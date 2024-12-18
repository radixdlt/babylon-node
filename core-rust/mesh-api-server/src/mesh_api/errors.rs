use crate::prelude::*;
use hyper::StatusCode;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::any::Any;
use strum::{Display, EnumIter, IntoEnumIterator};
use tower_http::catch_panic::ResponseForPanic;

#[derive(Debug, Clone, EnumIter, Display)]
#[repr(i32)]
pub(crate) enum ApiError {
    #[strum(serialize = "Endpoint not found")]
    EndpointNotFound = 1,
    #[strum(serialize = "Endpoint not supported")]
    EndpointNotSupported,
    #[strum(serialize = "Unexpected server error")]
    UnexpectedServerError,
    #[strum(serialize = "Invalid network")]
    InvalidNetwork,
    #[strum(serialize = "Invalid request")]
    InvalidRequest,
    #[strum(serialize = "Could not render response")]
    ResponseRenderingError,
    #[strum(serialize = "Invalid account")]
    InvalidAccount,
    #[strum(serialize = "Invalid currency")]
    InvalidCurrency,
    #[strum(serialize = "Transaction not found")]
    TransactionNotFound,
    #[strum(serialize = "Invalid number of signatures")]
    InvalidNumberOfSignatures,
    #[strum(serialize = "Invalid transaction")]
    InvalidTransaction,
    #[strum(serialize = "Invalid manifest instruction")]
    InvalidManifestInstruction,
    #[strum(serialize = "Named address not supported")]
    NamedAddressNotSupported,
    #[strum(serialize = "Instruction is not recognized")]
    UnrecognizedInstruction,
    #[strum(serialize = "Invalid number of public keys")]
    InvalidNumberOfPublicKeys,
    #[strum(serialize = "Invalid metadata")]
    InvalidMetadata,
    #[strum(serialize = "Invalid operation")]
    InvalidOperation,
    #[strum(serialize = "Invalid number of senders")]
    InvalidNumberOfSenders,
    #[strum(serialize = "Parent block not available")]
    ParentBlockNotAvailable,
    #[strum(serialize = "Invalid block identifier")]
    InvalidBlockIdentifier,
    #[strum(serialize = "Submit transaction error")]
    SubmitTransactionError,
    #[strum(serialize = "Get state history error")]
    GetStateHistoryError,
}

impl From<ApiError> for ResponseError {
    fn from(error: ApiError) -> Self {
        Self::new(error.clone() as i32, error.to_string(), false)
    }
}

pub fn list_available_api_errors() -> Vec<models::Error> {
    ApiError::iter()
        .map(|v| ResponseError::from(v).error)
        .collect()
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
        ResponseError::from(ApiError::UnexpectedServerError)
            .with_details("Panic caught during request-handling; see logged panic payload")
            .into_response()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseError {
    status_code: StatusCode,
    error: models::Error,
}

impl ResponseError {
    pub fn new(code: i32, error_message: impl Into<String>, retryable: bool) -> Self {
        Self {
            // "500" should be returned in case of unexpected error
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error: models::Error::new(code, error_message.into(), retryable),
        }
    }

    #[allow(unused)]
    pub fn retryable(self, retryable: bool) -> Self {
        Self {
            error: models::Error {
                retriable: retryable,
                ..self.error
            },
            ..self
        }
    }

    pub fn with_details(self, details_message: impl Into<String>) -> Self {
        Self {
            error: models::Error {
                details: Some(serde_json::json!({
                    "details_message": details_message.into(),
                })),
                ..self.error
            },
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn unique() -> Self {
        Self(
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ErrorResponseEvent {
    pub level: LogLevel,
    pub error: models::Error,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        let trace_id = TraceId::unique();
        let mut returned_body = self.error;

        if let Some(ref mut details) = returned_body.details {
            if let Some(map) = details.as_object_mut() {
                map.insert(
                    "trace_id".to_string(),
                    serde_json::Value::String(trace_id.0),
                );
            }
        } else {
            returned_body.details = Some(serde_json::json!({
                "trace_id": trace_id.0,
            }));
        }

        let error_response_event = ErrorResponseEvent {
            level: resolve_level(self.status_code),
            error: returned_body.clone(),
        };
        let mut framework_response = (self.status_code, Json(returned_body)).into_response();
        framework_response
            .extensions_mut()
            .insert(error_response_event);
        framework_response
    }
}

fn resolve_level(status_code: StatusCode) -> LogLevel {
    if status_code.is_server_error() {
        LogLevel::WARN
    } else {
        LogLevel::DEBUG
    }
}

/// A function to be used within a `map_response` layer in order to emit more customized events when
/// top-level `ErrorResponse` is returned.
/// In short, it is supposed to replace an `err(Debug)` within `#[tracing::instrument(...)]` of
/// every handler function which returns `Result<_, ResponseError<_>>`. It emits almost the same
/// information (except for emitting the path instead of the handler function name).
pub async fn emit_error_response_event(uri: Uri, response: Response) -> Response {
    let event = response.extensions().get::<ErrorResponseEvent>();
    if let Some(event) = event {
        let ErrorResponseEvent { level, error } = event;
        let internal_message = error
            .details
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "no_details".to_string())
            .to_string();
        match *level {
            LogLevel::TRACE => trace!(path = uri.path(), error = debug(error), internal_message),
            LogLevel::DEBUG => debug!(path = uri.path(), error = debug(error), internal_message),
            LogLevel::INFO => info!(path = uri.path(), error = debug(error), internal_message),
            LogLevel::WARN => warn!(path = uri.path(), error = debug(error), internal_message),
            LogLevel::ERROR => error!(path = uri.path(), error = debug(error), internal_message),
        }
    }
    response
}

pub(crate) fn assert_matching_network(
    network_identifier: &models::NetworkIdentifier,
    network_definition: &NetworkDefinition,
) -> Result<(), ResponseError> {
    if network_identifier.network != network_definition.logical_name {
        return Err(
            ResponseError::from(ApiError::InvalidNetwork).with_details(format!(
                "Invalid network - the network is actually: {}",
                network_definition.logical_name
            )),
        );
    } else if network_identifier.sub_network_identifier.is_some() {
        return Err(ResponseError::from(ApiError::InvalidNetwork)
            .with_details("Invalid network - subnetworks not supported".to_string()));
    }
    Ok(())
}
