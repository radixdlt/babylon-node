use crate::prelude::*;
use std::any::Any;

use tower_http::catch_panic::ResponseForPanic;

/// A marker trait for custom error details
pub trait ErrorDetails: serde::Serialize + Debug + Sized {
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

impl ErrorDetails for models::TransactionSubmitErrorDetails {
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

impl ErrorDetails for models::LtsTransactionSubmitErrorDetails {
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

impl ErrorDetails for models::TransactionPreviewV2ErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::TransactionPreviewV2ErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for models::StreamTransactionsErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::StreamTransactionsErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
}

impl ErrorDetails for models::StreamProofsErrorDetails {
    fn to_error_response(
        details: Option<Self>,
        code: i32,
        message: String,
        trace_id: Option<String>,
    ) -> models::ErrorResponse {
        models::ErrorResponse::StreamProofsErrorResponse {
            code,
            message,
            trace_id,
            details: details.map(Box::new),
        }
    }
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
        server_error::<()>("Unexpected server error").into_response()
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

        let mut response = (self.status_code, Json(body.clone())).into_response();
        response.extensions_mut().insert(body);
        response
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

pub(crate) fn assert_unbounded_endpoints_flag_enabled<E: ErrorDetails>(
    state: &CoreApiState,
) -> Result<(), ResponseError<E>> {
    if !state.flags.enable_unbounded_endpoints {
        return Err(client_error(
            "This endpoint is disabled as the response is potentially unbounded, and this node is configured with `enable_unbounded_endpoints` false.",
        ));
    }
    Ok(())
}

impl<E: ErrorDetails> From<PreviewerError> for ResponseError<E> {
    fn from(error: PreviewerError) -> Self {
        client_error(match error {
            PreviewerError::FromEngine(error) => match error {
                PreviewError::TransactionValidationError(error) => {
                    format!("Transaction validation error: {error:?}")
                }
            }
            PreviewerError::FromStateHistory(error) => match error {
                StateHistoryError::StateHistoryDisabled => {
                    "State history feature must be enabled (see the `db.historical_substate_values.enable` Node configuration flag)".to_string()
                }
                StateHistoryError::StateVersionInTooDistantPast { first_available_version } => {
                    format!("Cannot request state version past the earliest available {} (see the `state_hash_tree.state_version_history_length` Node configuration flag)", first_available_version)
                }
                StateHistoryError::StateVersionInFuture { current_version } => {
                    format!("Cannot request state version ahead of the current top-of-ledger {}", current_version)
                }
            }
        })
    }
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

/// Fails with a client error if the history of the transaction at the given state version (i.e.
/// its receipt or its identifiers) may have been deleted by the Node's (opt-in) ledger history GC.
/// This must be called on the same database snapshot which is then used for reading the history.
pub(crate) fn assert_transaction_history_not_pruned<E: ErrorDetails>(
    database: &impl QueryableTransactionStore,
    state_version: StateVersion,
) -> Result<(), ResponseError<E>> {
    match database.get_transaction_history_pruned_below() {
        Some(pruned_below) if state_version < pruned_below => Err(client_error(format!(
            "The transaction history at state version {} is not available from this node: \
            it has pruned its transaction history below state version {} \
            (see the `ledger_history.gc.*` node configuration). \
            Please use a node which keeps the full transaction history.",
            state_version.number(),
            pruned_below.number(),
        ))),
        _ => Ok(()),
    }
}

/// Returns a note to be appended to a "transaction not found" message (or an empty string): a Node
/// which pruned its hash-based transaction indices (using its opt-in ledger history GC) cannot find
/// an older transaction by its hash anymore.
pub(crate) fn transaction_hash_indices_pruned_note(
    database: &impl QueryableTransactionStore,
) -> String {
    match database.get_transaction_hash_indices_pruned_below() {
        Some(pruned_below) => format!(
            " Note: this node has pruned its transaction hash indices below state version {}, \
            so it cannot find a transaction committed before it (see the `ledger_history.gc.*` \
            node configuration).",
            pruned_below.number(),
        ),
        None => String::new(),
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

#[allow(dead_code)]
pub(crate) fn length_limit_error<E: ErrorDetails>() -> ResponseError<E> {
    ResponseError {
        status_code: StatusCode::PAYLOAD_TOO_LARGE,
        public_error_message: "length limit exceeded".into(),
        trace: None,
        details: None,
    }
}
