use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use postgres::Client;
use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;
use crate::da::processors::DbIncrease;

pub struct ProcessingContext<'a> {
    pub db_conn: &'a mut Client,
    pub stopwatch: Option<Instant>,
    pub existing_entities_by_address: LruLikeDictionary<String, Rc<DbEntityDefinition>>,
    pub existing_entities: LruLikeDictionary<DbEntityDefinitionLookup, Rc<DbEntityDefinition>>,
    pub most_recent_metadata_entry_hisotry: LruLikeDictionary<DbMetadataEntryHistoryLookup, Rc<DbMetadataEntryHistory>>,
    pub most_recent_metadata_aggregate_history: LruLikeDictionary<DbMetadataAggregateHistoryLookup, Rc<RefCell<DbMetadataAggregateHistory>>>,
    pub most_recent_role_assignment_entry_hisotry: LruLikeDictionary<DbRoleAssignmentEntryHistoryLookup, Rc<DbRoleAssignmentEntryHistory>>,
    pub most_recent_role_assignment_aggregate_history: LruLikeDictionary<DbRoleAssignmentAggregateHistoryLookup, Rc<RefCell<DbRoleAssignmentAggregateHistory>>>,
    pub increases: Vec<Box<dyn DbIncrease>>,
    pub ledger_tip: i64,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(db_conn: &'a mut Client, ledger_tip: i64) -> Self {
        Self {
            db_conn,
            stopwatch: None,
            existing_entities_by_address: LruLikeDictionary::new(1000000),
            existing_entities: LruLikeDictionary::new(1000000), // must be same as above
            most_recent_metadata_entry_hisotry: LruLikeDictionary::new(1000000),
            most_recent_metadata_aggregate_history: LruLikeDictionary::new(1000000),
            most_recent_role_assignment_entry_hisotry: LruLikeDictionary::new(1000000),
            most_recent_role_assignment_aggregate_history: LruLikeDictionary::new(1000000),
            increases: vec![],
            ledger_tip,
        }
    }

    pub fn elapsed(&self) -> u128 {
        self.stopwatch.as_ref().unwrap().elapsed().as_millis()
    }
}
