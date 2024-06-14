use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Instant, SystemTime};

use postgres::Client;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;

 #[derive(Debug)]
pub struct IncreaseA {
    pub entity_definitions: Vec<Rc<DbEntityDefinition>>,
}

#[derive(Debug)]
pub struct IncreaseB {
    pub last_state_version: i64,
    pub ledger_transactions: Vec<Rc<DbLedgerTransaction>>,
    pub metadata_entry_history: Vec<Rc<DbMetadataEntryHistory>>,
    pub metadata_aggregate_history: Vec<Rc<RefCell<DbMetadataAggregateHistory>>>,
}

pub struct ProcessingContext<'a> {
    pub db_conn: &'a mut Client,
    pub stopwatch: Option<Instant>,
    pub existing_entities: LruLikeDictionary<String, Rc<DbEntityDefinition>>,
    pub most_recent_entries: LruLikeDictionary<DbMetadataEntryHistoryLookup, Rc<DbMetadataEntryHistory>>,
    pub most_recent_aggregates: LruLikeDictionary<DbMetadataAggregateHistoryLookup, Rc<RefCell<DbMetadataAggregateHistory>>>,
    pub scan_results: Option<ScanResult>,
    pub increase_a: Option<Box<IncreaseA>>,
    pub increase_b: Option<Box<IncreaseB>>,
    pub min_state_version: Option<i64>,
}

impl<'a> ProcessingContext<'a> {
    pub fn new(db_conn: &'a mut Client) -> Self {
        Self {
            db_conn,
            stopwatch: None,
            existing_entities: LruLikeDictionary::new(1000000),
            most_recent_entries: LruLikeDictionary::new(1000000),
            most_recent_aggregates: LruLikeDictionary::new(1000000),
            scan_results: None,
            increase_a: None,
            increase_b: None,
            min_state_version: None,
        }
    }

    pub fn elapsed(&self) -> u128 {
        self.stopwatch.as_ref().unwrap().elapsed().as_millis()
    }
}

pub fn process_tx_stream_step_a(
    existing_entities: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
    seq: &DbSequences,
    sr: &ScanResult
) -> Box<IncreaseA> {
    let mut entity_definitions = vec![];

    for ed in &sr.entity_definitions {
        if let None = existing_entities.get(&ed.0.0) {
            let new_entity = DbEntityDefinition {
                id: seq.next_entity_definition_id(),
                from_state_version: *ed.1,
                address: ed.0.0.clone(),
            };

            let rc = Rc::new(new_entity);
            entity_definitions.push(Rc::clone(&rc));
            existing_entities.put(ed.0.0.clone(), rc);
        }
    }

    return Box::new(IncreaseA {
        entity_definitions,
    });
}

pub fn process_tx_stream_step_b(
    existing_entities: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
    most_recent_entries: &mut LruLikeDictionary<DbMetadataEntryHistoryLookup, Rc<DbMetadataEntryHistory>>,
    most_recent_aggregates: &mut LruLikeDictionary<DbMetadataAggregateHistoryLookup, Rc<RefCell<DbMetadataAggregateHistory>>>,
    sequences: &DbSequences,
    tx_stream: &[Tx]
) -> Box<IncreaseB> {
    let mut ledger_transactions = vec![];
    let mut metadata_entry_history = vec![];
    let mut metadata_aggregate_history = vec![];
    let mut last_state_version = None;

    for tx in tx_stream {
        last_state_version.replace(tx.state_version);

        let new_ledger_transaction = DbLedgerTransaction {
            state_version: tx.state_version,
            created_at: SystemTime::now(),
        };

        ledger_transactions.push(Rc::new(new_ledger_transaction));

        for change in &tx.changes {
            let entity = existing_entities.get(&change.entity_address.clone()).expect("ble, must exist");
            let previous_aggregate = most_recent_aggregates.get(&DbMetadataAggregateHistoryLookup { id: entity.id });
            let tmp_aggregate;

            let mut aggregate = match previous_aggregate {
                Some(&ref e) if e.borrow().from_state_version == tx.state_version => {
                    e.borrow_mut()
                }
                _ => {
                    let new_aggregate = DbMetadataAggregateHistory {
                        id: sequences.next_metadata_aggregate_history_id(),
                        from_state_version: tx.state_version,
                        entity_id: entity.id,
                        entry_ids: match previous_aggregate {
                            None => vec![],
                            Some(&ref e) => e.borrow().entry_ids.clone(),
                        },
                    };

                    tmp_aggregate = Rc::new(RefCell::new(new_aggregate));
                    metadata_aggregate_history.push(Rc::clone(&tmp_aggregate));

                    most_recent_aggregates.put(DbMetadataAggregateHistoryLookup { id: entity.id }, Rc::clone(&tmp_aggregate));

                    tmp_aggregate.borrow_mut()
                }
            };

            let entity_id = existing_entities.get(&change.entity_address).expect("ble, must exist").id;
            let key = change.metadata_key.clone().as_bytes().to_vec();
            let value = change.metadata_value.clone().as_bytes().to_vec();
            let lookup = DbMetadataEntryHistoryLookup { entity_id, key: key.clone() };
            let previous_entry = most_recent_entries.get(&lookup);
            let previous_position = if let Some(previous_entry) = previous_entry {
                aggregate.entry_ids.iter().position(|&x| x == previous_entry.id)
            } else {
                None
            };

            let new_entry_id = sequences.next_metadata_entry_history_id();
            let new_entry = DbMetadataEntryHistory {
                id: new_entry_id,
                from_state_version: tx.state_version,
                entity_id,
                key,
                value,
            };

            if let Some(previous_position) = previous_position {
                aggregate.entry_ids.remove(previous_position);
            }

            aggregate.entry_ids.insert(0, new_entry_id);

            let tmp_entry = Rc::new(new_entry);
            metadata_entry_history.push(Rc::clone(&tmp_entry));
            most_recent_entries.put(lookup, tmp_entry);
        }
    }

    return Box::new(IncreaseB {
        last_state_version: last_state_version.unwrap(),
        ledger_transactions,
        metadata_entry_history,
        metadata_aggregate_history,
    });
}

