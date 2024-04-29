use crate::core_api::*;

use crate::engine_prelude::*;
use state_manager::store::traits::*;
use state_manager::{
    LedgerProof, LedgerProofOrigin, ReadableRocks, StateManagerDatabase, StateVersion,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_stream_proofs(
    state: State<CoreApiState>,
    Json(request): Json<models::StreamProofsRequest>,
) -> Result<Json<models::StreamProofsResponse>, ResponseError<models::StreamProofsErrorDetails>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let filter = request.filter.unwrap_or(Box::new(StreamProofsFilterAny {
        from_state_version: None,
    }));

    let page_size = extract_valid_size(
        request.max_page_size,
        SizeRange {
            min: 1,
            default: 20,
            max: 100,
        },
    )
    .map_err(|err| err.into_response_error("max_page_size"))?;

    let continue_from_state_version =
        extract_continuation_token::<StateVersion>(request.continuation_token)
            .map_err(|err| err.into_response_error("continuation_token"))?;

    let database = state.state_manager.database.snapshot();

    use models::StreamProofsFilter::*;
    let mut proofs_iter = match *filter {
        StreamProofsFilterAny { from_state_version } => iterate_all_proofs(
            &database,
            continue_from_state_version,
            extract_from_state_version(&database, from_state_version)?,
        ),
        StreamProofsFilterNewEpochs { from_epoch } => iterate_end_of_epoch_proofs(
            &database,
            continue_from_state_version,
            extract_from_epoch(&mapping_context, &database, from_epoch)?,
        ),
        StreamProofsFilterProtocolUpdateInitializations { from_state_version } => {
            iterate_protocol_update_initialization_proofs(
                &database,
                continue_from_state_version,
                extract_from_state_version(&database, from_state_version)?,
            )
        }
        StreamProofsFilterProtocolUpdateExecution {
            from_state_version,
            protocol_version,
        } => iterate_protocol_update_execution_proofs(
            &database,
            continue_from_state_version,
            extract_from_state_version(&database, from_state_version)?,
            protocol_version,
        ),
    }?;

    let (page, continuation_token) = to_api_page(
        &mut proofs_iter,
        page_size,
        |proof| handlers::to_api_ledger_proof(&mapping_context, proof),
        |proof| proof.ledger_header.state_version,
    )?;

    Ok(Json(models::StreamProofsResponse {
        page,
        continuation_token,
    }))
}

fn iterate_all_proofs<'a>(
    database: &'a StateManagerDatabase<impl ReadableRocks>,
    continue_from_state_version: Option<StateVersion>,
    from_state_version: StateVersion,
) -> Result<
    Box<dyn Iterator<Item = LedgerProof> + 'a>,
    ResponseError<models::StreamProofsErrorDetails>,
> {
    let start = optional_max(from_state_version, continue_from_state_version);

    Ok(database.get_proof_iter(start))
}

fn iterate_end_of_epoch_proofs<'a>(
    database: &'a StateManagerDatabase<impl ReadableRocks>,
    continue_from_state_version: Option<StateVersion>,
    from_epoch: Epoch,
) -> Result<
    Box<dyn Iterator<Item = LedgerProof> + 'a>,
    ResponseError<models::StreamProofsErrorDetails>,
> {
    let continuation_next_epoch = match continue_from_state_version {
        Some(state_version) => match database.get_proof_iter(state_version).next() {
            Some(proof) => Some(proof.ledger_header.epoch.next().unwrap()),
            None => Err(client_error("continuation_token is not valid"))?,
        },
        None => None,
    };

    let start = optional_max(from_epoch, continuation_next_epoch);

    Ok(database.get_next_epoch_proof_iter(start))
}

fn iterate_protocol_update_initialization_proofs<'a>(
    database: &'a StateManagerDatabase<impl ReadableRocks>,
    continue_from_state_version: Option<StateVersion>,
    from_state_version: StateVersion,
) -> Result<
    Box<dyn Iterator<Item = LedgerProof> + 'a>,
    ResponseError<models::StreamProofsErrorDetails>,
> {
    let start = optional_max(from_state_version, continue_from_state_version);

    Ok(database.get_protocol_update_init_proof_iter(start))
}

fn iterate_protocol_update_execution_proofs<'a>(
    database: &'a StateManagerDatabase<impl ReadableRocks>,
    continue_from_state_version: Option<StateVersion>,
    from_state_version: StateVersion,
    protocol_version: Option<String>,
) -> Result<
    Box<dyn Iterator<Item = LedgerProof> + 'a>,
    ResponseError<models::StreamProofsErrorDetails>,
> {
    let start = optional_max(from_state_version, continue_from_state_version);

    let iter = database.get_protocol_update_execution_proof_iter(start);

    Ok(match protocol_version {
        Some(protocol_version) => Box::new(iter.filter(move |proof| {
            let LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name,
                ..
            } = &proof.origin
            else {
                return false;
            };
            protocol_version_name.as_str() == protocol_version.as_str()
        })),
        None => iter,
    })
}

fn extract_from_state_version(
    database: &StateManagerDatabase<impl ReadableRocks>,
    from_state_version: Option<i64>,
) -> Result<StateVersion, ResponseError<models::StreamProofsErrorDetails>> {
    let Some(from_state_version) = from_state_version else {
        return Ok(StateVersion::pre_genesis());
    };

    let from_state_version = extract_state_version(from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let max_state_version = database.max_state_version();

    // Allow requesting 1 past the end for good UX when streaming
    if from_state_version >= max_state_version.next().unwrap() {
        Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "from_state_version is past the end of the ledger",
            models::StreamProofsErrorDetails::StreamProofsErrorDetailsRequestedStateVersionOutOfBounds {
                max_ledger_state_version: to_api_state_version(max_state_version)?,
            }
        ))?;
    }

    Ok(from_state_version)
}

fn extract_from_epoch(
    mapping_context: &MappingContext,
    database: &StateManagerDatabase<impl ReadableRocks>,
    from_epoch: Option<i64>,
) -> Result<Epoch, ResponseError<models::StreamProofsErrorDetails>> {
    let Some(from_epoch) = from_epoch else {
        return Ok(Epoch::zero());
    };

    let from_epoch =
        extract_epoch(from_epoch).map_err(|err| err.into_response_error("from_epoch"))?;

    let max_new_epoch = database
        .max_completed_epoch()
        .unwrap_or(Epoch::zero())
        .next()
        .unwrap();

    // Allow requesting 1 past the end for good UX when streaming
    if from_epoch >= max_new_epoch.next().unwrap() {
        Err(detailed_error(
            StatusCode::BAD_REQUEST,
            "from_epoch is past the end of the ledger",
            models::StreamProofsErrorDetails::StreamProofsErrorDetailsRequestedEpochOutOfBounds {
                max_ledger_epoch: to_api_epoch(mapping_context, max_new_epoch)?,
            },
        ))?;
    }

    Ok(from_epoch)
}
