use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state), err(Debug))]
pub(crate) async fn handle_mempool_list(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, ResponseError<()>> {
    let state_manager = state.state_manager.read();
    assert_matching_network(&request.network, &state_manager.network)?;

    let mempool = state_manager.mempool.read();
    Ok(models::MempoolListResponse {
        contents: mempool
            .all_hashes_iter()
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
