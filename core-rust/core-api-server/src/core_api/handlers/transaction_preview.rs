use crate::core_api::*;
use radix_engine::{
    transaction::{PreviewError, PreviewResult, TransactionResult},
    types::{Bech32Encoder, RENodeId},
};
use radix_engine_interface::core::NetworkDefinition;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{LedgerTransactionReceipt, PreviewRequest};
use transaction::manifest;
use transaction::model::PreviewFlags;

pub(crate) async fn handle_transaction_preview(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionPreviewRequest>,
) -> Result<Json<models::TransactionPreviewResponse>, RequestHandlingError> {
    core_api_read_handler(state, request, handle_preview_internal)
}

#[tracing::instrument(level = "debug", skip(state_manager), err(Debug))]
fn handle_preview_internal(
    state_manager: &ActualStateManager,
    request: models::TransactionPreviewRequest,
) -> Result<models::TransactionPreviewResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let preview_request = parse_preview_request(&state_manager.network, request)?;

    let result = state_manager
        .preview(preview_request)
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                client_error(format!("Transaction validation error: {:?}", err))
            }
        })?;

    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    to_api_response(result, &bech32_encoder)
}

fn parse_preview_request(
    network: &NetworkDefinition,
    request: models::TransactionPreviewRequest,
) -> Result<PreviewRequest, RequestHandlingError> {
    let manifest_blobs: Vec<_> = request
        .blobs_hex
        .unwrap_or_default()
        .into_iter()
        .map(from_hex)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("blobs"))?;

    let manifest = manifest::compile(&request.manifest, network, manifest_blobs)
        .map_err(|err| client_error(format!("Invalid manifest - {:?}", err)))?;

    let signer_public_keys: Vec<_> = request
        .signer_public_keys
        .into_iter()
        .map(extract_api_public_key)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("signer_public_keys"))?;

    Ok(PreviewRequest {
        manifest,
        start_epoch_inclusive: extract_api_epoch(request.start_epoch_inclusive)
            .map_err(|err| err.into_response_error("start_epoch_inclusive"))?,
        end_epoch_exclusive: extract_api_epoch(request.end_epoch_exclusive)
            .map_err(|err| err.into_response_error("end_epoch_exclusive"))?,
        notary_public_key: request
            .notary_public_key
            .map(|pk| {
                extract_api_public_key(*pk)
                    .map_err(|err| err.into_response_error("notary_public_key"))
            })
            .transpose()?,
        notary_as_signatory: request.notary_as_signatory.unwrap_or(false),
        cost_unit_limit: extract_api_u32_as_i64(request.cost_unit_limit)
            .map_err(|err| err.into_response_error("cost_unit_limit"))?,
        tip_percentage: extract_api_u8_as_i32(request.tip_percentage)
            .map_err(|err| err.into_response_error("tip_percentage"))?,
        nonce: extract_api_u64_as_string(request.nonce)
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
    result: PreviewResult,
    bech32_encoder: &Bech32Encoder,
) -> Result<models::TransactionPreviewResponse, RequestHandlingError> {
    let receipt = result.receipt;

    let logs = receipt
        .execution
        .application_logs
        .iter()
        .map(
            |(level, message)| models::TransactionPreviewResponseLogsInner {
                level: level.to_string(),
                message: message.to_string(),
            },
        )
        .collect();

    let response = match &receipt.result {
        TransactionResult::Commit(commit_result) => {
            let api_resource_changes = commit_result
                .resource_changes
                .iter()
                .map(|v| {
                    Ok(models::ResourceChange {
                        resource_address: bech32_encoder
                            .encode_resource_address_to_string(&v.resource_address),
                        component_entity: Box::new(to_entity_reference(RENodeId::Component(
                            v.component_id,
                        ))?),
                        vault_entity: Box::new(to_entity_reference(RENodeId::Vault(v.vault_id))?),
                        amount: to_api_decimal(&v.amount),
                    })
                })
                .collect::<Result<_, MappingError>>()
                .map_err(|_| server_error("Can't map entity references"))?;

            let ledger_receipt: LedgerTransactionReceipt = receipt
                .try_into()
                .map_err(|_| server_error("Can't create a ledger receipt"))?;

            models::TransactionPreviewResponse {
                receipt: Box::new(to_api_receipt(bech32_encoder, ledger_receipt)?),
                resource_changes: api_resource_changes,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => models::TransactionPreviewResponse {
            receipt: Box::new(models::TransactionReceipt {
                status: models::TransactionStatus::Rejected,
                fee_summary: Box::new(to_api_fee_summary(
                    bech32_encoder,
                    receipt.execution.fee_summary,
                )?),
                state_updates: Box::default(),
                output: None,
                error_message: Some(format!("{:?}", reject_result)),
            }),
            resource_changes: vec![],
            logs,
        },
    };

    Ok(response)
}
