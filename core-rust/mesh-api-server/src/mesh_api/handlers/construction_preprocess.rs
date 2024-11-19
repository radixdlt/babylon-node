use crate::prelude::*;

pub(crate) async fn handle_construction_preprocess(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPreprocessRequest>,
) -> Result<Json<models::ConstructionPreprocessResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    // We assume that the withdrawing account (sender) will
    // - cover the transaction fee
    // - sign the transaction
    // Add it to the required_public_keys vector.
    let mut senders = Vec::new();
    for operation in request.operations {
        let operation_type =
            MeshApiOperationType::from_str(operation._type.as_str()).map_err(|_| {
                ResponseError::from(ApiError::InvalidOperation)
                    .with_details(format!("Invalid operation: {}", operation._type))
            })?;
        match operation_type {
            MeshApiOperationType::Withdraw => {
                let account = match operation.account {
                    None => Err(ExtractionError::NotFound),
                    Some(account) => extract_radix_account_address_from_account_identifier(
                        &ExtractionContext::new(&state.network),
                        &account,
                    ),
                }
                .map_err(|e| e.into_response_error("account"))?;
                senders.push(account);
            }
            MeshApiOperationType::Deposit => {}
            MeshApiOperationType::LockFee => {}
            MeshApiOperationType::FeeDistributed => {}
            MeshApiOperationType::TipDistributed => {}
            MeshApiOperationType::RoyaltyDistributed => {}
        }
    }

    if senders.len() != 1 {
        return Err(
            ResponseError::from(ApiError::InvalidNumberOfSenders).with_details(format!(
                "Expected exactly 1 sender (Withdraw operation), but found {}",
                senders.len()
            )),
        );
    }

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionpreprocessresponse for field
    // definitions
    Ok(Json(models::ConstructionPreprocessResponse {
        options: None,
        required_public_keys: Some(vec![to_api_account_identifier_from_global_address(
            &MappingContext::new(&state.network),
            senders[0],
        )?]),
    }))
}
