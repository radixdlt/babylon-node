use crate::prelude::*;

pub(crate) async fn handle_construction_preprocess(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionPreprocessRequest>,
) -> Result<Json<models::ConstructionPreprocessResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let mut fee_lockers = Vec::new();
    for operation in request.operations {
        match operation._type.as_str() {
            "LockFee" => {
                let account = extract_account_from_option(
                    &ExtractionContext::new(&state.network),
                    operation.account,
                )?;
                fee_lockers.push(account);
            }
            _ => {}
        }
    }

    if fee_lockers.len() != 1 {
        return Err(client_error(
            format!(
                "Expected exactly 1 LockFee operation, but found {}",
                fee_lockers.len()
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
            fee_lockers[0],
        )?]),
    }))
}
