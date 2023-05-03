use crate::core_api::*;
use radix_engine::types::ValidatorOffset;

use radix_engine::blueprints::epoch_manager::ValidatorSubstate;
use radix_engine::system::node_modules::access_rules::MethodAccessRulesSubstate;
use radix_engine_interface::types::{AccessRulesOffset, SysModuleId};
use state_manager::query::{dump_component_state, VaultData};
use std::ops::Deref;

use super::map_to_descendent_id;

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

    let validator_substate: ValidatorSubstate = read_mandatory_substate(
        database.deref(),
        validator_address.as_node_id(),
        SysModuleId::Object.into(),
        &ValidatorOffset::Validator.into(),
    )?;

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        validator_address.as_node_id(),
        SysModuleId::AccessRules.into(),
        &AccessRulesOffset::AccessRules.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), validator_address);

    let state_owned_vaults = component_dump
        .vaults
        .into_values()
        .map(|vault_data| match vault_data {
            VaultData::NonFungible {
                resource_address,
                amount,
                ids,
            } => to_api_non_fungible_resource_amount(
                &mapping_context,
                &resource_address,
                &amount,
                &ids,
            ),
            VaultData::Fungible {
                resource_address,
                amount,
            } => to_api_fungible_resource_amount(&mapping_context, &resource_address, &amount),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let descendent_ids = component_dump
        .descendents
        .into_iter()
        .filter(|(_, _, depth)| *depth > 0)
        .map(|(parent, node, depth)| map_to_descendent_id(parent, node, depth))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(models::StateValidatorResponse {
        address: to_api_component_address(&mapping_context, &validator_address),
        state: Some(to_api_validator_substate(
            &mapping_context,
            &validator_substate,
        )?),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
    .map(Json)
}
