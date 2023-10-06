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

use clokwerk::Interval;
use radix_engine::types::*;
use radix_engine_stores::hash_tree::tree_store::{
    NodeKey, ReadableTreeStore, StaleTreePart, TreeNode,
};
use std::collections::VecDeque;
use std::iter;
use std::ops::Deref;
use std::sync::Arc;

use crate::store::traits::gc::StateHashTreeGcStore;
use crate::store::traits::proofs::QueryableProofStore;
use crate::store::traits::StaleTreePartsV1;
use crate::store::StateManagerDatabase;
use crate::StateVersion;
use node_common::locks::RwLock;

/// A maximum number of JMT nodes collected into "batch delete" buffer.
/// Needed only to avoid OOM problems.
const DELETED_NODE_BUFFER_MAX_LEN: usize = 1000000;

/// A configuration for [`StateHashTreeGc`].
#[derive(Debug, Categorize, Encode, Decode, Clone, Default)]
pub struct StateHashTreeGcConfig {
    /// How often to run the GC, in seconds.
    pub interval_sec: u32,
    /// How many most recent state versions to keep in the state hash tree.
    pub state_version_history_length: usize,
    /// How many state versions to remove during a single GC run.
    /// This limit only exists to avoid overwhelming the DB on the first run after enabling the GC.
    pub max_deleted_state_versions_during_run: usize,
}

/// A garbage collector of sufficiently-old stale state hash tree nodes.
/// The implementation is suited for being driven by an external scheduler.
pub struct StateHashTreeGc {
    database: Arc<RwLock<StateManagerDatabase>>,
    config: StateHashTreeGcConfig,
}

impl StateHashTreeGc {
    /// Creates a new GC.
    pub fn new(database: Arc<RwLock<StateManagerDatabase>>, config: StateHashTreeGcConfig) -> Self {
        Self { database, config }
    }

    /// Performs a single GC run, which is supposed to permanently delete some number of the
    /// old-enough state hash tree nodes marked as stale.
    pub fn run(&self) {
        // Open an iterator of "stale tree parts" (batched by state version at which they became stale):
        let read_database = self.database.read();
        let to_state_version = read_database
            .max_state_version()
            .relative(i128::try_from(self.config.state_version_history_length).unwrap())
            .unwrap_or(StateVersion::pre_genesis());
        let stale_tree_parts_entries = read_database
            .get_stale_tree_parts_iter()
            .take(self.config.max_deleted_state_versions_during_run)
            .take_while(|(state_version, _)| state_version < &to_state_version);

        // Collect the stale node keys:
        let mut deleted_state_versions = Vec::new();
        let mut deleted_nodes = Vec::new();
        for (state_version, StaleTreePartsV1(stale_tree_parts)) in stale_tree_parts_entries {
            for stale_tree_part in stale_tree_parts {
                let part_keys: Box<dyn Iterator<Item = NodeKey>> = match stale_tree_part {
                    StaleTreePart::Node(key) => Box::new(iter::once(key)),
                    // In case of "delete partition", we have to traverse its entire subtree:
                    // Note: it is critical to do it DFS (i.e. delete a parent only after its children).
                    StaleTreePart::Subtree(subtree_root_key) => {
                        Box::new(DfsIterator::new(read_database.deref(), &subtree_root_key))
                    }
                };
                for key in part_keys {
                    deleted_nodes.push(key);
                    // Periodically rotate the collected buffer of node keys to delete:
                    if deleted_nodes.len() == DELETED_NODE_BUFFER_MAX_LEN {
                        read_database.batch_delete_node(deleted_nodes.iter());
                        deleted_nodes.clear();
                    }
                }
            }
            deleted_state_versions.push(state_version);
        }

        // Delete the last collected batch of keys, and then delete the processed "stale tree parts" records:
        read_database.batch_delete_node(deleted_nodes.iter());
        read_database.batch_delete_stale_tree_part(deleted_state_versions.iter());
    }

    /// Returns an interval to be used by the scheduler running this GC.
    pub fn interval(&self) -> Interval {
        Interval::Seconds(self.config.interval_sec)
    }
}

struct DfsIterator<'s, S> {
    tree_store: &'s S,
    levels: Vec<VecDeque<(NodeKey, bool)>>,
}

