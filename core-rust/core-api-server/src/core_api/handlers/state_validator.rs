use super::component_dump_to_vaults_and_nodes;
use crate::prelude::*;

pub(crate) async fn handle_state_validator(
    state: State<CoreApiState>,
    Json(request): Json<models::StateValidatorRequest>,
) -> Result<Json<models::StateValidatorResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let validator_address =
        extract_component_address(&extraction_context, &request.validator_address)
            .map_err(|err| err.into_response_error("validator_address"))?;

    if !request.validator_address.starts_with("validator_") {
        return Err(client_error(
            "Only validator addresses work for this endpoint. Try another endpoint instead.",
        ));
    }

    let database = state.state_manager.database.snapshot();

    let validator_state_substate = read_optional_main_field_substate(
        database.deref(),
        validator_address.as_node_id(),
        &ValidatorField::State.into(),
    )
    .ok_or_else(|| not_found_error("Validator not found".to_string()))?;

    let validator_readiness_signal_substate = read_mandatory_main_field_substate(
        database.deref(),
        validator_address.as_node_id(),
        &ValidatorField::ProtocolUpdateReadinessSignal.into(),
    )?;

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        validator_address.as_node_id(),
        RoleAssignmentPartitionOffset::Field.as_partition(ROLE_ASSIGNMENT_BASE_PARTITION),
        &RoleAssignmentField::Owner.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), validator_address);

    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::StateValidatorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &header.into(),
        )?),
        address: to_api_component_address(&mapping_context, &validator_address)?,
        state: Some(to_api_validator_state_substate(
            &mapping_context,
            &validator_state_substate,
        )?),
        protocol_update_readiness_signal: Some(
            to_api_validator_protocol_update_readiness_signal_substate(
                &mapping_context,
                &validator_readiness_signal_substate,
            )?,
        ),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
        vaults,
        descendent_nodes,
    }))
}
