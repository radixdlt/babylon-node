use crate::core_api::generated::models::ErrorResponse;

pub(crate) enum RequestHandlingError {
    ClientError(ErrorResponse),
    ServerError(ErrorResponse),
}

pub(crate) fn client_error(code: isize, message: &str) -> RequestHandlingError {
    RequestHandlingError::ClientError(ErrorResponse::new(code, message.to_string()))
}

pub(crate) fn server_error(code: isize, message: &str) -> RequestHandlingError {
    RequestHandlingError::ServerError(ErrorResponse::new(code, message.to_string()))
}

pub(crate) mod common_server_errors {
    use crate::core_api::errors::{server_error, RequestHandlingError};

    pub(crate) fn state_manager_lock_error() -> RequestHandlingError {
        server_error(1, "Internal server error: state manager lock")
    }

    pub(crate) fn unexpected_state(details: &str) -> RequestHandlingError {
        server_error(2, &format!("Unexpected state: {}", details))
    }
}
