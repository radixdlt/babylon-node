use std::error::Error;
use std::rc::Rc;

use postgres::Transaction;

use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;
use crate::da::processors::DbIncrease;
use crate::da::scan_result::ScanResult;

pub struct EntityDefinitionProcessor {
    entities_to_add: Vec<Rc<DbEntityDefinition>>,
}

impl EntityDefinitionProcessor {
    pub fn new() -> Self {
        Self {
            entities_to_add: vec![],
        }
    }

    pub fn process_scan_result(
        &mut self,
        sequences: &DbSequences,
        existing_entities_by_address: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
        existing_entities: &mut LruLikeDictionary<DbEntityDefinitionLookup, Rc<DbEntityDefinition>>,
        scan_result: &ScanResult
    ) -> Result<(), Box<dyn Error>> {
        for (lookup, state_version) in &scan_result.entity_definitions {
            if let None = existing_entities_by_address.get(&lookup.0) {
                let new_entity = DbEntityDefinition {
                    id: sequences.next_entity_definition_id(),
                    from_state_version: state_version.number() as i64,
                    address: lookup.0.clone(),
                };

                let rc = Rc::new(new_entity);
                self.entities_to_add.push(Rc::clone(&rc));
                existing_entities.put(rc.to_lookup(), Rc::clone(&rc));
                existing_entities_by_address.put(rc.address.clone(), rc);
            }
        }

        Ok(())
    }
}

impl DbIncrease for EntityDefinitionProcessor {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>> {
        let mut cnt = 0;

        cnt += persist_entity_definitions(client, &self.entities_to_add)?;

        Ok(cnt)
    }
}
