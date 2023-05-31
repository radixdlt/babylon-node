use crate::core_api::*;

use state_manager::query::TransactionIdentifierLoader;
use state_manager::store::traits::QueryableTransactionStore;
use state_manager::CommitBasedIdentifiers;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_network_status(
    state: State<CoreApiState>,
    Json(request): Json<models::NetworkStatusRequest>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let database = state.database.read();
    Ok(models::NetworkStatusResponse {
        post_genesis_state_identifier: database
            .get_committed_transaction_identifiers(1)
            .map(|identifiers| -> Result<_, MappingError> {
                Ok(Box::new(to_api_state_identifiers(identifiers.at_commit)?))
            })
            .transpose()?,
        current_state_identifier: Box::new(to_api_state_identifiers(
            database.get_top_commit_identifiers(),
        )?),
        pre_genesis_state_identifier: Box::new(to_api_state_identifiers(
            CommitBasedIdentifiers::pre_genesis(),
        )?),
    })
    .map(Json)
}

pub fn to_api_state_identifiers(
    identifiers: CommitBasedIdentifiers,
) -> Result<models::CommittedStateIdentifier, MappingError> {
    Ok(models::CommittedStateIdentifier {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
    })
}
