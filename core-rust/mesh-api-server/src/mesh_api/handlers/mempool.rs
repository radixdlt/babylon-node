use crate::prelude::*;

pub(crate) async fn handle_mempool(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkRequest>,
) -> Result<Json<models::MempoolResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    // let database = state.state_manager.database.snapshot();
    // let mapping_context = MappingContext::new(&state.network);

    Ok(Json(models::MempoolResponse::new(vec![])))
}
