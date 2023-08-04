use crate::core_api::*;

use radix_engine::types::*;
use state_manager::query::dump_component_state;
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

use super::component_dump_to_vaults_and_nodes;

pub(crate) async fn handle_state_access_controller(
    state: State<CoreApiState>,
    Json(request): Json<models::StateAccessControllerRequest>,
) -> Result<Json<models::StateAccessControllerResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let controller_address =
        extract_component_address(&extraction_context, &request.controller_address)
            .map_err(|err| err.into_response_error("controller_address"))?;

    if !request.controller_address.starts_with("accesscontroller_") {
        return Err(client_error("Only access controller addresses work for this endpoint. Try another endpoint instead."));
    }

    let database = state.database.read();

    let access_controller_substate = read_optional_main_field_substate(
        database.deref(),
        controller_address.as_node_id(),
        &AccessControllerField::AccessController.into(),
    )
    .ok_or_else(|| not_found_error("Access controller not found".to_string()))?;

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        controller_address.as_node_id(),
        ACCESS_RULES_FIELDS_PARTITION,
        &AccessRulesField::OwnerRole.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), controller_address);
    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    Ok(models::StateAccessControllerResponse {
        state_version: to_api_state_version(database.max_state_version())?,
        state: Some(to_api_access_controller_substate(
            &mapping_context,
            &access_controller_substate,
        )?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
        vaults,
        descendent_nodes,
    })
    .map(Json)
}
