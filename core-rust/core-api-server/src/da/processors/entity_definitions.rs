use std::error::Error;
use std::rc::Rc;
use postgres::Transaction;
use crate::da::scan_result::ScanResult;
use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;
use crate::da::processors::DbIncrease;

#[derive(Debug)]
pub struct EntityDefinitionsDbIncrease {
    entity_definitions: Vec<Rc<DbEntityDefinition>>,
}

impl DbIncrease for EntityDefinitionsDbIncrease {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>> {
        let mut cnt = 0;

        cnt += persist_entity_definitions(client, &self.entity_definitions)?;

        Ok(cnt)
    }
}

pub fn todo_rename_create_missing_entities(
    existing_entities_by_address: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
    existing_entities: &mut LruLikeDictionary<DbEntityDefinitionLookup, Rc<DbEntityDefinition>>,
    seq: &DbSequences,
    sr: &ScanResult
) -> EntityDefinitionsDbIncrease {
    let mut entity_definitions = vec![];

    for (lookup, state_version) in &sr.entity_definitions {
        if let None = existing_entities_by_address.get(&lookup.0) {
            let new_entity = DbEntityDefinition {
                id: seq.next_entity_definition_id(),
                from_state_version: state_version.number() as i64,
                address: lookup.0.clone(),
            };

            let rc = Rc::new(new_entity);
            entity_definitions.push(Rc::clone(&rc));
            existing_entities.put(rc.to_lookup(), Rc::clone(&rc));
            existing_entities_by_address.put(rc.address.clone(), rc);
        }
    }

    return EntityDefinitionsDbIncrease {
        entity_definitions
    };
}