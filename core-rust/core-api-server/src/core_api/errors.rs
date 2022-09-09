use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use radix_engine::types::NetworkDefinition;

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
        let coded_response: ErrorResponseWithCode = error_response.into();
        coded_response.into_response()
    }
}

pub type ErrorResponseWithCode = (StatusCode, Json<ErrorResponse>);

impl From<ErrorResponse> for ErrorResponseWithCode {
    fn from(error_response: ErrorResponse) -> Self {
        let http_code = u16::try_from(error_response.code).unwrap_or(500);
        let status_code = StatusCode::from_u16(http_code).expect("Http code was unexpected");

        let body = Json(error_response);
        (status_code, body)
    }
}

pub(crate) fn assert_matching_network(
    request_network: &str,
    network_definition: &NetworkDefinition,
) -> Result<(), RequestHandlingError> {
    if request_network != network_definition.logical_name {
        return Err(client_error(
            400,
            &format!(
                "Invalid network - the network is actually: {}",
                network_definition.logical_name
            ),
        ));
    }
    Ok(())
}

pub(crate) fn client_error(code: u32, message: &str) -> RequestHandlingError {
    RequestHandlingError::ClientError(ErrorResponse::new(code, message.to_string()))
}

pub(crate) fn server_error(code: u32, message: &str) -> RequestHandlingError {
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
