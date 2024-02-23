/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */
#![allow(dead_code)]
// TODO(historical-state): Remove the allow above once the impl here is used

use std::ops::Deref;
use std::{iter, mem};

use crate::engine_prelude::*;
use crate::store::traits::*;
use crate::StateVersion;

/// An implementation of a [`SubstateDatabase`] viewed at a specific [`StateVersion`].
///
/// This database is backed by:
/// - a [`ReadableTreeStore`] - a versioned source of ReNodes / Partitions / Substates metadata,
/// - and a [`SubstateUpsertValueStore`] - a raw history of Substate values' upserts.
pub struct StateHashTreeBasedSubstateDatabase<'t, 'v, T, V> {
    tree_store: &'t T,
    upsert_value_store: &'v V,
    at_state_version: StateVersion,
}

impl<'t, 'v, T: ReadableTreeStore, V: SubstateUpsertValueStore>
    StateHashTreeBasedSubstateDatabase<'t, 'v, T, V>
{
    /// Creates an instance backed by the given lower-level stores and scoped at the given version.
    pub fn new(
        tree_store: &'t T,
        upsert_value_store: &'v V,
        at_state_version: StateVersion,
    ) -> Self {
        Self {
            tree_store,
            upsert_value_store,
            at_state_version,
        }
    }
}

impl<'t, 'v, T: ReadableTreeStore, V: SubstateUpsertValueStore> SubstateDatabase
    for StateHashTreeBasedSubstateDatabase<'t, 'v, T, V>
{
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        // Performance note:
        // When reading from a tree-based store, getting a leaf has the same cost as starting an
        // iterator and taking its first element. The only possible savings would be available in
        // the "not found" case, which is rare in our use-cases.
        // Hence, for simplicity, we prefer to re-use a single (non-trivial) leaf-locating code.
        self.list_entries_from(partition_key, Some(sort_key))
            .next()
            .filter(|(first_ge_sort_key, _)| first_ge_sort_key == sort_key)
            .map(|(_, value)| value)
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let DbPartitionKey {
            node_key,
            partition_num,
        } = partition_key.clone();

        let node_tier_tree_browser =
            TreeStoreBrowser::new(self.tree_store, self.at_state_version.number());
        let node_leaf = node_tier_tree_browser.get_leaf(&NibblePath::new_even(node_key));
        let Some(node_leaf) = node_leaf else {
            return Box::new(iter::empty()); // The requested ReNode does not exist - treat as empty
        };

        let partition_tier_tree_browser = node_tier_tree_browser.nested(&node_leaf);
        let partition_leaf =
            partition_tier_tree_browser.get_leaf(&NibblePath::new_even(vec![partition_num]));
        let Some(partition_leaf) = partition_leaf else {
            return Box::new(iter::empty()); // The requested Partition does not exist - it is empty
        };

        let substate_tier_tree_browser = partition_tier_tree_browser.nested(&partition_leaf);
        let sort_key_bytes = from_sort_key
            .map(|sort_key| sort_key.0.clone())
            .unwrap_or_default();

        let partition_key = partition_key.clone(); // avoid lifetime dependency within iterator
        Box::new(substate_tier_tree_browser
            .iter_leafs_from(&NibblePath::new_even(sort_key_bytes))
            .map(move |ResolvedLeaf { nibble_path, last_hash_change_version }| {
                let sort_key = DbSortKey(nibble_path.bytes().to_vec());
                let value = self.upsert_value_store
                    .get_value(&partition_key, &sort_key, StateVersion::of(last_hash_change_version))
                    .expect("DB inconsistency: value not found for substate upsert found in tree");
                (sort_key, value)
            }))
    }
}

/// An implementation delegate allowing to browse a [`ReadableTreeStore`] at a specific version.
struct TreeStoreBrowser<T> {
    tree_store: T,
    version: Version,
}

impl<'t, T: Deref<Target = impl ReadableTreeStore> + Clone + 't> TreeStoreBrowser<T> {
    /// Creates an instance directly.
    pub fn new(tree_store: T, version: Version) -> Self {
        Self {
            tree_store,
            version,
        }
    }

