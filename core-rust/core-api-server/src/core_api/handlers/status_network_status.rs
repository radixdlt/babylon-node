use crate::prelude::*;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_network_status(
    state: State<CoreApiState>,
    Json(request): Json<models::NetworkStatusRequest>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let database = state.state_manager.database.snapshot();
    let (current_state_version, current_ledger_hashes) = database.get_top_ledger_hashes();
    let current_protocol_version = state
        .state_manager
        .protocol_manager
        .current_protocol_version();
    Ok(Json(models::NetworkStatusResponse {
        pre_genesis_state_identifier: Box::new(to_api_committed_state_identifiers(
            StateVersion::pre_genesis(),
            &LedgerHashes::pre_genesis(),
        )?),
        genesis_epoch_round: database
            .get_first_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_api_epoch_round(
                    &mapping_context,
                    &proof.ledger_header.into(),
                )?))
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
        post_genesis_epoch_round: database
            .get_post_genesis_epoch_proof()
            .map(|epoch_proof: LedgerProof| -> Result<_, MappingError> {
                let next_epoch = epoch_proof.ledger_header.next_epoch.ok_or_else(|| {
                    MappingError::UnexpectedGenesis {
                        message: "Post-genesis epoch proof didn't contain a next_epoch".to_string(),
                    }
                })?;
                Ok(Box::new(models::EpochRound {
                    epoch: to_api_epoch(&mapping_context, next_epoch.epoch)?,
                    round: to_api_round(Round::of(0))?,
                }))
            })
            .transpose()?,
        current_state_identifier: Box::new(to_api_committed_state_identifiers(
            current_state_version,
            &current_ledger_hashes,
        )?),
        current_epoch_round: database
            .get_latest_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_api_epoch_round(
                    &mapping_context,
                    &proof.ledger_header.into(),
                )?))
            })
            .transpose()?,
        current_protocol_version: current_protocol_version.to_string(),
    }))
}

pub fn to_api_epoch_round(
    context: &MappingContext,
    ledger_header: &LedgerStateSummary,
) -> Result<models::EpochRound, MappingError> {
    Ok(models::EpochRound {
        epoch: to_api_epoch(context, ledger_header.epoch)?,
        round: to_api_round(ledger_header.round)?,
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
