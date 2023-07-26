use node_common::locks::RwLock;
use crate::core_api::*;

#[tracing::instrument(level = "debug", skip(state))]
pub(crate) async fn handle_mempool_list(
    State(state): State<CoreApiState>,
    Json(request): Json<models::MempoolListRequest>,
) -> Result<Json<models::MempoolListResponse>, ResponseError<()>> {
    // that's my favorite endpoint for triggering a Core API panic:
    // curl localhost:3333/core/mempool/list -H 'Content-Type: application/json' -d '{"network":"bomb"}' -v
    if &request.network == "bomb" {
        let fakelock = RwLock::new("info");
        let instance = fakelock.write();
        if 2 > 1 {
            panic!("some bug");
        }
        println!("{}", instance.to_string()); // make sure the guard is not dropped until here
    }
    assert_matching_network(&request.network, &state.network)?;
    let mempool = state.mempool.read();
    Ok(models::MempoolListResponse {
        contents: mempool
            .all_hashes_iter()
            .map(
                |(intent_hash, payload_hash)| models::MempoolTransactionHashes {
                    intent_hash: to_api_intent_hash(intent_hash),
                    payload_hash: to_api_notarized_transaction_hash(payload_hash),
                },
            )
            .collect(),
    })
    .map(Json)
}
