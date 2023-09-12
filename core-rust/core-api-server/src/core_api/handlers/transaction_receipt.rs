use crate::core_api::handlers::to_api_committed_transaction;
use crate::core_api::*;

use state_manager::store::traits::*;
use state_manager::transaction::*;

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_transaction_receipt(
    state: State<CoreApiState>,
    Json(request): Json<models::TransactionReceiptRequest>,
) -> Result<Json<models::TransactionReceiptResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let intent_hash = extract_intent_hash(&extraction_context, request.intent_hash)
        .map_err(|err| err.into_response_error("intent_hash"))?;

    let database = state.state_manager.database.read();

    if !database.is_local_transaction_execution_index_enabled() {
        return Err(client_error(
            "This endpoint requires that the LocalTransactionExecutionIndex is enabled on the node. \
            To use this endpoint, you will need to enable the index in the config, wipe ledger and restart. \
            Please note the resync will take a while.",
        ));
    }

    let txn_state_version_opt = database.get_txn_state_version_by_identifier(&intent_hash);

    if let Some(txn_state_version) = txn_state_version_opt {
        let raw = database
            .get_committed_transaction(txn_state_version)
            .expect("Txn is missing");

        let receipt = database
            .get_committed_local_transaction_receipt(txn_state_version)
            .expect("Txn receipt is missing");

        let identifiers = database
            .get_committed_transaction_identifiers(txn_state_version)
            .expect("Txn identifiers are missing");

        let model = LedgerTransaction::from_raw(&raw).map_err(|error| {
            MappingError::CouldNotDecodeTransaction {
                state_version: txn_state_version,
                error,
            }
        })?;

        Ok(models::TransactionReceiptResponse {
            committed: Box::new(to_api_committed_transaction(
                Some(&database),
                &mapping_context,
                txn_state_version,
                raw,
                model,
                receipt,
                identifiers,
            )?),
        })
        .map(Json)
    } else {
        Err(not_found_error(format!(
            "Committed transaction not found with intent hash: {intent_hash:?}"
        )))
    }
}
