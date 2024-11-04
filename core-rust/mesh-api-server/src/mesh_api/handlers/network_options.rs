use crate::{mesh_api::generated::SCHEMA_VERSION, prelude::*};

pub(crate) async fn handle_network_options(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkIdentifier>,
) -> Result<Json<models::NetworkOptionsResponse>, ResponseError> {
    assert_matching_network(&request.network, &state.network)?;

    Ok(Json(models::NetworkOptionsResponse {
        version: Box::new(models::Version {
            rosetta_version: SCHEMA_VERSION.to_string(),
            // TODO get node version
            node_version: env!("VERSION_DISPLAY").to_string(),
            middleware_version: None,
            metadata: None,
        }),
        allow: Box::new(models::Allow {
            operation_statuses: vec![models::OperationStatus::new("STATUS".to_string(), true)],
            operation_types: vec!["TRANSFER".to_string()],
            errors: list_available_api_errors(),
            historical_balance_lookup: true,
            // TODO
            timestamp_start_index: None,
            // TODO
            call_methods: vec!["TODO".to_string()],
            // TODO
            balance_exemptions: vec![],
            mempool_coins: false,
            block_hash_case: None,
            transaction_hash_case: Some(models::Case::LowerCase),
        }),
    }))
}
