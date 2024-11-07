use crate::prelude::*;

pub(crate) async fn handle_network_list(
    state: State<MeshApiState>,
    Json(_request): Json<models::MetadataRequest>,
) -> Result<Json<models::NetworkListResponse>, ResponseError> {
    // See https://docs.cdp.coinbase.com/mesh/docs/models#networklistresponse for field definitions
    Ok(Json(models::NetworkListResponse {
        network_identifiers: vec![models::NetworkIdentifier {
            blockchain: "radix".to_string(),
            network: state.network.logical_name.to_string(),
            sub_network_identifier: None,
        }],
    }))
}
