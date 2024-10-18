use crate::prelude::*;

use super::component_dump_to_vaults_and_nodes;

pub(crate) async fn handle_state_access_controller(
    state: State<CoreApiState>,
    Json(request): Json<models::StateAccessControllerRequest>,
) -> Result<Json<models::StateAccessControllerResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    assert_unbounded_endpoints_flag_enabled(&state)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let controller_address =
        extract_component_address(&extraction_context, &request.controller_address)
            .map_err(|err| err.into_response_error("controller_address"))?;

    if !request.controller_address.starts_with("accesscontroller_") {
        return Err(client_error("Only access controller addresses work for this endpoint. Try another endpoint instead."));
    }

    let database = state.state_manager.database.snapshot();

    let access_controller_state_substate = read_optional_main_field_substate(
        database.deref(),
        controller_address.as_node_id(),
        &AccessControllerV2Field::State.into(),
    )
    .ok_or_else(|| not_found_error("Access controller not found".to_string()))?;

    let owner_substate = read_mandatory_substate(
        database.deref(),
        controller_address.as_node_id(),
        RoleAssignmentPartitionOffset::Field.as_partition(ROLE_ASSIGNMENT_BASE_PARTITION),
        &RoleAssignmentField::Owner.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), controller_address);
    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::StateAccessControllerResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &header.into(),
        )?),
        state: Some(to_api_access_controller_substate(
            &mapping_context,
            &access_controller_state_substate,
        )?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_substate,
        )?),
        vaults,
        descendent_nodes,
    }))
}
