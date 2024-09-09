use crate::engine_prelude::{ScryptoCategorize, ScryptoDecode, ScryptoEncode};

/// The ID of the node stored in the Address book (Secp256k1 public key)
#[derive(
    Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode, PartialOrd, Ord, PartialEq, Eq,
)]
#[sbor(transparent)]
pub struct NodeSecp256k1PublicKey(pub [u8; NodeSecp256k1PublicKey::LENGTH]);

impl NodeSecp256k1PublicKey {
    pub const LENGTH: usize = 33;

    pub fn new(id: [u8; Self::LENGTH]) -> Self {
        Self(id)
    }

    pub fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}

/// The Secp256K1 signature
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NodeSignature(pub [u8; NodeSignature::LENGTH]);

impl NodeSignature {
    pub const LENGTH: usize = 65; // v(1) + r(32) + s(32)

    pub fn new(signature: [u8; Self::LENGTH]) -> Self {
        Self(signature)
    }

    pub fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}
