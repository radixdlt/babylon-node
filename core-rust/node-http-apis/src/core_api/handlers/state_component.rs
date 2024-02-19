use crate::core_api::*;
use crate::engine_prelude::*;

use state_manager::query::{dump_component_state, ComponentStateDump, DescendantParentOpt};

use std::ops::Deref;

use super::map_to_vault_balance;

pub(crate) async fn handle_state_component(
    state: State<CoreApiState>,
    Json(request): Json<models::StateComponentRequest>,
) -> Result<Json<models::StateComponentResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    assert_unbounded_endpoints_flag_enabled(&state)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let component_address =
        extract_component_address(&extraction_context, &request.component_address)
            .map_err(|err| err.into_response_error("component_address"))?;

    if !request.component_address.starts_with("component_") {
        return Err(client_error("Only component addresses starting component_ currently work with this endpoint. Try another endpoint instead."));
    }

    let database = state.state_manager.database.snapshot();
    let type_info_substate = read_optional_substate(
        database.deref(),
        component_address.as_node_id(),
        TYPE_INFO_FIELD_PARTITION,
        &TypeInfoField::TypeInfo.into(),
    )
    .ok_or_else(|| not_found_error("Component not found".to_string()))?;

    let component_state_substate = read_mandatory_main_field_substate(
        database.deref(),
        component_address.as_node_id(),
        &ComponentField::State0.into(),
    )?;

    let component_royalty_substate =
        read_optional_substate::<ComponentRoyaltyAccumulatorFieldSubstate>(
            database.deref(),
            component_address.as_node_id(),
            ComponentRoyaltyPartitionOffset::Field.as_partition(ROYALTY_BASE_PARTITION),
            &RoyaltyField::RoyaltyAccumulator.into(),
        );

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        component_address.as_node_id(),
        RoleAssignmentPartitionOffset::Field.as_partition(ROLE_ASSIGNMENT_BASE_PARTITION),
        &RoleAssignmentField::Owner.into(),
    )?;

    let component_dump = dump_component_state(database.deref(), component_address);

    let (vaults, descendent_nodes) =
        component_dump_to_vaults_and_nodes(&mapping_context, component_dump)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::StateComponentResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        info: Some(to_api_type_info_substate(
            &mapping_context,
            &StateMappingLookups::default(),
            &type_info_substate,
        )?),
        state: Some(to_api_generic_scrypto_component_state_substate(
            &mapping_context,
            &component_state_substate,
        )?),
        royalty_accumulator: component_royalty_substate
            .map(|substate| to_api_component_royalty_substate(&mapping_context, &substate))
            .transpose()?
            .map(Box::new),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
        vaults,
        descendent_nodes,
    }))
}

pub(crate) fn component_dump_to_vaults_and_nodes(
    context: &MappingContext,
    component_dump: ComponentStateDump,
) -> Result<
    (
        Vec<models::VaultBalance>,
        Vec<models::StateComponentDescendentNode>,
    ),
    MappingError,
> {
    let vaults = component_dump
        .vaults
        .into_iter()
        .map(|(vault_id, vault_data)| map_to_vault_balance(context, vault_id, vault_data))
        .collect::<Result<Vec<_>, MappingError>>()?;

    let descendent_nodes = component_dump
        .descendents
        .into_iter()
        .filter(|(_, _, depth)| *depth > 0)
        .map(|(parent, node, depth)| map_to_descendent_id(context, parent, node, depth))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((vaults, descendent_nodes))
}

pub(crate) fn map_to_descendent_id(
    context: &MappingContext,
    parent: DescendantParentOpt,
    node_id: NodeId,
    depth: u32,
) -> Result<models::StateComponentDescendentNode, MappingError> {
    let parent = parent.unwrap();
    Ok(models::StateComponentDescendentNode {
        parent_entity: Box::new(to_api_entity_reference(context, &parent.0)?),
        parent_partition_number: parent.1 .0 as i32,
        parent_substate_key_hex: substate_key_to_hex(&parent.2),
        parent_substate_db_sort_key_hex: to_hex(SpreadPrefixKeyMapper::to_db_sort_key(&parent.2).0),
        entity: Box::new(to_api_entity_reference(context, &node_id)?),
        depth: depth as i32, // Won't go over 100 due to component dumper max depth
    })
}

pub(crate) fn substate_key_to_hex(substate_key: &SubstateKey) -> String {
    match substate_key {
        SubstateKey::Field(field_key) => to_hex([*field_key]),
        SubstateKey::Map(map_key) => to_hex(map_key),
        SubstateKey::Sorted((sort_key, map_key)) => to_hex([sort_key, map_key.as_slice()].concat()),
    }
}
