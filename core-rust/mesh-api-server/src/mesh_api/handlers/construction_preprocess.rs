use crate::prelude::*;

pub(crate) async fn handle_construction_preprocess(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPreprocessRequest>,
) -> Result<Json<models::ConstructionPreprocessResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionpreprocessresponse for field
    // definitions
    Ok(Json(models::ConstructionPreprocessResponse {
        options: None,
        required_public_keys: None,
    }))
}
