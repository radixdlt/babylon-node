use crate::core_api::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::types::ComponentOffset;
use radix_engine_common::types::NodeId;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
};

use radix_engine_interface::types::{
    AccessRulesOffset, AccountOffset, RoyaltyOffset, SysModuleId, TypeInfoOffset,
};
use radix_engine_queries::typed_substate_layout::AccountSubstate;
use radix_engine_queries::typed_substate_layout::MethodAccessRulesSubstate;
use state_manager::query::{dump_component_state, DescendantParentOpt, VaultData};
use std::ops::Deref;

pub(crate) async fn handle_state_component(
    state: State<CoreApiState>,
    Json(request): Json<models::StateComponentRequest>,
) -> Result<Json<models::StateComponentResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address =
        extract_component_address(&extraction_context, &request.component_address)
            .map_err(|err| err.into_response_error("component_address"))?;

    if !request.component_address.starts_with("component_")
        && !request.component_address.starts_with("account_")
    {
        // Until we have improvements to the state model for objects, only components should be supported here
        return Err(client_error("Only component addresses starting component_ or account_ currently work with this endpoint. Try another endpoint instead."));
    }

    let database = state.database.read();
    let type_info: TypeInfoSubstate = read_mandatory_substate(
        database.deref(),
        component_address.as_node_id(),
        SysModuleId::TypeInfo.into(),
        &TypeInfoOffset::TypeInfo.into(),
    )?;

    let component_state: Option<ComponentStateSubstate> = read_optional_substate(
        database.deref(),
        component_address.as_node_id(),
        SysModuleId::Object.into(),
        &ComponentOffset::State0.into(),
    );

    let account_state: Option<AccountSubstate> = read_optional_substate(
        database.deref(),
        component_address.as_node_id(),
        SysModuleId::Object.into(),
        &AccountOffset::Account.into(),
    );

    // TODO: royalty_* should be non-optional once fixed on the engine side
    let component_royalty_config: Option<ComponentRoyaltyConfigSubstate> = read_optional_substate(
        database.deref(),
        component_address.as_node_id(),
        SysModuleId::Royalty.into(),
        &RoyaltyOffset::RoyaltyConfig.into(),
    );

    let component_royalty_accumulator: Option<ComponentRoyaltyAccumulatorSubstate> =
        read_optional_substate(
            database.deref(),
            component_address.as_node_id(),
            SysModuleId::Royalty.into(),
            &RoyaltyOffset::RoyaltyAccumulator.into(),
        );

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        component_address.as_node_id(),
        SysModuleId::AccessRules.into(),
        &AccessRulesOffset::AccessRules.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), component_address);

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

    Ok(models::StateComponentResponse {
        info: Some(to_api_type_info_substate(&mapping_context, &type_info)?),
        state: if let Some(c) = component_state {
            Some(Box::new(to_api_component_state_substate(
                &mapping_context,
                &c,
            )?))
        } else {
            None
        },
        account: if let Some(a) = account_state {
            Some(Box::new(to_api_account_substate(&mapping_context, &a)?))
        } else {
            None
        },
        royalty_config: if let Some(r) = component_royalty_config {
            Some(Box::new(to_api_component_royalty_config_substate(
                &mapping_context,
                &r,
            )?))
        } else {
            None
        },
        royalty_accumulator: if let Some(r) = component_royalty_accumulator {
            Some(Box::new(to_api_component_royalty_accumulator_substate(&r)?))
        } else {
            None
        },
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
    .map(Json)
}

pub(crate) fn map_to_descendent_id(
    parent: DescendantParentOpt,
    node_id: NodeId,
    depth: u32,
) -> Result<models::StateComponentDescendentId, MappingError> {
    let parent = parent.unwrap();
    Ok(models::StateComponentDescendentId {
        parent_entity: Box::new(to_api_entity_reference(parent.0)?),
        parent_module_id: parent.1 .0 as i32,
        parent_sort_key: to_hex(&parent.2 .0),
        entity: Box::new(to_api_entity_reference(node_id)?),
        depth: depth as i32, // Won't go over 100 due to component dumper max depth
    })
}
