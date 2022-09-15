use crate::core_api::*;
use radix_engine::transaction::{PreviewError, PreviewResult, TransactionResult};
use scrypto::address::Bech32Encoder;
use scrypto::prelude::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{LedgerTransactionReceipt, PreviewRequest};
use transaction::model::{PreviewFlags, TransactionManifest};

pub(crate) async fn handle_transaction_preview(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionPreviewRequest>,
) -> Result<Json<models::TransactionPreviewResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_preview_internal)
}

fn handle_preview_internal(
    state_manager: &mut ActualStateManager,
    request: models::TransactionPreviewRequest,
) -> Result<models::TransactionPreviewResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let preview_request = parse_preview_request(request)?;

    let result = state_manager
        .preview(preview_request)
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                server_error(&format!("Transaction validation error: {:?}", err))
            }
        })?;

    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    to_api_response(result, &bech32_encoder)
}

fn parse_preview_request(
    request: models::TransactionPreviewRequest,
) -> Result<PreviewRequest, RequestHandlingError> {
    let manifest_bytes =
        from_hex(request.manifest).map_err(|err| err.into_response_error("manifest"))?;

    let manifest = scrypto_decode::<TransactionManifest>(&manifest_bytes)
        .map_err(|_| client_error("Invalid manifest - could not SBOR decode"))?;

    let signer_public_keys = request
        .signer_public_keys
        .into_iter()
        .map(|pk| {
            extract_api_public_key(pk).map_err(|err| err.into_response_error("signer_public_keys"))
        })
        .collect::<Result<Vec<PublicKey>, RequestHandlingError>>()?;

    Ok(PreviewRequest {
        manifest,
        cost_unit_limit: extract_api_u32_as_i64(request.cost_unit_limit)
            .map_err(|err| err.into_response_error("cost_unit_limit"))?,
        tip_percentage: extract_api_u32_as_i64(request.tip_percentage)
            .map_err(|err| err.into_response_error("tip_percentage"))?,
        nonce: extract_api_u64_as_string(request.nonce)
            .map_err(|err| err.into_response_error("nonce"))?,
        signer_public_keys,
        flags: PreviewFlags {
            unlimited_loan: request.flags.unlimited_loan,
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
                .map(|v| models::ResourceChange {
                    resource_address: bech32_encoder.encode_resource_address(&v.resource_address),
                    component_address: bech32_encoder
                        .encode_component_address(&v.component_address),
                    vault_entity_id: Box::new(to_vault_entity_id(&v.vault_id).into()),
                    amount_attos: to_api_decimal_attos(&v.amount),
                })
                .collect();

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
                fee_summary: Box::new(to_api_fee_summary(receipt.execution.fee_summary)),
                state_updates: Box::new(models::StateUpdates {
                    down_virtual_substates: vec![],
                    up_substates: vec![],
                    down_substates: vec![],
                    new_global_entities: vec![],
                }),
                output: None,
                error_message: Some(format!("{:?}", reject_result)),
            }),
            resource_changes: vec![],
            logs,
        },
    };

    Ok(response)
}
