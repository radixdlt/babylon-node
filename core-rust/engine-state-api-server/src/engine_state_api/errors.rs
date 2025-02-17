use crate::prelude::*;
use std::any::Any;

use hyper::StatusCode;

use rand::distributions::Alphanumeric;
use rand::Rng;
use tower_http::catch_panic::ResponseForPanic;

use super::models;

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
        ResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, "Unexpected server error")
            .with_internal_message("Panic caught during request-handling; see logged panic payload")
            .into_response()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseError {
    status_code: StatusCode,
    public_error_message: String,
    public_details: Option<models::ErrorDetails>,
    internal_message: Option<String>,
}

impl ResponseError {
    pub fn new(status_code: StatusCode, public_error_message: impl Into<String>) -> Self {
        Self {
            status_code,
            public_error_message: public_error_message.into(),
            public_details: None,
            internal_message: None,
        }
    }

    pub fn with_public_details(self, public_details: models::ErrorDetails) -> Self {
        Self {
            status_code: self.status_code,
            public_error_message: self.public_error_message,
            public_details: Some(public_details),
            internal_message: self.internal_message,
        }
    }

    pub fn with_internal_message(self, internal_message: impl Into<String>) -> Self {
        Self {
            status_code: self.status_code,
            public_error_message: self.public_error_message,
            public_details: self.public_details,
            internal_message: Some(internal_message.into()),
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
    pub error: models::ErrorResponse,
    pub internal_message: String,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        let trace_id = TraceId::unique();
        let returned_body = models::ErrorResponse {
            message: self.public_error_message,
            trace_id: trace_id.0.clone(),
            details: self.public_details.map(Box::new),
        };
        let error_response_event = ErrorResponseEvent {
            level: resolve_level(self.status_code),
            error: returned_body.clone(),
            internal_message: self
                .internal_message
                .unwrap_or_else(|| "no internal details available".to_string()),
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
        let ErrorResponseEvent {
            level,
            error,
            internal_message,
        } = event;
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

impl From<StateHistoryError> for ResponseError {
    fn from(error: StateHistoryError) -> Self {
        match error {
            StateHistoryError::StateHistoryDisabled => NodeFeatureDisabledError::new(
                "State history",
                "db.historical_substate_values.enable",
            )
            .into(),
            StateHistoryError::StateVersionInTooDistantPast {
                first_available_version,
            } => {
                ResponseError::new(
                    StatusCode::BAD_REQUEST,
                    "Cannot request state version past the earliest available",
                )
                .with_public_details(models::ErrorDetails::StateVersionInTooDistantPastDetails {
                    // best-effort conversion - we should not error-out within error-handling:
                    earliest_available_state_version: first_available_version.number() as i64,
                })
                .with_internal_message(
                    "See the `state_hash_tree.state_version_history_length` Node configuration",
                )
            }
            StateHistoryError::StateVersionInFuture { current_version } => {
                ResponseError::new(
                    StatusCode::BAD_REQUEST,
                    "Cannot request state version ahead of the current top-of-ledger",
                )
                .with_public_details(
                    models::ErrorDetails::StateVersionInFutureDetails {
                        // best-effort conversion - we should not error-out within error-handling:
                        current_state_version: current_version.number() as i64,
                    },
                )
            }
        }
    }
}

/// An error occurring when a Node feature required to handle the request is not configured.
/// To be translated into [`StatusCode::CONFLICT`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeFeatureDisabledError {
    public_feature_name: String,
    property_name: String,
}

impl NodeFeatureDisabledError {
    pub fn new(public_feature_name: impl Into<String>, property_name: impl Into<String>) -> Self {
        Self {
            public_feature_name: public_feature_name.into(),
            property_name: property_name.into(),
        }
    }
}

impl From<NodeFeatureDisabledError> for ResponseError {
    fn from(error: NodeFeatureDisabledError) -> Self {
        ResponseError::new(
            StatusCode::CONFLICT,
            format!(
                "{} feature is not enabled on this Node",
                error.public_feature_name
            ),
        )
        .with_internal_message(format!(
            "Missing `{}` Node configuration flag",
            error.property_name
        ))
    }
}
