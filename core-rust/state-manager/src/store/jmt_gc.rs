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

use crate::engine_prelude::*;
use node_common::locks::DbLock;
use std::iter;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::store::traits::gc::StateHashTreeGcStore;
use crate::store::traits::proofs::QueryableProofStore;
use crate::store::traits::StaleTreePartsV1;
use crate::{ActualStateManagerDatabase, StateVersion, StateVersionDelta};

/// A maximum number of JMT nodes collected into "batch delete" buffer.
/// Needed only to avoid OOM problems.
const DELETED_NODE_BUFFER_MAX_LEN: usize = 1000000;

/// A configuration for [`StateHashTreeGc`].
#[derive(Debug, Categorize, Encode, Decode, Clone, Default)]
pub struct StateHashTreeGcConfig {
    /// How often to run the GC, in seconds.
    /// This should be at least an order of magnitude shorter than an expected duration over which
    /// the [`state_version_history_length`] spans (to honour the precision of these settings).
    pub interval_sec: u32,
    /// How many most recent state versions to keep in the state hash tree.
    pub state_version_history_length: usize,
}

/// A garbage collector of sufficiently-old stale state hash tree nodes.
/// The implementation is suited for being driven by an external scheduler.
pub struct StateHashTreeGc {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    interval: Duration,
    history_len: StateVersionDelta,
}

impl StateHashTreeGc {
    /// Creates a new GC.
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        config: StateHashTreeGcConfig,
    ) -> Self {
        Self {
            database,
            interval: Duration::from_secs(u64::from(config.interval_sec)),
            history_len: StateVersionDelta::try_from(config.state_version_history_length).unwrap(),
        }
    }

    /// An interval between [`run()`]s, to be used by this instance's scheduler.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Performs a single GC run, which is supposed to permanently delete *all* old-enough state
    /// hash tree nodes marked as stale.
    ///
    /// *Note on concurrent database access:*
    /// The JMT's GC process, by its nature, only accesses "old" (i.e. not "top-of-ledger" new)
    /// JMT DB rows. For this reason, it can use the direct [`DbLock::access_direct()`] and
    /// effectively own these rows (for reads and deletes), without locking the database.
    pub fn run(&self) {
        let database = self.database.access_direct();
        let current_state_version = database.max_state_version();
        let to_state_version = current_state_version
            .relative(-self.history_len)
            .unwrap_or(StateVersion::pre_genesis());

        info!(
            "Starting a GC run: current state version is {:?}; pruning JMT up to version {:?}",
            current_state_version.number(),
            to_state_version.number(),
        );

        // Open an iterator of "stale tree parts" (batched by state version at which they became stale):
        let stale_entries = database
            .get_stale_tree_parts_iter()
            .take_while(|(state_version, _)| state_version < &to_state_version);

        // Collect the stale node keys into a "delete buffer":
        let mut deleted_state_versions = Vec::new();
        let mut deleted_nodes = Vec::new();
        for (state_version, StaleTreePartsV1(stale_tree_parts)) in stale_entries {
            for stale_tree_part in stale_tree_parts {
                let part_keys: Box<dyn Iterator<Item = NodeKey>> = match stale_tree_part {
                    StaleTreePart::Node(key) => Box::new(iter::once(key)),
                    // In case of "delete partition", we have to traverse its entire subtree:
                    // Note: it is critical to do a post-order DFS here (i.e. to delete a parent
                    // only after its children, in case this process is interrupted half-way
                    // and need to be resumed).
                    StaleTreePart::Subtree(subtree_root_key) => {
                        Box::new(iterate_dfs_post_order(database.deref(), subtree_root_key))
                    }
                };
                for key in part_keys {
                    deleted_nodes.push(key);
                    // Periodically rotate the collected buffer of node keys to delete:
                    if deleted_nodes.len() == DELETED_NODE_BUFFER_MAX_LEN {
                        info!("Flushing a full delete buffer at version {}", state_version);
                        database.batch_delete_node(deleted_nodes.iter());
                        deleted_nodes.clear();
                    }
                }
            }
            deleted_state_versions.push(state_version);
        }

        // Delete the last collected batch of keys, and then delete the processed "stale tree parts" records:
        info!("Flushing the last buffer ({} deletes)", deleted_nodes.len());
        database.batch_delete_node(deleted_nodes.iter());
        database.batch_delete_stale_tree_part(deleted_state_versions.iter());
    }
}

/// Iterates the node keys from the state hash tree's subtree starting at the given root key, in a
/// depth-first-search, post-order way (i.e. parent after children).
/// Note: the implementation will only traverse internal nodes, reading the leaves' state from their
/// parent's child-list. This means that it can return node keys of leaves that were already deleted
/// from the database (in a previous, incomplete GC run).
fn iterate_dfs_post_order<'s, S: ReadableTreeStore>(
    tree_store: &'s S,
    root_key: NodeKey,
) -> Box<dyn Iterator<Item = NodeKey> + 's> {
    let Some(root_node) = tree_store.get_node(&root_key) else {
        // A "top-level recovery" case: may happen when we resume an interrupted delete of a
        // state version (i.e. this entire subtree was one of the early entries within some
        // `StaleTreePartsV1`).
        return Box::new(iter::empty());
    };
    match root_node {
        TreeNode::Null => {
            // A special case: this subtree is empty.
            // Note: at the moment of writing this, this case is impossible in practice: we do
            // not delete ReNode-Tier tree, and we also do not store empty lower-Tier trees
            // (i.e. we delete their higher-Tier leaf counterpart instead). However, we can
            // return a correct empty result here (in case the above assumptions ever change).
            Box::new(iter::empty())
        }
        TreeNode::Leaf(_) => {
            // A special case: this subtree is just a single leaf.
            Box::new(iter::once(root_key))
        }
        TreeNode::Internal(root_internal_node) => {
            // A regular case: we have some nested internal nodes, use the DFS post-order recursion.
            Box::new(recurse_children_and_append_parent(
                tree_store,
                root_internal_node.children,
                root_key,
            ))
        }
    }
}

