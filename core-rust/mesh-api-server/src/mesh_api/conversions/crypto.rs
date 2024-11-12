use crate::prelude::*;
use models::SignatureType;

pub(crate) fn extract_public_key(
    public_key: &crate::mesh_api::generated::models::PublicKey,
) -> Result<PublicKey, ResponseError> {
    match public_key.curve_type {
        models::CurveType::Secp256k1 => Ok(PublicKey::Secp256k1(
            hex::decode(&public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Secp256k1PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(client_error(
                    format!("Invalid Secp256k1 public key: {}", public_key.hex_bytes),
                    false,
                ))?,
        )),
        models::CurveType::Edwards25519 => Ok(PublicKey::Ed25519(
            hex::decode(&public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Ed25519PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(client_error(
                    format!("Invalid Ed25519 public key: {}", public_key.hex_bytes),
                    false,
                ))?,
        )),
        _ => Err(client_error(
            format!("Invalid curve type: {:?}", &public_key.curve_type),
            false,
        )),
    }
}

pub(crate) fn extract_signature(
    signature: &crate::mesh_api::generated::models::Signature,
) -> Result<SignatureV1, ResponseError> {
    match signature.signature_type {
        SignatureType::Ecdsa => Ok(SignatureV1::Secp256k1(
            hex::decode(&signature.hex_bytes)
                .ok()
                .and_then(|bytes| Secp256k1Signature::try_from(bytes.as_slice()).ok())
                .ok_or(client_error(
                    format!("Invalid Secp256k1 signature: {}", signature.hex_bytes),
                    false,
                ))?,
        )),
        SignatureType::Ed25519 => Ok(SignatureV1::Ed25519(
            hex::decode(&signature.hex_bytes)
                .ok()
                .and_then(|bytes| Ed25519Signature::try_from(bytes.as_slice()).ok())
                .ok_or(client_error(
                    format!("Invalid Ed25519 signature: {}", signature.hex_bytes),
                    false,
                ))?,
        )),
        _ => Err(client_error(
            format!("Invalid signature type: {:?}", &signature.signature_type),
            false,
        )),
    }
}
