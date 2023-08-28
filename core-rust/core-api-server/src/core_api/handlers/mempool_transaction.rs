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

    let mut current_total_size = response.get_json_size();

    let mempool = state.state_manager.mempool.read();
    for payload_hash_str in request.payload_hashes.into_iter() {
        let payload_response =
            match extract_notarized_transaction_hash(&extraction_context, payload_hash_str.clone())
            {
                Ok(payload_hash) => match mempool.get_payload(&payload_hash) {
                    Some(mempool_transaction) => models::MempoolTransactionResponsePayloadsInner {
                        hash: payload_hash_str,
                        hex: Some(hex::encode(&mempool_transaction.raw.0)),
                        error: None,
                    },
                    None => models::MempoolTransactionResponsePayloadsInner {
                        hash: payload_hash_str,
                        hex: None,
                        error: Some("Payload hash not found in mempool".into()),
                    },
                },
                Err(_) => models::MempoolTransactionResponsePayloadsInner {
                    hash: payload_hash_str,
                    hex: None,
                    error: Some("Invalid payload hash".into()),
                },
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
