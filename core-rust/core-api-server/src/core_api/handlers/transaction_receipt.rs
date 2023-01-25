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
    let txn_state_version_opt = state_manager
        .store
        .get_txn_state_version_by_identifier(&intent_hash);

    if let Some(txn_state_version) = txn_state_version_opt {
        let txn = state_manager
            .store
            .get_committed_transaction(txn_state_version)
            .expect("Txn is missing");

        let receipt = state_manager
            .store
            .get_committed_transaction_receipt(txn_state_version)
            .expect("Txn receipt is missing");

        let identifiers = state_manager
            .store
            .get_committed_transaction_identifiers(txn_state_version)
            .expect("Txn identifiers are missing");

        Ok(models::TransactionReceiptResponse {
            committed: Box::new(to_api_committed_transaction(
                network,
                txn,
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
