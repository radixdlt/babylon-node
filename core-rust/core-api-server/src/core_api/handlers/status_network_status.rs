use crate::core_api::*;

use state_manager::query::TransactionIdentifierLoader;
use state_manager::store::traits::*;
use state_manager::{LedgerHashes, LedgerProof, StateVersion};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_network_status(
    state: State<CoreApiState>,
    Json(request): Json<models::NetworkStatusRequest>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let database = state.database.read();
    let (current_state_version, current_ledger_hashes) = database.get_top_ledger_hashes();
    Ok(models::NetworkStatusResponse {
        pre_genesis_state_identifier: Box::new(to_api_committed_state_identifiers(
            StateVersion::pre_genesis(),
            &LedgerHashes::pre_genesis(),
        )?),
        genesis_epoch_round: database
            .get_first_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_api_epoch_round(&mapping_context, &proof)?))
            })
            .transpose()?,
        post_genesis_state_identifier: database
            .get_post_genesis_epoch_proof()
            .and_then(|epoch_proof| {
                let state_version = epoch_proof.ledger_header.state_version;
                database
                    .get_committed_ledger_hashes(state_version)
                    .map(|ledger_hashes| (state_version, ledger_hashes))
            })
            .map(
                |(state_version, ledger_hashes)| -> Result<_, MappingError> {
                    Ok(Box::new(to_api_committed_state_identifiers(
                        state_version,
                        &ledger_hashes,
                    )?))
                },
            )
            .transpose()?,
        current_state_identifier: Some(Box::new(to_api_committed_state_identifiers(
            current_state_version,
            &current_ledger_hashes,
        )?)),
        current_epoch_round: database
            .get_last_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_api_epoch_round(&mapping_context, &proof)?))
            })
            .transpose()?,
        current_protocol_version: "babylon".to_string(),
    })
    .map(Json)
}

pub fn to_api_epoch_round(
    context: &MappingContext,
    ledger_proof: &LedgerProof,
) -> Result<models::EpochRound, MappingError> {
    Ok(models::EpochRound {
        epoch: to_api_epoch(context, ledger_proof.ledger_header.epoch)?,
        round: to_api_round(ledger_proof.ledger_header.round)?,
    })
}

pub fn to_api_committed_state_identifiers(
    state_version: StateVersion,
    ledger_hashes: &LedgerHashes,
) -> Result<models::CommittedStateIdentifier, MappingError> {
    Ok(models::CommittedStateIdentifier {
        state_version: to_api_state_version(state_version)?,
        state_tree_hash: to_api_state_tree_hash(&ledger_hashes.state_root),
        transaction_tree_hash: to_api_transaction_tree_hash(&ledger_hashes.transaction_root),
        receipt_tree_hash: to_api_receipt_tree_hash(&ledger_hashes.receipt_root),
    })
}
