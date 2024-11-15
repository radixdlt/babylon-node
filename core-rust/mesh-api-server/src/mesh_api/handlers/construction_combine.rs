use crate::prelude::*;

pub(crate) async fn handle_construction_combine(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionCombineRequest>,
) -> Result<Json<models::ConstructionCombineResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let signature = if request.signatures.len() == 1 {
        extract_signature(&request.signatures[0])
            .map_err(|e| e.into_response_error("signatures"))?
    } else {
        return Err(
            ResponseError::from(ApiError::InvalidNumberOfSignatures).with_details(format!(
                "Expected 1 signature, but received {}",
                request.signatures.len()
            )),
        );
    };

    let intent = RawTransactionIntent::from_hex(&request.unsigned_transaction)
        .ok()
        .and_then(|x| IntentV1::from_raw(&x).ok())
        .ok_or(
            ResponseError::from(ApiError::InvalidTransaction).with_details(format!(
                "Invalid unsigned transaction: {}",
                &request.unsigned_transaction
            )),
        )?;
    let tx = NotarizedTransactionV1 {
        signed_intent: SignedIntentV1 {
            intent,
            intent_signatures: IntentSignaturesV1 {
                signatures: Vec::new(),
            },
        },
        notary_signature: NotarySignatureV1(signature),
    };

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructioncombineresponse for field
    // definitions
    Ok(Json(models::ConstructionCombineResponse {
        signed_transaction: hex::encode(tx.to_raw().unwrap()),
    }))
}
