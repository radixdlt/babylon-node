use crate::prelude::*;

use super::{
    extract_preview_flags, to_api_receipt_logs, to_api_toolkit_receipt, to_rejection_receipt,
};

pub(crate) async fn handle_transaction_preview_v2(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionPreviewV2Request>,
) -> Result<
    Json<models::TransactionPreviewV2Response>,
    ResponseError<models::TransactionPreviewV2ErrorDetails>,
> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);

    let at_state_version = request
        .at_ledger_state
        .as_deref()
        .map(extract_ledger_state_selector)
        .transpose()
        .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let options = request.options.as_ref();
    let settings = PreviewV2Settings {
        include_radix_engine_toolkit_receipt: options
            .and_then(|o| o.radix_engine_toolkit_receipt)
            .unwrap_or(false),
        include_core_api_receipt: options.and_then(|o| o.core_api_receipt).unwrap_or(true),
        include_logs: options.and_then(|o| o.logs).unwrap_or(false),
    };

    let preview_flags = extract_preview_flags(request.flags.as_ref().map(|f| &**f));
    let disable_auth = preview_flags.disable_auth;

    let preview_executable =
        extract_preview_executable(&state.state_manager, preview_flags, &request)?;

    let result = state
        .state_manager
        .transaction_previewer
        .preview_executable(preview_executable, disable_auth, at_state_version)?;

    to_api_response(&mapping_context, result, settings).map(Json)
}

fn extract_preview_executable(
    state_manager: &StateManager,
    flags: PreviewFlags,
    request: &models::TransactionPreviewV2Request,
) -> Result<ExecutableTransaction, ResponseError<models::TransactionPreviewV2ErrorDetails>> {
    let raw_preview_transaction = match &request.preview_transaction {
        Some(models::PreviewTransaction::CompiledPreviewTransaction {
            preview_transaction_hex,
        }) => from_hex(preview_transaction_hex).map(RawPreviewTransaction::from_vec),
        None => Err(ExtractionError::MissingField),
    }
    .map_err(|err| err.into_response_error("preview_transaction"))?;

    validate_preview_transaction(state_manager, flags, raw_preview_transaction).map_err(|err| {
        let status_code = StatusCode::BAD_REQUEST;
        let public_message = "";
        let details =
            models::TransactionPreviewV2ErrorDetails::InvalidTransactionPreviewV2ErrorDetails {
                validation_error: format!("{:?}", err),
            };

        detailed_error(status_code, public_message, details)
    })
}

fn validate_preview_transaction(
    state_manager: &StateManager,
    flags: PreviewFlags,
    raw: RawPreviewTransaction,
) -> Result<ExecutableTransaction, TransactionValidationError> {
    let preview_transaction = PreviewTransactionV2::from_raw(&raw)
        .map_err(|err| TransactionValidationError::PrepareError(PrepareError::DecodeError(err)))?;
    let validated = preview_transaction
        .prepare_and_validate(state_manager.transaction_validator.read().deref())?;
    Ok(validated.create_executable(flags))
}

struct PreviewV2Settings {
    include_radix_engine_toolkit_receipt: bool,
    include_core_api_receipt: bool,
    include_logs: bool,
}

fn to_api_response(
    context: &MappingContext,
    result: ProcessedPreviewResult,
    settings: PreviewV2Settings,
) -> Result<
    models::TransactionPreviewV2Response,
    ResponseError<models::TransactionPreviewV2ErrorDetails>,
> {
    let engine_receipt = result.receipt;

    // Produce a toolkit transaction receipt for the transaction preview if it was requested in the
    // request opt-ins.
    let toolkit_receipt = if settings.include_radix_engine_toolkit_receipt {
        let receipt = to_api_toolkit_receipt(context, engine_receipt.clone())
            .ok_or(server_error("Can't produce toolkit transaction receipt."))?;
        Some(receipt)
    } else {
        None
    };

    let at_ledger_state = Box::new(to_api_ledger_state_summary(
        context,
        &result.base_ledger_state,
    )?);

    let execution_fee_data = ExecutionFeeData {
        fee_summary: engine_receipt.fee_summary,
        engine_costing_parameters: engine_receipt.costing_parameters,
        transaction_costing_parameters: engine_receipt.transaction_costing_parameters,
    };

    let response = match engine_receipt.result {
        TransactionResult::Commit(commit_result) => {
            let logs = if settings.include_logs {
                Some(to_api_receipt_logs(&commit_result))
            } else {
                None
            };

            let receipt = if settings.include_core_api_receipt {
                Some(Box::new(to_api_receipt(
                    None::<&ActualStateManagerDatabase>,
                    context,
                    LocalTransactionReceipt::new(
                        commit_result,
                        result.state_changes,
                        result.global_balance_summary,
                        execution_fee_data,
                    ),
                )?))
            } else {
                None
            };

            models::TransactionPreviewV2Response {
                at_ledger_state,
                receipt,
                radix_engine_toolkit_receipt: toolkit_receipt,
                logs,
            }
        }
        TransactionResult::Reject(reject_result) => {
            let receipt = if settings.include_core_api_receipt {
                Some(Box::new(to_rejection_receipt(
                    context,
                    execution_fee_data,
                    reject_result,
                )?))
            } else {
                None
            };

            let logs = if settings.include_logs {
                Some(vec![])
            } else {
                None
            };
            models::TransactionPreviewV2Response {
                at_ledger_state,
                receipt,
                radix_engine_toolkit_receipt: toolkit_receipt,
                logs,
            }
        }
        TransactionResult::Abort(_) => {
            panic!("Should not be aborting");
        }
    };

    Ok(response)
}
