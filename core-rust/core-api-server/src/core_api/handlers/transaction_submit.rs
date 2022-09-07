use crate::core_api::models::*;
use crate::core_api::*;
use axum::{Extension, Json};
use scrypto::crypto::sha256_twice;

use state_manager::jni::state_manager::ActualStateManager;
use state_manager::mempool::Mempool;
use state_manager::{MempoolError, TId, Transaction};

pub(crate) async fn handle_transaction_submit(
    state: Extension<CoreApiState>,
    request: Json<TransactionSubmitRequest>,
) -> Result<Json<TransactionSubmitResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_transaction_submit_internal)
}

fn handle_transaction_submit_internal(
    state_manager: &mut ActualStateManager,
    request: TransactionSubmitRequest,
) -> Result<TransactionSubmitResponse, RequestHandlingError> {
    let transaction_bytes = hex::decode(request.notarized_transaction)
        .map_err(|_| transaction_errors::invalid_transaction())?;

    let tid = sha256_twice(transaction_bytes.clone());

    let transaction = Transaction {
        payload: transaction_bytes,
        id: TId {
            bytes: tid.to_vec(),
        },
    };

    let result = state_manager.mempool.add_transaction(transaction);

    match result {
        Ok(_) => Ok(TransactionSubmitResponse::new(false)),
        Err(MempoolError::Duplicate) => Ok(TransactionSubmitResponse::new(true)),
        Err(MempoolError::Full {
            current_size: _,
            max_size: _,
        }) => Err(transaction_errors::mempool_is_full()),
        Err(MempoolError::TransactionValidationError(err)) => {
            Err(transaction_errors::transaction_validation_error(err))
        }
    }
}

mod transaction_errors {
    use crate::core_api::errors::{client_error, RequestHandlingError};
    use transaction::errors::TransactionValidationError;

    pub(crate) fn invalid_transaction() -> RequestHandlingError {
        client_error(11, "Invalid transaction payload")
    }

    pub(crate) fn mempool_is_full() -> RequestHandlingError {
        client_error(12, "Mempool is full")
    }

    pub(crate) fn transaction_validation_error(
        err: TransactionValidationError,
    ) -> RequestHandlingError {
        client_error(13, &format!("Transaction validation error: {:?}", err))
    }
}
