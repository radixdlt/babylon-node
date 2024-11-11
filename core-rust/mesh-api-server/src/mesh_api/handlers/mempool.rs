use crate::prelude::*;

pub(crate) async fn handle_mempool(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkRequest>,
) -> Result<Json<models::MempoolResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let mempool = state.state_manager.mempool.read();

    Ok(Json(models::MempoolResponse::new(
        mempool
            .all_hashes_iter()
            .map(|(intent_hash, _)| {
                Ok(models::TransactionIdentifier {
                    hash: to_api_transaction_hash_bech32m(&mapping_context, intent_hash)?,
                })
            })
            .collect::<Result<Vec<_>, MappingError>>()?,
    )))
}
