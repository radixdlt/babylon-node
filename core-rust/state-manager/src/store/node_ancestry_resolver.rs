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
use crate::{ChangeAction, SubstateChange};

use radix_engine::types::*;

/// A resolver of [`SubstateNodeAncestryRecord`]s for [`SubstateChange`]s.
pub struct NodeAncestryResolver {
    /// A map of created Nodes to their parent Substates, extracted from the [`SubstateChange`]s.
    created_node_to_parent: NonIterMap<NodeId, SubstateReference>,
    /// A map of existing Nodes to their root Nodes (i.e. `SubstateNodeAncestryRecord#root`), loaded
    /// from a [`SubstateNodeAncestryStore`] for each Node that does not have a parent in the
    /// [`created_node_to_parent`] map.
    top_created_node_to_existing_root: NonIterMap<NodeId, SubstateReference>,
}

impl NodeAncestryResolver {
    /// Resolves the [`SubstateNodeAncestryRecord`] for all newly-created Nodes found in the given
    /// [`SubstateChange`]s.
    /// The resolution considers the trees (or rather: potentially-incomplete bottom-up tree
    /// fragments) constructed within the given [`SubstateChange`]s, and the top fragments of these
    /// trees already-existing in the given [`SubstateNodeAncestryStore`].
    /// API note: the results are grouped by [`SubstateNodeAncestryRecord`] (since it may be shared
    /// by many child [`NodeId`]s).
    pub fn batch_resolve<S: SubstateNodeAncestryStore>(
        ancestry_store: &S,
        substate_changes: &[SubstateChange],
    ) -> impl Iterator<Item = (Vec<NodeId>, SubstateNodeAncestryRecord)> {
        // Index the new Nodes by their parent Substates (using `IndexedScryptoValue`):
        let parent_to_created_nodes = substate_changes
            .iter()
            .filter_map(|substate_change| match &substate_change.action {
                ChangeAction::Create(bytes) | ChangeAction::Update(bytes) => {
                    let owned_node_ids = IndexedScryptoValue::from_slice(bytes).unwrap().unpack().1;
                    if owned_node_ids.is_empty() {
                        return None;
                    }
                    let parent_reference = SubstateReference(
                        substate_change.node_id,
                        substate_change.partition_number,
                        substate_change.substate_key.clone(),
                    );
                    Some((parent_reference, owned_node_ids))
                }
                ChangeAction::Delete => None,
            })
            .collect::<IndexMap<_, _>>();

        // Invert the multimap (to obtain a Node -> parent Substate map):
        let created_node_to_parent = parent_to_created_nodes
            .iter()
            .flat_map(|(parent, children)| children.iter().map(|child| (*child, parent.clone())))
            .collect::<NonIterMap<_, _>>();

        // Find the parents which do _not_ know their parent within the `SubstateChange`s:
        let top_level_parent_node_ids = parent_to_created_nodes
            .keys()
            .filter(|parent| !created_node_to_parent.contains_key(&parent.0))
            .map(|parent| parent.0)
            .collect::<Vec<_>>();

        // Load the missing records of these parents from the `SubstateNodeAncestryStore`:
        let existing_records = ancestry_store.batch_get_ancestry(&top_level_parent_node_ids);

        // Construct a Node -> root map needed for the `NodeAncestryResolver` helper structure:
        let top_created_node_to_existing_root = top_level_parent_node_ids
            .into_iter()
            .zip(existing_records)
            .filter_map(|(parent, existing_record)| {
                existing_record.map(|record| (parent, record.root))
            })
            .collect::<NonIterMap<_, _>>();

        let resolver = Self {
            created_node_to_parent,
            top_created_node_to_existing_root,
        };

        // Simply query the fully-preloaded `NodeAncestryResolver` for each Node:
        parent_to_created_nodes
            .into_iter()
            .map(move |(parent, child_node_ids)| {
                let root = resolver.resolve_root(&parent);
                (child_node_ids, SubstateNodeAncestryRecord { parent, root })
            })
    }

