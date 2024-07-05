use std::collections::{HashMap, HashSet};
use std::error::Error;

use radix_substate_store_queries::typed_substate_layout::{TypedMetadataModuleSubstateKey, TypedSubstateKey};

use state_manager::StateVersion;

use crate::da::Tx;

pub struct ScanResult {
    pub entity_definitions: HashMap<ObservedEntityDefinitionLookup, StateVersion>,
    pub metadata_entry_history: HashSet<ObservedMetadataEntryHistoryLookup>,
    pub metadata_aggregate_history: HashSet<ObservedMetadataAggregateHistoryLookup>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedEntityDefinitionLookup(pub String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedMetadataEntryHistoryLookup(pub String, pub String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObservedMetadataAggregateHistoryLookup(pub String);

pub fn scan_tx_stream(tx_stream: &[Tx]) -> Result<ScanResult, Box<dyn Error>> {
    let mut entity_definitions = HashMap::new();
    let mut metadata_entry_history = HashSet::new();
    let mut metadata_aggregate_history = HashSet::new();

    for tx in tx_stream {
        for sc in &tx.substate_changes {
            entity_definitions.entry(ObservedEntityDefinitionLookup(sc.node_address.clone())).or_insert(tx.state_version);

            if let TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(entry)) = &sc.key {
                metadata_aggregate_history.insert(ObservedMetadataAggregateHistoryLookup(sc.node_address.clone()));
                metadata_entry_history.insert(ObservedMetadataEntryHistoryLookup(sc.node_address.clone(), entry.clone()));
            }

            // if let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::GenericKeyValueStoreKey(key)) = &sc.key {
            //
            // }
        }
    }

    return Ok(ScanResult {
        entity_definitions,
        metadata_entry_history,
        metadata_aggregate_history,
    })
}
