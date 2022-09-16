use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

pub(crate) async fn handle_v0_transaction_status(
    state: Extension<CoreApiState>,
    request: Json<models::V0TransactionStatusRequest>,
) -> Result<Json<models::V0TransactionStatusResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_status_internal)
}

use models::v0_transaction_payload_status::Status as PayloadStatus;
use models::v0_transaction_status_response::IntentStatus;

fn handle_v0_transaction_status_internal(
    state_manager: &mut ActualStateManager,
    request: models::V0TransactionStatusRequest,
) -> Result<models::V0TransactionStatusResponse, RequestHandlingError> {
    let intent_hash = extract_intent_hash(request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let committed_option = state_manager
        .store
        .get_committed_transaction_by_intent(&intent_hash);

    if let Some((stored_transaction, receipt, _)) = committed_option {
        let intent_status = match &receipt.status {
            state_manager::CommittedTransactionStatus::Success(_) => IntentStatus::CommittedSuccess,
            state_manager::CommittedTransactionStatus::Failure(_) => IntentStatus::CommittedFailure,
        };

        let payload_status = match &receipt.status {
            state_manager::CommittedTransactionStatus::Success(_) => {
                PayloadStatus::CommittedSuccess
            }
            state_manager::CommittedTransactionStatus::Failure(_) => {
                PayloadStatus::CommittedFailure
            }
        };

        let committed_payload = models::V0TransactionPayloadStatus {
            payload_hash: to_api_payload_hash(&stored_transaction.get_hash()),
            status: payload_status,
        };

        Ok(models::V0TransactionStatusResponse {
            intent_status,
            known_payloads: vec![committed_payload],
        })
    } else {
        // Need to implement properly!
        Ok(models::V0TransactionStatusResponse {
            intent_status: models::v0_transaction_status_response::IntentStatus::Unknown,
            known_payloads: vec![],
        })
    }
}
