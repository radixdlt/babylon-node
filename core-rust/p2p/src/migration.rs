use sbor::Sbor;

/// Status of the migration
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum MigrationStatus {
    Completed,
}
