use crate::core_api::models::*;
use crate::core_api::*;
use axum::{Extension, Json};
use radix_engine::transaction::{PreviewResult, TransactionResult};
use scrypto::address::Bech32Encoder;
use scrypto::crypto::EcdsaPublicKey;
use scrypto::prelude::scrypto_decode;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{LedgerTransactionReceipt, PreviewRequest};
use transaction::model::{PreviewFlags, TransactionManifest};

pub(crate) async fn handle_transaction_preview(
    state: Extension<CoreApiState>,
    request: Json<TransactionPreviewRequest>,
) -> Result<Json<TransactionPreviewResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_preview_internal)
}

fn handle_preview_internal(
    state_manager: &mut ActualStateManager,
    request: TransactionPreviewRequest,
) -> Result<TransactionPreviewResponse, RequestHandlingError> {
    let preview_request = parse_preview_request(request)?;

    let result = state_manager
        .preview(preview_request)
        .map_err(preview_errors::preview_error)?;

    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    to_api_response(result, &bech32_encoder)
}

fn parse_preview_request(
    request: TransactionPreviewRequest,
) -> Result<PreviewRequest, RequestHandlingError> {
    let manifest = hex::decode(request.manifest)
        .map(|manifest_bytes| {
            scrypto_decode::<TransactionManifest>(&manifest_bytes)
                .map_err(|_| preview_errors::invalid_manifest())
        })
        .map_err(|_| preview_errors::invalid_manifest())??;

    let signer_public_keys: Vec<EcdsaPublicKey> = request
        .signer_public_keys
        .into_iter()
        .flat_map(|s| {
            hex::decode(s.clone()).map(|pub_key_bytes| {
                EcdsaPublicKey::try_from(&pub_key_bytes[..])
                    .map_err(|_| preview_errors::invalid_signer_pub_key(&s))
            })
        })
        .collect::<Result<Vec<EcdsaPublicKey>, RequestHandlingError>>()?;

    Ok(PreviewRequest {
        manifest,
        cost_unit_limit: request.cost_unit_limit,
        tip_percentage: request.tip_percentage,
        nonce: request.nonce,
        signer_public_keys,
        flags: PreviewFlags {
            unlimited_loan: request.flags.unlimited_loan,
        },
    })
}

fn to_api_response(
    result: PreviewResult,
    bech32_encoder: &Bech32Encoder,
) -> Result<TransactionPreviewResponse, RequestHandlingError> {
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
                    vault_id: to_sbor_hex(&v.vault_id),
                    amount: v.amount.to_string(),
                })
                .collect();

            let ledger_receipt: LedgerTransactionReceipt = receipt.try_into().map_err(|_| {
                common_server_errors::unexpected_state("can't create a ledger receipt")
            })?;

            TransactionPreviewResponse {
                receipt: Box::new(
                    to_api_receipt(bech32_encoder, ledger_receipt).map_err(|_| {
                        common_server_errors::mapping_error("Unable to map receipt")
                    })?,
                ),
                resource_changes: api_resource_changes,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => TransactionPreviewResponse {
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

mod preview_errors {
    use crate::core_api::errors::{client_error, server_error, RequestHandlingError};
    use radix_engine::transaction::PreviewError;

    pub(crate) fn preview_error(err: PreviewError) -> RequestHandlingError {
        match err {
            PreviewError::TransactionValidationError(err) => {
                server_error(500, &format!("Transaction validation error: {:?}", err))
            }
        }
    }

    pub(crate) fn invalid_manifest() -> RequestHandlingError {
        client_error(400, "Invalid manifest")
    }

    pub(crate) fn invalid_signer_pub_key(raw_key: &str) -> RequestHandlingError {
        client_error(400, &format!("Invalid signer public key: {}", raw_key))
    }
}
