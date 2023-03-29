use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use radix_engine_interface::network::NetworkDefinition;

use super::models;
use models::{transaction_submit_error_details::TransactionSubmitErrorDetails, lts_transaction_submit_error_details::LtsTransactionSubmitErrorDetails};

/// A marker trait for custom error details
pub trait ErrorDetails: serde::Serialize + std::fmt::Debug + Sized {
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

        (self.status_code, Json(body)).into_response()
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

pub(crate) fn not_implemented<E: ErrorDetails>(message: impl Into<String>) -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::NOT_IMPLEMENTED,
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