use crate::core_api::models::*;
use crate::core_api::*;
use scrypto::crypto::sha256_twice;

use state_manager::jni::state_manager::ActualStateManager;
use state_manager::mempool::Mempool;
use state_manager::{MempoolError, TId, Transaction};

pub(crate) async fn handle_v0_transaction_submit(
    state: Extension<CoreApiState>,
    request: Json<V0TransactionSubmitRequest>,
) -> Result<Json<V0TransactionSubmitResponse>, RequestHandlingError> {
    core_api_handler(state, request, handle_v0_transaction_submit_internal)
}

fn handle_v0_transaction_submit_internal(
    state_manager: &mut ActualStateManager,
    request: V0TransactionSubmitRequest,
) -> Result<V0TransactionSubmitResponse, RequestHandlingError> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let transaction_bytes = hex::decode(request.notarized_transaction)
        .map_err(|_| errors::invalid_transaction())?;

    let tid = sha256_twice(transaction_bytes.clone());

    let transaction = Transaction {
        payload: transaction_bytes,
        id: TId {
            bytes: tid.to_vec(),
        },
    };

    let result = state_manager.mempool.add_transaction(transaction);

    match result {
        Ok(_) => Ok(V0TransactionSubmitResponse::new(false)),
        Err(MempoolError::Duplicate) => Ok(V0TransactionSubmitResponse::new(true)),
        Err(MempoolError::Full {
            current_size: _,
            max_size: _,
        }) => Err(errors::mempool_is_full()),
        Err(MempoolError::TransactionValidationError(err)) => {
            Err(errors::transaction_validation_error(err))
        }
    }
}

mod errors {
    use crate::core_api::errors::{client_error, RequestHandlingError};
    use transaction::errors::TransactionValidationError;

    pub(crate) fn invalid_transaction() -> RequestHandlingError {
        client_error(400, "Invalid transaction payload")
    }

    pub(crate) fn mempool_is_full() -> RequestHandlingError {
        client_error(400, "Mempool is full")
    }

    pub(crate) fn transaction_validation_error(
        err: TransactionValidationError,
    ) -> RequestHandlingError {
        client_error(400, &format!("Transaction validation error: {:?}", err))
    }
}
