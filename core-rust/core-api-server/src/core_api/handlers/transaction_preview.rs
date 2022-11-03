use crate::core_api::{generated::models::TransactionReadcallRequestTarget, *};
use models::transaction_readcall_response::TransactionReadcallResponse;
use radix_engine::{
    transaction::{PreviewError, PreviewResult, TransactionOutcome, TransactionResult},
    types::{
        Bech32Decoder, Bech32Encoder, ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoPackage,
        ScryptoReceiver,
    },
};
use scrypto::prelude::*;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::{LedgerTransactionReceipt, PreviewRequest};
use transaction::manifest;
use transaction::model::{Instruction, PreviewFlags, TransactionManifest, DEFAULT_COST_UNIT_LIMIT};

#[tracing::instrument(level = "debug", skip_all, err(Debug))]
pub(crate) async fn handle_transaction_readcall(
    Extension(state): Extension<CoreApiState>,
    Json(request): Json<models::TransactionReadcallRequest>,
) -> Result<Json<models::TransactionReadcallResponse>, RequestHandlingError> {
    let mut state_manager = state.state_manager.write();
    let bech32_decoder = Bech32Decoder::new(&state_manager.network);
    let bech32_encoder = Bech32Encoder::new(&state_manager.network);

    let args: Vec<_> = request
        .arguments
        .into_iter()
        .map(from_hex)
        .collect::<Result<_, _>>()
        .map_err(|_| client_error("Invalid hex character in arguments"))?;

    let call_target = request
        .target
        .ok_or_else(|| client_error("The `target` property should be present."))?;

    let requested_call = match call_target {
        TransactionReadcallRequestTarget::BlueprintFunctionIdentifier {
            package_address,
            blueprint_name,
            function_name,
        } => {
            let package_address =
                extract_package_address(&bech32_decoder, package_address.as_str())
                    .map_err(|extraction_error| server_error(format!("{extraction_error:?}")))?;

            Instruction::CallFunction {
                function_ident: ScryptoFunctionIdent {
                    package: ScryptoPackage::Global(package_address),
                    blueprint_name,
                    function_name,
                },
                args: args_from_bytes_vec!(args),
            }
        }
        TransactionReadcallRequestTarget::MethodIdentifier {
            component_address,
            method_name,
        } => {
            let component_address =
                extract_component_address(&bech32_decoder, component_address.as_str())
                    .map_err(|extraction_error| server_error(format!("{extraction_error:?}")))?;

            Instruction::CallMethod {
                method_ident: ScryptoMethodIdent {
                    receiver: ScryptoReceiver::Global(component_address),
                    method_name,
                },
                args: args_from_bytes_vec!(args),
            }
        }
    };

    let result = state_manager
        .preview(PreviewRequest {
            cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
            tip_percentage: 0,
            // TODO confirm this nonce, make this an actual nonce
            nonce: 490,
            manifest: TransactionManifest {
                instructions: vec![
                    Instruction::CallMethod {
                        method_ident: ScryptoMethodIdent {
                            receiver: ScryptoReceiver::Global(SYS_FAUCET_COMPONENT),
                            method_name: "lock_fee".to_string(),
                        },
                        args: args!(Decimal::from(100u32)),
                    },
                    requested_call,
                ],
                // TODO confirm this
                blobs: vec![],
            },
            flags: PreviewFlags {
                unlimited_loan: true,
                assume_all_signature_proofs: true,
            },
            // TODO confirm this
            signer_public_keys: vec![],
        })
        .map_err(|err| match err {
            PreviewError::TransactionValidationError(err) => {
                server_error(format!("Transaction validation error: {:?}", err))
            }
        })?;

    Ok(TransactionReadcallResponse {
        output: match result.receipt.result {
            TransactionResult::Commit(c) => match c.outcome {
                TransactionOutcome::Success(data) => data
                    .into_iter()
                    .skip(1) // Skip the result of `lock_fee`
                    .map(|line_output| {
                        scrypto_bytes_to_api_sbor_data(&bech32_encoder, &line_output)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .ok(),
                TransactionOutcome::Failure(f) => Err(server_error(format!("{f}")))?,
            },
            TransactionResult::Reject(r) => Err(server_error(format!("{:?}", r.error)))?,
        },
    })
    .map(Json)
}

pub(crate) async fn handle_transaction_preview(
    state: Extension<CoreApiState>,
    request: Json<models::TransactionPreviewRequest>,
) -> Result<Json<models::TransactionPreviewResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_preview_internal)
}

#[tracing::instrument(level = "debug", skip(state_manager), err(Debug))]
fn handle_preview_internal(
    state_manager: &mut ActualStateManager,
    request: models::TransactionPreviewRequest,
) -> Result<models::TransactionPreviewResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let preview_request = parse_preview_request(&state_manager.network, request)?;

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
        .map(|pk| extract_api_public_key(pk))
        .collect::<Result<_, _>>()
        .map_err(|err| err.into_response_error("signer_public_keys"))?;

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
            assume_all_signature_proofs: request.flags.assume_all_signature_proofs,
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
                    resource_address: bech32_encoder
                        .encode_resource_address_to_string(&v.resource_address),
                    component_entity: Box::new(to_entity_reference(
                        models::EntityType::Component,
                        &v.component_id,
                    )),
                    vault_entity: Box::new(to_entity_reference(
                        models::EntityType::Vault,
                        &v.vault_id,
                    )),
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
