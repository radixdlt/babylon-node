use crate::prelude::*;

pub(crate) async fn handle_mempool_transaction(
    state: State<MeshApiState>,
    Json(request): Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let extraction_context = ExtractionContext::new(&state.network);
    let mapping_context = MappingContext::new(&state.network);
    let mempool = &state.state_manager.mempool_manager;

    // Only user transactions might be present in mempool.
    // So it is safe to assume that transaction_identifier includes
    // `transaction intent_hash` and not `ledger_transaction_hash`
    let intent_hash = extract_transaction_intent_hash(
        &extraction_context,
        request.transaction_identifier.hash.clone(),
    )
    .map_err(|err| err.into_response_error("intent_hash"))?;

    let payload_hashes = mempool.get_mempool_payload_hashes_for_intent(&intent_hash);
    let notarized_transaction_hash = if payload_hashes.is_empty() {
        return Err(
            ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                "Transaction {} not found in mempool transactions",
                &request.transaction_identifier.hash
            )),
        );
    } else {
        payload_hashes.get(0).unwrap()
    };

    let user_transaction = match mempool.get_mempool_payload(&notarized_transaction_hash) {
        Some(transaction) => transaction.raw.into_typed().map_err(|_| {
            ResponseError::from(ApiError::InvalidTransaction).with_details(format!(
                "Invalid transaction hex: {:?}",
                notarized_transaction_hash
            ))
        })?,
        None => {
            return Err(
                ResponseError::from(ApiError::TransactionNotFound).with_details(format!(
                    "Transaction {} payload not found in mempool transactions",
                    &request.transaction_identifier.hash
                )),
            )
        }
    };

    let instructions = match user_transaction {
        UserTransaction::V1(notarized_transaction) => {
            notarized_transaction.signed_intent.intent.instructions.0
        }

        UserTransaction::V2(_) => {
            return Err(ResponseError::from(ApiError::InvalidTransaction)
                .with_details(format!("Transactions V2 not supported")))
        }
    };

    let database = state.state_manager.database.snapshot();
    let operations = to_mesh_api_operations_from_instructions_v1(
        &instructions,
        &mapping_context,
        database.deref(),
    )?;

    let transaction_identifier = Box::new(models::TransactionIdentifier {
        hash: to_api_transaction_hash_bech32m(&mapping_context, &intent_hash)?,
    });

    let transaction = Box::new(models::Transaction {
        transaction_identifier,
        operations,
        related_transactions: None,
        metadata: None,
    });

    Ok(Json(models::MempoolTransactionResponse {
        transaction,
        metadata: None,
    }))
}
