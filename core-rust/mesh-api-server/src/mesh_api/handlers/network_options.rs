use crate::{mesh_api::generated::SCHEMA_VERSION, prelude::*};

pub(crate) async fn handle_network_options(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkRequest>,
) -> Result<Json<models::NetworkOptionsResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier.network, &state.network)?;

    let database = state.state_manager.database.snapshot();

    let mut proof_iter = database.get_proof_iter(StateVersion::pre_genesis());

    // See https://docs.cdp.coinbase.com/mesh/docs/models#networkoptionsresponse for field
    // definitions
    Ok(Json(models::NetworkOptionsResponse {
        version: Box::new(models::Version {
            rosetta_version: SCHEMA_VERSION.to_string(),
            node_version: state.node_display_version.clone(),
            middleware_version: None,
            metadata: None,
        }),
        allow: Box::new(models::Allow {
            operation_statuses: vec![
                models::OperationStatus::new("SuccessStatus".to_string(), true),
                models::OperationStatus::new("FailureStatus".to_string(), false),
            ],
            // TODO::MESH Add enum with operation types
            // see comment https://github.com/radixdlt/babylon-node/pull/1013#discussion_r1830848173
            operation_types: vec![
                "Withdraw".to_string(),
                "Deposit".to_string(),
                "Mint".to_string(),
                "Burn".to_string(),
                "LockFee".to_string(),
            ],
            errors: list_available_api_errors(),
            historical_balance_lookup: false,
            timestamp_start_index: proof_iter.find_map(|p| {
                if p.ledger_header.proposer_timestamp_ms != 0 {
                    Some(
                        to_mesh_api_block_index_from_state_version(p.ledger_header.state_version)
                            .unwrap(),
                    )
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
