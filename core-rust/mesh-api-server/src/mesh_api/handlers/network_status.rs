use crate::prelude::*;

pub(crate) async fn handle_network_status(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkIdentifier>,
) -> Result<Json<models::NetworkStatusResponse>, ResponseError> {
    assert_matching_network(&request.network, &state.network)?;

    let database = state.state_manager.database.snapshot();

    let timestamp_substate =
        read_mandatory_main_field_substate::<ConsensusManagerProposerMilliTimestampFieldPayload>(
            database.deref(),
            CONSENSUS_MANAGER.as_node_id(),
            &ConsensusManagerField::ProposerMilliTimestamp.into(),
        )?
        .into_payload()
        .fully_update_and_into_latest_version();

    let genesis_block_identifier = database
        .get_first_proof()
        .map(|proof| -> Result<_, MappingError> {
            Ok(Box::new(ledger_header_to_block_identifier(
                &proof.ledger_header.into(),
            )?))
        })
        .unwrap_or_else(|| Err(MappingError::ProofNotFound))?;

    Ok(Json(models::NetworkStatusResponse {
        current_block_identifier: database
            .get_latest_proof()
            .map(|proof| -> Result<_, MappingError> {
                Ok(Box::new(ledger_header_to_block_identifier(
                    &proof.ledger_header.into(),
                )?))
            })
            .unwrap_or_else(|| Err(MappingError::ProofNotFound))?,

        current_block_timestamp: timestamp_substate.epoch_milli,
        oldest_block_identifier: Some(genesis_block_identifier.clone()),
        genesis_block_identifier,
        // sync_status not required.
        // Comparing the timestamp of the most recent blocks with current time is fine.
        sync_status: None,
        // This info is in the Java System API. Hard to get it. Setting empty vector for now.
        peers: Some(vec![]),
    }))
}
