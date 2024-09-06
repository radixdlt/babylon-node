use crate::core_api::*;
use crate::engine_prelude::*;

use std::ops::Range;

use radix_engine_toolkit::receipt::RuntimeToolkitTransactionReceipt;
use state_manager::transaction::ProcessedPreviewResult;
use state_manager::{
    ActualStateManagerDatabase, ExecutionFeeData, LocalTransactionReceipt, PreviewRequest,
};

pub(crate) async fn handle_transaction_preview(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionPreviewRequest>,
) -> Result<Json<models::TransactionPreviewResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let at_state_version = request
        .at_ledger_state
        .as_deref()
        .map(extract_ledger_state_selector)
        .transpose()
        .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let should_produce_toolkit_receipt = request
        .options
        .as_ref()
        .and_then(|opt_ins| opt_ins.radix_engine_toolkit_receipt)
        .unwrap_or(false);
    let preview_request = extract_preview_request(&state.network, request)?;

    let result = state
        .state_manager
        .transaction_previewer
        .preview(preview_request, at_state_version)?;

    to_api_response(&mapping_context, result, should_produce_toolkit_receipt).map(Json)
}

fn extract_preview_request(
    network: &NetworkDefinition,
    request: models::TransactionPreviewRequest,
) -> Result<PreviewRequest, ResponseError<()>> {
    let manifest_blobs: Vec<_> = request
        .blobs_hex
        .unwrap_or_default()
        .into_iter()
        .map(from_hex)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("blobs"))?;
    let manifest_blob_provider = BlobProvider::new_with_blobs(manifest_blobs);
    let manifest = manifest::compile(&request.manifest, network, manifest_blob_provider)
        .map_err(|err| client_error(format!("Invalid manifest - {err:?}")))?;

    let signer_public_keys: Vec<_> = request
        .signer_public_keys
        .into_iter()
        .map(extract_public_key)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("signer_public_keys"))?;

    Ok(PreviewRequest {
        manifest,
        explicit_epoch_range: Some(Range {
            start: extract_epoch(request.start_epoch_inclusive)
                .map_err(|err| err.into_response_error("start_epoch_inclusive"))?,
            end: extract_epoch(request.end_epoch_exclusive)
                .map_err(|err| err.into_response_error("end_epoch_exclusive"))?,
        }),
        notary_public_key: request
            .notary_public_key
            .map(|pk| {
                extract_public_key(*pk).map_err(|err| err.into_response_error("notary_public_key"))
            })
            .transpose()?,
        notary_is_signatory: request.notary_is_signatory.unwrap_or(false),
        tip_percentage: extract_u16_from_api_i32(request.tip_percentage)
            .map_err(|err| err.into_response_error("tip_percentage"))?,
        nonce: extract_u32_from_api_i64(request.nonce)
            .map_err(|err| err.into_response_error("nonce"))?,
        signer_public_keys,
        flags: PreviewFlags {
            use_free_credit: request.flags.use_free_credit,
            assume_all_signature_proofs: request.flags.assume_all_signature_proofs,
            skip_epoch_check: request.flags.skip_epoch_check,
            disable_auth: request.flags.disable_auth_checks.unwrap_or(false),
        },
        message: request
            .message
            .map(|message| {
                extract_message(*message).map_err(|err| err.into_response_error("message"))
            })
            .transpose()?
            .unwrap_or_else(|| MessageV1::None),
    })
}

