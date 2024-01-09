use crate::core_api::*;
use models::{
    target_identifier::TargetIdentifier,
    transaction_call_preview_response::TransactionCallPreviewResponse,
    transaction_status::TransactionStatus,
};
use radix_engine::{
    transaction::{PreviewError, TransactionOutcome, TransactionResult},
    types::Decimal,
};

use radix_engine_interface::constants::FAUCET;
use radix_engine_interface::manifest_args;
use radix_engine_interface::{
    blueprints::transaction_processor::InstructionOutput, data::scrypto::scrypto_encode,
};
use state_manager::PreviewRequest;
use transaction::prelude::*;

macro_rules! args_from_bytes_vec {
    ($args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(::radix_engine::types::manifest_decode(&arg).unwrap());
        }
        ::radix_engine::types::ManifestValue::Tuple { fields }
    }};
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn handle_transaction_callpreview(
    State(state): State<CoreApiState>,
    Json(request): Json<models::TransactionCallPreviewRequest>,
) -> Result<Json<models::TransactionCallPreviewResponse>, ResponseError<()>> {
    let extraction_context = ExtractionContext::new(&state.network);
    let mapping_context = MappingContext::new(&state.network);

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
        TargetIdentifier::BlueprintFunctionTargetIdentifier {
            package_address,
            blueprint_name,
            function_name,
        } => {
            let package_address =
                extract_package_address(&extraction_context, package_address.as_str())
                    .map_err(|err| err.into_response_error("target.package_address"))?;

            InstructionV1::CallFunction {
                blueprint_name,
                function_name,
                package_address: package_address.into(),
                args: args_from_bytes_vec!(args),
            }
        }
        TargetIdentifier::ComponentMethodTargetIdentifier {
            component_address,
            method_name,
        } => {
            let component_address =
                extract_component_address(&extraction_context, component_address.as_str())
                    .map_err(|err| err.into_response_error("target.component_address"))?;

            InstructionV1::CallMethod {
                address: component_address.into(),
                method_name,
                args: args_from_bytes_vec!(args),
            }
        }
    };

    let result = state
        .state_manager
        .transaction_previewer
        .read()
        .preview(PreviewRequest {
            manifest: TransactionManifestV1 {
                instructions: vec![
                    InstructionV1::CallMethod {
                        address: FAUCET.into(),
                        method_name: "lock_fee".to_string(),
                        args: manifest_args!(Decimal::from(100u32)).into(),
                    },
                    requested_call,
                ],
                blobs: btreemap!(),
            },
            explicit_epoch_range: None,
            notary_public_key: None,
            notary_is_signatory: true,
            tip_percentage: 0,
            nonce: 490,
            signer_public_keys: vec![],
            flags: PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: true,
                skip_epoch_check: true,
            },
            message: MessageV1::None,
        })
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                server_error(format!("Transaction validation error: {err:?}"))
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
                            let bytes = match line_output {
                                InstructionOutput::None => scrypto_encode(&()).unwrap(),
                                InstructionOutput::CallReturn(r) => r,
                            };
                            to_api_sbor_data_from_bytes(&mapping_context, &bytes)
                        })
                        .next()
                    {
                        None => None,
                        Some(Ok(output)) => Some(output),
                        // Decoding engine response should succeed
                        Some(Err(err)) => Err(server_error(format!("{err:?}")))?,
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
            TransactionResult::Abort(_) => {
                // TODO: Should remove this
                panic!("Should not be aborting");
            }
        }
    };

    Ok(TransactionCallPreviewResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &result.base_ledger_header,
        )?),
        error_message: error,
        output: output.map(Box::new),
        status,
    })
    .map(Json)
}
