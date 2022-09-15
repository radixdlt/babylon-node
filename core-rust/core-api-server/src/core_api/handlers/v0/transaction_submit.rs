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
    let transaction_bytes = from_hex(request.notarized_transaction)
        .map_err(|err| err.into_response_error("notarized_transaction"))?;

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
        }) => Err(client_error("Mempool is full")),
        Err(MempoolError::TransactionValidationError(err)) => Err(client_error(&format!(
            "Transaction validation error: {:?}",
            err
        ))),
    }
}
