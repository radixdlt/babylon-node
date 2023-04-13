use crate::core_api::*;
use radix_engine::types::Address;
use state_manager::{
    jni::state_manager::ActualStateManager,
    store::traits::{
        extensions::{
            AccountChangeIndexExtension, AccountChangeIndexStoreCapability,
            QueryableStoreIndexExtension,
        },
        QueryableProofStore, QueryableTransactionStore,
    },
};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_stream_account_transaction_outcomes(
    state: State<CoreApiState>,
    request: Json<models::LtsStreamAccountTransactionOutcomesRequest>,
) -> Result<Json<models::LtsStreamAccountTransactionOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_lts_stream_account_transaction_outcomes_internal,
    )
}

fn handle_lts_stream_account_transaction_outcomes_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStreamAccountTransactionOutcomesRequest,
) -> Result<models::LtsStreamAccountTransactionOutcomesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    if !request.account_address.starts_with("account_") {
        return Err(client_error(
            "Only component addresses starting with account_ work with this endpoint.",
        ));
    }

    let mapping_context = MappingContext::new(&state_manager.network);
    let extraction_context = ExtractionContext::new(&state_manager.network);

    let component_address =
        extract_component_address(&extraction_context, &request.account_address)
            .map_err(|err| err.into_response_error("account_address"))?;

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: usize = request
        .limit
        .try_into()
        .map_err(|_| client_error("limit cannot be negative"))?;

    if limit == 0 {
        return Err(client_error("limit must be positive"));
    }

    if limit > MAX_STREAM_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "limit must <= {MAX_STREAM_COUNT_PER_REQUEST}"
        )));
    }

    let query_account_change_index = state_manager.store().query_account_change_index();
    if !query_account_change_index.is_enabled() {
        return Err(server_error(
            "This endpoint requires that the AccountChangeIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config and restart the node. \
            Please note the index catchup build will take some time.",
        ));
    }

    let max_state_version = state_manager.store().max_state_version();

    let state_versions = query_account_change_index.get_state_versions(
        Address::Component(component_address),
        from_state_version,
        limit,
    );

    let committed_transaction_outcomes = state_versions
        .iter()
        .map(|state_version| {
            Ok(to_api_lts_committed_transaction_outcome(
                &mapping_context,
                state_manager
                    .store()
                    .get_committed_transaction_receipt(*state_version)
                    .expect("Transaction receipt index corrupted"),
                state_manager
                    .store()
                    .get_committed_transaction_identifiers(*state_version)
                    .expect("Transaction identifiers index corrupted"),
            )?)
        })
        .collect::<Result<Vec<models::LtsCommittedTransactionOutcome>, ResponseError<()>>>()?;

    Ok(models::LtsStreamAccountTransactionOutcomesResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count: state_versions
            .len()
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_outcomes,
    })
}
