use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub(crate) async fn handle_construction_metadata(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionMetadataRequest>,
) -> Result<Json<models::ConstructionMetadataResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let database = state.state_manager.database.snapshot();
    let current_epoch = database
        .get_latest_epoch_proof()
        .unwrap()
        .ledger_header
        .epoch;
    let nonce = rand::thread_rng().gen();

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionmetadataresponse for field
    // definitions
    Ok(Json(models::ConstructionMetadataResponse {
        metadata: serde_json::to_value(&ConstructionMetadata {
            start_epoch_inclusive: current_epoch.number(),
            end_epoch_exclusive: current_epoch.number() + 100,
            nonce,
            tip_percentage: 0,
        })
        .unwrap(),
        suggested_fee: None,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructionMetadata {
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u32,
    pub tip_percentage: u16,
}
