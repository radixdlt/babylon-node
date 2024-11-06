use crate::{mesh_api::generated::SCHEMA_VERSION, prelude::*};

pub(crate) async fn handle_network_options(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkIdentifier>,
) -> Result<Json<models::NetworkOptionsResponse>, ResponseError> {
    assert_matching_network(&request.network, &state.network)?;

    let database = state.state_manager.database.snapshot();

    let mut bundles_iter = database.get_committed_transaction_bundle_iter(StateVersion::of(1));

    Ok(Json(models::NetworkOptionsResponse {
        version: Box::new(models::Version {
            rosetta_version: SCHEMA_VERSION.to_string(),
            // TODO:MESH get node version
            node_version: "unknown".to_string(),
            middleware_version: None,
            metadata: None,
        }),
        allow: Box::new(models::Allow {
            operation_statuses: vec![
                models::OperationStatus::new("SuccessStatus".to_string(), true),
                models::OperationStatus::new("FailureStatus".to_string(), false),
            ],
            operation_types: vec![
                "Withdraw".to_string(),
                "Deposit".to_string(),
                "Mint".to_string(),
                "Burn".to_string(),
                "LockFee".to_string(),
            ],
            errors: list_available_api_errors(),
            historical_balance_lookup: false,
            timestamp_start_index: bundles_iter.find_map(|p| {
                if p.identifiers.proposer_timestamp_ms != 0 {
                    Some(to_api_state_version(p.state_version).unwrap())
                } else {
                    None
                }
            }),
            // This is for native RPC calls. Not needed for now.
            call_methods: vec![],
            // TODO:MESH
            balance_exemptions: vec![],
            mempool_coins: false,
            block_hash_case: None,
            transaction_hash_case: Some(models::Case::LowerCase),
        }),
    }))
}
