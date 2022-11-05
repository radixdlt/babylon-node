use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::TransactionIdentifierLoader;
use state_manager::CommittedTransactionIdentifiers;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_network_status(
    state: Extension<CoreApiState>,
    request: Json<models::NetworkStatusRequest>,
) -> Result<Json<models::NetworkStatusResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_network_status_internal)
}

pub(crate) fn handle_network_status_internal(
    state_manager: &ActualStateManager,
    request: models::NetworkStatusRequest,
) -> Result<models::NetworkStatusResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let pre_genesis =
        to_api_committed_state_identifier(CommittedTransactionIdentifiers::pre_genesis())?;

    Ok(models::NetworkStatusResponse {
        post_genesis_state_identifier: state_manager
            .store
            .get_transaction_identifiers(1)
            .map(|identifiers| -> Result<_, MappingError> {
                Ok(Box::new(to_api_committed_state_identifier(identifiers)?))
            })
            .transpose()?,
        current_state_identifier: Box::new(
            state_manager
                .store
                .get_top_of_ledger_transaction_identifiers()
                .map(|identifiers| -> Result<_, MappingError> {
                    to_api_committed_state_identifier(identifiers)
                })
                .transpose()?
                .unwrap_or_else(|| pre_genesis.clone()),
        ),
        pre_genesis_state_identifier: Box::new(pre_genesis),
    })
}

pub fn to_api_committed_state_identifier(
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::CommittedStateIdentifier, MappingError> {
    Ok(models::CommittedStateIdentifier {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
    })
}
