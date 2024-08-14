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

/// Timestamp of the various peer-related events
// At present it's just an alias for i64. Later we may want to replace it with struct using crono crate and 
// do something like shown below to transparently convert to/from internal representation 
// (once there will be real usage at Rust side).
// #[sbor(
//     as_type = "i64",
//     as_ref = "self.timestamp()",
//     from_value = "Self(DateTime::from_timestamp(value, 0))"
// )]
type PeerTimestamp = i64;

/// Peer address entry with all components
#[derive(Clone, Sbor)]
pub struct PeerAddress {
    pub encoded_uri: Vec<u8>,
    pub latest_connection_status: Option<ConnectionStatus>,
    pub last_seen: Option<PeerTimestamp>,
}

#[derive(Clone, Copy, Sbor)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

/// Address book entry
#[derive(Clone, Sbor)]
pub struct AddressBookEntry {
    pub node_id: AddressBookNodeId,
    pub banned_until: Option<PeerTimestamp>,
    pub known_addresses: Vec<PeerAddress>,
}
