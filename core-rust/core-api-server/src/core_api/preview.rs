use crate::core_api::conversions::{to_api_fee_summary, to_api_receipt, to_sbor_hex};
use crate::core_api::errors::{common_server_errors, RequestHandlingError};
use crate::core_api::generated::models::{TransactionPreviewRequest, TransactionPreviewResponse};
use crate::core_api::generated::{models, TransactionPreviewPostResponse};
use radix_engine::transaction::{PreviewResult, TransactionResult};
use scrypto::address::Bech32Encoder;
use scrypto::crypto::EcdsaPublicKey;
use scrypto::prelude::scrypto_decode;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{LedgerTransactionReceipt, PreviewRequest};
use std::sync::{Arc, Mutex};
use transaction::model::{PreviewFlags, TransactionManifest};

pub(crate) fn handle_preview(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: TransactionPreviewRequest,
) -> TransactionPreviewPostResponse {
    match handle_preview_internal(state_manager, request) {
        Ok(response) => TransactionPreviewPostResponse::TransactionPreviewResponse(response),
        Err(RequestHandlingError::ClientError(error_response)) => {
            TransactionPreviewPostResponse::ClientError(error_response)
        }
        Err(RequestHandlingError::ServerError(error_response)) => {
            TransactionPreviewPostResponse::ServerError(error_response)
        }
    }
}

fn handle_preview_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: TransactionPreviewRequest,
) -> Result<TransactionPreviewResponse, RequestHandlingError> {
    let preview_request = parse_preview_request(request)?;

    let mut locked_state_manager = state_manager
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

    let result = locked_state_manager
        .preview(preview_request)
        .map_err(preview_errors::engine_error)?;

    let bech32_encoder = Bech32Encoder::new(&locked_state_manager.network);

    to_api_response(result, &bech32_encoder)
}

fn parse_preview_request(
    request: TransactionPreviewRequest,
) -> Result<PreviewRequest, RequestHandlingError> {
    let cost_unit_limit: u32 = request
        .cost_unit_limit
        .parse()
        .map_err(|_| preview_errors::invalid_int_field("cost_unit_limit"))?;

    let tip_percentage: u32 = request
        .tip_percentage
        .parse()
        .map_err(|_| preview_errors::invalid_int_field("tip_percentage"))?;

    let nonce: u64 = request
        .nonce
        .parse()
        .map_err(|_| preview_errors::invalid_int_field("nonce"))?;

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
        cost_unit_limit,
        tip_percentage,
        nonce,
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
                receipt: to_api_receipt(bech32_encoder, ledger_receipt),
                resource_changes: api_resource_changes,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => TransactionPreviewResponse {
            receipt: models::TransactionReceipt {
                status: models::TransactionStatus::REJECTED,
                fee_summary: to_api_fee_summary(receipt.execution.fee_summary),
                state_updates: models::StateUpdates {
                    down_virtual_substates: vec![],
                    up_substates: vec![],
                    down_substates: vec![],
                    new_roots: vec![],
                },
                new_package_addresses: vec![],
                new_component_addresses: vec![],
                new_resource_addresses: vec![],
                output: None,
                error_message: Some(format!("{:?}", reject_result)),
            },
            resource_changes: vec![],
            logs,
        },
    };

    Ok(response)
}

mod preview_errors {
    use crate::core_api::errors::{client_error, RequestHandlingError};
    use radix_engine::transaction::PreviewError;

    pub(crate) fn engine_error(err: PreviewError) -> RequestHandlingError {
        client_error(11, &format!("Engine error: {:?}", err))
    }

    pub(crate) fn invalid_int_field(field: &str) -> RequestHandlingError {
        client_error(12, &format!("Invalid integer: {}", field))
    }

    pub(crate) fn invalid_manifest() -> RequestHandlingError {
        client_error(13, "Invalid manifest")
    }

    pub(crate) fn invalid_signer_pub_key(raw_key: &str) -> RequestHandlingError {
        client_error(14, &format!("Invalid signer public key: {}", raw_key))
    }
}
