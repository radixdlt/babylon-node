use crate::core_api::generated::models::*;
use crate::core_api::generated::{TransactionSubmitPostResponse, TransactionsPostResponse};
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::sha256_twice;
use scrypto::prelude::scrypto_encode;
use state_manager::jni::state_manager::ActualStateManager;
use state_manager::mempool::Mempool;
use state_manager::{MempoolError, TId, TemporaryTransactionReceipt, Transaction};
use std::cmp;
use std::sync::{Arc, Mutex};
use swagger::ApiError;
use transaction::model::NotarizedTransaction as EngineNotarizedTransaction;

pub(crate) fn handle_submit_transaction(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: TransactionSubmitRequest,
) -> Result<TransactionSubmitPostResponse, ApiError> {
    handle_submit_transaction_internal(state_manager, request)
        .map(TransactionSubmitPostResponse::TransactionSubmitResponse)
        .or_else(Ok)
}

fn handle_submit_transaction_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: TransactionSubmitRequest,
) -> Result<TransactionSubmitResponse, TransactionSubmitPostResponse> {
    let transaction_bytes = hex::decode(request.notarized_transaction)
        .map_err(|_| submit_client_error("Invalid transaction (malformed hex)"))?;

    let tid = sha256_twice(transaction_bytes.clone());

    let transaction = Transaction {
        payload: transaction_bytes,
        id: TId {
            bytes: tid.to_vec(),
        },
    };

    let mut locked_state_manager = state_manager
        .lock()
        .map_err(|_| submit_server_error("Internal server error (state manager lock)"))?;

    let result = locked_state_manager.mempool.add_transaction(transaction);

    match result {
        Ok(_) => Ok(TransactionSubmitResponse::new(false)),
        Err(MempoolError::Duplicate) => Ok(TransactionSubmitResponse::new(true)),
        Err(MempoolError::Full {
            current_size: _,
            max_size: _,
        }) => Err(submit_server_error("Mempool is full")),
        Err(MempoolError::TransactionValidationError(err)) => Err(submit_client_error(&format!(
            "Transaction validation error: {:?}",
            err
        ))),
    }
}

fn submit_client_error(message: &str) -> TransactionSubmitPostResponse {
    TransactionSubmitPostResponse::ClientError(ErrorResponse::new(400, message.to_string()))
}

fn submit_server_error(message: &str) -> TransactionSubmitPostResponse {
    TransactionSubmitPostResponse::ServerError(ErrorResponse::new(500, message.to_string()))
}

pub(crate) fn handle_transactions(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: CommittedTransactionsRequest,
) -> Result<TransactionsPostResponse, ApiError> {
    handle_transactions_internal(state_manager, request)
        .map(TransactionsPostResponse::CommittedTransactionsResponse)
        .or_else(Ok)
}

fn handle_transactions_internal(
    state_manager: Arc<Mutex<ActualStateManager>>,
    request: CommittedTransactionsRequest,
) -> Result<CommittedTransactionsResponse, TransactionsPostResponse> {
    let locked_state_manager = state_manager
        .lock()
        .map_err(|_| transactions_server_error("Internal server error (state manager lock)"))?;

    let initial_state_version: u64 = request
        .state_version
        .parse()
        .map_err(|_| transactions_client_error("Invalid state_version"))?;

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| transactions_client_error("Invalid limit"))?;

    let state_version_at_limit: u64 = initial_state_version
        .checked_add(limit)
        .and_then(|v| v.checked_sub(1))
        .ok_or_else(|| transactions_client_error("Invalid limit"))?;

    let up_to_state_version_inclusive = cmp::min(
        state_version_at_limit,
        locked_state_manager.proof_store.max_state_version(),
    );

    let mut txns = vec![];
    let mut state_version = initial_state_version;
    while state_version <= up_to_state_version_inclusive {
        if let Some(next_tid) = locked_state_manager.proof_store.get_tid(state_version) {
            let next_tx = locked_state_manager
                .transaction_store
                .get_transaction(&next_tid);
            txns.push((next_tx, state_version));
        }
        state_version += 1;
    }

    let api_txns = txns
        .into_iter()
        .map(|((tx, receipt), state_version)| {
            scrypto_decode(tx)
                .map(|notarized_tx| {
                    to_api_committed_transaction(notarized_tx, receipt.clone(), state_version)
                })
                .map_err(|_| transactions_server_error("Invalid committed txn payload"))
        })
        .collect::<Result<Vec<CommittedTransaction>, TransactionsPostResponse>>()?;

    Ok(CommittedTransactionsResponse {
        state_version: request.state_version,
        transactions: api_txns,
    })
}

fn to_api_committed_transaction(
    tx: EngineNotarizedTransaction,
    _receipt: TemporaryTransactionReceipt,
    state_version: u64,
) -> CommittedTransaction {
    let tx_hash = tx.hash();
    let signed_intent = tx.signed_intent;
    let signed_intent_hash = signed_intent.hash();
    let intent = signed_intent.intent;
    let intent_hash = intent.hash();
    let header = intent.header;

    CommittedTransaction {
        state_version: state_version.to_string(),
        notarized_transaction: NotarizedTransaction {
            hash: tx_hash.to_string(),
            signed_intent: SignedTransactionIntent {
                hash: signed_intent_hash.to_string(),
                intent: TransactionIntent {
                    hash: intent_hash.to_string(),
                    header: TransactionHeader {
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
                    manifest: hex::encode(scrypto_encode(&intent.manifest)),
                },
                intent_signatures: signed_intent
                    .intent_signatures
                    .into_iter()
                    .map(|(public_key, signature)| IntentSignature {
                        public_key: public_key.to_string(),
                        signature: signature.to_string(),
                    })
                    .collect(),
            },
            notary_signature: tx.notary_signature.to_string(),
        },
        receipt: TransactionReceipt {
            status: TransactionStatus::SUCCEEDED, // TODO: fixme (needs receipt)
            fee_summary: FeeSummary {
                // TODO: fixme
                loan_fully_repaid: true,
                cost_unit_limit: "0".to_string(),
                cost_unit_consumed: "0".to_string(),
                cost_unit_price: "0".to_string(),
                tip_percentage: "0".to_string(),
                xrd_burned: "0".to_string(),
                xrd_tipped: "0".to_string(),
            },
            output: Some(vec!["00".to_string()]), // TODO: fixme (needs receipt)
            error_message: None,                  // TODO: fixme (needs receipt)
        },
    }
}

fn transactions_server_error(message: &str) -> TransactionsPostResponse {
    TransactionsPostResponse::ServerError(ErrorResponse::new(500, message.to_string()))
}

fn transactions_client_error(message: &str) -> TransactionsPostResponse {
    TransactionsPostResponse::ClientError(ErrorResponse::new(400, message.to_string()))
}
