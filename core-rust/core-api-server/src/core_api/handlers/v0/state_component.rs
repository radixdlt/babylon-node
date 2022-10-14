use crate::core_api::*;
use radix_engine::engine::Substate;
use radix_engine::types::{Bech32Decoder, Bech32Encoder, ComponentAddress, RENodeId, SubstateId};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::dump_component;
use state_manager::store::traits::*;

pub(crate) async fn handle_v0_state_component(
    state: Extension<CoreApiState>,
    request: Json<models::V0StateComponentRequest>,
) -> Result<Json<models::V0StateComponentResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_v0_state_component_internal)
}

fn handle_v0_state_component_internal(
    state_manager: &ActualStateManager,
    request: models::V0StateComponentRequest,
) -> Result<models::V0StateComponentResponse, RequestHandlingError> {
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let component_address = extract_component_address(&bech32_decoder, &request.component_address)
        .map_err(|err| err.into_response_error("component_address"))?;

    let component_info_option = read_component_info(state_manager, &component_address)?;
    let component_state_option =
        read_component_state(&bech32_encoder, state_manager, &component_address)?;

    if component_info_option.is_none() && component_state_option.is_none() {
        return Err(not_found_error("Component not found"));
    }
    if !(component_info_option.is_some() && component_state_option.is_some()) {
        return Err(MappingError::InvalidComponentStateEntities {
            message: "Have only one of state and info substates".to_owned(),
        }
        .into());
    }

    let component_dump = dump_component(&state_manager.store, component_address)
        .map_err(|err| server_error(&format!("Error traversing component state: {:?}", err)))?;

    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let owned_vaults = component_dump
        .vaults
        .into_iter()
        .map(|vault| to_api_vault_substate(&bech32_encoder, &vault))
        .collect::<Result<Vec<_>, _>>()?;

    let descendent_ids = component_dump
        .descendents
        .into_iter()
        .filter(|(_, _, depth)| *depth > 0)
        .map(|(parent, node, depth)| map_to_descendent_id(parent, node, depth))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(models::V0StateComponentResponse {
        info: Some(component_info_option.unwrap()),
        state: Some(component_state_option.unwrap()),
        owned_vaults,
        descendent_ids,
    })
}

fn map_to_descendent_id(
    parent: Option<SubstateId>,
    node: RENodeId,
    depth: u32,
) -> Result<models::V0StateComponentDescendentId, MappingError> {
    Ok(models::V0StateComponentDescendentId {
        parent: Box::new(to_api_substate_id(parent.unwrap())?),
        entity_id: Box::new(to_api_entity_id(node)?),
        depth: depth as i32, // Won't go over 100 due to component dumper max depth
    })
}

fn read_component_info(
    state_manager: &ActualStateManager,
    component_address: &ComponentAddress,
) -> Result<Option<models::Substate>, MappingError> {
    let substate_id = SubstateId::ComponentInfo(*component_address);
    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        let bech32_encoder = Bech32Encoder::new(&state_manager.network);
        if let Substate::ComponentInfo(component_info) = output_value.substate {
            return Ok(Some(to_api_component_info_substate(
                &component_info,
                &bech32_encoder,
            )));
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Component info substate was not of the right type".to_owned(),
        });
    }
    Ok(None)
}

fn read_component_state(
    bech32_encoder: &Bech32Encoder,
    state_manager: &ActualStateManager,
    component_address: &ComponentAddress,
) -> Result<Option<models::Substate>, MappingError> {
    let substate_id = SubstateId::ComponentState(*component_address);
    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        if let Substate::ComponentState(component_state) = output_value.substate {
            return Ok(Some(to_api_component_state_substate(
                bech32_encoder,
                &component_state,
            )?));
        }
        return Err(MappingError::MismatchedSubstateId {
            message: "Component state substate was not of the right type".to_owned(),
        });
    }
    Ok(None)
}
