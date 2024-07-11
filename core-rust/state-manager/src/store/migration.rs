use sbor::Sbor;

/// Identifiers for the migrated stores
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum MigrationId {
    AddressBook,
    SafetyState,
}

/// Status of the migration
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum MigrationStatus {
    Completed,
}
