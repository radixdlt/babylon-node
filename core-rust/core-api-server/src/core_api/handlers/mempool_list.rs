use crate::prelude::*;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_mempool_list(
    State(state): State<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    assert_unbounded_endpoints_flag_enabled(&state)?;
    let mapping_context = MappingContext::new(&state.network);

    Ok(Json(models::MempoolListResponse {
        contents: state
            .state_manager
            .mempool_manager
            .get_mempool_all_hashes()
            .iter()
            .map(|(intent_hash, payload_hash)| {
                Ok(models::MempoolTransactionHashes {
                    intent_hash: to_api_transaction_intent_hash(intent_hash),
                    intent_hash_bech32m: to_api_hash_bech32m(&mapping_context, intent_hash)?,
                    payload_hash: to_api_notarized_transaction_hash(payload_hash),
                    payload_hash_bech32m: to_api_hash_bech32m(&mapping_context, payload_hash)?,
                })
            })
            .collect::<Result<Vec<_>, MappingError>>()?,
    }))
}