    /// Creates an instance for browsing a [`NestedTreeStore`] nested at the given leaf.
    /// This effectively allows to access a lower Tier tree (of a Radix-specific 3-Tier JMT).
    pub fn nested(&self, leaf: &ResolvedLeaf) -> TreeStoreBrowser<Rc<NestedTreeStore<T>>> {
        let nested_tree_store =
            NestedTreeStore::new(self.tree_store.clone(), leaf.nibble_path.bytes().to_vec());
        TreeStoreBrowser::new(Rc::new(nested_tree_store), leaf.last_hash_change_version)
    }

    /// Returns a specific leaf, if found by starting at the scoped version's root and following the
    /// given [`NibblePath`].
    pub fn get_leaf(&self, nibble_path: &NibblePath) -> Option<ResolvedLeaf> {
        self.iter_leafs_from(nibble_path)
            .next()
            .filter(|leaf| &leaf.nibble_path == nibble_path)
    }

    /// Returns an iterator of all leafs reachable from the scoped version's root, in
    /// lexicographical order, starting from the given one.
    pub fn iter_leafs_from(
        &self,
        nibble_path: &NibblePath,
    ) -> Box<dyn Iterator<Item = ResolvedLeaf> + 't> {
        recurse_until_leafs(
            self.tree_store.clone(),
            NodeKey::new_empty_path(self.version),
            nibble_path.nibbles().collect(),
        )
    }
}

/// Returns a lexicographically-sorted iterator of all the `tree_store`'s [`ResolvedLeaf`]s having
/// [`NibblePath`]s greater or equal to the given `from_nibbles`.
///
/// The algorithm:
/// - starts at the given `at_key`,
/// - then goes down the tree, guided by the given `from_nibbles` chain, for as long as it is
///   possible,
///   - Note: this means it will either locate exactly this nibble path, or - if it does not
///     exist - settle at its direct successor.
/// - and then continues as if it was a classic DFS all the way,
/// - but only leaf nodes are returned.
///
/// The implementation is a lazy recursive composite of child iterators.
fn recurse_until_leafs<'t, T: Deref<Target = impl ReadableTreeStore> + Clone + 't>(
    tree_store: T,
    at_key: NodeKey,
    from_nibbles: VecDeque<Nibble>,
) -> Box<dyn Iterator<Item = ResolvedLeaf> + 't> {
    let Some(node) = tree_store.get_node(&at_key) else {
        panic!("{:?} referenced but not found in the storage", at_key);
    };
    match node {
        TreeNode::Internal(internal) => {
            let mut child_from_nibbles = from_nibbles;
            let from_nibble = child_from_nibbles
                .pop_front()
                .unwrap_or_else(|| Nibble::from(0));
            Box::new(
                internal
                    .children
                    .into_iter()
                    .filter(move |child| child.nibble >= from_nibble)
                    .flat_map(move |child| {
                        let child_key = at_key.gen_child_node_key(child.version, child.nibble);
                        let child_from_nibbles = if child.nibble == from_nibble {
                            mem::take(&mut child_from_nibbles)
                        } else {
                            VecDeque::new()
                        };
                        recurse_until_leafs(tree_store.clone(), child_key, child_from_nibbles)
                    }),
            )
        }
        TreeNode::Leaf(leaf) => Box::new(
            Some(leaf)
                .filter(move |leaf| leaf.key_suffix.nibbles().ge(from_nibbles))
                .map(
                    |TreeLeafNode {
                         key_suffix,
                         last_hash_change_version,
                         ..
                     }| ResolvedLeaf {
                        nibble_path: NibblePath::from_iter(
                            at_key.nibble_path().nibbles().chain(key_suffix.nibbles()),
                        ),
                        last_hash_change_version,
                    },
                )
                .into_iter(),
        ),
        TreeNode::Null => Box::new(iter::empty()),
    }
}

