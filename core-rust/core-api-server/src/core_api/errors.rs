use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use radix_engine::types::NetworkDefinition;

use crate::core_api::*;

#[derive(Debug, Clone)]
pub(crate) struct RequestHandlingError(pub StatusCode, pub models::ErrorResponse);

impl IntoResponse for RequestHandlingError {
    fn into_response(self) -> Response {
        (self.0, Json(self.1)).into_response()
    }
}

pub(crate) fn assert_matching_network(
    request_network: &str,
    network_definition: &NetworkDefinition,
) -> Result<(), RequestHandlingError> {
    if request_network != network_definition.logical_name {
        return Err(client_error(&format!(
            "Invalid network - the network is actually: {}",
            network_definition.logical_name
        )));
    }
    Ok(())
}

// TODO - Replace ErrorResponse "code" with making them an Enum with different structured errors
// TODO - Add logging, metrics and tracing for all of these errors - require the error is passed in here
pub(crate) fn client_error(message: &str) -> RequestHandlingError {
    RequestHandlingError(
        StatusCode::BAD_REQUEST,
        models::ErrorResponse::new(400, message.to_string()),
    )
}

pub(crate) fn not_found_error(message: &str) -> RequestHandlingError {
    RequestHandlingError(
        StatusCode::NOT_FOUND,
        models::ErrorResponse::new(400, message.to_string()),
    )
}

pub(crate) fn server_error(public_message: &str) -> RequestHandlingError {
    RequestHandlingError(
        StatusCode::INTERNAL_SERVER_ERROR,
        models::ErrorResponse::new(500, public_message.to_string()),
    )
}