/// The recursive part of the [`iterate_dfs_post_order()`] logic.
fn recurse_children_and_append_parent<'s, S: ReadableTreeStore + 's>(
    tree_store: &'s S,
    children: Vec<TreeChildEntry>,
    parent_key: NodeKey,
) -> impl Iterator<Item = NodeKey> + 's {
    let parent_key_to_be_appended_after_children = iter::once(parent_key.clone());
    children
        .into_iter()
        .flat_map(move |child| -> Box<dyn Iterator<Item = NodeKey>> {
            let child_key = parent_key.gen_child_node_key(child.version, child.nibble);
            if child.is_leaf {
                // A terminal case: we do not need to recurse into children (nor load them from DB).
                // Not loading from the DB is an optimization to speed up the performance.
                // This can mean that we return children which are already deleted / no longer exist.
                // This is mentioned in the rust doc for `iterate_dfs_post_order`
                return Box::new(iter::once(child_key));
            }
            let Some(child_node) = tree_store.get_node(&child_key) else {
                // A mid-way "recovery" case: may happen when we resume an interrupted
                // delete of a particular subtree (and reach an already-deleted child).
                return Box::new(iter::empty());
            };
            let TreeNode::Internal(child_internal_node) = child_node else {
                panic!("unexpected non-leaf child: {:?}", child_node);
            };
            // A recursion case: this internal node has some child internal node.
            Box::new(recurse_children_and_append_parent(
                tree_store,
                child_internal_node.children,
                child_key,
            ))
        })
        // DFS post-order: as promised, list the parent after its children.
        .chain(parent_key_to_be_appended_after_children)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iterates_substates_from_deleted_partition_in_dfs_order() {
        // Arrange: we need a valid tree store state - easiest to just use JMT's infra
        let tree_store = TypedInMemoryTreeStore::new();

        // - Create a single node with 2 partitions; one of them (07) has an elaborate substate-tier tree:
        put_at_next_version(
            &tree_store,
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
            &tree_store,
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
            &tree_store,
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
        let stale_part_buffer = tree_store.stale_part_buffer.borrow();
        let mut stale_subtrees = stale_part_buffer
            .iter()
            .filter_map(|part| match part {
                StaleTreePart::Node(_) => None,
                StaleTreePart::Subtree(subtree_root_key) => Some(subtree_root_key),
            })
            .collect::<Vec<_>>();
        assert_eq!(stale_subtrees.len(), 1);
        let deleted_partition_root_key = stale_subtrees.remove(0).clone();
        // Note: "5f" is the tier separator byte - an implementation detail of our [`NestedTreeStore`].
        assert_eq!(
            deleted_partition_root_key,
            NodeKey::new(2, nibbles("c0ffee 5f 07 5f"))
        );

        // Act: Request a DFS iterator starting at the deleted partition's root
        let iterator = iterate_dfs_post_order(&tree_store, deleted_partition_root_key.clone());
        let iterated_node_keys = iterator.collect::<Vec<_>>();

        // Assert: The listed nodes are in DFS order
        assert_eq!(
            iterated_node_keys,
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
                // and the root at the end: (like DFS should)
                NodeKey::new(2, nibbles("c0ffee 5f 07 5f")), // root
            ]
        );

        // Follow-up: assert that remaining nodes keys are listed after partially executed deletes
        let mut deleted_keys = index_set_new();
        for node_key_to_delete in iterated_node_keys.clone() {
            tree_store
                .tree_nodes
                .borrow_mut()
                .remove(&node_key_to_delete);
            deleted_keys.insert(node_key_to_delete);
            let remaining_iterated_node_keys =
                iterate_dfs_post_order(&tree_store, deleted_partition_root_key.clone())
                    .collect::<IndexSet<_>>();
            assert_eq!(
                // We deliberately use union - the iterator is allowed to return already-deleted nodes:
                deleted_keys
                    .union(&remaining_iterated_node_keys)
                    .cloned()
                    .collect::<Vec<_>>(),
                iterated_node_keys
            )
        }
    }

    #[test]
    fn supports_degenerate_single_element_subtree() {
        // Arrange: A degenerate case with a single substate
        let tree_store = TypedInMemoryTreeStore::new();
        put_at_next_version(
            &tree_store,
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
        let iterator =
            iterate_dfs_post_order(&tree_store, NodeKey::new(1, nibbles("c0ffee 5f 03 5f")));

        // Assert: The single listed node key
        assert_eq!(
            iterator.collect::<Vec<_>>(),
            vec![NodeKey::new(1, nibbles("c0ffee 5f 03 5f"))], // the root is also the leaf afffff
        );
    }

    #[test]
    fn supports_already_deleted_entire_subtree() {
        // Arrange: A handcrafted stale part entry, for which a subtree does not exist
        let tree_store = TypedInMemoryTreeStore::new();
        tree_store
            .stale_part_buffer
            .borrow_mut()
            .push(StaleTreePart::Subtree(NodeKey::new(
                1,
                nibbles("c0ffee 5f 03 5f"),
            )));

        // Act: Request a DFS iterator starting at the partition's root
        let mut iterator =
            iterate_dfs_post_order(&tree_store, NodeKey::new(1, nibbles("c0ffee 5f 03 5f")));

        // Assert: Empty iterator
        assert!(iterator.next().is_none());
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