impl<'s, S: ReadableTreeStore> DfsIterator<'s, S> {
    pub fn new(tree_store: &'s S, root_key: &NodeKey) -> Self {
        let levels = Self::drill_levels(tree_store, root_key);
        Self { tree_store, levels }
    }

    fn drill_levels(tree_store: &S, start_key: &NodeKey) -> Vec<VecDeque<(NodeKey, bool)>> {
        let mut levels = Vec::new();
        let mut at_key = start_key;
        // Drill down the leftmost descendants chain, in order to:
        // - find the starting point of the DFS iteration;
        // - and record the single chain of "siblings of ancestors" (to easily continue the traversal).
        loop {
            let Some(at_node) = tree_store.get_node(at_key) else {
                break; // let's silently tolerate nodes deleted by any previous incomplete GC run
            };
            match at_node {
                TreeNode::Internal(internal) => {
                    let level = internal
                        .children
                        .into_iter()
                        .map(|entry| {
                            (
                                at_key.gen_child_node_key(entry.version, entry.nibble),
                                entry.is_leaf,
                            )
                        })
                        .collect::<VecDeque<_>>();
                    levels.push(level);
                    let (leftmost_child_key, is_leaf) = levels
                        .last()
                        .expect("we literally just pushed an element into it")
                        .front()
                        .expect("if internal node exists, then it has children");
                    if *is_leaf {
                        break;
                    } else {
                        at_key = leftmost_child_key;
                    }
                }
                TreeNode::Leaf(_) => {
                    // This special case may only happen on the initialization (i.e. from `new()`);
                    // As seen in the branch above, we normally do not even attempt to load the
                    // final leaf node from DB.
                    levels.push(VecDeque::from([(at_key.clone(), true)]));
                    break;
                }
                TreeNode::Null => {
                    // This case does not occur in practice at all (we do not keep empty lower-tier
                    // trees in the DB - we remove their higher-tier leaf node instead).
                    break;
                }
            }
        }
        levels
    }
}

impl<'s, S: ReadableTreeStore> Iterator for DfsIterator<'s, S> {
    type Item = NodeKey;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(mut current_deepest_level) = self.levels.pop() else {
            return None;
        };
        let (returned_key, _is_leaf) = current_deepest_level.pop_front().unwrap();
        if let Some((sibling_key, is_leaf)) = current_deepest_level.front() {
            if *is_leaf {
                self.levels.push(current_deepest_level);
            } else {
                let new_deepest_levels = Self::drill_levels(self.tree_store, sibling_key);
                self.levels.push(current_deepest_level);
                self.levels.extend(new_deepest_levels);
            }
        }
        Some(returned_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_engine_store_interface::interface::{
        DatabaseUpdate, DatabaseUpdates, DbSortKey, NodeDatabaseUpdates, PartitionDatabaseUpdates,
    };
    use radix_engine_stores::hash_tree::put_at_next_version;
    use radix_engine_stores::hash_tree::tree_store::{NibblePath, TypedInMemoryTreeStore};

    #[test]
    fn iterates_substates_from_deleted_partition_in_dfs_order() {
        // Arrange: we need a valid tree store state - easiest to just use JMT's infra
        let mut tree_store = TypedInMemoryTreeStore::new();

        // - Create a single node with 2 partitions; one of them (07) has an elaborate substate-tier tree:
        put_at_next_version(
            &mut tree_store,
            None,
            &DatabaseUpdates {
                node_updates: indexmap!(
                    bytes("c0ffee") => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            7 => PartitionDatabaseUpdates::Delta {
                                substate_updates: indexmap!(
                                    DbSortKey(bytes("dec0")) => DatabaseUpdate::Set(vec![1]),
                                    DbSortKey(bytes("beef")) => DatabaseUpdate::Set(vec![2]),
                                    DbSortKey(bytes("ee")) => DatabaseUpdate::Set(vec![3]),
                                    DbSortKey(bytes("bada55")) => DatabaseUpdate::Set(vec![4]),
                                )
                            },
                            64 => PartitionDatabaseUpdates::Delta {
                                substate_updates: indexmap!(
                                    DbSortKey(bytes("af")) => DatabaseUpdate::Set(vec![0]),
                                )
                            }
                        )
                    }
                ),
            },
        );

