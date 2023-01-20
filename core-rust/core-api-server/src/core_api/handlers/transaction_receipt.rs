use crate::core_api::handlers::to_api_committed_transaction;
use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_transaction_receipt(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionReceiptRequest>,
) -> Result<Json<models::TransactionReceiptResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_transaction_receipt_internal)
}

fn handle_transaction_receipt_internal(
    state_manager: &ActualStateManager,
    request: models::TransactionReceiptRequest,
) -> Result<models::TransactionReceiptResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let intent_hash = extract_intent_hash(request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let network = &state_manager.network;
    let committed_option = state_manager
        .store
        .get_committed_transaction_by_identifier(&intent_hash);

    if let Some((ledger_transaction, receipt, identifiers)) = committed_option {
        Ok(models::TransactionReceiptResponse {
            committed: Box::new(to_api_committed_transaction(
                network,
                ledger_transaction,
                receipt,
                identifiers,
            )?),
        })
    } else {
        Err(not_found_error(format!(
            "Committed transaction not found with intent hash: {}",
            intent_hash
        )))
    }
}
