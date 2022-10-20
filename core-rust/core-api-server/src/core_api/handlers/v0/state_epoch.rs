use crate::core_api::*;
use radix_engine::model::PersistedSubstate;
use radix_engine::types::SubstateId;
use scrypto::constants::SYS_SYSTEM_COMPONENT;
use scrypto::engine::types::{GlobalAddress, GlobalOffset, RENodeId, SubstateOffset, SystemOffset};
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_v0_state_epoch(
    state: Extension<CoreApiState>,
) -> Result<Json<models::V0StateEpochResponse>, RequestHandlingError> {
    core_api_read_handler(state, Json(()), handle_v0_state_epoch_internal)
}

fn handle_v0_state_epoch_internal(
    state_manager: &ActualStateManager,
    _request: (),
) -> Result<models::V0StateEpochResponse, RequestHandlingError> {
    let substate_id = SubstateId(
        RENodeId::Global(GlobalAddress::Component(SYS_SYSTEM_COMPONENT)),
        SubstateOffset::Global(GlobalOffset::Global),
    );
    if let Some(output_value) = state_manager.store.get_substate(&substate_id) {
        if let PersistedSubstate::GlobalRENode(global) = output_value.substate {
            let derefed = global.node_deref();
            let substate_id = SubstateId(derefed, SubstateOffset::System(SystemOffset::System));
            let info_output = state_manager.store.get_substate(&substate_id);
            if let Some(PersistedSubstate::System(system)) = info_output.map(|o| o.substate) {
                return Ok(models::V0StateEpochResponse {
                    epoch: to_api_epoch(system.epoch)?,
                });
            }
        }
    }
    Err(MappingError::MismatchedSubstateId {
        message: "System Substate not found".to_owned(),
    }
    .into())
}