        // - Add some more substates to the partition 07 (so that JMT contains nodes of different versions!)
        put_at_next_version(
            &mut tree_store,
            Some(1),
            &DatabaseUpdates {
                node_updates: indexmap!(
                    bytes("c0ffee") => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            7 => PartitionDatabaseUpdates::Delta {
                                substate_updates: indexmap!(
                                    DbSortKey(bytes("deafd00d")) => DatabaseUpdate::Set(vec![5]),
                                    DbSortKey(bytes("dead")) => DatabaseUpdate::Set(vec![6]),
                                    DbSortKey(bytes("b000000000")) => DatabaseUpdate::Set(vec![7]),
                                )
                            },
                        )
                    }
                ),
            },
        );

        // - Delete the extremely large partition 07:
        put_at_next_version(
            &mut tree_store,
            Some(2),
            &DatabaseUpdates {
                node_updates: indexmap!(
                    bytes("c0ffee") => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            7 => PartitionDatabaseUpdates::Reset {
                                new_substate_values: indexmap!()
                            }
                        )
                    }
                ),
            },
        );

        // - Expect that subtree of our partition became stale:
        let mut stale_subtrees = tree_store
            .stale_part_buffer
            .iter()
            .filter_map(|part| match part {
                StaleTreePart::Node(_) => None,
                StaleTreePart::Subtree(subtree_root_key) => Some(subtree_root_key),
            })
            .collect::<Vec<_>>();
        assert_eq!(stale_subtrees.len(), 1);
        let deleted_partition_root_key = stale_subtrees.remove(0);
        // Note: "5f" is the tier separator byte - an implementation detail of our [`NestedTreeStore`].
        assert_eq!(
            deleted_partition_root_key,
            &NodeKey::new(2, nibbles("c0ffee 5f 07 5f"))
        );

        // Act: Request a DFS iterator starting at the deleted partition's root
        let iterator = DfsIterator::new(&tree_store, deleted_partition_root_key);

        // Assert: The listed nodes are in DFS order
        assert_eq!(
            iterator.collect::<Vec<_>>(),
            vec![
                // this starts leftmost and completes larger and larger subtrees: (like DFS should)
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f b0")), // leaf b000000000
                NodeKey::new(1, nibbles("c0ffee 5f 07 5f ba")), // leaf bada55
                NodeKey::new(1, nibbles("c0ffee 5f 07 5f be")), // leaf beef
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f b")),  // parent of these ^ three
                // drills down the next sibling's subtree: (like DFS should)
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f dead")), // leaf dead
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f deaf")), // leaf deafd00d
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f dea")),  // parent of these ^ two
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f dec")), // leaf dec0 (sibling of that parent)
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f de")),  // parent of these ^ two
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f d")), // parent of this ^ one (yup, "long common prefix")
                // and visits the rightmost top-level leaf sibling too: (like DFS should)
                NodeKey::new(1, nibbles("c0ffee 5f 07 5f e")), // leaf ee
            ]
        );
    }

    #[test]
    fn supports_degenerate_single_element_subtree() {
        // Arrange: A degenerate case with a single substate
        let mut tree_store = TypedInMemoryTreeStore::new();
        put_at_next_version(
            &mut tree_store,
            None,
            &DatabaseUpdates {
                node_updates: indexmap!(
                    bytes("c0ffee") => NodeDatabaseUpdates {
                        partition_updates: indexmap!(
                            3 => PartitionDatabaseUpdates::Delta {
                                substate_updates: indexmap!(
                                    DbSortKey(bytes("afffff")) => DatabaseUpdate::Set(vec![0]),
                                )
                            }
                        )
                    }
                ),
            },
        );

        // Act: Request a DFS iterator starting at the partition's root
        let iterator = DfsIterator::new(&tree_store, &NodeKey::new(1, nibbles("c0ffee 5f 03 5f")));

        // Assert: The single listed node key
        assert_eq!(
            iterator.collect::<Vec<_>>(),
            vec![NodeKey::new(1, nibbles("c0ffee 5f 03 5f"))], // the root is also the leaf afffff
        );
    }

    fn bytes(string: &str) -> Vec<u8> {
        hex::decode(string.replace(' ', "")).unwrap()
    }

    fn nibbles(string: &str) -> NibblePath {
        let mut string = string.replace(' ', "");
        if string.len() % 2 == 0 {
            NibblePath::new_even(hex::decode(&string).unwrap())
        } else {
            string.push('0');
            NibblePath::new_odd(hex::decode(&string).unwrap())
        }
    }
}
