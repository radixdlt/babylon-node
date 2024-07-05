use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use postgres::Transaction;
use crate::da::{SubstateChange, Tx};
use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;
use crate::da::processors::DbIncrease;

pub struct MetadataProcessor {
    entries_to_add: Vec<Rc<DbMetadataEntryHistory>>,
    aggregates_to_add: Vec<Rc<RefCell<DbMetadataAggregateHistory>>>,
}

impl MetadataProcessor {
    pub fn new() -> Self {
        Self {
            entries_to_add: vec![],
            aggregates_to_add: vec![],
        }
    }

    pub fn process_change(
        &mut self,
        change: &SubstateChange,
        tx: &Tx,
        sequences: &DbSequences,
        existing_entities: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
        most_recent_entries: &mut LruLikeDictionary<DbMetadataEntryHistoryLookup, Rc<DbMetadataEntryHistory>>,
        most_recent_aggregates: &mut LruLikeDictionary<DbMetadataAggregateHistoryLookup, Rc<RefCell<DbMetadataAggregateHistory>>>
    ) -> Result<(), Box<dyn Error>> {

        let entity = existing_entities.get(&change.node_address.clone()).expect("ble, must exist");
        let previous_aggregate = most_recent_aggregates.get(&DbMetadataAggregateHistoryLookup { id: entity.id });
        let tmp_aggregate;

        let mut aggregate = match previous_aggregate {
            Some(&ref e) if e.borrow().from_state_version == tx.state_version.number() as i64 => {
                e.borrow_mut()
            }
            _ => {
                let new_aggregate = DbMetadataAggregateHistory {
                    id: sequences.next_metadata_aggregate_history_id(),
                    from_state_version: tx.state_version.number() as i64,
                    entity_id: entity.id,
                    entry_ids: match previous_aggregate {
                        None => vec![],
                        Some(&ref e) => e.borrow().entry_ids.clone(),
                    },
                };

                tmp_aggregate = Rc::new(RefCell::new(new_aggregate));
                self.aggregates_to_add.push(Rc::clone(&tmp_aggregate));

                most_recent_aggregates.put(DbMetadataAggregateHistoryLookup { id: entity.id }, Rc::clone(&tmp_aggregate));

                tmp_aggregate.borrow_mut()
            }
        };

        return Ok(());

        // let entity_id = existing_entities.get(&change.node_address).expect("ble, must exist").id;
        // let key = change.metadata_key.clone().as_bytes().to_vec();
        // let value = change.metadata_value.clone().as_bytes().to_vec();
        // let lookup = DbMetadataEntryHistoryLookup { entity_id, key: key.clone() };
        // let previous_entry = most_recent_entries.get(&lookup);
        // let previous_position = if let Some(previous_entry) = previous_entry {
        //     aggregate.entry_ids.iter().position(|&x| x == previous_entry.id)
        // } else {
        //     None
        // };
        //
        // let new_entry_id = sequences.next_metadata_entry_history_id();
        // let new_entry = DbMetadataEntryHistory {
        //     id: new_entry_id,
        //     from_state_version: tx.state_version,
        //     entity_id,
        //     key,
        //     value,
        // };
        //
        // if let Some(previous_position) = previous_position {
        //     aggregate.entry_ids.remove(previous_position);
        // }
        //
        // aggregate.entry_ids.insert(0, new_entry_id);
        //
        // let tmp_entry = Rc::new(new_entry);
        // self.entries_to_add.push(Rc::clone(&tmp_entry));
        // most_recent_entries.put(lookup, tmp_entry);
        //
        // Ok(())
    }
}

impl DbIncrease for MetadataProcessor {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>> {
        let mut cnt = 0;

        cnt += persist_metadata_entry_history(client, &self.entries_to_add)?;
        cnt += persist_metadata_aggregate_history(client, &self.aggregates_to_add)?;

        Ok(cnt)
    }
}
