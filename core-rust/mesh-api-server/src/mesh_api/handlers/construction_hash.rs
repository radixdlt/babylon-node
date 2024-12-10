use crate::prelude::*;

pub(crate) async fn handle_construction_hash(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionHashRequest>,
) -> Result<Json<models::TransactionIdentifierResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let intent_hash = RawNotarizedTransaction::from_hex(&request.signed_transaction)
        .map_err(|_| {
            ResponseError::from(ApiError::InvalidTransaction).with_details(format!(
                "Invalid transaction hex: {}",
                &request.signed_transaction
            ))
        })?
        .prepare(PreparationSettings::latest_ref())
        .map_err(|err| {
            ResponseError::from(ApiError::InvalidTransaction)
                .with_details(format!("Failed to prepare user transaction: {:?}", err))
        })?
        .transaction_intent_hash();

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
