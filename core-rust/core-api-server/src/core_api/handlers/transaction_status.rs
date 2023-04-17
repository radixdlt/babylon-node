use std::collections::{HashMap, HashSet};

use crate::core_api::*;

use state_manager::{
    DetailedTransactionOutcome, HasUserPayloadHash, RejectionReason, UserPayloadHash,
};

use models::transaction_payload_status::Status as PayloadStatus;
use models::TransactionIntentStatus as IntentStatus;
use state_manager::mempool::pending_transaction_result_cache::PendingTransactionRecord;
use state_manager::query::StateManagerSubstateQueries;
use state_manager::store::traits::*;

#[tracing::instrument(err(Debug), skip(state))]
pub(crate) async fn handle_transaction_status(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionStatusRequest>,
) -> Result<Json<models::TransactionStatusResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new_for_uncommitted_data(&state.network);

    let intent_hash = extract_intent_hash(request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let state_manager = state.state_manager.read();
    let database = state.database.read();

    let txn_state_version_opt = database.get_txn_state_version_by_identifier(&intent_hash);

    let mut known_pending_payloads = state_manager
        .pending_transaction_result_cache
        .peek_all_known_payloads_for_intent(&intent_hash);

    let current_epoch = database.get_epoch();

    let invalid_from_epoch = known_pending_payloads
        .iter()
        .next()
        .map(|p| p.1.intent_invalid_from_epoch);

    let intent_is_permanently_rejected = invalid_from_epoch.map_or(false, |invalid_from_epoch| {
        current_epoch >= invalid_from_epoch
    }) || known_pending_payloads.iter().any(|p| {
        p.1.earliest_permanent_rejection
            .as_ref()
            .map_or(false, |r| r.marks_permanent_rejection_for_intent())
    });

    if let Some(txn_state_version) = txn_state_version_opt {
        let txn = database
            .get_committed_transaction(txn_state_version)
            .expect("Txn is missing");

        let local_detailed_outcome = database
            .get_committed_transaction_receipt(txn_state_version)
            .expect("Txn receipt is missing")
            .local_execution
            .outcome;

        let payload_hash = txn
            .user()
            .expect("Only user transactions should be able to be looked up by intent hash")
            .user_payload_hash();

        // Remove the committed payload from the rejection list if it's present
        known_pending_payloads.remove(&payload_hash);

        let (intent_status, payload_status, outcome, error_message) = match local_detailed_outcome {
            DetailedTransactionOutcome::Success(_) => (
                IntentStatus::CommittedSuccess,
                PayloadStatus::CommittedSuccess,
                "SUCCESS",
                None,
            ),
            DetailedTransactionOutcome::Failure(reason) => (
                IntentStatus::CommittedFailure,
                PayloadStatus::CommittedFailure,
                "FAILURE",
                Some(format!("{reason:?}")),
            ),
        };

        let committed_payload = models::TransactionPayloadStatus {
            payload_hash: to_api_payload_hash(&payload_hash),
            status: payload_status,
            error_message,
        };

        let mut known_payloads = vec![committed_payload];
        known_payloads.append(&mut map_rejected_payloads_due_to_known_commit(
            known_pending_payloads,
        ));

        return Ok(models::TransactionStatusResponse {
            intent_status,
            status_description: format!("The transaction has been committed to the ledger, with an outcome of {outcome}. For more information, use the /transaction/receipt endpoint."),
            invalid_from_epoch: None,
            known_payloads,
        }).map(Json);
    }

    let mempool_payloads_hashes = state_manager
        .mempool
        .read()
        .get_payload_hashes_for_intent(&intent_hash);

    if !mempool_payloads_hashes.is_empty() {
        let mempool_payloads = mempool_payloads_hashes
            .iter()
            .map(|payload_hash| models::TransactionPayloadStatus {
                payload_hash: to_api_payload_hash(payload_hash),
                status: PayloadStatus::InMempool,
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
            intent_status: IntentStatus::InMempool,
            status_description: "At least one payload for the intent is in this node's mempool. This node believes it's possible the intent might be able to be committed. Whilst the transaction continues to live in the mempool, you can use the /mempool/transaction endpoint to read its payload.".to_owned(),
            invalid_from_epoch: invalid_from_epoch.map(|epoch| to_api_epoch(&mapping_context, epoch)).transpose()?,
            known_payloads,
        }).map(Json);
    }

    let known_payloads = map_pending_payloads_not_in_mempool(known_pending_payloads);

    let response = if intent_is_permanently_rejected {
        models::TransactionStatusResponse {
            intent_status: IntentStatus::PermanentRejection,
            status_description: "Based on the results from executing a payload for this intent, the node believes the intent is permanently rejected - this means that any transaction payload containing the intent should never be able to be committed.".to_owned(),
            invalid_from_epoch: None,
            known_payloads,
        }
    } else {
        let (status, description) = if known_payloads.is_empty() {
            (
                IntentStatus::NotSeen,
                "No payloads for this intent have been seen recently by this node.",
            )
        } else {
            let any_payloads_not_rejected = known_payloads
                .iter()
                .any(|p| p.status == PayloadStatus::NotInMempool);
            if any_payloads_not_rejected {
                (IntentStatus::FateUncertain, "At least one payload for this intent was not rejected at its last execution, it's unknown whether it will be committed or not.")
            } else {
                (IntentStatus::FateUncertainButLikelyRejection, "All known payloads were rejected at their last execution. But none of these rejections implied that the intent itself is permanently rejected. It may still be possible for the intent to be committed.")
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
    known_rejected_payloads: HashMap<UserPayloadHash, PendingTransactionRecord>,
) -> Vec<models::TransactionPayloadStatus> {
    known_rejected_payloads
        .into_iter()
        .map(|(payload_hash, transaction_record)| {
            let rejection_reason_to_use = transaction_record
                .most_applicable_status()
                .unwrap_or(&RejectionReason::IntentHashCommitted);
            models::TransactionPayloadStatus {
                payload_hash: to_api_payload_hash(&payload_hash),
                status: PayloadStatus::PermanentlyRejected,
                error_message: Some(rejection_reason_to_use.to_string()),
            }
        })
        .collect::<Vec<_>>()
}

fn map_pending_payloads_not_in_mempool(
    known_payloads_not_in_mempool: HashMap<UserPayloadHash, PendingTransactionRecord>,
) -> Vec<models::TransactionPayloadStatus> {
    known_payloads_not_in_mempool
        .into_iter()
        .map(|(payload_hash, transaction_record)| {
            match transaction_record.most_applicable_status() {
                Some(reason) => models::TransactionPayloadStatus {
                    payload_hash: to_api_payload_hash(&payload_hash),
                    status: if reason.is_permanent_for_payload() {
                        PayloadStatus::PermanentlyRejected
                    } else {
                        PayloadStatus::TransientlyRejected
                    },
                    error_message: Some(reason.to_string()),
                },
                None => models::TransactionPayloadStatus {
                    payload_hash: to_api_payload_hash(&payload_hash),
                    status: PayloadStatus::NotInMempool,
                    error_message: None,
                },
            }
        })
        .collect::<Vec<_>>()
}
