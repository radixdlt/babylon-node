use crate::core_api::*;

pub(crate) async fn handle_mempool_transaction(
    state: State<CoreApiState>,
    Json(request): Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let payload_hash = extract_notarized_transaction_hash(request.payload_hash)
        .map_err(|err| err.into_response_error("payload_hash"))?;

    let mempool = state.radix_node.mempool.read();
    match mempool.get_payload(&payload_hash) {
        Some(mempool_transaction) => Ok(models::MempoolTransactionResponse {
            payload_hex: hex::encode(&mempool_transaction.raw.0),
        }),
        None => Err(not_found_error(
            "Transaction with given payload hash is not in the mempool",
        )),
    }
    .map(Json)
}
