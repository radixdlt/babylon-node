use crate::prelude::*;

pub(crate) async fn handle_construction_hash(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionHashRequest>,
) -> Result<Json<models::TransactionIdentifierResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let intent_hash = RawNotarizedTransaction::from_hex(&request.signed_transaction)
        .ok()
        .and_then(|raw| raw.prepare(PreparationSettings::latest_ref()).ok())
        .and_then(|tx| Some(tx.hashes()))
        .ok_or(
            ResponseError::from(ApiError::InvalidTransaction).with_details(format!(
                "Invalid transaction: {}",
                request.signed_transaction
            )),
        )?
        .transaction_intent_hash;

    let transaction_identifier = to_mesh_api_transaction_identifier_from_hash(
        to_api_transaction_hash_bech32m(&MappingContext::new(&state.network), &intent_hash)?,
    );

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionhashresponse for field
    // definitions
    Ok(Json(models::TransactionIdentifierResponse {
        transaction_identifier: Box::new(transaction_identifier),
        metadata: None,
    }))
}
