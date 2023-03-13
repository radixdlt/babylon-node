use crate::core_api::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::query::TransactionIdentifierLoader;
use state_manager::store::traits::QueryableTransactionStore;
use state_manager::CommittedTransactionIdentifiers;

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_status_network_status(
    state: Extension<CoreApiState>,
    request: Json<models::NetworkStatusRequest>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_status_network_status_internal)
}

pub(crate) fn handle_status_network_status_internal(
    state_manager: &ActualStateManager,
    request: models::NetworkStatusRequest,
) -> Result<models::NetworkStatusResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;
    Ok(models::NetworkStatusResponse {
        post_genesis_state_identifier: state_manager
            .store()
            .get_committed_transaction_identifiers(1)
            .map(|identifiers| -> Result<_, MappingError> {
                Ok(Box::new(to_api_committed_state_identifier(identifiers)?))
            })
            .transpose()?,
        current_state_identifier: Box::new(to_api_committed_state_identifier(
            state_manager.store().get_top_transaction_identifiers(),
        )?),
        pre_genesis_state_identifier: Box::new(to_api_committed_state_identifier(
            CommittedTransactionIdentifiers::pre_genesis(),
        )?),
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
