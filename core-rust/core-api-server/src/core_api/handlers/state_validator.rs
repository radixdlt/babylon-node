use crate::core_api::*;
use radix_engine::types::*;

use radix_engine::blueprints::consensus_manager::*;
use radix_engine::system::node_modules::access_rules::*;
use state_manager::query::dump_component_state;
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

    let database = state.database.read();

    let validator_substate: ValidatorSubstate = read_mandatory_main_field_substate(
        database.deref(),
        validator_address.as_node_id(),
        &ValidatorField::Validator.into(),
    )?;

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        validator_address.as_node_id(),
        ACCESS_RULES_FIELD_PARTITION,
        &AccessRulesField::AccessRules.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), validator_address);

    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    Ok(models::StateValidatorResponse {
        address: to_api_component_address(&mapping_context, &validator_address)?,
        state: Some(to_api_validator_substate(
            &mapping_context,
            &validator_substate,
        )?),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        vaults,
        descendent_nodes,
    })
    .map(Json)
}
