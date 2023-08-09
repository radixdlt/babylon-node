use crate::core_api::*;
use radix_engine::types::*;

use state_manager::query::dump_component_state;
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

use super::component_dump_to_vaults_and_nodes;

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

    let database = state.state_manager.database.read();

    let validator_substate = read_optional_main_field_substate(
        database.deref(),
        validator_address.as_node_id(),
        &ValidatorField::Validator.into(),
    )
    .ok_or_else(|| not_found_error("Validator not found".to_string()))?;

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        validator_address.as_node_id(),
        ROLE_ASSIGNMENT_FIELDS_PARTITION,
        &RoleAssignmentField::OwnerRole.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), validator_address);

    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    let header = database
        .get_last_proof()
        .expect("proof for outputted state must exist")
        .ledger_header;

    Ok(models::StateValidatorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        address: to_api_component_address(&mapping_context, &validator_address)?,
        state: Some(to_api_validator_substate(
            &mapping_context,
            &validator_substate,
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
