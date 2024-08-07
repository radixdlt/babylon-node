use crate::store::p2p::address_book_components::AddressBookNodeId;
use crate::store::p2p::migration::{MigrationId, MigrationStatus};
use crate::store::typed_cf_api::{
    AddressBookNodeIdDbCodec, DefaultCf, DirectDbCodec, MigrationIdDbCodec, MigrationStatusDbCodec
    , UnitDbCodec,
};

/// Address book and safety state store migration status. Filled once during the migration.
pub struct MigrationStatusCf;
impl DefaultCf for MigrationStatusCf {
    type Key = MigrationId;
    type Value = MigrationStatus;

    const DEFAULT_NAME: &'static str = "migration_status";
    type KeyCodec = MigrationIdDbCodec;
    type ValueCodec = MigrationStatusDbCodec;
}

/// Address book
pub struct AddressBookCf;
impl DefaultCf for AddressBookCf {
    type Key = AddressBookNodeId;
    type Value = Vec<u8>;

    const DEFAULT_NAME: &'static str = "address_book";
    type KeyCodec = AddressBookNodeIdDbCodec;
    type ValueCodec = DirectDbCodec;
}

/// Safety store
pub struct SafetyStoreCf;
impl DefaultCf for SafetyStoreCf {
    type Key = ();
    type Value = Vec<u8>;

    const DEFAULT_NAME: &'static str = "safety_store";
    type KeyCodec = UnitDbCodec;
    type ValueCodec = DirectDbCodec;
}

pub struct HighPriorityPeersCf;
impl DefaultCf for HighPriorityPeersCf {
    type Key = ();
    type Value = Vec<u8>;

    const DEFAULT_NAME: &'static str = "high_priority_peers";
    type KeyCodec = UnitDbCodec;
    type ValueCodec = DirectDbCodec;
}
