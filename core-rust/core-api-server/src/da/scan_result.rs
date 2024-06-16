use std::collections::{HashMap, HashSet};

pub struct ScanResult {
    pub entity_definitions: HashMap<ObservedEntityDefinitionLookup, i64>,
    pub metadata_entry_history: HashSet<ObservedMetadataEntryHistoryLookup>,
    pub metadata_aggregate_history: HashSet<ObservedMetadataAggregateHistoryLookup>,
}

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

// TODO get rid of those two
pub struct Tx {
    pub state_version: i64,
    pub changes: Vec<Change>,
}

pub struct Change {
    pub entity_address: String,
    pub metadata_key: String,
    pub metadata_value: String,
}
