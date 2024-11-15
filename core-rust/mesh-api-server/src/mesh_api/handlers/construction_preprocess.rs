use crate::prelude::*;

pub(crate) async fn handle_construction_preprocess(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPreprocessRequest>,
) -> Result<Json<models::ConstructionPreprocessResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mut senders = Vec::new();
    for operation in request.operations {
        let operation_type = MeshApiOperationTypes::from_str(operation._type.as_str())
            .map_err(|_| client_error(format!("Invalid operation: {}", operation._type), false))?;
        match operation_type {
            MeshApiOperationTypes::Withdraw => {
                let account = extract_account_from_option(
                    &ExtractionContext::new(&state.network),
                    operation.account,
                )?;
                senders.push(account);
            }
            _ => {}
        }
    }

    if senders.len() != 1 {
        return Err(client_error(
            format!(
                "Expected exactly 1 sender (Withdraw operation), but found {}",
                senders.len()
            ),
            false,
        ));
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
