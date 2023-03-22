use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_state_account_resource_balance(
    state: State<CoreApiState>,
    request: Json<models::LtsStateAccountResourceBalanceRequest>,
) -> Result<Json<models::LtsStateAccountResourceBalanceResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_state_account_resource_balance_internal,
    )
}

fn handle_rc_state_account_resource_balance_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStateAccountResourceBalanceRequest,
) -> Result<models::LtsStateAccountResourceBalanceResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let _mapping_context = MappingContext::new(&state_manager.network);

    todo!();
}
