use std::collections::{HashMap, HashSet};

use crate::core_api::*;

use state_manager::{
    AlreadyCommittedError, DetailedTransactionOutcome, RejectionReason, StateVersion,
};

use state_manager::mempool::pending_transaction_result_cache::PendingTransactionRecord;
use state_manager::query::StateManagerSubstateQueries;
use state_manager::store::traits::*;
use transaction::prelude::*;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_transaction_status(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionStatusRequest>,
) -> Result<Json<models::TransactionStatusResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new_for_uncommitted_data(&state.network);

    let intent_hash = extract_intent_hash(request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let pending_transaction_result_cache =
        state.state_manager.pending_transaction_result_cache.read();
    let mut known_pending_payloads =
        pending_transaction_result_cache.peek_all_known_payloads_for_intent(&intent_hash);
    drop(pending_transaction_result_cache);

    let database = state.state_manager.database.read();

    if !database.is_local_transaction_execution_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the LocalTransactionExecutionIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config, wipe ledger and restart. \
            Please note the resync will take a while.",
        ));
    }

    let txn_state_version_opt = database.get_txn_state_version_by_identifier(&intent_hash);
    let current_epoch = database.get_epoch();

    let invalid_from_epoch = known_pending_payloads
        .iter()
        .filter_map(|p| p.1.intent_invalid_from_epoch)
        .next();

    let intent_is_permanently_rejected = invalid_from_epoch.map_or(false, |invalid_from_epoch| {
        current_epoch >= invalid_from_epoch
    }) || known_pending_payloads.iter().any(|p| {
        p.1.earliest_permanent_rejection
            .as_ref()
            .map_or(false, |r| r.marks_permanent_rejection_for_intent())
    });

    if let Some(txn_state_version) = txn_state_version_opt {
        let identifiers = database
            .get_committed_transaction_identifiers(txn_state_version)
            .expect("Txn identifiers are missing")
            .payload
            .typed;

        let local_detailed_outcome = database
            .get_committed_local_transaction_execution(txn_state_version)
            .expect("Txn local execution is missing")
            .outcome;
        drop(database);

        let user_identifiers = identifiers
            .user()
            .expect("Only user transactions should be able to be looked up by intent hash");
        let notarized_transaction_hash = user_identifiers.notarized_transaction_hash;

        // Remove the committed payload from the rejection list if it's present
        known_pending_payloads.remove(notarized_transaction_hash);

        let (intent_status, payload_status, outcome, error_message) = match local_detailed_outcome {
            DetailedTransactionOutcome::Success(_) => (
                models::TransactionIntentStatus::CommittedSuccess,
                models::TransactionPayloadStatus::CommittedSuccess,
                "SUCCESS",
                None,
            ),
            DetailedTransactionOutcome::Failure(reason) => (
                models::TransactionIntentStatus::CommittedFailure,
                models::TransactionPayloadStatus::CommittedFailure,
                "FAILURE",
                Some(format!("{reason:?}")),
            ),
        };

        let committed_payload = models::TransactionPayloadDetails {
            payload_hash: to_api_notarized_transaction_hash(
                user_identifiers.notarized_transaction_hash,
            ),
            state_version: Some(to_api_state_version(txn_state_version)?),
            status: payload_status,
            error_message,
        };

        let mut known_payloads = vec![committed_payload];
        known_payloads.append(&mut map_rejected_payloads_due_to_known_commit(
            known_pending_payloads,
            txn_state_version,
            notarized_transaction_hash,
        ));

        return Ok(models::TransactionStatusResponse {
            intent_status,
            status_description: format!("The transaction has been committed to the ledger, with an outcome of {outcome}. For more information, use the /transaction/receipt endpoint."),
            invalid_from_epoch: None,
            known_payloads,
        }).map(Json);
    }

    let mempool = state.state_manager.mempool.read();
    let mempool_payloads_hashes = mempool.get_payload_hashes_for_intent(&intent_hash);
    drop(mempool);

    if !mempool_payloads_hashes.is_empty() {
        let mempool_payloads = mempool_payloads_hashes
            .iter()
            .map(|payload_hash| models::TransactionPayloadDetails {
                payload_hash: to_api_notarized_transaction_hash(payload_hash),
                state_version: None,
                status: models::TransactionPayloadStatus::InMempool,
                error_message: None,
            })
            .collect::<Vec<_>>();

        let mempool_payloads_hashes: HashSet<_> = mempool_payloads_hashes.into_iter().collect();

        let known_payloads_not_in_mempool = known_pending_payloads
            .into_iter()
            .filter(|p| !mempool_payloads_hashes.contains(&p.0))
            .collect();

        let mut known_payloads = mempool_payloads;
        known_payloads.append(&mut map_pending_payloads_not_in_mempool(
            known_payloads_not_in_mempool,
        ));

        return Ok(models::TransactionStatusResponse {
            intent_status: models::TransactionIntentStatus::InMempool,
            status_description: "At least one payload for the intent is in this node's mempool. This node believes it's possible the intent might be able to be committed. Whilst the transaction continues to live in the mempool, you can use the /mempool/transaction endpoint to read its payload.".to_owned(),
            invalid_from_epoch: invalid_from_epoch.map(|epoch| to_api_epoch(&mapping_context, epoch)).transpose()?,
            known_payloads,
        }).map(Json);
    }

    let known_payloads = map_pending_payloads_not_in_mempool(known_pending_payloads);

    let response = if intent_is_permanently_rejected {
        models::TransactionStatusResponse {
            intent_status: models::TransactionIntentStatus::PermanentRejection,
            status_description: "Based on the results from executing a payload for this intent, the node believes the intent is permanently rejected - this means that any transaction payload containing the intent should never be able to be committed.".to_owned(),
            invalid_from_epoch: None,
            known_payloads,
        }
    } else {
        let (status, description) = if known_payloads.is_empty() {
            (
                models::TransactionIntentStatus::NotSeen,
                "No payloads for this intent have been seen recently by this node.",
            )
        } else {
            let any_payloads_not_rejected = known_payloads
                .iter()
                .any(|p| p.status == models::TransactionPayloadStatus::NotInMempool);
            if any_payloads_not_rejected {
                (
                    models::TransactionIntentStatus::FateUncertain,
                    "At least one payload for this intent was not rejected at its last execution, it's unknown whether it will be committed or not."
                )
            } else {
                (
                    models::TransactionIntentStatus::FateUncertainButLikelyRejection,
                    "All known payloads were rejected at their last execution. But none of these rejections implied that the intent itself is permanently rejected. It may still be possible for the intent to be committed."
                )
            }
        };
        models::TransactionStatusResponse {
            intent_status: status,
            status_description: description.to_owned(),
            invalid_from_epoch: invalid_from_epoch
                .map(|epoch| to_api_epoch(&mapping_context, epoch))
                .transpose()?,
            known_payloads,
        }
    };

    Ok(response).map(Json)
}

