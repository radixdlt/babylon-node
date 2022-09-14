use scrypto::crypto::{
    EcdsaPublicKey, EcdsaSignature, Ed25519PublicKey, Ed25519Signature, PublicKey, Signature,
    SignatureWithPublicKey,
};

use crate::core_api::*;

pub fn to_api_public_key(public_key: PublicKey) -> models::PublicKey {
    match public_key {
        PublicKey::Ecdsa(key) => models::PublicKey::EcdsaSecp256k1PublicKey {
            key_bytes: to_hex(key.to_vec()),
        },
        PublicKey::Ed25519(key) => models::PublicKey::EddsaEd25519PublicKey {
            key_bytes: to_hex(key.to_vec()),
        },
    }
}

pub fn to_api_signature(signature: Signature) -> models::Signature {
    match signature {
        Signature::Ecdsa(sig) => models::Signature::EcdsaSecp256k1Signature {
            signature_bytes: to_hex(sig.to_vec()),
        },
        Signature::Ed25519(sig) => models::Signature::EddsaEd25519Signature {
            signature_bytes: to_hex(sig.to_vec()),
        },
    }
}

pub fn to_api_signature_with_public_key(
    sig_with_public_key: SignatureWithPublicKey,
) -> models::SignatureWithPublicKey {
    match sig_with_public_key {
        SignatureWithPublicKey::Ecdsa(sig) => {
            models::SignatureWithPublicKey::EcdsaSecp256k1SignatureWithPublicKey {
                recoverable_signature: Box::new(models::EcdsaSecp256k1Signature {
                    key_type: models::PublicKeyType::EcdsaSecp256k1,
                    signature_bytes: to_hex(sig.to_vec()),
                }),
            }
        }
        SignatureWithPublicKey::Ed25519(pub_key, sig) => {
            models::SignatureWithPublicKey::EddsaEd25519SignatureWithPublicKey {
                signature: Box::new(models::EddsaEd25519Signature {
                    key_type: models::PublicKeyType::EddsaEd25519,
                    signature_bytes: to_hex(sig.to_vec()),
                }),
                public_key: Box::new(models::EddsaEd25519PublicKey {
                    key_type: models::PublicKeyType::EddsaEd25519,
                    key_bytes: to_hex(pub_key.to_vec()),
                }),
            }
        }
    }
}

#[allow(dead_code)]
pub fn extract_api_signature(signature: models::Signature) -> Result<Signature, ExtractionError> {
    Ok(match signature {
        models::Signature::EcdsaSecp256k1Signature { signature_bytes } => Signature::Ecdsa(
            EcdsaSignature::try_from(from_hex(signature_bytes)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
        models::Signature::EddsaEd25519Signature { signature_bytes } => Signature::Ed25519(
            Ed25519Signature::try_from(from_hex(signature_bytes)?.as_ref())
                .map_err(|_| ExtractionError::InvalidSignature)?,
        ),
    })
}

pub fn extract_api_public_key(public_key: models::PublicKey) -> Result<PublicKey, ExtractionError> {
    Ok(match public_key {
        models::PublicKey::EcdsaSecp256k1PublicKey { key_bytes } => PublicKey::Ecdsa(
            EcdsaPublicKey::try_from(from_hex(key_bytes)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
        models::PublicKey::EddsaEd25519PublicKey { key_bytes } => PublicKey::Ed25519(
            Ed25519PublicKey::try_from(from_hex(key_bytes)?.as_ref())
                .map_err(|_| ExtractionError::InvalidPublicKey)?,
        ),
    })
}
