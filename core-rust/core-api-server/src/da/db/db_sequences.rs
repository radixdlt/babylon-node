use std::cell::Cell;
use std::collections::HashMap;
use std::error::Error;
use postgres::{Client, Transaction};
use crate::da::db::*;

pub struct DbSequences {
    next_entity_definitions_id: Cell<i64>,
    next_metadata_entry_history_id: Cell<i64>,
    next_metadata_aggregate_history_id: Cell<i64>,
}

impl DbSequences {
    pub fn new(postgres_db: &mut Client) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            next_entity_definitions_id: Cell::new(read_next_sequence_id(postgres_db, "entity_definitions")?),
            next_metadata_entry_history_id: Cell::new(read_next_sequence_id(postgres_db, "metadata_entry_history")?),
            next_metadata_aggregate_history_id: Cell::new(read_next_sequence_id(postgres_db, "metadata_aggregate_history")?),
        })
    }

    pub fn persist(&self, postgres_db: &mut Transaction) -> Result<(), Box<dyn Error>> {
        let values = HashMap::from([
            ("entity_definitions", self.next_entity_definitions_id.get()),
            ("metadata_entry_history", self.next_metadata_entry_history_id.get()),
            ("metadata_aggregate_history", self.next_metadata_aggregate_history_id.get()),
        ]);

        persist_sequences(postgres_db, values)?;

        Ok(())
    }

    pub fn next_entity_definition_id(&self) -> i64 {
        let curr = self.next_entity_definitions_id.get();
        self.next_entity_definitions_id.set(curr + 1);
        curr
    }

    pub fn next_metadata_entry_history_id(&self) -> i64 {
        let curr = self.next_metadata_entry_history_id.get();
        self.next_metadata_entry_history_id.set(curr + 1);
        curr
    }

    pub fn next_metadata_aggregate_history_id(&self) -> i64 {
        let curr = self.next_metadata_aggregate_history_id.get();
        self.next_metadata_aggregate_history_id.set(curr + 1);
        curr
    }
}