fn map_rejected_payloads_due_to_known_commit(
    known_rejected_payloads: HashMap<NotarizedTransactionHash, PendingTransactionRecord>,
    committed_state_version: StateVersion,
    committed_notarized_transaction_hash: &NotarizedTransactionHash,
) -> Vec<models::TransactionPayloadDetails> {
    known_rejected_payloads
        .into_iter()
        .map(|(notarized_transaction_hash, transaction_record)| {
            let error_string_to_use = transaction_record
                .most_applicable_status()
                .map(|reason| reason.to_string())
                // Note: in theory, we should not see the "no rejection" for any transaction here,
                // since we only enter this method after seeing their intent hash committed by a
                // different payload. However, the cache update happens asynchronously after the
                // commit, and we may see a "not-yet-updated" entry - luckily, in such case, we can
                // precisely tell the transaction's status ourselves:
                .unwrap_or_else(|| {
                    RejectionReason::AlreadyCommitted(AlreadyCommittedError {
                        notarized_transaction_hash,
                        committed_state_version,
                        committed_notarized_transaction_hash: *committed_notarized_transaction_hash,
                    })
                    .to_string()
                });
            models::TransactionPayloadDetails {
                payload_hash: to_api_notarized_transaction_hash(&notarized_transaction_hash),
                state_version: None,
                status: models::TransactionPayloadStatus::PermanentlyRejected,
                error_message: Some(error_string_to_use),
            }
        })
        .collect::<Vec<_>>()
}

fn map_pending_payloads_not_in_mempool(
    known_payloads_not_in_mempool: HashMap<NotarizedTransactionHash, PendingTransactionRecord>,
) -> Vec<models::TransactionPayloadDetails> {
    known_payloads_not_in_mempool
        .into_iter()
        .map(|(payload_hash, transaction_record)| {
            match transaction_record.most_applicable_status() {
                Some(reason) => models::TransactionPayloadDetails {
                    payload_hash: to_api_notarized_transaction_hash(&payload_hash),
                    state_version: None,
                    status: if reason.is_permanent_for_payload() {
                        models::TransactionPayloadStatus::PermanentlyRejected
                    } else {
                        models::TransactionPayloadStatus::TransientlyRejected
                    },
                    error_message: Some(reason.to_string()),
                },
                None => models::TransactionPayloadDetails {
                    payload_hash: to_api_notarized_transaction_hash(&payload_hash),
                    state_version: None,
                    status: models::TransactionPayloadStatus::NotInMempool,
                    error_message: None,
                },
            }
        })
        .collect::<Vec<_>>()
}
