use radix_engine::blueprints::resource::NonFungibleResourceManagerIdTypeSubstate;
use std::ops::Deref;

use crate::core_api::*;
use radix_engine_interface::types::{NonFungibleResourceManagerOffset, SysModuleId};

use crate::core_api::models::StateNonFungibleResponse;

pub(crate) async fn handle_state_non_fungible(
    state: State<CoreApiState>,
    Json(request): Json<models::StateNonFungibleRequest>,
) -> Result<Json<StateNonFungibleResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let _mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let database = state.database.read();

    let _id_type_substate: NonFungibleResourceManagerIdTypeSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        SysModuleId::Object.into(),
        &NonFungibleResourceManagerOffset::IdType.into(),
    )?;

    let _non_fungible_id =
        extract_non_fungible_id_from_simple_representation(&request.non_fungible_id)
            .map_err(|err| err.into_response_error("non_fungible_id"))?;

    // TODO: bring this check back
    /*
    if non_fungible_id.id_type() != id_type_substate {
        return Err(ExtractionError::WrongNonFungibleIdType {
            expected: non_fungible_id_type,
            actual: non_fungible_id.id_type(),
        }
        .into_response_error("non_fungible_id"));
    }
     */

    // TODO: fixme
    Ok(StateNonFungibleResponse { non_fungible: None }).map(Json)
}
