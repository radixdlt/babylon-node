use radix_engine::types::PublicKey;
use radix_engine_interface::crypto::{Ed25519PublicKey, Secp256k1PublicKey};
use transaction::model::{SignatureV1, SignatureWithPublicKeyV1};
use transaction::prelude::{Ed25519Signature, Secp256k1Signature};

use crate::core_api::*;

pub fn to_api_public_key(public_key: &PublicKey) -> models::PublicKey {
    match public_key {
        PublicKey::Secp256k1(key) => models::PublicKey::EcdsaSecp256k1PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
        PublicKey::Ed25519(key) => models::PublicKey::EddsaEd25519PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
    }
}

pub fn to_api_ecdsa_secp256k1_public_key(
    key: &Secp256k1PublicKey,
) -> models::EcdsaSecp256k1PublicKey {
    models::EcdsaSecp256k1PublicKey {
        key_type: models::PublicKeyType::EcdsaSecp256k1,
        key_hex: to_hex(key.0),
    }
}

pub fn to_api_signature(signature: &SignatureV1) -> models::Signature {
    match signature {
        SignatureV1::Secp256k1(sig) => models::Signature::EcdsaSecp256k1Signature {
            signature_hex: to_hex(sig.to_vec()),
        },
        SignatureV1::Ed25519(sig) => models::Signature::EddsaEd25519Signature {
            signature_hex: to_hex(sig.to_vec()),
        },
    }
}

pub fn to_api_signature_with_public_key(
    sig_with_public_key: &SignatureWithPublicKeyV1,
) -> models::SignatureWithPublicKey {
    match sig_with_public_key {
        SignatureWithPublicKeyV1::Secp256k1 { signature } => {
            models::SignatureWithPublicKey::EcdsaSecp256k1SignatureWithPublicKey {
                recoverable_signature: Box::new(models::EcdsaSecp256k1Signature {
                    key_type: models::PublicKeyType::EcdsaSecp256k1,
                    signature_hex: to_hex(signature.to_vec()),
                }),
            }
        }
        SignatureWithPublicKeyV1::Ed25519 {
            public_key,
            signature,
        } => models::SignatureWithPublicKey::EddsaEd25519SignatureWithPublicKey {
            signature: Box::new(models::EddsaEd25519Signature {
                key_type: models::PublicKeyType::EddsaEd25519,
                signature_hex: to_hex(signature.to_vec()),
            }),
            public_key: Box::new(models::EddsaEd25519PublicKey {
                key_type: models::PublicKeyType::EddsaEd25519,
                key_hex: to_hex(public_key.to_vec()),
            }),
        },
    }
}

#[allow(dead_code)]
pub fn extract_api_signature(signature: models::Signature) -> Result<SignatureV1, ExtractionError> {
    Ok(match signature {
        models::Signature::EcdsaSecp256k1Signature { signature_hex } => SignatureV1::Secp256k1(
            Secp256k1Signature::try_from(from_hex(signature_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
        models::Signature::EddsaEd25519Signature { signature_hex } => SignatureV1::Ed25519(
            Ed25519Signature::try_from(from_hex(signature_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
    })
}

pub fn extract_api_public_key(public_key: models::PublicKey) -> Result<PublicKey, ExtractionError> {
    Ok(match public_key {
        models::PublicKey::EcdsaSecp256k1PublicKey { key_hex } => PublicKey::Secp256k1(
            Secp256k1PublicKey::try_from(from_hex(key_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
        models::PublicKey::EddsaEd25519PublicKey { key_hex } => PublicKey::Ed25519(
            Ed25519PublicKey::try_from(from_hex(key_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
    })
}
