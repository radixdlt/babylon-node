use crate::prelude::*;

pub(crate) async fn handle_construction_preprocess(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPreprocessRequest>,
) -> Result<Json<models::ConstructionPreprocessResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mut senders = Vec::new();
    for operation in request.operations {
        let operation_type =
            MeshApiOperationTypes::from_str(operation._type.as_str()).map_err(|_| {
                ResponseError::from(ApiError::InvalidOperation)
                    .with_details(format!("Invalid operation: {}", operation._type))
            })?;
        match operation_type {
            MeshApiOperationTypes::Withdraw => {
                let account = extract_account_address_from_option(
                    &ExtractionContext::new(&state.network),
                    operation.account,
                )
                .map_err(|e| e.into_response_error("account"))?;
                senders.push(account);
            }
            _ => {}
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
        required_public_keys: Some(vec![to_mesh_api_account_from_address(
            &MappingContext::new(&state.network),
            senders[0],
        )?]),
    }))
}
