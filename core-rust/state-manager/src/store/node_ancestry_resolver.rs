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

use crate::store::traits::*;
use crate::{ChangeAction, SubstateReference};
use std::collections::hash_map::Entry;

use radix_engine::types::*;

/// A parent Substate and its owned Nodes.
/// This structure may be used to represent only directly owned Nodes (i.e. all immediate children),
/// or all transitively owned Nodes (i.e. all descendants: children, grand-children, grand-grand-...).
type OwnedNodeSet = (SubstateReference, Vec<NodeId>);

/// A static resolver of [`SubstateNodeAncestryRecord`]s for [`SubstateChange`]s.
pub struct NodeAncestryResolver {}

impl NodeAncestryResolver {
    /// Resolves the [`SubstateNodeAncestryRecord`] for all newly-created Nodes found in the given
    /// substate changes.
    /// The resolution considers the trees (or rather: potentially-incomplete bottom-up tree
    /// fragments) constructed within the given [`ChangeAction`]s, and the top fragments of these
    /// trees already-existing in the given [`SubstateNodeAncestryStore`].
    /// API note: the results are grouped by [`SubstateNodeAncestryRecord`] (since it may be shared
    /// by many child [`NodeId`]s).
    pub fn batch_resolve<S: SubstateNodeAncestryStore>(
        ancestry_store: &S,
        substate_changes: impl Iterator<Item = (SubstateReference, ChangeAction)>,
    ) -> impl Iterator<Item = (Vec<NodeId>, SubstateNodeAncestryRecord)> {
        // Gather the Nodes owned by upserted parent Substates (using `IndexedScryptoValue`).
        // This effectively builds a forest of upserted nodes, using a "parent to child-list map" representation.
        let directly_owned_node_sets =
            Self::extract_owned_node_sets(substate_changes).collect::<Vec<_>>();

        // Prune the child-lists in the above forest so that they only contain nodes which are parents of some other nodes.
        // This limits further computation, since we only need to know a root of each parent anyway.
        let directly_owned_parent_sets =
            Self::clone_parents_from_node_sets(&directly_owned_node_sets);

        // Split the above "parents-only" forest into separate trees and flatten them.
        // This way we obtain all parents owned by each topmost parent within `SubstateChange`s.
        let transitively_owned_parent_sets =
            Self::propagate_transitively_owned_sets(directly_owned_parent_sets).collect::<Vec<_>>();

        // Resolve the existing roots of the topmost parents found above.
        // This queries the DB and then applies the "any Node without DB entry is a root" rule.
        let topmost_parent_roots = Self::batch_get_existing_root_or_same_substate(
            ancestry_store,
            transitively_owned_parent_sets
                .iter()
                .map(|(parent, _)| parent),
        );

        // Combine the `transitively_owned_parent_sets` (lower parts of the trees, constructed from
        // `SubstateChange`s) and `topmost_parent_roots` (roots of the trees, loaded from DB).
        let parent_to_root = transitively_owned_parent_sets
            .iter()
            .zip(topmost_parent_roots)
            .flat_map(|((topmost_parent, transitively_owned_parents), root)| {
                // interesting gotcha: the topmost parent is also "a parent" (which won't be on any child-list)
                let topmost_parent_entry = if topmost_parent.0 != root.0 {
                    // so, if it is _not_ a root itself, then we must include its entry in the `parent_to_root` map
                    vec![(topmost_parent.0, root.clone())]
                } else {
                    // ... but if root, we do not include it (although technically such entry would be correct)
                    vec![]
                };
                transitively_owned_parents
                    .iter()
                    .map(move |owned_parent| (*owned_parent, root.clone()))
                    .chain(topmost_parent_entry)
            })
            .collect::<NonIterMap<_, _>>();

        // Iterate again through all the upserted parents + Nodes (gathered in the beginning) and
        // return them together with roots read from the precomputed map above.
        directly_owned_node_sets
            .into_iter()
            .map(move |(parent, child_nodes)| {
                let root = parent_to_root
                    .get(&parent.0)
                    .cloned()
                    .unwrap_or_else(|| parent.clone());
                (child_nodes, SubstateNodeAncestryRecord { parent, root })
            })
    }

