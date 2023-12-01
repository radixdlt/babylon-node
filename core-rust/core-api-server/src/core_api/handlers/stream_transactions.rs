use radix_engine::track::{NodeStateUpdates, PartitionStateUpdates};
use std::iter;

use crate::core_api::*;

use radix_engine::types::hash;
use radix_engine_store_interface::interface::DatabaseUpdate;

use state_manager::store::{traits::*, StateManagerDatabase};
use state_manager::transaction::*;
use state_manager::{
    CommittedTransactionIdentifiers, LedgerHeader, LedgerProof, LocalTransactionReceipt,
    StateVersion,
};

use transaction::manifest;
use transaction::prelude::*;

use super::to_api_committed_state_identifiers;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_stream_transactions(
    state: State<CoreApiState>,
    Json(request): Json<models::StreamTransactionsRequest>,
) -> Result<
    Json<models::StreamTransactionsResponse>,
    ResponseError<models::StreamTransactionsErrorDetails>,
> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new_for_transaction_stream(&state.network)
        .with_sbor_formats(&request.sbor_format_options)
        .with_transaction_formats(&request.transaction_format_options)
        .with_substate_formats(&request.substate_format_options);

    let from_state_version = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| client_error("limit cannot be negative"))?;

    if limit == 0 {
        return Err(client_error("limit must be positive"));
    }

    if limit > MAX_BATCH_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "limit must <= {MAX_BATCH_COUNT_PER_REQUEST}"
        )));
    }

    let limit = limit.try_into().expect("limit out of usize bounds");

    let database = state.state_manager.database.read_current();

    if !database.is_local_transaction_execution_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the LocalTransactionExecutionIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config, wipe ledger and restart. \
            Please note the resync will take a while.",
        ));
    }

    let max_ledger_state_version = to_api_state_version(database.max_state_version())?;

    let previous_state_identifiers = match from_state_version.previous() {
        Ok(previous_state_version) => {
            if previous_state_version.number() == 0 {
                None
            } else {
                let identifiers = database
                    .get_committed_transaction_identifiers(previous_state_version)
                    .ok_or_else(|| detailed_error(
                        StatusCode::BAD_REQUEST,
                        "The requested state version is out of bounds",
                        models::StreamTransactionsErrorDetails::RequestedStateVersionOutOfBoundsErrorDetails {
                            max_ledger_state_version
                        }
                    ))?;
                Some(Box::new(to_api_committed_state_identifiers(
                    previous_state_version,
                    &identifiers.resultant_ledger_hashes,
                )?))
            }
        }
        Err(_) => None,
    };

    let mut response = models::StreamTransactionsResponse {
        previous_state_identifiers,
        from_state_version: to_api_state_version(from_state_version)?,
        count: MAX_BATCH_COUNT_PER_REQUEST as i32, // placeholder to get a better size approximation for the header
        max_ledger_state_version,
        transactions: Vec::new(),
        proofs: None,
    };

    let mut proofs = Vec::new();

    // Reserve enough for the "header" fields
    let mut current_total_size = response.get_json_size();
    let bundles_iter = database.get_committed_transaction_bundle_iter(from_state_version);
    let proofs_iter = if request.include_proofs.is_some_and(|value| value) {
        database.get_proof_iter(from_state_version)
    } else {
        Box::new(iter::empty())
    };
    let transactions_and_proofs_iter = TransactionAndProofIterator::new(bundles_iter, proofs_iter);
    for (bundle, maybe_proof) in transactions_and_proofs_iter.take(limit) {
        let CommittedTransactionBundle {
            state_version,
            raw,
            receipt,
            identifiers,
        } = bundle;
        let model = LedgerTransaction::from_raw(&raw).map_err(|error| {
            MappingError::CouldNotDecodeTransaction {
                state_version,
                error,
            }
        })?;
        let committed_transaction = to_api_committed_transaction(
            Some(&database),
            &mapping_context,
            state_version,
            raw,
            model,
            receipt,
            identifiers,
        )?;
        current_total_size += committed_transaction.get_json_size();
        response.transactions.push(committed_transaction);

        if let Some(proof) = maybe_proof {
            let api_proof = to_api_ledger_proof(&mapping_context, proof)?;
            current_total_size += api_proof.get_json_size();
            proofs.push(api_proof);
        }

        if current_total_size > CAP_BATCH_RESPONSE_WHEN_ABOVE_BYTES {
            break;
        }
    }

    if request.include_proofs.is_some_and(|value| value) {
        response.proofs = Some(proofs);
    }

    let count: i32 = {
        let transaction_count = response.transactions.len();
        if transaction_count > MAX_BATCH_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    response.count = count;

    Ok(response).map(Json)
}