fn to_api_response(
    context: &MappingContext,
    result: ProcessedPreviewResult,
    should_include_toolkit_receipt: bool,
) -> Result<models::TransactionPreviewResponse, ResponseError<()>> {
    let engine_receipt = result.receipt;
    let versioned_engine_receipt = engine_receipt.clone().into_versioned();

    // This is interpreted by the toolkit in the wallet. This will be removed with the release of
    // the cuttlefish protocol update.
    let encoded_receipt = to_hex(scrypto_encode(&versioned_engine_receipt).unwrap());

    // Produce a toolkit transaction receipt for the transaction preview if it was requested in the
    // request opt-ins.
    let toolkit_receipt = if should_include_toolkit_receipt {
        Some(
            RuntimeToolkitTransactionReceipt::try_from(versioned_engine_receipt.clone())
                .ok()
                .and_then(|value| {
                    value
                        .into_serializable_receipt(&context.address_encoder)
                        .ok()
                })
                .and_then(|value| serde_json::to_value(&value).ok())
                .ok_or(server_error("Can't produce toolkit transaction receipt."))?,
        )
    } else {
        None
    };

    let at_ledger_state = Box::new(to_api_ledger_state_summary(
        context,
        &result.base_ledger_state,
    )?);

    let execution_fee_data = ExecutionFeeData {
        fee_summary: engine_receipt.fee_summary,
        engine_costing_parameters: engine_receipt.costing_parameters,
        transaction_costing_parameters: engine_receipt.transaction_costing_parameters,
    };

    let response = match engine_receipt.result {
        TransactionResult::Commit(commit_result) => {
            let mut instruction_resource_changes = Vec::new();

            let execution_trace = commit_result
                .execution_trace
                .as_ref()
                .expect("preview should have been executed with execution trace enabled");
            for (index, resource_changes) in &execution_trace.resource_changes {
                let resource_changes: Vec<models::ResourceChange> = resource_changes
                    .iter()
                    .map(|v| {
                        Ok(models::ResourceChange {
                            resource_address: to_api_resource_address(
                                context,
                                &v.resource_address,
                            )?,
                            component_entity: Box::new(to_api_entity_reference(
                                context, &v.node_id,
                            )?),
                            vault_entity: Box::new(to_api_entity_reference(context, &v.vault_id)?),
                            amount: to_api_decimal(&v.amount),
                        })
                    })
                    .collect::<Result<_, MappingError>>()
                    .map_err(|_| server_error("Can't map entity references"))?;

                let instruction = models::InstructionResourceChanges {
                    index: to_api_index_as_i64(*index)?,
                    resource_changes,
                };

                instruction_resource_changes.push(instruction);
            }

            let logs = commit_result
                .application_logs
                .iter()
                .map(
                    |(level, message)| models::TransactionPreviewResponseLogsInner {
                        level: level.to_string(),
                        message: message.to_string(),
                    },
                )
                .collect();

            let local_receipt = LocalTransactionReceipt::new(
                commit_result,
                result.state_changes,
                result.global_balance_summary,
                execution_fee_data,
            );

            models::TransactionPreviewResponse {
                at_ledger_state,
                encoded_receipt,
                receipt: Box::new(to_api_receipt(
                    None::<&ActualStateManagerDatabase>,
                    context,
                    local_receipt,
                )?),
                radix_engine_toolkit_receipt: toolkit_receipt,
                instruction_resource_changes,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => models::TransactionPreviewResponse {
            at_ledger_state,
            encoded_receipt,
            receipt: Box::new(models::TransactionReceipt {
                status: models::TransactionStatus::Rejected,
                fee_summary: Box::new(to_api_fee_summary(
                    context,
                    &execution_fee_data.fee_summary,
                )?),
                fee_source: None,
                fee_destination: None,
                costing_parameters: Box::new(to_api_costing_parameters(
                    context,
                    &execution_fee_data.engine_costing_parameters,
                    &execution_fee_data.transaction_costing_parameters,
                )?),
                state_updates: Box::default(),
                events: None,
                output: None,
                next_epoch: None,
                error_message: Some(format!("{reject_result:?}")),
            }),
            radix_engine_toolkit_receipt: toolkit_receipt,
            instruction_resource_changes: vec![],
            logs: vec![],
        },
        TransactionResult::Abort(_) => {
            panic!("Should not be aborting");
        }
    };

    Ok(response)
}

fn extract_message(message: models::TransactionMessage) -> Result<MessageV1, ExtractionError> {
    Ok(match message {
        models::TransactionMessage::PlaintextTransactionMessage { mime_type, content } => {
            MessageV1::Plaintext(PlaintextMessageV1 {
                mime_type,
                message: match *content {
                    models::PlaintextMessageContent::StringPlaintextMessageContent { value } => {
                        MessageContentsV1::String(value)
                    }
                    models::PlaintextMessageContent::BinaryPlaintextMessageContent {
                        value_hex,
                    } => MessageContentsV1::Bytes(from_hex(value_hex)?),
                },
            })
        }
        models::TransactionMessage::EncryptedTransactionMessage {
            encrypted_hex,
            curve_decryptor_sets,
        } => MessageV1::Encrypted(EncryptedMessageV1 {
            encrypted: AesGcmPayload(from_hex(encrypted_hex)?),
            decryptors_by_curve: curve_decryptor_sets
                .into_iter()
                .map(|curve_decryptor_set| -> Result<_, ExtractionError> {
                    let dh_ephemeral_public_key =
                        extract_public_key(curve_decryptor_set.dh_ephemeral_public_key.unwrap())?;
                    let decryptors = curve_decryptor_set
                        .decryptors
                        .into_iter()
                        .map(|decryptor| -> Result<_, ExtractionError> {
                            Ok((
                                PublicKeyFingerprint(copy_u8_array(&from_hex(
                                    decryptor.public_key_fingerprint_hex,
                                )?)),
                                AesWrapped128BitKey(copy_u8_array(&from_hex(
                                    decryptor.aes_wrapped_key_hex,
                                )?)),
                            ))
                        })
                        .collect::<Result<_, _>>()?;
                    let descryptors_by_curve = match dh_ephemeral_public_key {
                        PublicKey::Secp256k1(dh_ephemeral_public_key) => {
                            DecryptorsByCurve::Secp256k1 {
                                dh_ephemeral_public_key,
                                decryptors,
                            }
                        }
                        PublicKey::Ed25519(dh_ephemeral_public_key) => DecryptorsByCurve::Ed25519 {
                            dh_ephemeral_public_key,
                            decryptors,
                        },
                    };
                    Ok((descryptors_by_curve.curve_type(), descryptors_by_curve))
                })
                .collect::<Result<_, _>>()?,
        }),
    })
}
