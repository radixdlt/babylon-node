use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_mempool_list(
    State(state): State<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let mempool = state.state_manager.mempool.read();
    Ok(models::MempoolListResponse {
        contents: mempool
            .all_hashes_iter()
            .map(|(intent_hash, payload_hash)| {
                Ok(models::MempoolTransactionHashes {
                    intent_hash: to_api_intent_hash(intent_hash),
                    intent_hash_bech32m: to_api_hash_bech32m(&mapping_context, intent_hash)?,
                    payload_hash: to_api_notarized_transaction_hash(payload_hash),
                    payload_hash_bech32m: to_api_hash_bech32m(&mapping_context, payload_hash)?,
                })
            })
            .collect::<Result<Vec<_>, MappingError>>()?,
    })
    .map(Json)
}