    /// Inspects the given substate changes (using the [`IndexedScryptoValue`]) to find the Nodes
    /// *directly* owned by each upserted Substate.
    fn extract_owned_node_sets(
        substate_changes: impl Iterator<Item = (SubstateReference, ChangeAction)>,
    ) -> impl Iterator<Item = OwnedNodeSet> {
        substate_changes.filter_map(|(substate_reference, action)| {
            let created_directly_owned_nodes = match &action {
                ChangeAction::Create(new) => {
                    IndexedScryptoValue::from_slice(new).unwrap().unpack().1
                }
                ChangeAction::Update { new, previous } => {
                    let new_directly_owned_nodes =
                        IndexedScryptoValue::from_slice(new).unwrap().unpack().1;
                    let previous_directly_owned_nodes = IndexedScryptoValue::from_slice(previous)
                        .unwrap()
                        .unpack()
                        .1;
                    let previous_directly_owned_node_set =
                        HashSet::from_iter(previous_directly_owned_nodes); // performance-only
                    new_directly_owned_nodes
                        .into_iter()
                        .filter(|node| !previous_directly_owned_node_set.contains(node))
                        .collect::<Vec<_>>()
                }
                ChangeAction::Delete => Vec::new(),
            };
            if created_directly_owned_nodes.is_empty() {
                return None;
            }
            Some((substate_reference, created_directly_owned_nodes))
        })
    }

