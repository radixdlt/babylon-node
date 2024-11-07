use models::AccountIdentifier;

use crate::prelude::*;

pub(crate) async fn handle_construction_derive(
    state: State<MeshApiState>,
    Json(request): Json<models::ConstructionDeriveRequest>,
) -> Result<Json<models::ConstructionDeriveResponse>, ResponseError> {
    assert_matching_network(&request.network_identifier, &state.network)?;

    let public_key = match request.public_key.curve_type {
        models::CurveType::Secp256k1 => PublicKey::Secp256k1(
            from_hex(&request.public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Secp256k1PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(
                    ResponseError::from(ApiError::InvalidRequest).with_details(format!(
                        "Invalid Secp256k1 public key: {}",
                        request.public_key.hex_bytes
                    )),
                )?,
        ),
        models::CurveType::Edwards25519 => PublicKey::Ed25519(
            from_hex(&request.public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Ed25519PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(
                    ResponseError::from(ApiError::InvalidRequest).with_details(format!(
                        "Invalid Ed25519 public key: {}",
                        request.public_key.hex_bytes
                    )),
                )?,
        ),
        _ => {
            return Err(
                ResponseError::from(ApiError::InvalidRequest).with_details(format!(
                    "Invalid curve type: {:?}",
                    request.public_key.curve_type
                )),
            )
        }
    };
    let address = state
        .address_encoder()
        .encode(ComponentAddress::preallocated_account_from_public_key(&public_key).as_bytes())
        .expect("Failed to encode account address");

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
