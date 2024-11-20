use crate::prelude::*;
use models::SignatureType;

pub(crate) fn extract_public_key(
    public_key: &crate::mesh_api::generated::models::PublicKey,
) -> Result<PublicKey, ExtractionError> {
    // https://docs.cdp.coinbase.com/mesh/docs/models#curvetype
    // - secp256k1 is SEC compressed, in line with canonical Radix encoding
    // - ed25519 is y (255-bits) + x-sign-bit (1-bit), in line with canonical Radix encoding
    match public_key.curve_type {
        models::CurveType::Secp256k1 => Ok(PublicKey::Secp256k1(
            hex::decode(&public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Secp256k1PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(ExtractionError::InvalidSecp256k1PublicKey(
                    public_key.hex_bytes.clone(),
                ))?,
        )),
        models::CurveType::Edwards25519 => Ok(PublicKey::Ed25519(
            hex::decode(&public_key.hex_bytes)
                .ok()
                .and_then(|bytes| Ed25519PublicKey::try_from(bytes.as_slice()).ok())
                .ok_or(ExtractionError::InvalidEd25519PublicKey(
                    public_key.hex_bytes.clone(),
                ))?,
        )),
        t => Err(ExtractionError::InvalidCurveType(t)),
    }
}

pub(crate) fn extract_signature(
    signature: &crate::mesh_api::generated::models::Signature,
) -> Result<SignatureV1, ExtractionError> {
    // https://docs.cdp.coinbase.com/mesh/docs/models#signaturetype
    // - ecdsa_recovery is r (32-bytes) + s (32-bytes) + v (1-byte), so Radix encoding needs reformatting
    // - ed25519 is R (32-bytes) + s (32-bytes), in line with canonical Radix encoding
    // - We don't support ecdsa (no recovery)
    match signature.signature_type {
        SignatureType::EcdsaRecovery => Ok(SignatureV1::Secp256k1(
            hex::decode(&signature.hex_bytes)
                .ok()
                .and_then(|mut bytes| {
                    // Mesh uses r + s + v
                    // Radix uses v + r + s
                    if let Some(v) = bytes.pop() {
                        bytes.insert(0, v);
                    }
                    Secp256k1Signature::try_from(bytes.as_slice()).ok()
                })
                .ok_or(ExtractionError::InvalidSecp256k1Signature(
                    signature.hex_bytes.clone(),
                ))?,
        )),
        SignatureType::Ed25519 => Ok(SignatureV1::Ed25519(
            hex::decode(&signature.hex_bytes)
                .ok()
                .and_then(|bytes| Ed25519Signature::try_from(bytes.as_slice()).ok())
                .ok_or(ExtractionError::InvalidEd25519Signature(
                    signature.hex_bytes.clone(),
                ))?,
        )),
        t => Err(ExtractionError::InvalidSignatureType(t)),
    }
}
