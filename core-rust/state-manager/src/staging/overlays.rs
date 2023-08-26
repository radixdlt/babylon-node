use radix_engine_store_interface::interface::{DatabaseUpdate, DbSortKey, PartitionEntry};
use std::cmp::Ordering;
use std::hash::Hash;
use std::iter::Peekable;

use crate::store::traits::{SubstateNodeAncestryRecord, SubstateNodeAncestryStore};
use radix_engine_common::types::NodeId;
use utils::prelude::NonIterMap;

pub struct SubstateOverlayIterator<'a> {
    root_db: Peekable<Box<dyn Iterator<Item = PartitionEntry> + 'a>>,
    overlay: Peekable<Box<dyn Iterator<Item = (DbSortKey, DatabaseUpdate)> + 'a>>,
}

impl<'a> SubstateOverlayIterator<'a> {
    pub fn new(
        root_db: Peekable<Box<dyn Iterator<Item = PartitionEntry> + 'a>>,
        overlay: Peekable<Box<dyn Iterator<Item = (DbSortKey, DatabaseUpdate)> + 'a>>,
    ) -> Self {
        Self { root_db, overlay }
    }
}

impl<'a> Iterator for SubstateOverlayIterator<'a> {
    type Item = PartitionEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.root_db.peek(), self.overlay.peek()) {
            (Some((l, _)), Some((r, _))) => match l.cmp(r) {
                Ordering::Less => self.root_db.next(),
                Ordering::Equal => {
                    let _discarded_root_entry = self.root_db.next();
                    let overlay_entry = self.overlay.next().unwrap();
                    match overlay_entry.1 {
                        DatabaseUpdate::Set(value) => Some((overlay_entry.0, value)),
                        DatabaseUpdate::Delete => self.next(),
                    }
                }
                Ordering::Greater => {
                    let next = self.overlay.next().unwrap();
                    match next.1 {
                        DatabaseUpdate::Set(value) => Some((next.0, value)),
                        DatabaseUpdate::Delete => self.next(),
                    }
                }
            },
            (None, Some(_)) => {
                let mut next = self.overlay.next();
                while let Some((k, database_update)) = &next {
                    match database_update {
                        DatabaseUpdate::Set(value) => return Some((k.clone(), value.clone())),
                        DatabaseUpdate::Delete => {
                            next = self.overlay.next();
                        }
                    }
                }
                None
            }
            (Some(_), None) => self.root_db.next(),
            (None, None) => None,
        }
    }
}

pub trait MapLike<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
}

impl<K: Hash + Eq, V: Clone> MapLike<K, V> for NonIterMap<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        NonIterMap::get(self, key)
    }
}

impl<K: Hash + Eq, V: Clone> MapLike<K, V> for im::HashMap<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        im::HashMap::get(self, key)
    }
}

pub struct MapSubstateNodeAncestryStore<'m, M> {
    map: &'m M,
}

impl<'m, M> MapSubstateNodeAncestryStore<'m, M> {
    pub fn wrap(map: &'m M) -> Self {
        Self { map }
    }
}

impl<'m, M: MapLike<NodeId, SubstateNodeAncestryRecord>> SubstateNodeAncestryStore
    for MapSubstateNodeAncestryStore<'m, M>
{
    fn get_ancestry(&self, node_id: &NodeId) -> Option<SubstateNodeAncestryRecord> {
        self.map.get(node_id).cloned()
    }
}

pub struct StagedSubstateNodeAncestryStore<'s, U, O> {
    underlying: &'s U,
    overlay: &'s O,
}

impl<'s, U, O> StagedSubstateNodeAncestryStore<'s, U, O> {
    pub fn new(underlying: &'s U, overlay: &'s O) -> Self {
        Self {
            underlying,
            overlay,
        }
    }
}

impl<'s, U: SubstateNodeAncestryStore, O: SubstateNodeAncestryStore> SubstateNodeAncestryStore
    for StagedSubstateNodeAncestryStore<'s, U, O>
{
    fn batch_get_ancestry<'a>(
        &self,
        node_ids: impl IntoIterator<Item = &'a NodeId>,
    ) -> Vec<Option<SubstateNodeAncestryRecord>> {
        let node_ids = Vec::from_iter(node_ids);
        let overlay_results = self.overlay.batch_get_ancestry(node_ids.iter().cloned());
        let outside_overlay_node_ids = node_ids
            .into_iter()
            .zip(overlay_results.iter())
            .filter(|(_, result)| result.is_none())
            .map(|(node_id, _)| node_id)
            .collect::<Vec<_>>();
        let mut underlying_result_iter = self
            .underlying
            .batch_get_ancestry(outside_overlay_node_ids)
            .into_iter();
        overlay_results
            .into_iter()
            .map(|overlay_result| {
                overlay_result.map(Some).unwrap_or_else(|| {
                    underlying_result_iter
                        .next()
                        .expect("less results than IDs")
                })
            })
            .collect()
    }
}
