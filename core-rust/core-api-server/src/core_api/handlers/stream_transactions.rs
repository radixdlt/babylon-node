use crate::prelude::*;

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

    let from_state_version = extract_state_version(request.from_state_version)
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

    let database = state.state_manager.database.snapshot();

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
        Box::new(core::iter::empty())
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
            database.deref(),
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

    Ok(Json(response))
}

static ZERO_HASH: Hash = Hash([0; Hash::LENGTH]);

pub fn to_api_ledger_proof(
    mapping_context: &MappingContext,
    proof: LedgerProof,
) -> Result<models::LedgerProof, MappingError> {
    let api_origin = match &proof.origin {
        LedgerProofOrigin::Consensus {
            opaque,
            timestamped_signatures,
        } => {
            let api_timestamped_signatures = timestamped_signatures
                .iter()
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
                            signature_hex: to_hex(
                                timestamped_validator_signature.signature.to_vec(),
                            ),
                        }),
                    })
                })
                .collect::<Result<_, _>>()?;

            models::LedgerProofOrigin::ConsensusLedgerProofOrigin {
                opaque_hash: to_api_hash(opaque),
                timestamped_signatures: api_timestamped_signatures,
            }
        }
        LedgerProofOrigin::ProtocolUpdate {
            protocol_version_name,
            config_hash,
            batch_group_index,
            batch_group_name,
            batch_index,
            batch_name,
            is_end_of_update,
        } => {
            if protocol_version_name == &ProtocolVersionName::babylon() {
                models::LedgerProofOrigin::GenesisLedgerProofOrigin {
                    protocol_version_name: protocol_version_name.as_str().to_string(),
                    genesis_opaque_hash: to_api_hash(
                        config_hash
                            .as_ref()
                            .expect("Genesis always has a config hash"),
                    ),
                    batch_group_idx: to_api_index_as_i64(*batch_group_index)?,
                    batch_group_name: batch_group_name.to_string(),
                    batch_idx: to_api_index_as_i64(*batch_index)?,
                    batch_name: batch_name.to_string(),
                    is_end_of_update: *is_end_of_update,
                }
            } else {
                models::LedgerProofOrigin::ProtocolUpdateLedgerProofOrigin {
                    protocol_version_name: protocol_version_name.to_string(),
                    config_hash: to_api_hash(config_hash.as_ref().unwrap_or(&ZERO_HASH)),
                    batch_group_idx: to_api_index_as_i64(*batch_group_index)?,
                    batch_group_name: batch_group_name.to_string(),
                    batch_idx: to_api_index_as_i64(*batch_index)?,
                    batch_name: batch_name.to_string(),
                    is_end_of_update: *is_end_of_update,
                }
            }
        }
    };
    Ok(models::LedgerProof {
        ledger_header: Box::new(to_api_ledger_header(mapping_context, proof.ledger_header)?),
        origin: Some(api_origin),
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
                significant_protocol_update_readiness: None,
            }))
        }
        None => None,
    };
    let next_protocol_version = ledger_header
        .next_protocol_version
        .map(|version| version.to_string());

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
        next_protocol_version,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_committed_transaction(
    database: &StateManagerDatabase<impl ReadableRocks>,
    context: &MappingContext,
    state_version: StateVersion,
    raw_ledger_transaction: RawLedgerTransaction,
    ledger_transaction: LedgerTransaction,
    receipt: LocalTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::CommittedTransaction, MappingError> {
    let balance_changes = if context.transaction_options.include_balance_changes {
        Some(Box::new(to_api_balance_changes(
            database, context, &receipt,
        )?))
    } else {
        None
    };

    Ok(models::CommittedTransaction {
        resultant_state_identifiers: Box::new(to_api_committed_state_identifiers(
            state_version,
            &identifiers.resultant_ledger_hashes,
        )?),
        ledger_transaction: Some(to_api_ledger_transaction(
            context,
            &raw_ledger_transaction,
            &ledger_transaction,
            &identifiers.transaction_hashes,
        )?),
        receipt: Box::new(to_api_receipt(Some(database), context, receipt)?),
        balance_changes,
        proposer_timestamp_ms: identifiers.proposer_timestamp_ms,
    })
}

pub fn to_api_ledger_transaction(
    context: &MappingContext,
    raw_ledger_transaction: &RawLedgerTransaction,
    ledger_transaction: &LedgerTransaction,
    hashes: &LedgerTransactionHashes,
) -> Result<models::LedgerTransaction, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_ledger {
        Some(to_hex(raw_ledger_transaction.as_slice()))
    } else {
        None
    };

    Ok(match ledger_transaction {
        LedgerTransaction::UserV1(tx) => {
            let user_hashes =
                hashes
                    .as_user()
                    .ok_or_else(|| MappingError::MismatchedTransactionIdentifiers {
                        message: "Transaction hashes for notarized transaction were not user"
                            .to_string(),
                    })?;
            models::LedgerTransaction::UserLedgerTransaction {
                payload_hex,
                notarized_transaction: Box::new(to_api_notarized_transaction_v1(
                    context,
                    tx,
                    &user_hashes,
                )?),
            }
        }
        LedgerTransaction::UserV2(tx) => {
            let user_hashes =
                hashes
                    .as_user()
                    .ok_or_else(|| MappingError::MismatchedTransactionIdentifiers {
                        message: "Transaction hashes for notarized transaction were not user"
                            .to_string(),
                    })?;
            models::LedgerTransaction::UserLedgerTransactionV2 {
                payload_hex,
                notarized_transaction: Box::new(to_api_notarized_transaction_v2(
                    context,
                    tx,
                    &user_hashes,
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
            name: tx.name.clone(),
            flashed_state_updates: Box::new(to_api_flashed_state_updates(
                context,
                &tx.state_updates,
            )?),
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_notarized_transaction_v1(
    context: &MappingContext,
    notarized: &NotarizedTransactionV1,
    user_hashes: &UserTransactionHashes,
) -> Result<models::NotarizedTransaction, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_notarized {
        Some(to_hex(
            notarized
                .to_raw()
                .map_err(|err| MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding user payload sbor".to_string(),
                })?
                .as_slice(),
        ))
    } else {
        None
    };

    Ok(models::NotarizedTransaction {
        hash: to_api_notarized_transaction_hash(&user_hashes.notarized_transaction_hash),
        hash_bech32m: to_api_hash_bech32m(context, &user_hashes.notarized_transaction_hash)?,
        payload_hex,
        signed_intent: Box::new(to_api_signed_intent(
            context,
            &notarized.signed_intent,
            &user_hashes.transaction_intent_hash,
            &user_hashes.signed_transaction_intent_hash,
        )?),
        notary_signature: Some(to_api_signature(&notarized.notary_signature.0)),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_signed_intent(
    context: &MappingContext,
    signed_intent: &SignedIntentV1,
    transaction_intent_hash: &TransactionIntentHash,
    signed_transaction_intent_hash: &SignedTransactionIntentHash,
) -> Result<models::SignedTransactionIntent, MappingError> {
    Ok(models::SignedTransactionIntent {
        hash: to_api_signed_transaction_intent_hash(signed_transaction_intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, signed_transaction_intent_hash)?,
        intent: Box::new(to_api_intent_v1(
            context,
            &signed_intent.intent,
            transaction_intent_hash,
        )?),
        intent_signatures: signed_intent
            .intent_signatures
            .signatures
            .iter()
            .map(|s| to_api_signature_with_public_key(&s.0))
            .collect(),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_intent_v1(
    context: &MappingContext,
    intent: &IntentV1,
    transaction_intent_hash: &TransactionIntentHash,
) -> Result<models::TransactionIntent, MappingError> {
    let instructions = if context.transaction_options.include_manifest {
        let readable_manifest = TransactionManifestV1::from_intent(intent);
        Some(
            manifest::decompile(&readable_manifest, &context.network_definition).map_err(
                |err| MappingError::InvalidManifest {
                    message: format!("Failed to decompile a transaction manifest: {err:?}"),
                },
            )?,
        )
    } else {
        None
    };

    let IntentV1 {
        header,
        blobs,
        message,
        instructions: _,
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
        hash: to_api_transaction_intent_hash(transaction_intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, transaction_intent_hash)?,
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

fn to_api_encrypted_message_v2(
    context: &MappingContext,
    encrypted: &EncryptedMessageV2,
) -> Result<models::TransactionMessage, MappingError> {
    let EncryptedMessageV2 {
        encrypted,
        decryptors_by_curve,
    } = encrypted;
    Ok(models::TransactionMessage::EncryptedTransactionMessage {
        encrypted_hex: to_hex(&encrypted.0),
        curve_decryptor_sets: decryptors_by_curve
            .values()
            .map(|decryptor_set| to_api_decryptor_set_v2(context, decryptor_set))
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

fn to_api_decryptor_set_v2(
    context: &MappingContext,
    decryptor_set: &DecryptorsByCurveV2,
) -> Result<models::EncryptedMessageCurveDecryptorSet, MappingError> {
    let (dh_ephemeral_public_key, decryptors) = match decryptor_set {
        DecryptorsByCurveV2::Ed25519 {
            dh_ephemeral_public_key,
            decryptors,
        } => (PublicKey::Ed25519(*dh_ephemeral_public_key), decryptors),
        DecryptorsByCurveV2::Secp256k1 {
            dh_ephemeral_public_key,
            decryptors,
        } => (PublicKey::Secp256k1(*dh_ephemeral_public_key), decryptors),
    };
    Ok(models::EncryptedMessageCurveDecryptorSet {
        dh_ephemeral_public_key: Some(to_api_public_key(&dh_ephemeral_public_key)),
        decryptors: decryptors
            .iter()
            .map(|(public_key_fingerprint, aes_wrapped_key)| {
                to_api_decryptor_v2(context, public_key_fingerprint, aes_wrapped_key)
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

fn to_api_decryptor_v2(
    _context: &MappingContext,
    public_key_fingerprint: &PublicKeyFingerprint,
    aes_wrapped_key: &AesWrapped256BitKey,
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
        proposer_timestamp: Box::new(to_api_clamped_instant_from_epoch_milli(
            *proposer_timestamp_ms,
        )),
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
        Some(to_hex(
            system_transaction
                .to_raw()
                .map_err(|err| MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding system transaction sbor".to_string(),
                })?
                .as_slice(),
        ))
    } else {
        None
    };
    Ok(models::SystemTransaction { payload_hex })
}

fn to_api_balance_changes(
    database: &StateManagerDatabase<impl ReadableRocks>,
    context: &MappingContext,
    receipt: &LocalTransactionReceipt,
) -> Result<models::CommittedTransactionBalanceChanges, MappingError> {
    let local_execution = &receipt.local_execution;

    Ok(models::CommittedTransactionBalanceChanges {
        fungible_entity_balance_changes: to_api_lts_fungible_balance_changes(
            database,
            context,
            &local_execution.fee_summary,
            &local_execution.fee_source,
            &local_execution.fee_destination,
            &local_execution
                .global_balance_summary
                .global_balance_changes,
        )?,
        non_fungible_entity_balance_changes: to_api_lts_entity_non_fungible_balance_changes(
            context,
            &local_execution
                .global_balance_summary
                .global_balance_changes,
        )?,
    })
}

pub fn to_api_flashed_state_updates(
    context: &MappingContext,
    state_updates: &StateUpdates,
) -> Result<models::FlashedStateUpdates, MappingError> {
    let mut deleted_partitions = Vec::new();
    let mut set_substates = Vec::new();
    let mut deleted_substates = Vec::new();
    for (node_id, updates) in &state_updates.by_node {
        match updates {
            NodeStateUpdates::Delta { by_partition } => {
                for (partition_number, partition_updates) in by_partition {
                    match partition_updates {
                        PartitionStateUpdates::Delta { by_substate } => {
                            for (key, update) in by_substate {
                                match update {
                                    DatabaseUpdate::Set(value) => {
                                        set_substates.push(to_api_flash_set_substate(
                                            context,
                                            node_id,
                                            *partition_number,
                                            key,
                                            value,
                                        )?);
                                    }
                                    DatabaseUpdate::Delete => {
                                        deleted_substates.push(to_api_direct_substate_id(
                                            context,
                                            node_id,
                                            *partition_number,
                                            key,
                                        )?);
                                    }
                                }
                            }
                        }
                        PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                            new_substate_values,
                        }) => {
                            deleted_partitions.push(to_api_partition_id(
                                context,
                                node_id,
                                *partition_number,
                            )?);
                            for (key, value) in new_substate_values {
                                set_substates.push(to_api_flash_set_substate(
                                    context,
                                    node_id,
                                    *partition_number,
                                    key,
                                    value,
                                )?);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(models::FlashedStateUpdates {
        deleted_partitions,
        set_substates,
        deleted_substates,
    })
}

fn to_api_flash_set_substate(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    value: &DbSubstateValue,
) -> Result<models::FlashSetSubstate, MappingError> {
    let typed_substate_key =
        create_typed_substate_key(context, node_id, partition_number, substate_key)?;
    Ok(models::FlashSetSubstate {
        substate_id: Box::new(to_api_substate_id(
            context,
            node_id,
            partition_number,
            substate_key,
            &typed_substate_key,
        )?),
        value: Box::new(to_api_substate_value(
            context,
            &StateMappingLookups::default(),
            &typed_substate_key,
            value,
        )?),
    })
}

fn to_api_direct_substate_id(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<models::SubstateId, MappingError> {
    let typed_substate_key =
        create_typed_substate_key(context, node_id, partition_number, substate_key)?;
    to_api_substate_id(
        context,
        node_id,
        partition_number,
        substate_key,
        &typed_substate_key,
    )
}

pub fn to_api_notarized_transaction_v2(
    context: &MappingContext,
    notarized: &NotarizedTransactionV2,
    user_hashes: &UserTransactionHashes,
) -> Result<models::NotarizedTransactionV2, MappingError> {
    let payload_hex = if context.transaction_options.include_raw_notarized {
        Some(to_hex(
            notarized
                .to_raw()
                .map_err(|err| MappingError::SborEncodeError {
                    encode_error: err,
                    message: "Error encoding user payload sbor".to_string(),
                })?
                .as_slice(),
        ))
    } else {
        None
    };

    let NotarizedTransactionV2 {
        signed_transaction_intent,
        notary_signature,
    } = notarized;

    Ok(models::NotarizedTransactionV2 {
        hash: to_api_notarized_transaction_hash(&user_hashes.notarized_transaction_hash),
        hash_bech32m: to_api_hash_bech32m(context, &user_hashes.notarized_transaction_hash)?,
        payload_hex,
        signed_transaction_intent: Box::new(to_api_signed_transaction_intent_v2(
            context,
            signed_transaction_intent,
            &user_hashes.transaction_intent_hash,
            &user_hashes.signed_transaction_intent_hash,
        )?),
        notary_signature: Some(to_api_signature(&notary_signature.0)),
    })
}

pub fn to_api_signed_transaction_intent_v2(
    context: &MappingContext,
    signed: &SignedTransactionIntentV2,
    transaction_intent_hash: &TransactionIntentHash,
    signed_transaction_intent_hash: &SignedTransactionIntentHash,
) -> Result<models::SignedTransactionIntentV2, MappingError> {
    let SignedTransactionIntentV2 {
        transaction_intent,
        transaction_intent_signatures,
        non_root_subintent_signatures,
    } = signed;
    Ok(models::SignedTransactionIntentV2 {
        hash: to_api_signed_transaction_intent_hash(signed_transaction_intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, signed_transaction_intent_hash)?,
        transaction_intent: Box::new(to_api_transaction_intent_v2(
            context,
            transaction_intent,
            transaction_intent_hash,
        )?),
        transaction_intent_signatures: Box::new(to_api_intent_signatures_v2(
            transaction_intent_signatures.signatures.as_slice(),
        )),
        non_root_subintent_signatures: non_root_subintent_signatures
            .by_subintent
            .iter()
            .map(|sigs| to_api_intent_signatures_v2(&sigs.signatures))
            .collect(),
    })
}

pub fn to_api_intent_signatures_v2(
    intent_signatures: &[IntentSignatureV1],
) -> models::IntentSignatures {
    models::IntentSignatures {
        signatures: intent_signatures
            .iter()
            .map(|s| to_api_signature_with_public_key(&s.0))
            .collect(),
    }
}

pub fn to_api_transaction_intent_v2(
    context: &MappingContext,
    transaction_intent: &TransactionIntentV2,
    transaction_intent_hash: &TransactionIntentHash,
) -> Result<models::TransactionIntentV2, MappingError> {
    let TransactionIntentV2 {
        transaction_header,
        root_intent_core,
        non_root_subintents,
    } = transaction_intent;
    Ok(models::TransactionIntentV2 {
        hash: to_api_transaction_intent_hash(transaction_intent_hash),
        hash_bech32m: to_api_hash_bech32m(context, transaction_intent_hash)?,
        transaction_header: Box::new(to_api_transaction_header_v2(transaction_header)?),
        root_intent_core: Box::new(to_api_intent_core_v2(context, root_intent_core)?),
        non_root_subintents: non_root_subintents
            .0
            .iter()
            // TODO:CUTTLEFISH - fix the subintent hashes
            .map(|subintent| {
                to_api_subintent_v2(context, subintent, &SubintentHash(Hash([0; Hash::LENGTH])))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_transaction_header_v2(
    transaction_header: &TransactionHeaderV2,
) -> Result<models::TransactionHeaderV2, MappingError> {
    let TransactionHeaderV2 {
        notary_public_key,
        notary_is_signatory,
        tip_basis_points,
    } = transaction_header;
    Ok(models::TransactionHeaderV2 {
        notary_public_key: Some(to_api_public_key(notary_public_key)),
        notary_is_signatory: *notary_is_signatory,
        tip_basis_points: to_api_u32_as_i64(*tip_basis_points),
    })
}

pub fn to_api_subintent_v2(
    context: &MappingContext,
    subintent: &SubintentV2,
    subintent_hash: &SubintentHash,
) -> Result<models::SubintentV2, MappingError> {
    let SubintentV2 { intent_core } = subintent;
    Ok(models::SubintentV2 {
        hash: to_api_subintent_hash(subintent_hash),
        hash_bech32m: to_api_hash_bech32m(context, subintent_hash)?,
        intent_core: Box::new(to_api_intent_core_v2(context, intent_core)?),
    })
}

pub fn to_api_intent_core_v2(
    context: &MappingContext,
    intent_core: &IntentCoreV2,
) -> Result<models::IntentCoreV2, MappingError> {
    let instructions = if context.transaction_options.include_manifest {
        let readable_manifest = TransactionManifestV2::from_intent_core(intent_core);
        Some(
            manifest::decompile(&readable_manifest, &context.network_definition).map_err(
                |err| MappingError::InvalidManifest {
                    message: format!("Failed to decompile a transaction manifest: {err:?}"),
                },
            )?,
        )
    } else {
        None
    };

    let IntentCoreV2 {
        header,
        blobs,
        message,
        children,
        instructions: _,
    } = intent_core;

    let intent_header = {
        let IntentHeaderV2 {
            network_id,
            start_epoch_inclusive,
            end_epoch_exclusive,
            min_proposer_timestamp_inclusive,
            max_proposer_timestamp_exclusive,
            intent_discriminator,
        } = header;

        models::IntentHeaderV2 {
            network_id: (*network_id).into(),
            start_epoch_inclusive: to_api_epoch(context, *start_epoch_inclusive)?,
            end_epoch_exclusive: to_api_epoch(context, *end_epoch_exclusive)?,
            min_proposer_timestamp_inclusive: min_proposer_timestamp_inclusive
                .as_ref()
                .map(|instant| Box::new(to_api_scrypto_instant(instant))),
            max_proposer_timestamp_exclusive: max_proposer_timestamp_exclusive
                .as_ref()
                .map(|instant| Box::new(to_api_scrypto_instant(instant))),
            intent_discriminator: to_api_u64_as_string(*intent_discriminator),
        }
    };

    let children_specifiers = {
        let ChildSubintentSpecifiersV2 { children } = children;
        children
            .iter()
            .map(|child_specifier| to_api_hash_bech32m(context, &child_specifier.hash))
            .collect::<Result<_, _>>()?
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
            MessageV2::None => None,
            MessageV2::Plaintext(plaintext) => {
                Some(Box::new(to_api_plaintext_message(context, plaintext)?))
            }
            MessageV2::Encrypted(encrypted) => {
                Some(Box::new(to_api_encrypted_message_v2(context, encrypted)?))
            }
        }
    } else {
        None
    };

    Ok(models::IntentCoreV2 {
        intent_header: Box::new(intent_header),
        children_specifiers,
        instructions,
        blobs_hex,
        message,
    })
}
