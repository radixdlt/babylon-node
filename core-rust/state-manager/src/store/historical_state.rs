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

use substate_store_impls::hash_tree::entity_tier::EntityTier;

use crate::engine_prelude::*;
use crate::store::traits::*;
use crate::StateVersion;

/// An implementation of a [`SubstateDatabase`] viewed at a specific [`StateVersion`].
///
/// This database is backed by:
/// - a [`ReadableTreeStore`] - a versioned source of ReNodes / Partitions / Substates metadata,
/// - and a [`LeafSubstateValueStore`] - a store of Substate values' associated with their leafs.
pub struct StateHashTreeBasedSubstateDatabase<'t, T> {
    tree_store: &'t T,
    at_state_version: StateVersion,
}

impl<'t, T: ReadableTreeStore + LeafSubstateValueStore> StateHashTreeBasedSubstateDatabase<'t, T> {
    /// Creates an instance backed by the given lower-level stores and scoped at the given version.
    pub fn new(tree_store: &'t T, at_state_version: StateVersion) -> Self {
        Self {
            tree_store,
            at_state_version,
        }
    }

    fn create_entity_tier(&self) -> EntityTier<'t, T> {
        EntityTier::new(
            self.tree_store,
            Some(self.at_state_version.number()).filter(|v| *v > 0),
        )
    }
}

impl<'t, T: ReadableTreeStore + LeafSubstateValueStore> SubstateDatabase
    for StateHashTreeBasedSubstateDatabase<'t, T>
{
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.create_entity_tier()
            .get_entity_partition_tier(partition_key.node_key.clone())
            .get_partition_substate_tier(partition_key.partition_num)
            .get_substate_summary(sort_key)
            .and_then(|summary| {
                self.tree_store
                    .get_associated_value(&summary.state_tree_leaf_key)
            })
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        Box::new(
            self.create_entity_tier()
                .get_entity_partition_tier(partition_key.node_key.clone())
                .get_partition_substate_tier(partition_key.partition_num)
                .into_iter_substate_summaries_from(from_sort_key)
                .map(|substate| {
                    let value = self
                        .tree_store
                        .get_associated_value(&substate.state_tree_leaf_key)
                        .expect("DB inconsistency: associated value not found for leaf key");
                    (substate.sort_key, value)
                }),
        )
    }
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

    #[test]
    fn lists_partition_substates_from_different_starting_keys() {
        let mut test_stores = TestStores::new_empty();
        let the_only_version = test_stores.put_substate_changes(vec![
            change(8, 7, 1, Some(3)),
            change(8, 7, 2, Some(5)),
            change(8, 7, 4, Some(7)),
            change(8, 7, 7, Some(9)),
        ]);

        let subject = test_stores.create_subject(the_only_version);
        let all_substates = subject
            .list_entries_from(&partition_key(8, 7), None)
            .collect::<Vec<_>>();
        assert_eq!(
            all_substates,
            vec![
                (sort_key(1), from_seed(3)),
                (sort_key(2), from_seed(5)),
                (sort_key(4), from_seed(7)),
                (sort_key(7), from_seed(9)),
            ]
        );

        let from_existent = subject
            .list_entries_from(&partition_key(8, 7), Some(&sort_key(2)))
            .collect::<Vec<_>>();
        assert_eq!(from_existent, all_substates[1..]);

        let from_non_existent = subject
            .list_entries_from(&partition_key(8, 7), Some(&sort_key(3)))
            .collect::<Vec<_>>();
        assert_eq!(from_non_existent, all_substates[2..]);

        let from_lt_min = subject
            .list_entries_from(&partition_key(8, 7), Some(&sort_key(0)))
            .collect::<Vec<_>>();
        assert_eq!(from_lt_min, all_substates);

        let from_gt_max = subject
            .list_entries_from(&partition_key(8, 7), Some(&sort_key(9)))
            .collect::<Vec<_>>();
        assert_eq!(from_gt_max, vec![]);
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
        current_version: StateVersion,
    }

    impl TestStores {
        pub fn new_empty() -> Self {
            Self {
                tree_store: TypedInMemoryTreeStore::new().storing_substate_values(),
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
        ) -> StateHashTreeBasedSubstateDatabase<TypedInMemoryTreeStore> {
            StateHashTreeBasedSubstateDatabase::new(&self.tree_store, at_state_version)
        }

        fn apply_database_updates(&mut self, database_updates: &DatabaseUpdates) -> StateVersion {
            let current_version = self.current_version;
            put_at_next_version(
                &self.tree_store,
                Some(current_version.number()).filter(|number| *number > 0u64),
                database_updates,
            );
            let next_version = current_version.next().expect("too high version in a test");
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

    impl LeafSubstateValueStore for TypedInMemoryTreeStore {
        fn get_associated_value(
            &self,
            state_tree_leaf_key: &StoredTreeNodeKey,
        ) -> Option<DbSubstateValue> {
            self.associated_substate_values
                .borrow()
                .get(state_tree_leaf_key)
                .cloned()
        }
    }
}
