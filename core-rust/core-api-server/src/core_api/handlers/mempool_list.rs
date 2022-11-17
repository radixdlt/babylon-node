use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_mempool_list(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, RequestHandlingError> {
    let state_manager = state.state_manager.read();
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
    .map(Json)
}
