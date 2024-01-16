use axum::body::BoxBody;
use axum::http::Uri;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use std::any::Any;

use hyper::StatusCode;

use rand::distributions::Alphanumeric;
use rand::Rng;
use tower_http::catch_panic::ResponseForPanic;
use tracing::{debug, error, info, trace, warn, Level};

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
    pub level: Level,
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

fn resolve_level(status_code: StatusCode) -> Level {
    if status_code.is_server_error() {
        Level::WARN
    } else {
        Level::DEBUG
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
            Level::TRACE => trace!(path = uri.path(), error = debug(error), internal_message),
            Level::DEBUG => debug!(path = uri.path(), error = debug(error), internal_message),
            Level::INFO => info!(path = uri.path(), error = debug(error), internal_message),
            Level::WARN => warn!(path = uri.path(), error = debug(error), internal_message),
            Level::ERROR => error!(path = uri.path(), error = debug(error), internal_message),
        }
    }
    response
}
