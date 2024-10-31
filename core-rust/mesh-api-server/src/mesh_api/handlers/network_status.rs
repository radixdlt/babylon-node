use crate::prelude::*;

pub(crate) async fn handle_network_status(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkIdentifier>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);

    let database = state.state_manager.database.snapshot();

    let timestamp_substate =
        read_mandatory_main_field_substate::<ConsensusManagerProposerMilliTimestampFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::ProposerMilliTimestamp.into(),
        )?
        .into_payload()
        .fully_update_and_into_latest_version();

    Ok(Json(models::NetworkStatusResponse {
        current_block_identifier: database
            .get_latest_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_block_identifier(
                    &mapping_context,
                    &proof.ledger_header.into(),
                )?))
            })
            .unwrap_or_else(|| Err(MappingError::ProofNotFound))?,

        current_block_timestamp: timestamp_substate.epoch_milli,
        genesis_block_identifier: database
            .get_first_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(to_block_identifier(
                    &mapping_context,
                    &proof.ledger_header.into(),
                )?))
            })
            .unwrap_or_else(|| Err(MappingError::ProofNotFound))?,

        oldest_block_identifier: None,
        // TODO crate::mesh_api::generated::models::SyncStatus
        sync_status: None,
        // TODO crate::mesh_api::generated::models::Peer
        peers: None,
    }))
}

pub fn to_block_identifier(
    context: &MappingContext,
    ledger_header: &LedgerStateSummary,
) -> Result<models::BlockIdentifier, MappingError> {
    Ok(models::BlockIdentifier {
        index: to_api_epoch(context, ledger_header.epoch)?,
        hash: format!("round {}", to_api_round(ledger_header.round)?),
    })
}
