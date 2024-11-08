use crate::prelude::*;
use models::AccountIdentifier;

pub(crate) async fn handle_construction_derive(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionDeriveRequest>,
) -> Result<Json<models::ConstructionDeriveResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let public_key = assert_public_key(&request.public_key)?;
    let address = state.public_key_to_address(public_key);

    // See https://docs.cdp.coinbase.com/mesh/docs/models#constructionderiveresponse for field
    // definitions
    Ok(Json(models::ConstructionDeriveResponse {
        address: None, // deprecated
        account_identifier: Some(Box::new(AccountIdentifier {
            address,
            sub_account: None,
            metadata: None,
        })),
        metadata: None,
    }))
}
