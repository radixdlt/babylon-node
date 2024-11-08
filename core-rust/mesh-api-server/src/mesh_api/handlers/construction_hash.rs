use models::TransactionIdentifier;
use radix_transactions::validation::TransactionValidator;

use crate::prelude::*;

pub(crate) async fn handle_construction_hash(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionHashRequest>,
) -> Result<Json<models::TransactionIdentifierResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let tx = RawNotarizedTransaction::from_hex(&request.signed_transaction)
        .ok()
        .and_then(|raw| NotarizedTransactionV1::from_raw(&raw).ok())
        .and_then(|tx| {
            tx.prepare_and_validate(&TransactionValidator::new_with_latest_config(
                &state.network,
            ))
            .ok()
        })
        .ok_or(client_error(
            format!("Invalid transaction: {}", request.signed_transaction),
            false,
        ))?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionhashresponse for field
    // definitions
    Ok(Json(models::TransactionIdentifierResponse {
        transaction_identifier: Box::new(TransactionIdentifier {
            hash: state
                .hash_encoder()
                .encode(&tx.transaction_intent_hash())
                .unwrap(),
        }),
        metadata: None,
    }))
}
