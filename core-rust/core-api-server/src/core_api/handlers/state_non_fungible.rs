use crate::core_api::*;

use crate::core_api::models::StateNonFungibleResponse;
use radix_engine::blueprints::resource::NonFungibleResourceManagerSubstate;
use radix_engine_interface::types::{NonFungibleResourceManagerOffset, SysModuleId};
use std::ops::Deref;

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
    let resource_manager: NonFungibleResourceManagerSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        SysModuleId::Object.into(),
        &NonFungibleResourceManagerOffset::ResourceManager.into(),
    )?;

    let non_fungible_id_type = resource_manager.id_type;

    let non_fungible_id =
        extract_non_fungible_id_from_simple_representation(&request.non_fungible_id)
            .map_err(|err| err.into_response_error("non_fungible_id"))?;

    if non_fungible_id.id_type() != non_fungible_id_type {
        return Err(ExtractionError::WrongNonFungibleIdType {
            expected: non_fungible_id_type,
            actual: non_fungible_id.id_type(),
        }
        .into_response_error("non_fungible_id"));
    }

    // TODO: figure out how nfts are stored and use a correct substate
    /*
        let xyz_substate: Option<???> = read_optional_substate(
            database.deref(),
            resource_manager.non_fungible_table.as_node_id(),
            SysModuleId::Object.into(),
            scrypto_encode(&non_fungible_id), // TODO: this is probably wrong.... check correct nft offset
        ).unwrap_or(
            not_found_error(
                "The specified non-fungible id does not exist under that non-fungible resource"));
    */

    Ok(StateNonFungibleResponse {
        /* TODO: fixme
        non_fungible: Some(to_api_xyz_substate(
            &mapping_context,
            &xyz_substate,
        )?),
         */
        non_fungible: None,
    })
    .map(Json)
}