    /// Resolves the root Substate of the given one by traversing the internal maps:
    /// - the given Substate is assumed to originate from the [`SubstateChange`]s, so we first
    ///   follow the parent links through the [`created_node_to_parent`];
    /// - after this walk ends, we check the [`top_created_node_to_existing_root`], which may contain a hit
    ///   (i.e. when [`SubstateChange`] did _not_ create the entire tree, including root).
    /// - if there is no hit after the above, then it means that the given Substate is a root, and
    ///   it will be returned itself.
    pub fn resolve_root(&self, substate: &SubstateReference) -> SubstateReference {
        let mut at_substate = substate;
        loop {
            let parent = self.created_node_to_parent.get(&at_substate.0);
            if let Some(parent) = parent {
                at_substate = parent;
            } else {
                break;
            }
        }
        self.top_created_node_to_existing_root
            .get(&at_substate.0)
            .unwrap_or(at_substate)
            .clone()
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
            change(
                substate(3, 12, 23),
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(4))),
                },
            ),
            // some new child Node ID = 5 (right under a root Node ID = 1):
            change(
                substate(1, 14, 24),
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(5))),
                },
            ),
        ];

        // Act
        let new_index_entries =
            NodeAncestryResolver::batch_resolve(&existing_index_entries, &substate_changes)
                .flat_map(|(key_batch, value)| {
                    key_batch.into_iter().map(move |key| (key, value.clone()))
                })
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
            change(
                substate(6, 10, 29),
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(7))),
                },
            ),
            // some new grand-child Node ID = 6 (under an existing non-root Node ID = 2):
            change(
                substate(2, 14, 24),
                Value::Custom {
                    value: ScryptoCustomValue::Own(Own(node_id(6))),
                },
            ),
        ];

        // Act
        let new_index_entries =
            NodeAncestryResolver::batch_resolve(&existing_index_entries, &substate_changes)
                .flat_map(|(key_batch, value)| {
                    key_batch.into_iter().map(move |key| (key, value.clone()))
                })
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
    pub fn newly_created_root_nodes_are_not_recorded_in_index() {
        // Arrange
        let existing_index_entries = hashmap!(
            // some unrelated existing entry:
            node_id(2) => record(substate(1, 11, 25), substate(1, 11, 25)),
        );
        let substate_changes = [
            // some new root Node ID = 6 (which does not own any other Node):
            change(substate(6, 10, 29), ScryptoValue::I16 { value: 666 }),
        ];

        // Act
        let new_index_entries =
            NodeAncestryResolver::batch_resolve(&existing_index_entries, &substate_changes)
                .collect::<Vec<_>>();

        // Assert
        assert_eq!(new_index_entries, vec![]);
    }

    fn node_id(seed: u8) -> NodeId {
        NodeId([seed; NodeId::LENGTH])
    }

    fn substate(node_id_seed: u8, partition: u8, substate_key_seed: u8) -> SubstateReference {
        SubstateReference(
            node_id(node_id_seed),
            PartitionNumber(partition),
            SubstateKey::Tuple(substate_key_seed),
        )
    }

    fn record(parent: SubstateReference, root: SubstateReference) -> SubstateNodeAncestryRecord {
        SubstateNodeAncestryRecord { parent, root }
    }

    fn change(substate: SubstateReference, new_value: impl ScryptoEncode) -> SubstateChange {
        SubstateChange {
            node_id: substate.0,
            partition_number: substate.1,
            substate_key: substate.2,
            action: ChangeAction::Update(scrypto_encode(&new_value).unwrap()),
        }
    }

    impl SubstateNodeAncestryStore for HashMap<NodeId, SubstateNodeAncestryRecord> {
        fn get_ancestry(&self, node_id: &NodeId) -> Option<SubstateNodeAncestryRecord> {
            self.get(node_id).cloned()
        }
    }
}
