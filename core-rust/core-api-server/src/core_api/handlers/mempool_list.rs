use crate::core_api::*;

use state_manager::jni::state_manager::ActualStateManager;

pub(crate) async fn handle_mempool_list(
    state: Extension<CoreApiState>,
    request: Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_mempool_list_internal)
}

#[tracing::instrument(level = "debug", skip(state_manager), err(Debug))]
fn handle_mempool_list_internal(
    state_manager: &mut ActualStateManager,
    request: models::MempoolListRequest,
) -> Result<models::MempoolListResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    Ok(models::MempoolListResponse {
        contents: state_manager
            .mempool
            .list_all_hashes()
            .into_iter()
            .map(
                |(intent_hash, payload_hash)| models::MempoolTransactionHashes {
                    intent_hash: to_api_intent_hash(intent_hash),
                    payload_hash: to_api_payload_hash(payload_hash),
                },
            )
            .collect(),
    })
}
