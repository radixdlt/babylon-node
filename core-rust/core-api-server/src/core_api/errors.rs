use axum::body::BoxBody;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use models::stream_proofs_error_details::StreamProofsErrorDetails;
use std::any::Any;

use crate::engine_prelude::*;
use hyper::StatusCode;
use tower_http::catch_panic::ResponseForPanic;

use super::{models, CoreApiState};
use crate::core_api::models::StreamTransactionsErrorDetails;
use models::{
    lts_transaction_submit_error_details::LtsTransactionSubmitErrorDetails,
    transaction_submit_error_details::TransactionSubmitErrorDetails,
};
use state_manager::historical_state::StateHistoryError;
use state_manager::transaction::PreviewerError;

/// A marker trait for custom error details
pub trait ErrorDetails: serde::Serialize + Debug + Sized {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse;
}

impl ErrorDetails for () {
    fn to_error_response(
        _details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::BasicErrorResponse {
            code,
            message,
            trace_id,
        }
    }
}

impl ErrorDetails for TransactionSubmitErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::TransactionSubmitErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for LtsTransactionSubmitErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::LtsTransactionSubmitErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for StreamTransactionsErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::StreamTransactionsErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for StreamProofsErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::StreamProofsErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InternalServerErrorResponseForPanic;

impl ResponseForPanic for InternalServerErrorResponseForPanic {
    type ResponseBody = BoxBody;

    fn response_for_panic(
        &mut self,
        _panic_payload: Box<dyn Any + Send + 'static>,
    ) -> Response<Self::ResponseBody> {
        // Please note that we deliberately do *not*:
        // - log the panic payload (since the default panic handler already does this);
        // - include the panic payload in the response (it may contain sensitive details).
        server_error::<()>("Unexpected server error").into_response()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseError<E: ErrorDetails> {
    status_code: StatusCode,
    public_error_message: String,
    trace: Option<LogTraceId>,
    details: Option<E>,
}

#[derive(Debug, Clone)]
pub struct LogTraceId(pub String);

impl<E: ErrorDetails> IntoResponse for ResponseError<E> {
    fn into_response(self) -> Response {
        let body = E::to_error_response(
            self.details,
            self.status_code.as_u16() as i32,
            self.public_error_message,
            self.trace.map(|x| x.0),
        );

        let mut response = (self.status_code, Json(body.clone())).into_response();
        response.extensions_mut().insert(body);
        response
    }
}

pub(crate) fn assert_matching_network<E: ErrorDetails>(
    request_network: &str,
    network_definition: &NetworkDefinition,
) -> Result<(), ResponseError<E>> {
    if request_network != network_definition.logical_name {
        return Err(client_error(format!(
            "Invalid network - the network is actually: {}",
            network_definition.logical_name
        )));
    }
    Ok(())
}

pub(crate) fn assert_unbounded_endpoints_flag_enabled<E: ErrorDetails>(
    state: &CoreApiState,
) -> Result<(), ResponseError<E>> {
    if !state.flags.enable_unbounded_endpoints {
        return Err(client_error(
            "This endpoint is disabled as the response is potentially unbounded, and this node is configured with `enable_unbounded_endpoints` false.",
        ));
    }
    Ok(())
}

impl<E: ErrorDetails> From<PreviewerError> for ResponseError<E> {
    fn from(error: PreviewerError) -> Self {
        client_error(match error {
            PreviewerError::FromEngine(error) => match error {
                PreviewError::TransactionValidationError(error) => {
                    format!("Transaction validation error: {error:?}")
                }
            }
            PreviewerError::FromStateHistory(error) => match error {
                StateHistoryError::StateHistoryDisabled => {
                    "State history feature must be enabled (see the `db.historical_substate_values.enable` Node configuration flag)".to_string()
                }
                StateHistoryError::StateVersionInTooDistantPast { first_available_version } => {
                    format!("Cannot request state version past the earliest available {} (see the `state_hash_tree.state_version_history_length` Node configuration flag)", first_available_version)
                }
                StateHistoryError::StateVersionInFuture { current_version } => {
                    format!("Cannot request state version ahead of the current top-of-ledger {}", current_version)
                }
            }
        })
    }
}

// TODO - Add logging, metrics and tracing for all of these errors - require the error is passed in here
pub(crate) fn client_error<E: ErrorDetails>(message: impl Into<String>) -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::BAD_REQUEST,
        public_error_message: message.into(),
        trace: None,
        details: None,
    }
}

pub(crate) fn not_found_error<E: ErrorDetails>(message: impl Into<String>) -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::NOT_FOUND,
        public_error_message: message.into(),
        trace: None,
        details: None,
    }
}

pub(crate) fn server_error<E: ErrorDetails>(public_message: impl Into<String>) -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        public_error_message: public_message.into(),
        trace: None,
        details: None,
    }
}

pub(crate) fn detailed_error<E: ErrorDetails>(
    status_code: StatusCode,
    public_message: impl Into<String>,
    details: impl Into<E>,
) -> ResponseError<E> {
    ResponseError {
        status_code,
        public_error_message: public_message.into(),
        trace: None,
        details: Some(details.into()),
    }
}

pub(crate) fn length_limit_error<E: ErrorDetails>() -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::PAYLOAD_TOO_LARGE,
        public_error_message: "length limit exceeded".into(),
        trace: None,
        details: None,
    }
}
