use node_common::store::typed_cf_api::SborDbCodec;
use node_common::store::typed_cf_api::{
    DefaultCf, DirectDbCodec, UnitDbCodec,
};
use crate::address_book_components::AddressBookNodeId;
use crate::migration::MigrationStatus;
use crate::typed_cf_api::AddressBookNodeIdDbCodec;

/// Address book and safety state store migration status. Filled once during the migration.
pub struct MigrationStatusCf;
impl DefaultCf for MigrationStatusCf {
    type Key = ();
    type Value = MigrationStatus;

    const DEFAULT_NAME: &'static str = "migration_status";
    type KeyCodec = UnitDbCodec;
    type ValueCodec = SborDbCodec<MigrationStatus>;
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
