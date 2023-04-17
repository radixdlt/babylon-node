use crate::core_api::*;
use radix_engine::blueprints::resource::{
    FungibleResourceManagerSubstate, NonFungibleResourceManagerSubstate,
};
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::{ResourceAddress, ResourceManagerOffset, SubstateOffset};
use radix_engine_interface::api::types::{AccessRulesOffset, NodeModuleId, RENodeId};
use std::ops::Deref;

enum ManagerByType {
    Fungible(FungibleResourceManagerSubstate),
    NonFungible(NonFungibleResourceManagerSubstate),
}

pub(crate) async fn handle_state_resource(
    state: State<CoreApiState>,
    Json(request): Json<models::StateResourceRequest>,
) -> Result<Json<models::StateResourceResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let database = state.database.read();

    let manager = match &resource_address {
        ResourceAddress::Fungible(_) => {
            let substate_offset =
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let loaded_substate = read_mandatory_substate(
                database.deref(),
                RENodeId::GlobalObject(resource_address.into()),
                NodeModuleId::SELF,
                &substate_offset,
            )?;
            let PersistedSubstate::ResourceManager(substate) = loaded_substate else {
                return Err(wrong_substate_type(substate_offset));
            };
            ManagerByType::Fungible(substate)
        }
        ResourceAddress::NonFungible(_) => {
            let substate_offset =
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let loaded_substate = read_mandatory_substate(
                database.deref(),
                RENodeId::GlobalObject(resource_address.into()),
                NodeModuleId::SELF,
                &substate_offset,
            )?;
            let PersistedSubstate::NonFungibleResourceManager(substate) = loaded_substate else {
                return Err(wrong_substate_type(substate_offset));
            };
            ManagerByType::NonFungible(substate)
        }
    };
    let access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::AccessRules,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    let vault_access_rules = {
        let substate_offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let loaded_substate = read_mandatory_substate(
            database.deref(),
            RENodeId::GlobalObject(resource_address.into()),
            NodeModuleId::AccessRules1,
            &substate_offset,
        )?;
        let PersistedSubstate::MethodAccessRules(substate) = loaded_substate else {
            return Err(wrong_substate_type(substate_offset));
        };
        substate
    };

    Ok(models::StateResourceResponse {
        manager: Some(match &manager {
            ManagerByType::Fungible(manager) => {
                to_api_fungible_resource_manager_substate(&mapping_context, manager)?
            }
            ManagerByType::NonFungible(manager) => {
                to_api_non_fungible_resource_manager_substate(&mapping_context, manager)?
            }
        }),
        access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &access_rules,
        )?),
        vault_access_rules: Some(to_api_access_rules_chain_substate(
            &mapping_context,
            &vault_access_rules,
        )?),
    })
    .map(Json)
}
