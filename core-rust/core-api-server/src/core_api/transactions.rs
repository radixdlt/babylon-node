use crate::core_api::conversions::to_api_receipt;
use crate::core_api::errors::{common_server_errors, RequestHandlingError};
use crate::core_api::generated::models;
use crate::core_api::generated::models::{
    CommittedTransactionsRequest, CommittedTransactionsResponse, TransactionSubmitRequest,
    TransactionSubmitResponse,
};
use crate::core_api::generated::{TransactionSubmitPostResponse, TransactionsPostResponse};
use scrypto::address::Bech32Encoder;
use scrypto::buffer::scrypto_decode;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::sha256_twice;

use state_manager::jni::state_manager::ActualStateManager;
use state_manager::mempool::Mempool;
use state_manager::store::{QueryableProofStore, QueryableTransactionStore};
use state_manager::{LedgerTransactionReceipt, MempoolError, TId, Transaction};
use std::cmp;
use std::sync::{Arc, Mutex};
use transaction::manifest;
use transaction::model::NotarizedTransaction as EngineNotarizedTransaction;

pub(crate) fn handle_submit_transaction(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: TransactionSubmitRequest,
) -> TransactionSubmitPostResponse {
    match handle_submit_transaction_internal(state_manager, request) {
        Ok(response) => TransactionSubmitPostResponse::TransactionSubmitResponse(response),
        Err(RequestHandlingError::ClientError(error_response)) => {
            TransactionSubmitPostResponse::ClientError(error_response)
        }
        Err(RequestHandlingError::ServerError(error_response)) => {
            TransactionSubmitPostResponse::ServerError(error_response)
        }
    }
}

fn handle_submit_transaction_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
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

    let mut locked_state_manager = state_manager
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

    let result = locked_state_manager.mempool.add_transaction(transaction);

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

pub(crate) fn handle_get_committed_transactions(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: CommittedTransactionsRequest,
) -> TransactionsPostResponse {
    match handle_get_committed_transactions_internal(state_manager, request) {
        Ok(response) => TransactionsPostResponse::CommittedTransactionsResponse(response),
        Err(RequestHandlingError::ClientError(error_response)) => {
            TransactionsPostResponse::ClientError(error_response)
        }
        Err(RequestHandlingError::ServerError(error_response)) => {
            TransactionsPostResponse::ServerError(error_response)
        }
    }
}

fn handle_get_committed_transactions_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: CommittedTransactionsRequest,
) -> Result<CommittedTransactionsResponse, RequestHandlingError> {
    let start_state_version: u64 = request
        .start_state_version
        .parse()
        .map_err(|_| transaction_errors::invalid_int_field("state_version"))?;

    if start_state_version < 1 {
        return Err(transaction_errors::invalid_start_state_version());
    }

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| transaction_errors::invalid_int_field("limit"))?;

    let state_version_at_limit: u64 = start_state_version
        .checked_add(limit)
        .and_then(|v| v.checked_sub(1))
        .ok_or_else(|| transaction_errors::invalid_int_field("limit"))?;

    let locked_state_manager = state_manager
        .lock()
        .map_err(|_| common_server_errors::state_manager_lock_error())?;

    let up_to_state_version_inclusive = cmp::min(
        state_version_at_limit,
        locked_state_manager.store.max_state_version(),
    );

    let mut txns = vec![];
    let mut state_version = start_state_version;
    while state_version <= up_to_state_version_inclusive {
        let next_tid = locked_state_manager
            .store
            .get_tid(state_version)
            .ok_or_else(|| {
                transaction_errors::missing_transaction_at_state_version(state_version)
            })?;
        let next_tx = locked_state_manager.store.get_transaction(&next_tid);
        txns.push((next_tx, state_version));
        state_version += 1;
    }

    let network = locked_state_manager.network.clone();

    let api_txns = txns
        .into_iter()
        .map(|((tx, receipt), state_version)| {
            scrypto_decode(&tx)
                .map(|notarized_tx| {
                    to_api_committed_transaction(&network, notarized_tx, receipt, state_version)
                })
                .map_err(|_| transaction_errors::invalid_committed_txn())
        })
        .collect::<Result<Vec<models::CommittedTransaction>, RequestHandlingError>>()?;

    let start_state_version = if api_txns.is_empty() {
        0
    } else {
        start_state_version
    };

    Ok(CommittedTransactionsResponse {
        start_state_version: start_state_version.to_string(),
        max_state_version: up_to_state_version_inclusive.to_string(),
        transactions: api_txns,
    })
}

fn to_api_committed_transaction(
    network: &NetworkDefinition,
    tx: EngineNotarizedTransaction,
    receipt: LedgerTransactionReceipt,
    state_version: u64,
) -> models::CommittedTransaction {
    let bech32_encoder = Bech32Encoder::new(network);
    let receipt = to_api_receipt(&bech32_encoder, receipt);

    models::CommittedTransaction {
        state_version: state_version.to_string(),
        notarized_transaction: to_api_notarized_transaction(tx, &bech32_encoder),
        receipt,
    }
}

fn to_api_notarized_transaction(
    tx: EngineNotarizedTransaction,
    bech32_encoder: &Bech32Encoder,
) -> models::NotarizedTransaction {
    let tx_hash = tx.hash();
    let signed_intent = tx.signed_intent;
    let signed_intent_hash = signed_intent.hash();
    let intent = signed_intent.intent;
    let intent_hash = intent.hash();
    let header = intent.header;

    models::NotarizedTransaction {
        hash: tx_hash.to_string(),
        signed_intent: models::SignedTransactionIntent {
            hash: signed_intent_hash.to_string(),
            intent: models::TransactionIntent {
                hash: intent_hash.to_string(),
                header: models::TransactionHeader {
                    version: header.version as isize,
                    network_id: header.network_id as isize,
                    start_epoch_inclusive: header.start_epoch_inclusive.to_string(),
                    end_epoch_exclusive: header.end_epoch_exclusive.to_string(),
                    nonce: header.nonce.to_string(),
                    notary_public_key: header.notary_public_key.to_string(),
                    notary_as_signatory: header.notary_as_signatory,
                    cost_unit_limit: header.cost_unit_limit.to_string(),
                    tip_percentage: header.tip_percentage.to_string(),
                },
                manifest: manifest::decompile(&intent.manifest, bech32_encoder)
                    .expect("Failed to decompile a transaction manifest"),
            },
            intent_signatures: signed_intent
                .intent_signatures
                .into_iter()
                .map(|(public_key, signature)| models::IntentSignature {
                    public_key: public_key.to_string(),
                    signature: signature.to_string(),
                })
                .collect(),
        },
        notary_signature: tx.notary_signature.to_string(),
    }
}

mod transaction_errors {
    use crate::core_api::errors::{client_error, server_error, RequestHandlingError};
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

    pub(crate) fn invalid_int_field(field: &str) -> RequestHandlingError {
        client_error(14, &format!("Invalid integer: {}", field))
    }

    pub(crate) fn invalid_start_state_version() -> RequestHandlingError {
        client_error(
            15,
            "start_state_version is invalid (minimum state version is 1)",
        )
    }

    pub(crate) fn invalid_committed_txn() -> RequestHandlingError {
        server_error(16, "Internal server error: invalid committed txn payload")
    }

    pub(crate) fn missing_transaction_at_state_version(state_version: u64) -> RequestHandlingError {
        server_error(
            17,
            &format!(
                "A transaction is missing at state version {}",
                state_version
            ),
        )
    }
}
