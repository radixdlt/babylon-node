use crate::core_api::*;

use state_manager::jni::state_manager::ActualStateManager;

use super::to_api_notarized_transaction;

pub(crate) async fn handle_mempool_transaction(
    state: Extension<CoreApiState>,
    request: Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_mempool_list_internal)
}

fn handle_mempool_list_internal(
    state_manager: &ActualStateManager,
    request: models::MempoolTransactionRequest,
) -> Result<models::MempoolTransactionResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let mapping_context = MappingContext::new(&state_manager.network);

    let payload_hash = extract_payload_hash(request.payload_hash)
        .map_err(|err| err.into_response_error("payload_hash"))?;

    match state_manager.mempool.get_payload(&payload_hash) {
        Some(pending_transaction) => Ok(models::MempoolTransactionResponse {
            notarized_transaction: Box::new(to_api_notarized_transaction(
                &mapping_context,
                &pending_transaction.payload,
            )?),
        }),
        None => Err(not_found_error(
            "Transaction with given payload hash is not in the mempool",
        )),
    }
}