/// A relevant information from a [`TreeLeafNode`] (specifically, with a resolved [`NibblePath`]).
#[derive(Clone, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ResolvedLeaf {
    nibble_path: NibblePath,
    last_hash_change_version: Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updated_substate_has_different_values_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, Some(from_seed(8)));
    }

    #[test]
    fn reset_substate_has_different_values_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 =
            test_stores.reset_partition(&partition_key(8, 7), vec![(sort_key(2), from_seed(9))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, Some(from_seed(9)));
    }

    #[test]
    fn unchanged_substate_has_same_value_across_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 6), &sort_key(3));
        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 6), &sort_key(3));
        assert_eq!(value_v2, value_v1);
    }

    #[test]
    fn substate_created_later_has_no_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores
            .put_substate_changes(vec![change(8, 7, 2, Some(8)), change(8, 1, 9, Some(1))]);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 1), &sort_key(9));
        assert_eq!(value_v1, None);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 1), &sort_key(9));
        assert_eq!(value_v2, Some(from_seed(1)));
    }

    #[test]
    fn substate_deleted_later_still_has_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 =
            test_stores.put_substate_changes(vec![change(8, 7, 2, None), change(8, 1, 9, Some(1))]);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, None);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));
    }

    #[test]
    fn substate_inside_deleted_partition_still_has_value_earlier() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 6, 3, Some(6)),
            change(8, 7, 2, Some(7)),
            change(9, 2, 5, Some(2)),
        ]);
        let v2 = test_stores.reset_partition(&partition_key(8, 7), vec![]);

        let subject_v2 = test_stores.create_subject(v2);
        let value_v2 = subject_v2.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v2, None);

        let subject_v1 = test_stores.create_subject(v1);
        let value_v1 = subject_v1.get_substate(&partition_key(8, 7), &sort_key(2));
        assert_eq!(value_v1, Some(from_seed(7)));
    }

    #[test]
    fn lists_partition_substates_at_different_versions() {
        let mut test_stores = TestStores::new_empty();
        let v1 = test_stores.put_substate_changes(vec![
            change(8, 7, 2, Some(7)), // to be changed
            change(8, 7, 4, Some(9)), // to be unchanged
            change(8, 7, 1, Some(3)), // to be deleted
            change(8, 6, 3, Some(6)), // unrelated partition
            change(9, 2, 5, Some(2)), // unrelated node
        ]);
        let v2 = test_stores.put_substate_changes(vec![
            change(8, 7, 2, Some(8)), // changed value
            change(8, 7, 9, Some(1)), // added substate
            change(8, 7, 1, None),    // deleted substate
            change(8, 1, 8, Some(2)), // unrelated change
        ]);

        let subject_v1 = test_stores.create_subject(v1);
        let substates_v1 = subject_v1
            .list_entries_from(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            substates_v1,
            vec![
                (sort_key(1), from_seed(3)),
                (sort_key(2), from_seed(7)),
                (sort_key(4), from_seed(9)),
            ]
        );

        let subject_v2 = test_stores.create_subject(v2);
        let substates_v2 = subject_v2
            .list_entries_from(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            substates_v2,
            vec![
                (sort_key(2), from_seed(8)),
                (sort_key(4), from_seed(9)),
                (sort_key(9), from_seed(1)),
            ]
        );
    }

    // Only test utils below:

    type SingleSubstateChange = (DbSubstateKey, DatabaseUpdate);

    fn change(
        node_key_seed: u8,
        partition_num: u8,
        sort_key_seed: u8,
        value_seed: Option<u8>,
    ) -> SingleSubstateChange {
        change_exact(
            node_key(node_key_seed),
            partition_num,
            from_seed(sort_key_seed),
            value_seed.map(from_seed),
        )
    }

    pub fn change_exact(
        node_key: Vec<u8>,
        partition_num: u8,
        sort_key: Vec<u8>,
        value: Option<Vec<u8>>,
    ) -> SingleSubstateChange {
        (
            (
                DbPartitionKey {
                    node_key,
                    partition_num,
                },
                DbSortKey(sort_key),
            ),
            value
                .map(DatabaseUpdate::Set)
                .unwrap_or(DatabaseUpdate::Delete),
        )
    }

    fn node_key(node_key_seed: u8) -> DbNodeKey {
        const RANDOM_ENTITY_TYPES: [EntityType; 3] = [
            EntityType::GlobalAccount,
            EntityType::GlobalPackage,
            EntityType::GlobalValidator,
        ];
        let entity_type = RANDOM_ENTITY_TYPES[node_key_seed as usize % RANDOM_ENTITY_TYPES.len()];
        let node_id = NodeId::new(entity_type as u8, &[node_key_seed; NodeId::RID_LENGTH]);
        SpreadPrefixKeyMapper::to_db_node_key(&node_id)
    }

    fn partition_key(node_key_seed: u8, partition_num: u8) -> DbPartitionKey {
        DbPartitionKey {
            node_key: node_key(node_key_seed),
            partition_num,
        }
    }

    fn sort_key(sort_key_seed: u8) -> DbSortKey {
        DbSortKey(from_seed(sort_key_seed))
    }

    fn from_seed(seed: u8) -> Vec<u8> {
        vec![seed; seed as usize]
    }

    struct TestStores {
        tree_store: TypedInMemoryTreeStore,
        upsert_value_store: MemorySubstateUpsertValueStore,
        current_version: StateVersion,
    }

    impl TestStores {
        pub fn new_empty() -> Self {
            Self {
                tree_store: TypedInMemoryTreeStore::new(),
                upsert_value_store: MemorySubstateUpsertValueStore::default(),
                current_version: StateVersion::pre_genesis(),
            }
        }

        pub fn put_substate_changes(
            &mut self,
            changes: impl IntoIterator<Item = SingleSubstateChange>,
        ) -> StateVersion {
            self.apply_database_updates(&DatabaseUpdates::from_delta_maps(
                Self::index_to_delta_maps(changes),
            ))
        }

        pub fn reset_partition(
            &mut self,
            partition_key: &DbPartitionKey,
            values: impl IntoIterator<Item = (DbSortKey, DbSubstateValue)>,
        ) -> StateVersion {
            let DbPartitionKey {
                node_key,
                partition_num,
            } = partition_key.clone();
            self.apply_database_updates(&DatabaseUpdates {
                node_updates: indexmap!(
                    node_key => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            partition_num => PartitionDatabaseUpdates::Reset {
                                new_substate_values: values.into_iter().collect()
                            }
                        )
                    }
                ),
            })
        }

        pub fn create_subject(
            &self,
            at_state_version: StateVersion,
        ) -> StateHashTreeBasedSubstateDatabase<
            TypedInMemoryTreeStore,
            MemorySubstateUpsertValueStore,
        > {
            StateHashTreeBasedSubstateDatabase::new(
                &self.tree_store,
                &self.upsert_value_store,
                at_state_version,
            )
        }

        fn apply_database_updates(&mut self, database_updates: &DatabaseUpdates) -> StateVersion {
            let current_version = self.current_version;
            put_at_next_version(
                &self.tree_store,
                Some(current_version.number()).filter(|number| *number > 0u64),
                database_updates,
            );
            let next_version = current_version.next().expect("too high version in a test");
            self.upsert_value_store
                .put_at_version(next_version, database_updates);
            self.current_version = next_version;
            self.current_version
        }

        fn index_to_delta_maps(
            changes: impl IntoIterator<Item = SingleSubstateChange>,
        ) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>> {
            let mut delta_maps =
                index_map_new::<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>();
            for change in changes {
                let ((partition_key, sort_key), update) = change;
                delta_maps
                    .entry(partition_key)
                    .or_default()
                    .insert(sort_key, update);
            }
            delta_maps
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Default)]
    struct MemorySubstateUpsertValueStore {
        memory: BTreeMap<(DbPartitionKey, DbSortKey, StateVersion), DbSubstateValue>,
    }

    impl MemorySubstateUpsertValueStore {
        pub fn put_at_version(&mut self, version: StateVersion, updates: &DatabaseUpdates) {
            for (node_key, node_updates) in &updates.node_updates {
                for (partition_num, partition_updates) in &node_updates.partition_updates {
                    let upserted_values = match partition_updates {
                        PartitionDatabaseUpdates::Delta { substate_updates } => substate_updates
                            .iter()
                            .filter_map(|(sort_key, update)| match update {
                                DatabaseUpdate::Set(value) => Some((sort_key, value)),
                                DatabaseUpdate::Delete => None,
                            })
                            .collect::<Vec<_>>(),
                        PartitionDatabaseUpdates::Reset {
                            new_substate_values,
                        } => new_substate_values.iter().collect::<Vec<_>>(),
                    };
                    for (sort_key, value) in upserted_values {
                        self.memory.insert(
                            (
                                DbPartitionKey {
                                    node_key: node_key.clone(),
                                    partition_num: *partition_num,
                                },
                                sort_key.clone(),
                                version,
                            ),
                            value.clone(),
                        );
                    }
                }
            }
        }
    }

    impl SubstateUpsertValueStore for MemorySubstateUpsertValueStore {
        fn get_value(
            &self,
            partition_key: &DbPartitionKey,
            sort_key: &DbSortKey,
            upserted_at_state_version: StateVersion,
        ) -> Option<DbSubstateValue> {
            self.memory
                .get(&(
                    partition_key.clone(),
                    sort_key.clone(),
                    upserted_at_state_version,
                ))
                .cloned()
        }
    }
}
