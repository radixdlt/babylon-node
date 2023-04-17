use crate::core_api::*;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ComponentOffset, RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::api::types::{
    AccessRulesOffset, AccountOffset, NodeModuleId, RoyaltyOffset, TypeInfoOffset,
};

use state_manager::query::{dump_component_state, VaultData};
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
    let type_info = {
        let substate_offset = SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::TypeInfo,
            &substate_offset,
        )?;
        let PersistedSubstate::TypeInfo(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };
    let component_state = {
        let substate_offset = SubstateOffset::Component(ComponentOffset::State0);
        let loaded_substate_opt = read_optional_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        );
        match loaded_substate_opt {
            Some(PersistedSubstate::ComponentState(substate)) => Some(substate),
            Some(..) => return Err(wrong_substate_type(substate_offset)),
            None => None,
        }
    };
    let account_state = {
        let substate_offset = SubstateOffset::Account(AccountOffset::Account);
        let loaded_substate_opt = read_optional_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::SELF,
            &substate_offset,
        );
        match loaded_substate_opt {
            Some(PersistedSubstate::Account(substate)) => Some(substate),
            Some(..) => return Err(wrong_substate_type(substate_offset)),
            None => None,
        }
    };
    // TODO: royalty_* should be non-optional once fixed on the engine side
    let component_royalty_config = {
        let substate_offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig);
        let loaded_substate_opt = read_optional_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::ComponentRoyalty,
            &substate_offset,
        );
        match loaded_substate_opt {
            Some(PersistedSubstate::ComponentRoyaltyConfig(substate)) => Some(substate),
            Some(..) => return Err(wrong_substate_type(substate_offset)),
            None => None,
        }
    };
    let component_royalty_accumulator = {
        let substate_offset = SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator);
        let loaded_substate_opt = read_optional_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::ComponentRoyalty,
            &substate_offset,
        );
        match loaded_substate_opt {
            Some(PersistedSubstate::ComponentRoyaltyAccumulator(substate)) => Some(substate),
            Some(..) => return Err(wrong_substate_type(substate_offset)),
            None => None,
        }
    };
    let component_access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(component_address.into()),
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let component_dump = dump_component_state(database.deref(), component_address)
        .map_err(|err| server_error(format!("Error traversing component state: {err:?}")))?;

    let state_owned_vaults = component_dump
        .vaults
        .into_iter()
        .map(|vault| match vault {
            VaultData::NonFungible {
                resource_address,
                ids,
            } => to_api_non_fungible_resource_amount(&mapping_context, &resource_address, &ids),
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
        access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &component_access_rules,
        )?),
        state_owned_vaults,
        descendent_ids,
    })
    .map(Json)
}

pub(crate) fn map_to_descendent_id(
    parent: Option<SubstateId>,
    node: RENodeId,
    depth: u32,
) -> Result<models::StateComponentDescendentId, MappingError> {
    Ok(models::StateComponentDescendentId {
        parent: Box::new(to_api_substate_id(parent.unwrap())?),
        entity: Box::new(to_api_entity_reference(node)?),
        depth: depth as i32, // Won't go over 100 due to component dumper max depth
    })
}
