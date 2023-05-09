use crate::core_api::*;

use radix_engine::types::hash;

use state_manager::store::traits::*;
use state_manager::transaction::{LedgerTransaction, ValidatorTransaction};
use state_manager::{
    CommittedTransactionIdentifiers, IntentHash, LocalTransactionReceipt, SignaturesHash,
    UserPayloadHash,
};

use radix_engine_interface::data::manifest::manifest_encode;
use std::collections::HashMap;
use transaction::manifest;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, SystemTransaction, TransactionIntent,
    TransactionManifest,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_stream_transactions(
    state: State<CoreApiState>,
    Json(request): Json<models::StreamTransactionsRequest>,
) -> Result<Json<models::StreamTransactionsResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: u64 = request
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

    let limit = limit.try_into().expect("limit out of usize bounds");

    let database = state.database.read();

    let max_state_version = database.max_state_version();

    let mut response = models::StreamTransactionsResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count: MAX_STREAM_COUNT_PER_REQUEST as i32, // placeholder to get a better size aproximation for the header
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        transactions: Vec::new(),
    };

    // Reserve enough for the "header" fields
    let mut current_total_size = response.get_json_size();
    for (ledger_transaction, receipt, identifiers) in database
        .get_committed_transaction_bundle_iter(from_state_version)
        .take(limit)
    {
        let committed_transaction = to_api_committed_transaction(
            &mapping_context,
            ledger_transaction,
            receipt,
            identifiers,
        )?;

        let committed_transaction_size = committed_transaction.get_json_size();
        current_total_size += committed_transaction_size;

        response.transactions.push(committed_transaction);

        if current_total_size > CAP_STREAM_RESPONSE_WHEN_ABOVE_BYTES {
            break;
        }
    }

    let count: i32 = {
        let transaction_count = response.transactions.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    response.count = count;

    Ok(response).map(Json)
}

#[tracing::instrument(skip_all)]
pub fn to_api_committed_transaction(
    context: &MappingContext,
    ledger_transaction: LedgerTransaction,
    receipt: LocalTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::CommittedTransaction, MappingError> {
    let receipt = to_api_receipt(context, receipt)?;

    Ok(models::CommittedTransaction {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
        ledger_transaction: Some(to_api_ledger_transaction(context, &ledger_transaction)?),
        receipt: Box::new(receipt),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_ledger_transaction(
    context: &MappingContext,
    ledger_transaction: &LedgerTransaction,
) -> Result<models::LedgerTransaction, MappingError> {
    Ok(match ledger_transaction {
        LedgerTransaction::User(tx) => models::LedgerTransaction::UserLedgerTransaction {
            payload_hex: to_hex(ledger_transaction.create_payload().map_err(|err| {
                MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding user payload sbor".to_string(),
                }
            })?),
            notarized_transaction: Box::new(to_api_notarized_transaction(context, tx)?),
        },
        LedgerTransaction::Validator(tx) => models::LedgerTransaction::ValidatorLedgerTransaction {
            payload_hex: to_hex(ledger_transaction.create_payload().map_err(|err| {
                MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding validator payload sbor".to_string(),
                }
            })?),
            validator_transaction: Box::new(to_api_validator_transaction(context, tx)?),
        },
        LedgerTransaction::System(tx) => models::LedgerTransaction::SystemLedgerTransaction {
            payload_hex: to_hex(ledger_transaction.create_payload().map_err(|err| {
                MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding system payload sbor".to_string(),
                }
            })?),
            system_transaction: Box::new(to_api_system_transaction(context, tx)?),
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_notarized_transaction(
    context: &MappingContext,
    tx: &NotarizedTransaction,
) -> Result<models::NotarizedTransaction, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let payload = tx.to_bytes().map_err(|err| MappingError::SborEncodeError {
        encode_error: err,
        message: "Error encoding user payload sbor".to_string(),
    })?;
    let payload_hash = UserPayloadHash::for_transaction(tx);

    Ok(models::NotarizedTransaction {
        hash: to_api_payload_hash(&payload_hash),
        payload_hex: to_hex(payload),
        signed_intent: Box::new(to_api_signed_intent(context, &tx.signed_intent)?),
        notary_signature: Some(to_api_signature(&tx.notary_signature)),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_signed_intent(
    context: &MappingContext,
    signed_intent: &SignedTransactionIntent,
) -> Result<models::SignedTransactionIntent, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let signed_intent_hash = SignaturesHash::for_signed_intent(signed_intent);

    Ok(models::SignedTransactionIntent {
        hash: to_api_signed_intent_hash(&signed_intent_hash),
        intent: Box::new(to_api_intent(context, &signed_intent.intent)?),
        intent_signatures: signed_intent
            .intent_signatures
            .iter()
            .map(to_api_signature_with_public_key)
            .collect(),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_intent(
    context: &MappingContext,
    intent: &TransactionIntent,
) -> Result<models::TransactionIntent, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let intent_hash = IntentHash::for_intent(intent);
    let header = &intent.header;

    Ok(models::TransactionIntent {
        hash: to_api_intent_hash(&intent_hash),
        header: Box::new(models::TransactionHeader {
            version: header.version.into(),
            network_id: header.network_id.into(),
            start_epoch_inclusive: to_api_epoch(context, header.start_epoch_inclusive)?,
            end_epoch_exclusive: to_api_epoch(context, header.end_epoch_exclusive)?,
            nonce: to_api_u64_as_string(header.nonce),
            notary_public_key: Some(to_api_public_key(&header.notary_public_key)),
            notary_as_signatory: header.notary_as_signatory,
            cost_unit_limit: to_api_u32_as_i64(header.cost_unit_limit),
            tip_percentage: to_api_u16_as_i32(header.tip_percentage),
        }),
        manifest: Box::new(to_api_manifest(context, &intent.manifest)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_manifest(
    context: &MappingContext,
    manifest: &TransactionManifest,
) -> Result<models::TransactionManifest, MappingError> {
    Ok(models::TransactionManifest {
        instructions: manifest::decompile(&manifest.instructions, &context.network_definition)
            .map_err(|err| MappingError::InvalidManifest {
                message: format!(
                    "Failed to decompile a transaction manifest: {err:?}, instructions: {:?}",
                    &manifest.instructions
                ),
            })?,
        blobs_hex: manifest
            .blobs
            .iter()
            .map(|blob| (to_hex(hash(blob)), to_hex(blob)))
            .collect::<HashMap<String, String>>(),
    })
}

pub fn to_api_validator_transaction(
    context: &MappingContext,
    validator_transaction: &ValidatorTransaction,
) -> Result<models::ValidatorTransaction, MappingError> {
    Ok(match validator_transaction {
        ValidatorTransaction::RoundUpdate {
            proposer_timestamp_ms,
            consensus_epoch,
            round_in_epoch,
        } => models::ValidatorTransaction::RoundUpdateValidatorTransaction {
            proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(
                *proposer_timestamp_ms,
            )?),
            consensus_epoch: to_api_epoch(context, *consensus_epoch)?,
            round_in_epoch: to_api_round(*round_in_epoch)?,
        },
    })
}

pub fn to_api_system_transaction(
    _context: &MappingContext,
    system_transaction: &SystemTransaction,
) -> Result<models::SystemTransaction, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let payload =
        manifest_encode(system_transaction).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Error encoding user system sbor".to_string(),
        })?;
    Ok(models::SystemTransaction {
        payload_hex: to_hex(payload),
    })
}
