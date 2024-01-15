use axum::body::BoxBody;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
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
        server_error("Unexpected server error").into_response()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseError {
    status_code: StatusCode,
    public_error_message: String,
    trace: LogTraceId,
    details: Option<models::ErrorDetails>,
}

#[derive(Debug, Clone)]
pub struct LogTraceId(pub String);

impl LogTraceId {
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

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(models::ErrorResponse {
                message: self.public_error_message,
                trace_id: self.trace.0,
                details: self.details.map(Box::new),
            }),
        )
            .into_response()
    }
}

pub(crate) fn client_error(
    message: impl Into<String>,
    details: models::ErrorDetails,
) -> ResponseError {
    ResponseError {
        status_code: StatusCode::BAD_REQUEST,
        public_error_message: message.into(),
        trace: LogTraceId::unique(),
        details: Some(details),
    }
}

pub(crate) fn server_error(message: impl Into<String>) -> ResponseError {
    ResponseError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        public_error_message: message.into(),
        trace: LogTraceId::unique(),
        details: None,
    }
}

pub(crate) fn not_found_error(message: impl Into<String>) -> ResponseError {
    ResponseError {
        status_code: StatusCode::NOT_FOUND,
        public_error_message: message.into(),
        trace: LogTraceId::unique(),
        details: None,
    }
}
