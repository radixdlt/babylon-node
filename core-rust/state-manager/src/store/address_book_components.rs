use crate::engine_prelude::Sbor;


/// The ID of the node stored in the Address book (Secp256k1 public key)
#[derive(Clone, Copy, Sbor)]
pub struct AddressBookNodeId(pub [u8; AddressBookNodeId::LENGTH]);

impl AddressBookNodeId {
    pub const LENGTH: usize = 33;

    pub fn new(id: [u8; Self::LENGTH]) -> Self {
        Self(id)
    }

    pub fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}

/// Address book entry
#[derive(Clone, Sbor)]
pub struct AddressBookEntry {
    pub node_id: AddressBookNodeId,
    pub banned_until: Option<i64>,
    pub known_addresses: Vec<String>,
}
