use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_state_account_all_resource_balances(
    state: State<CoreApiState>,
    request: Json<models::RcStateAccountAllResourceBalancesRequest>,
) -> Result<Json<models::RcStateAccountAllResourceBalancesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_state_account_all_resource_balances_internal,
    )
}

fn handle_rc_state_account_all_resource_balances_internal(
    state_manager: &ActualStateManager,
    request: models::RcStateAccountAllResourceBalancesRequest,
) -> Result<models::RcStateAccountAllResourceBalancesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    let _mapping_context = MappingContext::new(&state_manager.network);

    todo!();
}
