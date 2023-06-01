use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_mempool_list(
    State(state): State<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mempool = state.mempool.read();
    Ok(models::MempoolListResponse {
        contents: mempool
            .all_hashes_iter()
            .map(
                |(intent_hash, payload_hash)| models::MempoolTransactionHashes {
                    intent_hash: to_api_intent_hash(intent_hash),
                    payload_hash: to_api_notarized_transaction_hash(payload_hash),
                },
            )
            .collect(),
    })
    .map(Json)
}