    /// Clones the given [`OwnedNodeSet`]s, but only keeping on the child-lists the Nodes which are
    /// themselves parents of some other Nodes.
    fn clone_parents_from_node_sets(
        node_sets: &[OwnedNodeSet],
    ) -> impl Iterator<Item = OwnedNodeSet> + '_ {
        let mut parent_nodes = node_sets
            .iter()
            .map(|(parent, _)| parent.0)
            .collect::<HashSet<_>>();
        node_sets.iter().map(move |(parent, owned_nodes)| {
            (
                parent.clone(),
                owned_nodes
                    .iter()
                    .filter_map(|node| parent_nodes.take(node))
                    .collect::<Vec<_>>(),
            )
        })
    }

    /// Fetches the existing roots of the given Substates.
    /// The results are returned in the same order as the inputs, and for every unknown root the
    /// input is treated as a new root.
    fn batch_get_existing_root_or_same_substate<'a, S: SubstateNodeAncestryStore>(
        ancestry_store: &S,
        substates: impl IntoIterator<Item = &'a SubstateReference>,
    ) -> impl Iterator<Item = SubstateReference> + 'a {
        let substates = substates.into_iter().collect::<Vec<_>>();
        let unique_nodes = substates
            .iter()
            .map(|substate| substate.0)
            .collect::<HashSet<_>>();
        let existing_records = ancestry_store.batch_get_ancestry(unique_nodes.iter());
        let node_to_existing_root = unique_nodes
            .into_iter()
            .zip(existing_records)
            .filter_map(|(node, opt_record)| opt_record.map(|record| (node, record.root)))
            .collect::<NonIterMap<_, _>>();
        substates.into_iter().map(move |substate| {
            node_to_existing_root
                .get(&substate.0)
                .unwrap_or(substate)
                .clone()
        })
    }

    /// Propagates all children of the given [`OwnedNodeSet`]s up to their topmost ancestor.
    /// In other words: Transforms the given set of "parent-to-direct-children" entries into a set
    /// of "topmost-ancestor-to-all-descendants" entries.
    /// In yet other words: Splits the given forest into separate trees and flattens them.
    ///
    /// Simplified example (which ignores the "parent is Substate, child is Node" aspect):
    /// - input:
    ///   1 _directly_ owns 2, 3
    ///   2 _directly_ owns 4,
    ///   7 _directly_ owns 8, 9, 10
    ///   4 _directly_ owns 5, 6,
    /// - output:
    ///   1 _transitively_ owns 2, 3, 4, 5, 6
    ///   7 _transitively_ owns 8, 9, 10
    ///
    /// Please see the unit test to find an example which takes "parent is Substate, child is Node"
    /// into account.
    fn propagate_transitively_owned_sets(
        directly_owned_sets: impl IntoIterator<Item = OwnedNodeSet>,
    ) -> impl Iterator<Item = OwnedNodeSet> {
        // Perform a single pass to split the entries into 2 flavors: ones that belong to topmost
        // parent Nodes, and ones that belong to some Substate lower in some tree.
        let mut topmost_node_to_directly_owned_sets = index_map_new::<NodeId, Vec<OwnedNodeSet>>();
        let mut other_node_to_directly_owned_sets = NonIterMap::<NodeId, Vec<OwnedNodeSet>>::new();
        for (parent, children) in directly_owned_sets {
            match other_node_to_directly_owned_sets.entry(parent.0) {
                Entry::Occupied(mut occupied) => {
                    occupied.get_mut().push((parent, children.clone()));
                }
                Entry::Vacant(_) => {
                    topmost_node_to_directly_owned_sets
                        .entry(parent.0)
                        .or_insert_with(Vec::new)
                        .push((parent, children.clone()));
                }
            }
            for child in children {
                if let Some(demoted_sets) = topmost_node_to_directly_owned_sets.remove(&child) {
                    other_node_to_directly_owned_sets.insert(child, demoted_sets);
                } else {
                    other_node_to_directly_owned_sets
                        .entry(child)
                        .or_insert_with(Vec::new);
                }
            }
        }
        // Transfer the child lists from `other_node_to_directly_owned_sets` to their respective
        // `topmost_node_to_directly_owned_sets`.
        topmost_node_to_directly_owned_sets
            .into_iter()
            .flat_map(|(_, owned_sets)| owned_sets)
            .map(move |(topmost_parent, directly_owned_nodes)| {
                // Re-use the "resulting transitive child-list" as a work queue, to avoid recursion.
                let mut transitively_owned_nodes = directly_owned_nodes;
                let mut index = 0;
                while let Some(node) = transitively_owned_nodes.get(index) {
                    transitively_owned_nodes.extend(
                        other_node_to_directly_owned_sets
                            .remove(node)
                            .into_iter()
                            .flatten()
                            .flat_map(|other_owned_set| other_owned_set.1),
                    );
                    index += 1;
                }
                (topmost_parent, transitively_owned_nodes)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::Value;

    #[test]
    pub fn newly_created_child_nodes_are_recorded_under_their_existing_parents() {
        // Arrange
        let existing_index_entries = hashmap!(
            // a classic entry, specifying some immediate parent and the root high up:
            node_id(3) => record(substate(2, 17, 29), substate(1, 19, 21)),
            // an entry of the "immediate parent" mentioned above (its parent == root):
            node_id(2) => record(substate(1, 11, 25), substate(1, 11, 25)),
            // and NO entry for node_id(1), since it is a root
        );
        let substate_changes = [
            // some new grand-grand-child Node ID = 4 (under Node ID = 3):
            (
                substate(3, 12, 23),
                create(Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(4))),
                }),
            ),
            // some new child Node ID = 5 (right under a root Node ID = 1):
            (
                substate(1, 14, 24),
                create(Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(5))),
                }),
            ),
        ];

        // Act
        let new_index_entries = NodeAncestryResolver::batch_resolve(
            &existing_index_entries,
            substate_changes.into_iter(),
        )
        .flat_map(|(key_batch, value)| key_batch.into_iter().map(move |key| (key, value.clone())))
        .collect::<HashMap<_, _>>();

        // Assert
        assert_eq!(
            new_index_entries,
            hashmap!(
                node_id(4) => record(substate(3, 12, 23), substate(1, 19, 21)),
                node_id(5) => record(substate(1, 14, 24), substate(1, 14, 24))
            )
        );
    }

    #[test]
    pub fn newly_created_child_nodes_are_recorded_under_their_newly_created_parents() {
        // Arrange
        let existing_index_entries = hashmap!(
            node_id(2) => record(substate(1, 11, 25), substate(1, 11, 25)),
        );
        let substate_changes = [
            // some new grand-grand-child Node ID = 7 (under the new Node ID = 6 seen *below*):
            (
                substate(6, 10, 29),
                create(Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(7))),
                }),
            ),
            // some new grand-child Node ID = 6 (under an existing non-root Node ID = 2):
            (
                substate(2, 14, 24),
                create(Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(6))),
                }),
            ),
        ];

        // Act
        let new_index_entries = NodeAncestryResolver::batch_resolve(
            &existing_index_entries,
            substate_changes.into_iter(),
        )
        .flat_map(|(key_batch, value)| key_batch.into_iter().map(move |key| (key, value.clone())))
        .collect::<HashMap<_, _>>();

        // Assert
        assert_eq!(
            new_index_entries,
            hashmap!(
                node_id(7) => record(substate(6, 10, 29), substate(1, 11, 25)),
                node_id(6) => record(substate(2, 14, 24), substate(1, 11, 25))
            )
        );
    }

    #[test]
    pub fn owned_child_nodes_in_new_value_are_ignored_if_they_were_owned_in_previous_value() {
        // Arrange
        let existing_index_entries = hashmap!(
            // pre-existence of roots is irrelevant to this test
        );
        let substate_changes = [
            // a simpler "create" case - both nodes owned here were definitely just created:
            (
                substate(1, 12, 23),
                create(Value::Tuple {
                    fields: vec![
                        Value::Custom {
                            value: ScryptoCustomValue::Own(Own(node_id(2))),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Own(Own(node_id(3))),
                        },
                    ],
                }),
            ),
            // an "update" case - Node ID = 5 has existed before under the same parent:
            (
                substate(4, 11, 21),
                update(
                    Value::Tuple {
                        fields: vec![
                            Value::Custom {
                                value: ScryptoCustomValue::Own(Own(node_id(5))),
                            },
                            Value::Custom {
                                value: ScryptoCustomValue::Own(Own(node_id(6))),
                            },
                        ],
                    },
                    Value::Tuple {
                        fields: vec![
                            Value::Custom {
                                value: ScryptoCustomValue::Own(Own(node_id(5))),
                            },
                            Value::Custom {
                                value: ScryptoCustomValue::Own(Own(node_id(7))),
                            },
                        ],
                    },
                ),
            ),
        ];

        // Act
        let new_index_entries = NodeAncestryResolver::batch_resolve(
            &existing_index_entries,
            substate_changes.into_iter(),
        )
        .flat_map(|(key_batch, value)| key_batch.into_iter().map(move |key| (key, value.clone())))
        .collect::<HashMap<_, _>>();

        // Assert
        assert_eq!(
            new_index_entries,
            hashmap!(
                // Nodes owned by a value from Create:
                node_id(2) => record(substate(1, 12, 23), substate(1, 12, 23)),
                node_id(3) => record(substate(1, 12, 23), substate(1, 12, 23)),
                // a Node newly-owned by a value from Update:
                node_id(6) => record(substate(4, 11, 21), substate(4, 11, 21)),
                // no record produced for Node ID = 6 from that Update; it was already owned before.
            )
        );
    }

    #[test]
    pub fn newly_created_root_nodes_are_not_recorded_in_index() {
        // Arrange
        let existing_index_entries = hashmap!(
            // some unrelated existing entry:
            node_id(2) => record(substate(1, 11, 25), substate(1, 11, 25)),
        );
        let substate_changes = [
            // some new root Node ID = 6 (which does not own any other Node):
            (
                substate(6, 10, 29),
                create(ScryptoValue::I16 { value: 666 }),
            ),
            // some new root Node ID = 7 (which owns a leaf Node ID = 8 and a mid-parent Node ID = 9):
            (
                substate(7, 11, 23),
                create(Value::Tuple {
                    fields: vec![
                        Value::Custom {
                            value: ScryptoCustomValue::Own(Own(node_id(8))),
                        },
                        Value::Custom {
                            value: ScryptoCustomValue::Own(Own(node_id(9))),
                        },
                    ],
                }),
            ),
            // an arbitrary parent ID = 9 (just so that root Node ID = 7 can own a non-trivial tree):
            (
                substate(9, 12, 25),
                create(Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(10))),
                }),
            ),
        ];

        // Act
        let new_index_entries = NodeAncestryResolver::batch_resolve(
            &existing_index_entries,
            substate_changes.into_iter(),
        )
        .flat_map(|(key_batch, value)| key_batch.into_iter().map(move |key| (key, value.clone())))
        .collect::<HashMap<_, _>>();

        // Assert
        assert_eq!(
            new_index_entries,
            hashmap!(
                // entries of new non-roots (all correctly indicating Node ID = 7 as their root)
                node_id(8) => record(substate(7, 11, 23), substate(7, 11, 23)),
                node_id(9) => record(substate(7, 11, 23), substate(7, 11, 23)),
                node_id(10) => record(substate(9, 12, 25), substate(7, 11, 23)),
                // no entry for root Node ID = 7 (which is a newly created topmost parent, but still a root)
            )
        );
    }

    #[test]
    pub fn propagates_transitively_owned_child_nodes_to_topmost_parent_node() {
        // Arrange: a setup from the rustdoc example of `propagate_transitively_owned_sets()`
        let directly_owned_sets = vec![
            (substate(1, 0, 0), vec![node_id(2), node_id(3)]),
            (substate(2, 0, 0), vec![node_id(4)]),
            (substate(7, 0, 0), vec![node_id(8), node_id(9), node_id(10)]),
            (substate(4, 0, 0), vec![node_id(5), node_id(6)]),
        ];

        // Act: call the low-level helper directly, since only its non-trivial logic is under test
        let transitively_owned_sets =
            NodeAncestryResolver::propagate_transitively_owned_sets(directly_owned_sets);

        // Assert: only the topmost substates are returned, each with its entire transitive owned Node set
        assert_eq!(
            transitively_owned_sets.collect::<Vec<_>>(),
            vec![
                (
                    substate(1, 0, 0),
                    vec![node_id(2), node_id(3), node_id(4), node_id(5), node_id(6)]
                ),
                (substate(7, 0, 0), vec![node_id(8), node_id(9), node_id(10)]),
            ]
        );
    }

    #[test]
    pub fn propagates_transitively_owned_child_nodes_to_distinct_substates_of_same_parent_node() {
        // Arrange: 2 different substates of Node ID = 3 form 2 trees of the forest
        let directly_owned_sets = vec![
            (substate(2, 1, 1), vec![node_id(5), node_id(7)]),
            (substate(3, 1, 1), vec![node_id(2), node_id(6)]),
            (substate(3, 8, 8), vec![node_id(9)]),
        ];

        // Act: call the low-level helper directly, since only its non-trivial logic is under test
        let transitively_owned_sets =
            NodeAncestryResolver::propagate_transitively_owned_sets(directly_owned_sets);

        // Assert: only the topmost substates are returned, each with its entire transitive owned Node set
        assert_eq!(
            transitively_owned_sets.collect::<Vec<_>>(),
            vec![
                (
                    substate(3, 1, 1),
                    vec![node_id(2), node_id(6), node_id(5), node_id(7)]
                ),
                (substate(3, 8, 8), vec![node_id(9)]),
            ]
        );
    }

    fn node_id(seed: u8) -> NodeId {
        NodeId([seed; NodeId::LENGTH])
    }

    fn substate(node_id_seed: u8, partition: u8, substate_key_seed: u8) -> SubstateReference {
        SubstateReference(
            node_id(node_id_seed),
            PartitionNumber(partition),
            SubstateKey::Field(substate_key_seed),
        )
    }

    fn record(parent: SubstateReference, root: SubstateReference) -> SubstateNodeAncestryRecord {
        SubstateNodeAncestryRecord { parent, root }
    }

    fn create(new_value: impl ScryptoEncode) -> ChangeAction {
        ChangeAction::Create(scrypto_encode(&new_value).unwrap())
    }

    fn update(new_value: impl ScryptoEncode, previous_value: impl ScryptoEncode) -> ChangeAction {
        ChangeAction::Update {
            new: scrypto_encode(&new_value).unwrap(),
            previous: scrypto_encode(&previous_value).unwrap(),
        }
    }

    impl SubstateNodeAncestryStore for HashMap<NodeId, SubstateNodeAncestryRecord> {
        fn get_ancestry(&self, node_id: &NodeId) -> Option<SubstateNodeAncestryRecord> {
            self.get(node_id).cloned()
        }
    }
}
