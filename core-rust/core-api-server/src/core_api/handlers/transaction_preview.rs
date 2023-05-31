use crate::core_api::*;
use radix_engine::transaction::{PreviewError, TransactionResult};
use radix_engine_common::data::scrypto::scrypto_encode;
use radix_engine_interface::network::NetworkDefinition;
use std::ops::Range;

use state_manager::transaction::ProcessedPreviewResult;
use state_manager::{LocalTransactionReceipt, PreviewRequest, ProcessedTransactionReceipt};
use transaction::manifest;
use transaction::model::PreviewFlags;

pub(crate) async fn handle_transaction_preview(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionPreviewRequest>,
) -> Result<Json<models::TransactionPreviewResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let preview_request = extract_preview_request(&state.network, request)?;

    let result = state
        .transaction_previewer
        .preview(preview_request)
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                client_error(format!("Transaction validation error: {err:?}"))
            }
        })?;

    to_api_response(&mapping_context, result).map(Json)
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

    let manifest = manifest::compile(&request.manifest, network, manifest_blobs)
        .map_err(|err| client_error(format!("Invalid manifest - {err:?}")))?;

    let signer_public_keys: Vec<_> = request
        .signer_public_keys
        .into_iter()
        .map(extract_api_public_key)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("signer_public_keys"))?;

    Ok(PreviewRequest {
        manifest,
        explicit_epoch_range: Some(Range {
            start: extract_api_epoch(request.start_epoch_inclusive)
                .map_err(|err| err.into_response_error("start_epoch_inclusive"))?,
            end: extract_api_epoch(request.end_epoch_exclusive)
                .map_err(|err| err.into_response_error("end_epoch_exclusive"))?,
        }),
        notary_public_key: request
            .notary_public_key
            .map(|pk| {
                extract_api_public_key(*pk)
                    .map_err(|err| err.into_response_error("notary_public_key"))
            })
            .transpose()?,
        notary_is_signatory: request.notary_is_signatory.unwrap_or(false),
        tip_percentage: extract_api_u16_as_i32(request.tip_percentage)
            .map_err(|err| err.into_response_error("tip_percentage"))?,
        nonce: extract_api_u32_as_i64(request.nonce)
            .map_err(|err| err.into_response_error("nonce"))?,
        signer_public_keys,
        flags: PreviewFlags {
            unlimited_loan: request.flags.unlimited_loan,
            assume_all_signature_proofs: request.flags.assume_all_signature_proofs,
            permit_duplicate_intent_hash: request.flags.permit_duplicate_intent_hash,
            permit_invalid_header_epoch: request.flags.permit_invalid_header_epoch,
        },
    })
}

fn to_api_response(
    context: &MappingContext,
    result: ProcessedPreviewResult,
) -> Result<models::TransactionPreviewResponse, ResponseError<()>> {
    let receipt = result.receipt;
    let substate_changes = match result.processed_receipt {
        ProcessedTransactionReceipt::Commit(commit) => {
            commit.local_receipt.on_ledger.substate_changes
        }
        _ => vec![],
    };

    let encoded_receipt = to_hex(scrypto_encode(&receipt).unwrap());

    let response = match receipt.result {
        TransactionResult::Commit(commit_result) => {
            let mut instruction_resource_changes = Vec::new();

            for (index, resource_changes) in &receipt.execution_trace.resource_changes {
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
                    index: i32::try_from(*index).unwrap(),
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

            let local_receipt = LocalTransactionReceipt::from((
                commit_result,
                substate_changes,
                receipt.execution_trace,
            ));

            models::TransactionPreviewResponse {
                encoded_receipt,
                receipt: Box::new(to_api_receipt(context, local_receipt)?),
                instruction_resource_changes,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => models::TransactionPreviewResponse {
            encoded_receipt,
            receipt: Box::new(models::TransactionReceipt {
                status: models::TransactionStatus::Rejected,
                fee_summary: None,
                state_updates: Box::default(),
                events: None,
                output: None,
                next_epoch: None,
                error_message: Some(format!("{reject_result:?}")),
            }),
            instruction_resource_changes: vec![],
            logs: vec![],
        },
        TransactionResult::Abort(_) => {
            panic!("Should not be aborting");
        }
    };

    Ok(response)
}