pub struct ScanResult {
    pub entity_definitions: HashMap<ObservedEntityDefinitionLookup, i64>,
    pub metadata_entry_history: HashSet<ObservedMetadataEntryHistoryLookup>,
    pub metadata_aggregate_history: HashSet<ObservedMetadataAggregateHistoryLookup>,
}

// TODO avoid primitive types
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedEntityDefinitionLookup(pub String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedMetadataEntryHistoryLookup(pub String, pub String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedMetadataAggregateHistoryLookup(pub String);

pub fn scan_tx_stream(tx_stream: &[Tx]) -> ScanResult {
    let mut entity_definitions = HashMap::new();
    let mut metadata_entry_history = HashSet::new();
    let mut metadata_aggregate_history = HashSet::new();

    for tx in tx_stream {
        for change in &tx.changes {
            entity_definitions.entry(ObservedEntityDefinitionLookup(change.entity_address.clone())).or_insert(tx.state_version);
            metadata_entry_history.insert(ObservedMetadataEntryHistoryLookup(change.entity_address.clone(), change.metadata_key.clone()));
            metadata_aggregate_history.insert(ObservedMetadataAggregateHistoryLookup(change.entity_address.clone()));
        }
    }

    return ScanResult {
        entity_definitions,
        metadata_entry_history,
        metadata_aggregate_history,
    }
}

// some boilerplate code below

pub struct Tx {
    pub state_version: i64,
    pub changes: Vec<Change>,
}

pub struct Change {
    pub entity_address: String,
    pub metadata_key: String,
    pub metadata_value: String,
}

#[allow(unused)]
pub struct FetchContext {
    pub next_state_version: i64,
    random: StdRng,
    resources: Vec<String>,
}

#[allow(unused)]
impl FetchContext {
    pub fn new(ledger_tip: Option<i64>) -> Self {
        Self {
            next_state_version: ledger_tip.unwrap_or(0) + 1,
            random: StdRng::seed_from_u64(42),
            resources: vec![
                "Cake".to_owned(),
                "Candy".to_owned(),
                "Chocolate".to_owned(),
                "Lollypop".to_owned()],
        }
    }

    fn next_state_version(&mut self) -> i64 {
        let val = self.next_state_version;
        self.next_state_version += 1;
        return val;
    }
}

#[allow(unused)]
pub fn fetch_sample_tx_stream(context: &mut FetchContext) -> Vec<Tx> {
    let mut res = vec![];

    // TODO should be easy to rewrite with range + map().collect() similarly to C#'s LINQ
    for _ in 1..context.random.gen_range(5..16) {
        let mut changes = vec![];

        for _ in 0..context.random.gen_range(1..6) {
            // TODO: what sucks here? new string allocations, unwrap()s and this deref()
            let metadata_key = String::from_utf8(vec![b'a' + context.random.gen_range(0..26); 3]).unwrap();
            let metadata_value = context.random.gen_range(0..100).to_string();
            let mut entity_address = String::from(context.resources.get(context.random.gen_range(0..context.resources.len())).unwrap().deref());
            entity_address.push((b'A' + context.random.gen_range(0..26)) as char);

            changes.push(Change {
                entity_address,
                metadata_key,
                metadata_value,
            });
        }

        res.push(Tx {
            state_version: context.next_state_version(),
            changes,
        })
    }

    return res;
}

#[allow(unused)]
pub fn print_tx_stream(tx_stream: &[Tx]) {
    println!("### TX STREAM (metadata changes) ###");
    println!("tx #STATE_VERSION: ENTITY.KEY = VALUE[; ENTITY.KEY  = VALUE]*");
    println!();

    for tx in tx_stream {
        let changes = tx.changes.iter().map(|c| format!("{}.{} = {}", c.entity_address, c.metadata_key, c.metadata_value)).collect::<Vec<String>>().join("; ");

        println!("tx #{}: {}", tx.state_version, changes);
    }
}

#[allow(unused)]
pub fn print_entries(entries: &[Rc<DbMetadataEntryHistory>]) {
    println!("### ENTRIES ###");
    println!("ID; FROM_STATE_VERSION; ENTITY_ID; KEY; VALUE");
    println!();

    for entry in entries {
        println!("{}; {}; {}; {}; {}", entry.id, entry.from_state_version, entry.entity_id, String::from_utf8(entry.key.clone()).unwrap(), String::from_utf8(entry.value.clone()).unwrap());
    }
}

#[allow(unused)]
pub fn print_aggregates(aggregates: &[Rc<RefCell<DbMetadataAggregateHistory>>]) {
    println!("### AGGREGATES ###");
    println!("ID; FROM_STATE_VERSION; ENTITY_ID; ENTRY_IDS");
    println!();

    for aggregate in aggregates {
        let aggregate = &aggregate.borrow();
        let entry_ids = aggregate.entry_ids.iter().copied().map(|x| x.to_string()).collect::<Vec<String>>().join(",");

        println!("{}; {}; {}; {}", aggregate.id, aggregate.from_state_version, aggregate.entity_id, entry_ids);
    }
}
