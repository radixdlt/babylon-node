use radix_engine::types::{
    EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey, EddsaEd25519Signature,
    PublicKey, Signature, SignatureWithPublicKey,
};

use crate::core_api::*;

pub fn to_api_public_key(public_key: &PublicKey) -> models::PublicKey {
    match public_key {
        PublicKey::EcdsaSecp256k1(key) => models::PublicKey::EcdsaSecp256k1PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
        PublicKey::EddsaEd25519(key) => models::PublicKey::EddsaEd25519PublicKey {
            key_hex: to_hex(key.to_vec()),
        },
    }
}

pub fn to_api_signature(signature: &Signature) -> models::Signature {
    match signature {
        Signature::EcdsaSecp256k1(sig) => models::Signature::EcdsaSecp256k1Signature {
            signature_hex: to_hex(sig.to_vec()),
        },
        Signature::EddsaEd25519(sig) => models::Signature::EddsaEd25519Signature {
            signature_hex: to_hex(sig.to_vec()),
        },
    }
}

pub fn to_api_signature_with_public_key(
    sig_with_public_key: &SignatureWithPublicKey,
) -> models::SignatureWithPublicKey {
    match sig_with_public_key {
        SignatureWithPublicKey::EcdsaSecp256k1 { signature } => {
            models::SignatureWithPublicKey::EcdsaSecp256k1SignatureWithPublicKey {
                recoverable_signature: Box::new(models::EcdsaSecp256k1Signature {
                    key_type: models::PublicKeyType::EcdsaSecp256k1,
                    signature_hex: to_hex(signature.to_vec()),
                }),
            }
        }
        SignatureWithPublicKey::EddsaEd25519 {
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
pub fn extract_api_signature(signature: models::Signature) -> Result<Signature, ExtractionError> {
    Ok(match signature {
        models::Signature::EcdsaSecp256k1Signature { signature_hex } => Signature::EcdsaSecp256k1(
            EcdsaSecp256k1Signature::try_from(from_hex(signature_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
        models::Signature::EddsaEd25519Signature { signature_hex } => Signature::EddsaEd25519(
            EddsaEd25519Signature::try_from(from_hex(signature_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
    })
}

pub fn extract_api_public_key(public_key: models::PublicKey) -> Result<PublicKey, ExtractionError> {
    Ok(match public_key {
        models::PublicKey::EcdsaSecp256k1PublicKey { key_hex } => PublicKey::EcdsaSecp256k1(
            EcdsaSecp256k1PublicKey::try_from(from_hex(key_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
        models::PublicKey::EddsaEd25519PublicKey { key_hex } => PublicKey::EddsaEd25519(
            EddsaEd25519PublicKey::try_from(from_hex(key_hex)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
    })
}
