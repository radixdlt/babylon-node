use crate::core_api::*;

pub(crate) async fn handle_mempool_transaction(
    state: State<CoreApiState>,
    Json(request): Json<models::MempoolTransactionRequest>,
) -> Result<Json<models::MempoolTransactionResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    if request.payload_hashes.len() > MAX_BATCH_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "you can query up to {MAX_BATCH_COUNT_PER_REQUEST} payload hashes"
        )));
    }

    let mut response = models::MempoolTransactionResponse {
        count: MAX_BATCH_COUNT_PER_REQUEST as i32, // placeholder to get a better size aproximation for the header
        payloads: vec![],
    };

    let extraction_context = ExtractionContext::new(&state.network);
    let mapping_context = MappingContext::new_for_uncommitted_data(&state.network);

    let mut current_total_size = response.get_json_size();

    let mempool = state.state_manager.mempool.read();
    for payload_hash_str in request.payload_hashes.into_iter() {
        let payload_hash =
            extract_notarized_transaction_hash(&extraction_context, payload_hash_str)
                .map_err(|err| err.into_response_error("payload_hashes"))?;

        let (hex, error) = match mempool.get_payload(&payload_hash) {
            Some(mempool_transaction) => (Some(hex::encode(&mempool_transaction.raw.0)), None),
            None => (None, Some("Payload hash not found in mempool".into())),
        };

        let payload_response = models::MempoolTransactionResponsePayloadsInner {
            hash: to_api_notarized_transaction_hash(&payload_hash),
            hash_bech32m: to_api_hash_bech32m(&mapping_context, &payload_hash)?,
            hex,
            error,
        };

        current_total_size += payload_response.get_json_size();

        response.payloads.push(payload_response);

        if current_total_size > CAP_BATCH_RESPONSE_WHEN_ABOVE_BYTES {
            break;
        }
    }

    response.count = response.payloads.len() as i32;

    Ok(response).map(Json)
}
