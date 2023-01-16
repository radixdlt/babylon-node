use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use radix_engine_interface::node::NetworkDefinition;

/// A marker trait for custom error details
pub trait ErrorDetails: serde::Serialize + std::fmt::Debug {}

impl ErrorDetails for () {}

/// Note - We create our own generic error response model instead of using
/// the auto-generated ones, to enable us to genericize the error handling logic.
/// We should ensure manually that the types match this model.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct ErrorResponse<E: ErrorDetails> {
    /// A numeric code corresponding to the given HTTP error code.
    #[serde(rename = "code")]
    pub code: i32,
    /// A human-readable error message.
    #[serde(rename = "message")]
    pub message: String,
    /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
    #[serde(rename = "trace_id", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// The details of the error, if present.
    #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
    pub details: Option<E>,
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
        let body = ErrorResponse::<E> {
            code: self.status_code.as_u16() as i32,
            message: self.public_error_message,
            trace_id: self.trace.map(|x| x.0),
            details: self.details,
        };

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

// TODO - Replace ErrorResponse "code" with making them an Enum with different structured errors
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
