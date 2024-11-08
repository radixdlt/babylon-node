use crate::{mesh_api::generated::SCHEMA_VERSION, prelude::*};

pub(crate) async fn handle_network_options(
    state: State<MeshApiState>,
    Json(request): Json<models::NetworkRequest>,
) -> Result<Json<models::NetworkOptionsResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();

    let mut proof_iter = database.get_proof_iter(StateVersion::pre_genesis());

    // TODO:MESH for some reason the version sometimes is empty
    let node_version = if state.node_display_version.is_empty() {
        "unknown"
    } else {
        &state.node_display_version
    };

    // See https://docs.cdp.coinbase.com/mesh/docs/models#networkoptionsresponse for field
    // definitions
    Ok(Json(models::NetworkOptionsResponse {
        version: Box::new(models::Version {
            rosetta_version: SCHEMA_VERSION.to_string(),
            node_version: node_version.to_string(),
            middleware_version: None,
            metadata: None,
        }),
        allow: Box::new(models::Allow {
            operation_statuses: MeshApiOperationStatus::iter()
                .map(|s| models::OperationStatus::new(s.to_string(), s.into()))
                .collect(),
            operation_types: MeshApiOperationTypes::iter()
                .map(|o| o.to_string())
                .collect(),
            errors: list_available_api_errors(),
            historical_balance_lookup: false,
            timestamp_start_index: proof_iter.find_map(|p| {
                // Observed that some timestamp are 0 or 1
                if p.ledger_header.proposer_timestamp_ms > 1 {
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
