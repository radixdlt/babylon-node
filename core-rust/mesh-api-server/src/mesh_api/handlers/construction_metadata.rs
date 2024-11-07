use crate::prelude::*;

pub(crate) async fn handle_construction_metadata(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionMetadataRequest>,
) -> Result<Json<models::ConstructionMetadataResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionmetadataresponse for field
    // definitions
    Ok(Json(models::ConstructionMetadataResponse {
        metadata: serde_json::Value::Null,
        suggested_fee: None,
    }))
}
