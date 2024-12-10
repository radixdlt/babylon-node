use crate::prelude::*;
use radix_transactions::validation::TransactionValidator;

// This method only accepts transactions constructed with the Mesh API,
// which are V1 at the moment.
// Also the number of supported V1 instructions is limited to some basic ones.
// (see `construction_payloads.rs` and parse_instructions() below for more details).
pub(crate) async fn handle_construction_parse(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionParseRequest>,
) -> Result<Json<models::ConstructionParseResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let transaction_bytes = hex::decode(&request.transaction).map_err(|_| {
        ResponseError::from(ApiError::InvalidTransaction)
            .with_details(format!("Invalid transaction hex: {}", &request.transaction))
    })?;

    let (instructions, signers) = if request.signed {
        let transaction = match manifest_decode::<AnyTransaction>(&transaction_bytes) {
            Ok(AnyTransaction::NotarizedTransactionV1(transaction)) => transaction,
            Ok(_) => {
                return Err(ResponseError::from(ApiError::InvalidTransaction)
                    .with_details("Only V1 notarized transactions are supported in the Mesh API parse endpoint"));
            }
            Err(_) => {
                return Err(ResponseError::from(ApiError::InvalidTransaction)
                    .with_details(format!("Invalid transaction: {}", &request.transaction)))
            }
        };

        let validated = transaction
            .prepare_and_validate(&TransactionValidator::new_with_latest_config(
                &state.network,
            ))
            .map_err(|e| {
                ResponseError::from(ApiError::InvalidTransaction)
                    .with_details(format!("Transaction validation error: {:?}", e))
            })?;

        let instructions = transaction.signed_intent.intent.instructions.0;
        let signers = validated.signer_keys;
        (instructions, signers)
    } else {
        let prepared_intent = PreparedIntentV1::prepare(
            &RawTransactionIntent::from_vec(transaction_bytes),
            &PreparationSettings::latest(),
        )
        .unwrap();
        #[allow(deprecated)]
        let instructions = prepared_intent.instructions.inner.0;
        let signers = index_set_new();
        (instructions, signers)
    };

    let mapping_context = MappingContext::new(&state.network);
    let database = state.state_manager.database.access_direct();
    let operations = to_mesh_api_operations_from_instructions_v1(
        &instructions,
        &mapping_context,
        database.deref(),
    )?;

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionparseresponse for field
    // definitions
    Ok(Json(models::ConstructionParseResponse {
        operations,
        signers: None,
        account_identifier_signers: Some(
            signers
                .into_iter()
                .map(|x| -> Result<models::AccountIdentifier, MappingError> {
                    to_api_account_identifier_from_public_key(&mapping_context, x)
                })
                .collect::<Result<Vec<_>, MappingError>>()?,
        ),
        metadata: None,
    }))
}
