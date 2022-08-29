use crate::core_api::errors::{common_server_errors, RequestHandlingError};
use crate::core_api::generated::models::*;
use crate::core_api::generated::TransactionPreviewPostResponse;
use radix_engine::transaction::{PreviewResult, TransactionOutcome, TransactionResult};
use scrypto::address::Bech32Encoder;
use scrypto::crypto::EcdsaPublicKey;
use scrypto::prelude::scrypto_decode;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::PreviewRequest;
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

    to_api_response(result, bech32_encoder)
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
    bech32_encoder: Bech32Encoder,
) -> Result<TransactionPreviewResponse, RequestHandlingError> {
    let execution = result.receipt.execution;
    let fee_summary = execution.fee_summary;

    let api_fee_summary = FeeSummary {
        loan_fully_repaid: fee_summary.loan_fully_repaid,
        cost_unit_limit: fee_summary.cost_unit_limit.to_string(),
        cost_unit_consumed: fee_summary.cost_unit_consumed.to_string(),
        cost_unit_price: fee_summary.cost_unit_price.to_string(),
        tip_percentage: fee_summary.tip_percentage.to_string(),
        xrd_burned: fee_summary.burned.to_string(),
        xrd_tipped: fee_summary.tipped.to_string(),
    };

    let logs = execution
        .application_logs
        .into_iter()
        .map(|(level, message)| TransactionPreviewResponseLogsInner {
            level: level.to_string(),
            message,
        })
        .collect();

    let response = match result.receipt.result {
        TransactionResult::Commit(commit_result) => {
            let new_package_addresses = commit_result
                .entity_changes
                .new_package_addresses
                .into_iter()
                .map(|addr| bech32_encoder.encode_package_address(&addr))
                .collect();

            let new_component_addresses = commit_result
                .entity_changes
                .new_component_addresses
                .into_iter()
                .map(|addr| bech32_encoder.encode_component_address(&addr))
                .collect();

            let new_resource_addresses = commit_result
                .entity_changes
                .new_resource_addresses
                .into_iter()
                .map(|addr| bech32_encoder.encode_resource_address(&addr))
                .collect();

            let (status, output, error_message) = match commit_result.outcome {
                TransactionOutcome::Success(output) => {
                    let output_hex = output.into_iter().map(hex::encode).collect();
                    (TransactionStatus::SUCCEEDED, Some(output_hex), None)
                }
                TransactionOutcome::Failure(error) => (
                    TransactionStatus::FAILED,
                    None,
                    Some(format!("{:?}", error)),
                ),
            };

            TransactionPreviewResponse {
                transaction_status: status,
                transaction_fee: api_fee_summary,
                logs,
                new_package_addresses,
                new_component_addresses,
                new_resource_addresses,
                output,
                error_message,
            }
        }
        TransactionResult::Reject(reject_result) => TransactionPreviewResponse {
            transaction_status: TransactionStatus::REJECTED,
            transaction_fee: api_fee_summary,
            logs,
            new_package_addresses: vec![],
            new_component_addresses: vec![],
            new_resource_addresses: vec![],
            output: None,
            error_message: Some(format!("{:?}", reject_result)),
        },
    };

    Ok(response)
}

mod preview_errors {
    use crate::core_api::errors::{client_error, RequestHandlingError};
    use radix_engine::transaction::PreviewError;

    pub(crate) fn engine_error(err: PreviewError) -> RequestHandlingError {
        client_error(1, &format!("Engine error: {:?}", err))
    }

    pub(crate) fn invalid_int_field(field: &str) -> RequestHandlingError {
        client_error(2, &format!("Invalid integer: {}", field))
    }

    pub(crate) fn invalid_manifest() -> RequestHandlingError {
        client_error(3, "Invalid manifest")
    }

    pub(crate) fn invalid_signer_pub_key(raw_key: &str) -> RequestHandlingError {
        client_error(4, &format!("Invalid signer public key: {}", raw_key))
    }
}
