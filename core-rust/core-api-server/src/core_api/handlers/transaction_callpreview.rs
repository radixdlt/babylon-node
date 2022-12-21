use crate::core_api::*;
use models::{
    transaction_call_preview_response::TransactionCallPreviewResponse,
    transaction_status::TransactionStatus, TransactionCallPreviewRequestTarget,
};
use radix_engine::{
    transaction::{PreviewError, TransactionOutcome, TransactionResult},
    types::{Bech32Decoder, Bech32Encoder, Decimal, FAUCET_COMPONENT},
};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::args;
use state_manager::PreviewRequest;
use transaction::args_from_bytes_vec;
use transaction::model::{BasicInstruction, PreviewFlags, TransactionManifest};

#[tracing::instrument(level = "debug", skip_all, err(Debug))]
pub(crate) async fn handle_transaction_callpreview(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::TransactionCallPreviewRequest>,
) -> Result<Json<models::TransactionCallPreviewResponse>, RequestHandlingError> {
    let state_manager = state.state_manager.read();
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let args: Vec<_> = request
        .arguments
        .into_iter()
        .map(from_hex)
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("arguments"))?;

    let call_target = request
        .target
        .ok_or_else(|| client_error("Missing target from request".to_string()))?;

    let requested_call = match call_target {
        TransactionCallPreviewRequestTarget::BlueprintFunctionIdentifier {
            package_address,
            blueprint_name,
            function_name,
        } => {
            let package_address =
                extract_package_address(&bech32_decoder, package_address.as_str())
                    .map_err(|err| err.into_response_error("target.package_address"))?;

            BasicInstruction::CallFunction {
                blueprint_name,
                function_name,
                package_address,
                args: args_from_bytes_vec!(args),
            }
        }
        TransactionCallPreviewRequestTarget::ComponentMethodIdentifier {
            component_address,
            method_name,
        } => {
            let component_address =
                extract_component_address(&bech32_decoder, component_address.as_str())
                    .map_err(|err| err.into_response_error("target.component_address"))?;

            BasicInstruction::CallMethod {
                component_address,
                method_name,
                args: args_from_bytes_vec!(args),
            }
        }
    };

    let epoch = state_manager.get_epoch();

    let result = state_manager
        .preview(PreviewRequest {
            manifest: TransactionManifest {
                instructions: vec![
                    BasicInstruction::CallMethod {
                        component_address: FAUCET_COMPONENT,
                        method_name: "lock_fee".to_string(),
                        args: args!(Decimal::from(100u32)),
                    },
                    requested_call,
                ],
                blobs: vec![],
            },
            start_epoch_inclusive: epoch,
            end_epoch_exclusive: epoch + 100,
            notary_public_key: None,
            notary_as_signatory: true,
            cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
            tip_percentage: 0,
            nonce: 490,
            signer_public_keys: vec![],
            flags: PreviewFlags {
                permit_invalid_header_epoch: true,
                permit_duplicate_intent_hash: true,
                unlimited_loan: true,
                assume_all_signature_proofs: true,
            },
        })
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                server_error(format!("Transaction validation error: {:?}", err))
            }
        })?;

    let (status, output, error) = {
        match result.receipt.result {
            TransactionResult::Commit(c) => match c.outcome {
                TransactionOutcome::Success(data) => {
                    let output = match data
                        .into_iter()
                        .skip(1) // Skip the result of `lock_fee`
                        .map(|line_output| {
                            scrypto_bytes_to_api_sbor_data(&bech32_encoder, &line_output.as_vec())
                        })
                        .next()
                    {
                        None => None,
                        Some(Ok(output)) => Some(output),
                        // Decoding engine response should succeed
                        Some(Err(err)) => Err(server_error(format!("{:?}", err)))?,
                    };

                    (TransactionStatus::Succeeded, output, None)
                }
                TransactionOutcome::Failure(f) => {
                    (TransactionStatus::Failed, None, Some(format!("{f:?}")))
                }
            },
            TransactionResult::Reject(r) => {
                (TransactionStatus::Rejected, None, Some(format!("{r:?}")))
            }
        }
    };

    Ok(TransactionCallPreviewResponse {
        error_message: error,
        output: output.map(Box::new),
        status,
    })
    .map(Json)
}
