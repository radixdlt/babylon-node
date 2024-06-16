mod db_reader;
mod db_sequences;
mod db_writer;

use std::time::SystemTime;
pub use db_reader::*;
pub use db_sequences::*;
pub use db_writer::*;

#[derive(Debug)]
pub struct DbLedgerTransaction {
    pub state_version: i64,
    pub created_at: SystemTime,
}

#[derive(Debug)]
pub struct DbEntityDefinition {
    pub id: i64,
    pub from_state_version: i64,
    pub address: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DbEntityDefinitionLookup {
    pub id: i64,
}

impl DbEntityDefinition {
    pub fn to_lookup(&self) -> DbEntityDefinitionLookup {
        DbEntityDefinitionLookup {
            id: self.id,
        }
    }
}

#[derive(Debug)]
pub struct DbMetadataEntryHistory {
    pub id: i64,
    pub from_state_version: i64,
    pub entity_id: i64,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DbMetadataEntryHistoryLookup {
    pub entity_id: i64,
    pub key: Vec<u8>,
}

impl DbMetadataEntryHistory {
    pub fn to_lookup(&self) -> DbMetadataEntryHistoryLookup {
        DbMetadataEntryHistoryLookup {
            entity_id: self.entity_id,
            key: self.key.clone(),
        }
    }
}

#[derive(Debug)]
pub struct DbMetadataAggregateHistory {
    pub id: i64,
    pub from_state_version: i64,
    pub entity_id: i64,
    pub entry_ids: Vec<i64>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DbMetadataAggregateHistoryLookup {
    pub id: i64,
}

impl DbMetadataAggregateHistory {
    pub fn to_lookup(&self) -> DbMetadataAggregateHistoryLookup {
        DbMetadataAggregateHistoryLookup {
            id: self.id,
        }
    }
}
