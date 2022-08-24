use crate::core_api::generated::models::*;
use crate::core_api::generated::{models, TransactionPreviewPostResponse};
use radix_engine::transaction::{PreviewResult, TransactionStatus};
use scrypto::address::Bech32Encoder;
use scrypto::crypto::EcdsaPublicKey;
use scrypto::prelude::scrypto_decode;
use state_manager::{PreviewRequest, StateManager};
use std::sync::{Arc, Mutex};
use swagger::ApiError;
use transaction::model::{PreviewFlags, TransactionManifest};

pub(crate) fn handle_preview(
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
    request: TransactionPreviewRequest,
) -> Result<TransactionPreviewPostResponse, ApiError> {
    handle_preview_internal(state_manager, request)
        .map(TransactionPreviewPostResponse::TransactionPreviewResponse)
        .or_else(Ok)
}

fn handle_preview_internal(
    state_manager: Arc<Mutex<dyn StateManager + Send + Sync>>,
    request: TransactionPreviewRequest,
) -> Result<TransactionPreviewResponse, TransactionPreviewPostResponse> {
    let preview_request = parse_preview_request(request)?;

    let mut locked_state_manager = state_manager
        .lock()
        .map_err(|_| server_error("Internal server error (state manager lock)"))?;

    let result = locked_state_manager
        .preview(preview_request)
        .map_err(|e| client_error(&format!("{:?}", e)))?;

    let bech32_encoder = Bech32Encoder::new_from_network(locked_state_manager.network());

    to_api_response(result, bech32_encoder)
}

fn parse_preview_request(
    request: TransactionPreviewRequest,
) -> Result<PreviewRequest, TransactionPreviewPostResponse> {
    let cost_unit_limit: u32 = request
        .cost_unit_limit
        .try_into()
        .map_err(|_| client_error("Invalid cost_unit_limit"))?;

    let tip_percentage: u32 = request
        .tip_percentage
        .try_into()
        .map_err(|_| client_error("Invalid tip_percentage"))?;

    let nonce: u64 = request
        .nonce
        .try_into()
        .map_err(|_| client_error("Invalid nonce"))?;

    let manifest = hex::decode(request.manifest)
        .map(|manifest_bytes| {
            scrypto_decode::<TransactionManifest>(&manifest_bytes)
                .map_err(|_| client_error("Invalid manifest"))
        })
        .map_err(|_| client_error("Invalid manifest (malformed hex)"))??;

    let signer_public_keys: Vec<EcdsaPublicKey> = request
        .signer_public_keys
        .into_iter()
        .flat_map(|s| {
            hex::decode(s).map(|pub_key_bytes| {
                EcdsaPublicKey::try_from(&pub_key_bytes[..])
                    .map_err(|_| client_error("Invalid signer public key"))
                    .map_err(|_| client_error("Invalid signer public key (malformed hex)"))
            })
        })
        .collect::<Result<Vec<EcdsaPublicKey>, TransactionPreviewPostResponse>>()?;

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
) -> Result<TransactionPreviewResponse, TransactionPreviewPostResponse> {
    let (status, output, error_message) = match result.receipt.status {
        TransactionStatus::Succeeded(output) => {
            let output_hex = output.into_iter().map(hex::encode).collect();
            (models::TransactionStatus::SUCCEEDED, Some(output_hex), None)
        }
        TransactionStatus::Failed(error) => (
            models::TransactionStatus::FAILED,
            None,
            Some(format!("{:?}", error)),
        ),
        TransactionStatus::Rejected => (models::TransactionStatus::REJECTED, None, None),
    };

    let fee_summary = &result.receipt.fee_summary;

    let logs = result
        .receipt
        .application_logs
        .into_iter()
        .map(|(level, message)| TransactionPreviewResponseLogsInner {
            level: level.to_string(),
            message,
        })
        .collect();

    let new_package_addresses = result
        .receipt
        .new_package_addresses
        .into_iter()
        .map(|addr| {
            bech32_encoder
                .encode_package_address(&addr)
                .map_err(|_| server_error("Unexpected error, failed to encode package address"))
        })
        .collect::<Result<Vec<String>, TransactionPreviewPostResponse>>()?;

    let new_component_addresses = result
        .receipt
        .new_component_addresses
        .into_iter()
        .map(|addr| {
            bech32_encoder
                .encode_component_address(&addr)
                .map_err(|_| server_error("Unexpected error, failed to encode component address"))
        })
        .collect::<Result<Vec<String>, TransactionPreviewPostResponse>>()?;

    let new_resource_addresses = result
        .receipt
        .new_resource_addresses
        .into_iter()
        .map(|addr| {
            bech32_encoder
                .encode_resource_address(&addr)
                .map_err(|_| server_error("Unexpected error, failed to encode resource address"))
        })
        .collect::<Result<Vec<String>, TransactionPreviewPostResponse>>()?;

    Ok(TransactionPreviewResponse {
        transaction_status: status,
        transaction_fee: FeeSummary {
            loan_fully_repaid: fee_summary.loan_fully_repaid,
            cost_unit_limit: fee_summary.cost_unit_limit.to_string(),
            cost_unit_consumed: fee_summary.cost_unit_consumed.to_string(),
            cost_unit_price: fee_summary.cost_unit_price.to_string(),
            tip_percentage: fee_summary.tip_percentage.to_string(),
            xrd_burned: fee_summary.burned.to_string(),
            xrd_tipped: fee_summary.tipped.to_string(),
        },
        logs,
        new_package_addresses,
        new_component_addresses,
        new_resource_addresses,
        output,
        error_message,
    })
}

fn client_error(message: &str) -> TransactionPreviewPostResponse {
    TransactionPreviewPostResponse::ClientError(ErrorResponse::new(400, message.to_string()))
}

fn server_error(message: &str) -> TransactionPreviewPostResponse {
    TransactionPreviewPostResponse::ServerError(ErrorResponse::new(500, message.to_string()))
}
