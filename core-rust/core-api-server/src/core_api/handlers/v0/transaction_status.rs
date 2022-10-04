use std::collections::HashMap;

use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{HasUserPayloadHash, UserPayloadHash};

use state_manager::mempool::transaction_rejection_cache::RejectionRecord;
use state_manager::store::traits::*;

#[tracing::instrument(err(Debug), skip(state))]
pub(crate) async fn handle_v0_transaction_status(
    state: Extension<CoreApiState>,
    request: Json<models::V0TransactionStatusRequest>,
) -> Result<Json<models::V0TransactionStatusResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_v0_transaction_status_internal)
}

use models::v0_transaction_payload_status::Status as PayloadStatus;
use models::v0_transaction_status_response::IntentStatus;

fn handle_v0_transaction_status_internal(
    state_manager: &ActualStateManager,
    request: models::V0TransactionStatusRequest,
) -> Result<models::V0TransactionStatusResponse, RequestHandlingError> {
    let intent_hash = extract_intent_hash(request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let committed_option = state_manager
        .store
        .get_committed_transaction_by_identifier(&intent_hash);

    let mut rejected_payloads = state_manager
        .rejection_cache
        .peek_all_rejected_payloads_for_intent(&intent_hash);

    if let Some((stored_transaction, receipt, _)) = committed_option {
        let payload_hash = stored_transaction.user_payload_hash();

        // Remove the committed payload from the rejection list if it's present
        rejected_payloads.remove(&payload_hash);

        let intent_status = match &receipt.status {
            state_manager::CommittedTransactionStatus::Success(_) => IntentStatus::CommittedSuccess,
            state_manager::CommittedTransactionStatus::Failure(_) => IntentStatus::CommittedFailure,
        };

        let (payload_status, error_message) = match &receipt.status {
            state_manager::CommittedTransactionStatus::Success(_) => {
                (PayloadStatus::CommittedSuccess, None)
            }
            state_manager::CommittedTransactionStatus::Failure(reason) => {
                (PayloadStatus::CommittedFailure, Some(reason.to_owned()))
            }
        };

        let committed_payload = models::V0TransactionPayloadStatus {
            payload_hash: to_api_payload_hash(&payload_hash),
            status: payload_status,
            error_message,
        };

        let mut known_payloads = vec![committed_payload];
        known_payloads.append(&mut map_rejected_payloads(rejected_payloads));

        return Ok(models::V0TransactionStatusResponse {
            intent_status,
            known_payloads,
        });
    }

    let mempool_payloads_hashes = state_manager
        .mempool
        .get_payload_hashes_for_intent(&intent_hash);

    if !mempool_payloads_hashes.is_empty() {
        let mempool_payloads = mempool_payloads_hashes
            .into_iter()
            .map(|payload_hash| models::V0TransactionPayloadStatus {
                payload_hash: to_api_payload_hash(&payload_hash),
                status: PayloadStatus::InMempool,
                error_message: None,
            })
            .collect::<Vec<_>>();

        let mut known_payloads = mempool_payloads;
        known_payloads.append(&mut map_rejected_payloads(rejected_payloads));

        return Ok(models::V0TransactionStatusResponse {
            intent_status: models::v0_transaction_status_response::IntentStatus::InMempool,
            known_payloads,
        });
    }

    let known_payloads = map_rejected_payloads(rejected_payloads);

    let intent_status = if !known_payloads.is_empty() {
        models::v0_transaction_status_response::IntentStatus::Rejected
    } else {
        models::v0_transaction_status_response::IntentStatus::Unknown
    };

    Ok(models::V0TransactionStatusResponse {
        intent_status,
        known_payloads,
    })
}

fn map_rejected_payloads(
    known_rejected_payloads: HashMap<UserPayloadHash, RejectionRecord>,
) -> Vec<models::V0TransactionPayloadStatus> {
    known_rejected_payloads
        .into_iter()
        .map(
            |(payload_hash, rejection_record)| models::V0TransactionPayloadStatus {
                payload_hash: to_api_payload_hash(&payload_hash),
                status: PayloadStatus::Rejected,
                error_message: Some(rejection_record.reason.to_string()),
            },
        )
        .collect::<Vec<_>>()
}
