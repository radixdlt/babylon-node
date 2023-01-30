use crate::core_api::*;

use radix_engine::types::hash;

use radix_engine_interface::data::scrypto_encode;

use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;
use state_manager::transaction::{LedgerTransaction, ValidatorTransaction};
use state_manager::{
    CommittedTransactionIdentifiers, IntentHash, LedgerTransactionReceipt, SignaturesHash,
    UserPayloadHash,
};

use std::collections::HashMap;
use transaction::manifest;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, SystemTransaction, TransactionIntent,
    TransactionManifest,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_stream_transactions(
    state: Extension<CoreApiState>,
    request: Json<models::StreamTransactionsRequest>,
) -> Result<Json<models::StreamTransactionsResponse>, ResponseError<()>> {
    core_api_read_handler(state, request, handle_stream_transactions_internal)
}

const MAX_TXN_COUNT_PER_REQUEST: u16 = 10000;

#[tracing::instrument(err(Debug), skip(state_manager))]
fn handle_stream_transactions_internal(
    state_manager: &ActualStateManager,
    request: models::StreamTransactionsRequest,
) -> Result<models::StreamTransactionsResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| client_error("limit cannot be negative"))?;

    if limit == 0 {
        return Err(client_error("limit must be positive"));
    }

    if limit > MAX_TXN_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "limit must <= {}",
            MAX_TXN_COUNT_PER_REQUEST
        )));
    }

    let max_state_version = state_manager.staged_store.root.max_state_version();

    let txns = state_manager
        .staged_store
        .root
        .get_committed_transaction_bundles(
            from_state_version,
            limit.try_into().expect("limit out of usize bounds"),
        );

    let mapping_context = MappingContext::new(&state_manager.network);

    let api_txns = txns
        .into_iter()
        .map(|(ledger_transaction, receipt, identifiers)| {
            Ok(to_api_committed_transaction(
                &mapping_context,
                ledger_transaction,
                receipt,
                identifiers,
            )?)
        })
        .collect::<Result<Vec<models::CommittedTransaction>, ResponseError<()>>>()?;

    let start_state_version = if api_txns.is_empty() {
        0
    } else {
        from_state_version
    };

    let count: i32 = {
        let transaction_count = api_txns.len();
        if transaction_count > MAX_TXN_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    Ok(models::StreamTransactionsResponse {
        from_state_version: to_api_state_version(start_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        transactions: api_txns,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_committed_transaction(
    context: &MappingContext,
    ledger_transaction: LedgerTransaction,
    receipt: LedgerTransactionReceipt,
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
            start_epoch_inclusive: to_api_epoch(header.start_epoch_inclusive)?,
            end_epoch_exclusive: to_api_epoch(header.end_epoch_exclusive)?,
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
    _context: &MappingContext,
    validator_transaction: &ValidatorTransaction,
) -> Result<models::ValidatorTransaction, MappingError> {
    Ok(match validator_transaction {
        ValidatorTransaction::RoundUpdate {
            proposer_timestamp_ms,
            consensus_epoch,
            round_in_epoch,
        } => models::ValidatorTransaction::TimeUpdateValidatorTransaction {
            proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(
                *proposer_timestamp_ms,
            )?),
            consensus_epoch: to_api_epoch(*consensus_epoch)?,
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
        scrypto_encode(system_transaction).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Error encoding user system sbor".to_string(),
        })?;
    Ok(models::SystemTransaction {
        payload_hex: to_hex(payload),
    })
}
