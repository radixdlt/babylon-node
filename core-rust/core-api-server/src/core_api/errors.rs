use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;

use crate::core_api::generated::models::ErrorResponse;

#[derive(Debug, Clone)]
pub(crate) enum RequestHandlingError {
    ClientError(ErrorResponse),
    ServerError(ErrorResponse),
}

impl IntoResponse for RequestHandlingError {
    fn into_response(self) -> Response {
        let error_response = match self {
            Self::ClientError(r) => r,
            Self::ServerError(r) => r,
        };
        let http_code = u16::try_from(error_response.code).unwrap_or(500);
        let status_code = StatusCode::from_u16(http_code).expect("Http code was unexpected");

        let body = Json(error_response);
        (status_code, body).into_response()
    }
}

pub(crate) fn client_error(code: i32, message: &str) -> RequestHandlingError {
    RequestHandlingError::ClientError(ErrorResponse::new(code, message.to_string()))
}

pub(crate) fn server_error(code: i32, message: &str) -> RequestHandlingError {
    RequestHandlingError::ServerError(ErrorResponse::new(code, message.to_string()))
}

// TODO - Add logging, metrics and tracing for all of these errors - require the error is passed in here
pub(crate) mod common_server_errors {
    use crate::core_api::errors::{server_error, RequestHandlingError};

    pub(crate) fn state_manager_lock_error() -> RequestHandlingError {
        server_error(500, "Internal server error: state manager lock")
    }

    pub(crate) fn unexpected_state(details: &str) -> RequestHandlingError {
        server_error(500, &format!("Unexpected state: {}", details))
    }

    pub(crate) fn mapping_error(details: &str) -> RequestHandlingError {
        server_error(
            500,
            &format!("Unexpected state, mapping failed: {}", details),
        )
    }
}
