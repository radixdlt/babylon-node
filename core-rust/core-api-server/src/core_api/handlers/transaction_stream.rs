use crate::core_api::*;

use radix_engine::types::hash;
use radix_engine::types::Bech32Encoder;
use radix_engine_interface::core::NetworkDefinition;

use state_manager::jni::state_manager::ActualStateManager;
use state_manager::store::traits::*;
use state_manager::transaction::{LedgerTransaction, ValidatorTransaction};
use state_manager::{
    CommittedTransactionIdentifiers, IntentHash, LedgerTransactionReceipt, SignaturesHash,
    UserPayloadHash,
};
use std::cmp;
use std::collections::HashMap;
use transaction::manifest;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, TransactionIntent, TransactionManifest,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_transaction_stream(
    state: Extension<CoreApiState>,
    request: Json<models::CommittedTransactionsRequest>,
) -> Result<Json<models::CommittedTransactionsResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_transaction_stream_internal)
}

const MAX_TXN_COUNT_PER_REQUEST: u16 = 10000;

#[tracing::instrument(err(Debug), skip(state_manager))]
fn handle_transaction_stream_internal(
    state_manager: &ActualStateManager,
    request: models::CommittedTransactionsRequest,
) -> Result<models::CommittedTransactionsResponse, RequestHandlingError> {
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
        return Err(client_error(&format!(
            "limit must <= {}",
            MAX_TXN_COUNT_PER_REQUEST
        )));
    }

    let state_version_at_limit: u64 = from_state_version
        .checked_add(limit)
        .and_then(|v| v.checked_sub(1))
        .ok_or_else(|| client_error("start_state_version + limit - 1 out of u64 bounds"))?;

    let max_state_version = state_manager.store.max_state_version();

    if max_state_version < from_state_version {
        return Ok(models::CommittedTransactionsResponse {
            from_state_version: to_api_state_version(from_state_version)?,
            count: 0,
            max_ledger_state_version: to_api_state_version(max_state_version)?,
            transactions: vec![],
        });
    }

    let up_to_state_version_inclusive = cmp::min(state_version_at_limit, max_state_version);

    let mut txns = vec![];
    let mut state_version = from_state_version;
    while state_version <= up_to_state_version_inclusive {
        let next_tid = state_manager
            .store
            .get_payload_hash(state_version)
            .ok_or_else(|| {
                server_error(&format!(
                    "A transaction id is missing at state version {}",
                    state_version
                ))
            })?;
        let next_tx = state_manager
            .store
            .get_committed_transaction(&next_tid)
            .ok_or_else(|| {
                server_error(&format!(
                    "A transaction is missing at state version {}",
                    state_version
                ))
            })?;
        txns.push((next_tx, state_version));
        state_version += 1;
    }

    let network = state_manager.network.clone();

    let api_txns = txns
        .into_iter()
        .map(
            |((ledger_transaction, receipt, identifiers), state_version)| {
                if identifiers.state_version != state_version {
                    Err(server_error(&format!(
                        "Loaded state version {} doesn't match its stored state version {}",
                        state_version, identifiers.state_version
                    )))?
                }
                let api_tx = to_api_committed_transaction(
                    &network,
                    ledger_transaction,
                    receipt,
                    identifiers,
                )?;

                Ok(api_tx)
            },
        )
        .collect::<Result<Vec<models::CommittedTransaction>, RequestHandlingError>>()?;

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

    Ok(models::CommittedTransactionsResponse {
        from_state_version: to_api_state_version(start_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        transactions: api_txns,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_committed_transaction(
    network: &NetworkDefinition,
    ledger_transaction: LedgerTransaction,
    receipt: LedgerTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::CommittedTransaction, MappingError> {
    let bech32_encoder = Bech32Encoder::new(network);
    let receipt = to_api_receipt(&bech32_encoder, receipt)?;

    Ok(models::CommittedTransaction {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
        ledger_transaction: Some(to_api_ledger_transaction(&ledger_transaction, network)?),
        receipt: Box::new(receipt),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_ledger_transaction(
    ledger_transaction: &LedgerTransaction,
    network: &NetworkDefinition,
) -> Result<models::LedgerTransaction, MappingError> {
    Ok(match ledger_transaction {
        LedgerTransaction::User(tx) => models::LedgerTransaction::UserLedgerTransaction {
            payload_hex: to_hex(ledger_transaction.create_payload()),
            notarized_transaction: Box::new(to_api_notarized_transaction(tx, network)?),
        },
        LedgerTransaction::Validator(tx) => models::LedgerTransaction::ValidatorLedgerTransaction {
            payload_hex: to_hex(ledger_transaction.create_payload()),
            validator_transaction: Box::new(to_api_validator_transaction(tx, network)?),
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_notarized_transaction(
    tx: &NotarizedTransaction,
    network: &NetworkDefinition,
) -> Result<models::NotarizedTransaction, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let payload = tx.to_bytes();
    let payload_hash = UserPayloadHash::for_transaction(tx);

    Ok(models::NotarizedTransaction {
        hash: to_api_payload_hash(&payload_hash),
        payload_hex: to_hex(payload),
        signed_intent: Box::new(to_api_signed_intent(&tx.signed_intent, network)?),
        notary_signature: Some(to_api_signature(&tx.notary_signature)),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_signed_intent(
    signed_intent: &SignedTransactionIntent,
    network: &NetworkDefinition,
) -> Result<models::SignedTransactionIntent, MappingError> {
    // NOTE: We don't use the .hash() method on the struct impls themselves,
    //       because they use the wrong hash function
    let signed_intent_hash = SignaturesHash::for_signed_intent(signed_intent);

    Ok(models::SignedTransactionIntent {
        hash: to_api_signed_intent_hash(&signed_intent_hash),
        intent: Box::new(to_api_intent(&signed_intent.intent, network)?),
        intent_signatures: signed_intent
            .intent_signatures
            .iter()
            .map(to_api_signature_with_public_key)
            .collect(),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_intent(
    intent: &TransactionIntent,
    network: &NetworkDefinition,
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
            tip_percentage: to_api_u32_as_i64(header.tip_percentage),
        }),
        manifest: Box::new(to_api_manifest(&intent.manifest, network)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_manifest(
    manifest: &TransactionManifest,
    network: &NetworkDefinition,
) -> Result<models::TransactionManifest, MappingError> {
    Ok(models::TransactionManifest {
        instructions: manifest::decompile(&manifest.instructions, network).map_err(|err| {
            MappingError::InvalidManifest {
                message: format!(
                    "Failed to decompile a transaction manifest: {err:?}, instructions: {:?}",
                    &manifest.instructions
                ),
            }
        })?,
        blobs_hex: manifest
            .blobs
            .iter()
            .map(|blob| (to_hex(hash(blob)), to_hex(blob)))
            .collect::<HashMap<String, String>>(),
    })
}

pub fn to_api_validator_transaction(
    validator_transaction: &ValidatorTransaction,
    _network: &NetworkDefinition,
) -> Result<models::ValidatorTransaction, MappingError> {
    Ok(match validator_transaction {
        ValidatorTransaction::EpochUpdate(epoch) => {
            models::ValidatorTransaction::EpochUpdateValidatorTransaction {
                epoch: to_api_epoch(*epoch)?,
            }
        }
    })
}