pub fn to_api_ledger_proof(
    mapping_context: &MappingContext,
    proof: LedgerProof,
) -> Result<models::LedgerProof, MappingError> {
    let timestamped_signatures = proof
        .timestamped_signatures
        .into_iter()
        .map(|timestamped_validator_signature| {
            Ok(models::TimestampedValidatorSignature {
                validator_key: Box::new(to_api_ecdsa_secp256k1_public_key(
                    &timestamped_validator_signature.key,
                )),
                validator_address: to_api_component_address(
                    mapping_context,
                    &timestamped_validator_signature.validator_address,
                )?,
                timestamp_ms: timestamped_validator_signature.timestamp_ms,
                signature: Box::new(models::EcdsaSecp256k1Signature {
                    key_type: models::PublicKeyType::EcdsaSecp256k1,
                    signature_hex: to_hex(timestamped_validator_signature.signature.to_vec()),
                }),
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(models::LedgerProof {
        opaque_hash: to_api_hash(&proof.opaque),
        ledger_header: Box::new(to_api_ledger_header(mapping_context, proof.ledger_header)?),
        timestamped_signatures,
    })
}

pub fn to_api_hash(hash: &Hash) -> String {
    to_hex(hash)
}

pub fn to_api_ledger_header(
    mapping_context: &MappingContext,
    ledger_header: LedgerHeader,
) -> Result<models::LedgerHeader, MappingError> {
    let next_epoch = match ledger_header.next_epoch {
        Some(next_epoch) => {
            let validators = next_epoch
                .validator_set
                .into_iter()
                .map(|active_validator_info| {
                    Ok(models::ActiveValidator {
                        address: to_api_component_address(
                            mapping_context,
                            &active_validator_info.address,
                        )?,
                        key: Box::new(to_api_ecdsa_secp256k1_public_key(
                            &active_validator_info.key,
                        )),
                        stake: to_api_decimal(&active_validator_info.stake),
                    })
                })
                .collect::<Result<_, _>>()?;
            Some(Box::new(models::NextEpoch {
                epoch: to_api_epoch(mapping_context, next_epoch.epoch)?,
                validators,
            }))
        }
        None => None,
    };
    Ok(models::LedgerHeader {
        epoch: to_api_epoch(mapping_context, ledger_header.epoch)?,
        round: to_api_round(ledger_header.round)?,
        state_version: to_api_state_version(ledger_header.state_version)?,
        hashes: Box::new(models::LedgerHashes {
            state_tree_hash: to_api_state_tree_hash(&ledger_header.hashes.state_root),
            transaction_tree_hash: to_api_transaction_tree_hash(
                &ledger_header.hashes.transaction_root,
            ),
            receipt_tree_hash: to_api_receipt_tree_hash(&ledger_header.hashes.receipt_root),
        }),
        consensus_parent_round_timestamp_ms: ledger_header.consensus_parent_round_timestamp_ms,
        proposer_timestamp_ms: ledger_header.proposer_timestamp_ms,
        next_epoch,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_committed_transaction(
    database: Option<&StateManagerDatabase>,
    context: &MappingContext,
    state_version: StateVersion,
    raw_ledger_transaction: RawLedgerTransaction,
    ledger_transaction: LedgerTransaction,
    receipt: LocalTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::CommittedTransaction, MappingError> {
    let receipt = to_api_receipt(database, context, receipt)?;

    Ok(models::CommittedTransaction {
        resultant_state_identifiers: Box::new(to_api_committed_state_identifiers(
            state_version,
            &identifiers.resultant_ledger_hashes,
        )?),
        ledger_transaction: Some(to_api_ledger_transaction(
            context,
            &raw_ledger_transaction,
            &ledger_transaction,
            &identifiers.payload,
        )?),
        proposer_timestamp_ms: identifiers.proposer_timestamp_ms,
        receipt: Box::new(receipt),
    })
}

pub fn to_api_ledger_transaction(
    context: &MappingContext,
    raw_ledger_transaction: &RawLedgerTransaction,
    ledger_transaction: &LedgerTransaction,
    payload_identifiers: &PayloadIdentifiers,
) -> Result<models::LedgerTransaction, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_ledger {
        Some(to_hex(&raw_ledger_transaction.0))
    } else {
        None
    };

    Ok(match ledger_transaction {
        LedgerTransaction::UserV1(tx) => {
            let user_identifiers = payload_identifiers.typed.user().ok_or_else(|| {
                MappingError::MismatchedTransactionIdentifiers {
                    message: "Transaction hashes for notarized transaction were not user"
                        .to_string(),
                }
            })?;
            models::LedgerTransaction::UserLedgerTransaction {
                payload_hex,
                notarized_transaction: Box::new(to_api_notarized_transaction(
                    context,
                    tx,
                    user_identifiers.intent_hash,
                    user_identifiers.signed_intent_hash,
                    user_identifiers.notarized_transaction_hash,
                )?),
            }
        }
        LedgerTransaction::RoundUpdateV1(tx) => {
            models::LedgerTransaction::RoundUpdateLedgerTransaction {
                payload_hex,
                round_update_transaction: Box::new(to_api_round_update_transaction(context, tx)?),
            }
        }
        LedgerTransaction::Genesis(tx) => match tx.as_ref() {
            GenesisTransaction::Flash => models::LedgerTransaction::GenesisLedgerTransaction {
                payload_hex,
                is_flash: true,
                system_transaction: None,
            },
            GenesisTransaction::Transaction(tx) => {
                models::LedgerTransaction::GenesisLedgerTransaction {
                    payload_hex,
                    is_flash: false,
                    system_transaction: Some(Box::new(to_api_system_transaction(context, tx)?)),
                }
            }
        },
        LedgerTransaction::FlashV1(tx) => models::LedgerTransaction::FlashLedgerTransaction {
            payload_hex,
            flash_transaction: Box::new(to_api_flash_transaction(context, tx)?),
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_notarized_transaction(
    context: &MappingContext,
    notarized: &NotarizedTransactionV1,
    intent_hash: &IntentHash,
    signed_intent_hash: &SignedIntentHash,
    notarized_transaction_hash: &NotarizedTransactionHash,
) -> Result<models::NotarizedTransaction, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_notarized {
        Some(to_hex(notarized.to_payload_bytes().map_err(|err| {
            MappingError::SborEncodeError {
                encode_error: err,
                message: "Error encoding user payload sbor".to_string(),
            }
        })?))
    } else {
        None
    };

    Ok(models::NotarizedTransaction {
        hash: to_api_notarized_transaction_hash(notarized_transaction_hash),
        hash_bech32m: to_api_hash_bech32m(context, notarized_transaction_hash)?,
        payload_hex,
        signed_intent: Box::new(to_api_signed_intent(
            context,
            &notarized.signed_intent,
            intent_hash,
            signed_intent_hash,
        )?),
        notary_signature: Some(to_api_signature(&notarized.notary_signature.0)),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_signed_intent(
    context: &MappingContext,
    signed_intent: &SignedIntentV1,
    intent_hash: &IntentHash,
    signed_intent_hash: &SignedIntentHash,
) -> Result<models::SignedTransactionIntent, MappingError> {
    Ok(models::SignedTransactionIntent {
        hash: to_api_signed_intent_hash(signed_intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, signed_intent_hash)?,
        intent: Box::new(to_api_intent(context, &signed_intent.intent, intent_hash)?),
        intent_signatures: signed_intent
            .intent_signatures
            .signatures
            .iter()
            .map(|s| to_api_signature_with_public_key(&s.0))
            .collect(),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_intent(
    context: &MappingContext,
    intent: &IntentV1,
    intent_hash: &IntentHash,
) -> Result<models::TransactionIntent, MappingError> {
    let IntentV1 {
        header,
        instructions,
        blobs,
        message,
    } = intent;

    let header = Box::new(models::TransactionHeader {
        network_id: header.network_id.into(),
        start_epoch_inclusive: to_api_epoch(context, header.start_epoch_inclusive)?,
        end_epoch_exclusive: to_api_epoch(context, header.end_epoch_exclusive)?,
        nonce: to_api_u32_as_i64(header.nonce),
        notary_public_key: Some(to_api_public_key(&header.notary_public_key)),
        notary_is_signatory: header.notary_is_signatory,
        tip_percentage: to_api_u16_as_i32(header.tip_percentage),
    });

    let instructions = if context.transaction_options.include_manifest {
        Some(
            manifest::decompile(&instructions.0, &context.network_definition).map_err(|err| {
                MappingError::InvalidManifest {
                    message: format!(
                        "Failed to decompile a transaction manifest: {err:?}, instructions: {:?}",
                        &instructions
                    ),
                }
            })?,
        )
    } else {
        None
    };

    let blobs_hex = if context.transaction_options.include_blobs {
        Some(
            blobs
                .blobs
                .iter()
                .map(|blob| (to_hex(hash(&blob.0)), to_hex(&blob.0)))
                .collect(),
        )
    } else {
        None
    };

    let message = if context.transaction_options.include_message {
        match message {
            MessageV1::None => None,
            MessageV1::Plaintext(plaintext) => Some(to_api_plaintext_message(context, plaintext)?),
            MessageV1::Encrypted(encrypted) => Some(to_api_encrypted_message(context, encrypted)?),
        }
        .map(Box::new)
    } else {
        None
    };

    Ok(models::TransactionIntent {
        hash: to_api_intent_hash(intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, intent_hash)?,
        header,
        instructions,
        blobs_hex,
        message,
    })
}

fn to_api_plaintext_message(
    context: &MappingContext,
    plaintext: &PlaintextMessageV1,
) -> Result<models::TransactionMessage, MappingError> {
    Ok(models::TransactionMessage::PlaintextTransactionMessage {
        mime_type: plaintext.mime_type.clone(),
        content: Box::new(to_api_plaintext_message_content(
            context,
            &plaintext.message,
        )?),
    })
}

fn to_api_plaintext_message_content(
    _context: &MappingContext,
    content: &MessageContentsV1,
) -> Result<models::PlaintextMessageContent, MappingError> {
    Ok(match content {
        MessageContentsV1::String(string) => {
            models::PlaintextMessageContent::StringPlaintextMessageContent {
                value: string.clone(),
            }
        }
        MessageContentsV1::Bytes(bytes) => {
            models::PlaintextMessageContent::BinaryPlaintextMessageContent {
                value_hex: to_hex(bytes),
            }
        }
    })
}

fn to_api_encrypted_message(
    context: &MappingContext,
    encrypted: &EncryptedMessageV1,
) -> Result<models::TransactionMessage, MappingError> {
    Ok(models::TransactionMessage::EncryptedTransactionMessage {
        encrypted_hex: to_hex(&encrypted.encrypted.0),
        curve_decryptor_sets: encrypted
            .decryptors_by_curve
            .values()
            .map(|decryptor_set| to_api_decryptor_set(context, decryptor_set))
            .collect::<Result<_, _>>()?,
    })
}

fn to_api_decryptor_set(
    context: &MappingContext,
    decryptor_set: &DecryptorsByCurve,
) -> Result<models::EncryptedMessageCurveDecryptorSet, MappingError> {
    let (dh_ephemeral_public_key, decryptors) = match decryptor_set {
        DecryptorsByCurve::Ed25519 {
            dh_ephemeral_public_key,
            decryptors,
        } => (PublicKey::Ed25519(*dh_ephemeral_public_key), decryptors),
        DecryptorsByCurve::Secp256k1 {
            dh_ephemeral_public_key,
            decryptors,
        } => (PublicKey::Secp256k1(*dh_ephemeral_public_key), decryptors),
    };
    Ok(models::EncryptedMessageCurveDecryptorSet {
        dh_ephemeral_public_key: Some(to_api_public_key(&dh_ephemeral_public_key)),
        decryptors: decryptors
            .iter()
            .map(|(public_key_fingerprint, aes_wrapped_key)| {
                to_api_decryptor(context, public_key_fingerprint, aes_wrapped_key)
            })
            .collect::<Result<_, _>>()?,
    })
}

fn to_api_decryptor(
    _context: &MappingContext,
    public_key_fingerprint: &PublicKeyFingerprint,
    aes_wrapped_key: &AesWrapped128BitKey,
) -> Result<models::EncryptedMessageDecryptor, MappingError> {
    Ok(models::EncryptedMessageDecryptor {
        public_key_fingerprint_hex: to_hex(public_key_fingerprint.0),
        aes_wrapped_key_hex: to_hex(aes_wrapped_key.0),
    })
}

pub fn to_api_round_update_transaction(
    context: &MappingContext,
    round_update_transaction: &RoundUpdateTransactionV1,
) -> Result<models::RoundUpdateTransaction, MappingError> {
    let RoundUpdateTransactionV1 {
        proposer_timestamp_ms,
        epoch,
        round,
        leader_proposal_history,
    } = round_update_transaction;
    Ok(models::RoundUpdateTransaction {
        proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(*proposer_timestamp_ms)?),
        epoch: to_api_epoch(context, *epoch)?,
        round_in_epoch: to_api_round(*round)?,
        leader_proposal_history: Box::new(models::LeaderProposalHistory {
            gap_round_leaders: leader_proposal_history
                .gap_round_leaders
                .iter()
                .map(|leader| to_api_active_validator_index(*leader))
                .collect(),
            current_leader: Box::new(to_api_active_validator_index(
                leader_proposal_history.current_leader,
            )),
            is_fallback: leader_proposal_history.is_fallback,
        }),
    })
}

pub fn to_api_system_transaction(
    context: &MappingContext,
    system_transaction: &SystemTransactionV1,
) -> Result<models::SystemTransaction, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_system {
        Some(to_hex(system_transaction.to_payload_bytes().map_err(
            |err| MappingError::SborEncodeError {
                encode_error: err,
                message: "Error encoding system transaction sbor".to_string(),
            },
        )?))
    } else {
        None
    };
    Ok(models::SystemTransaction { payload_hex })
}

pub fn to_api_flash_transaction(
    context: &MappingContext,
    flash_transaction: &FlashTransactionV1,
) -> Result<models::FlashTransaction, MappingError> {
    let mut flashed_substates = vec![];
    for (node_id, updates) in &flash_transaction.state_updates.by_node {
        match updates {
            NodeStateUpdates::Delta { by_partition } => {
                for (partition_number, partition_updates) in by_partition {
                    match partition_updates {
                        PartitionStateUpdates::Delta { by_substate } => {
                            for (key, update) in by_substate {
                                let address = to_api_entity_address(context, node_id)?;
                                let partition_num = to_api_u8_as_i32(partition_number.0);
                                let substate_key = to_api_substate_key(key);
                                match update {
                                    DatabaseUpdate::Set(value) => {
                                        let substate_value = to_hex(value);
                                        let flashed_substate = models::FlashedSubstate::new(
                                            address,
                                            partition_num,
                                            substate_key,
                                            substate_value,
                                        );
                                        flashed_substates.push(flashed_substate);
                                    }
                                    DatabaseUpdate::Delete => {
                                        unimplemented!("Flash transactions with deleted substates not yet supported.");
                                    }
                                }
                            }
                        }
                        PartitionStateUpdates::Batch(..) => {
                            unimplemented!("Flash transactions with partition state updates not yet supported.");
                        }
                    }
                }
            }
        }
    }

    Ok(models::FlashTransaction::new(flashed_substates))
}
