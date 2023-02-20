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

use radix_engine::blueprints::resource::VaultSubstate;
use radix_engine::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::system::substates::PersistedSubstate;

use radix_engine::types::{
    AccessControllerOffset, ComponentOffset, EpochManagerOffset, GlobalOffset, KeyValueStoreOffset,
    RENodeId, SubstateId, SubstateOffset, ValidatorOffset, VaultOffset,
};
use radix_engine_interface::api::types::{AccountOffset, NodeModuleId};

#[derive(Debug)]
pub enum StateTreeTraverserError {
    ExpectedNodeSubstateNotInStore(SubstateId),
    UnexpectedPersistedNode(RENodeId),
    MaxDepthExceeded,
}

pub struct ComponentStateTreeTraverser<
    's,
    'v,
    S: ReadableSubstateStore + QueryableSubstateStore,
    V: StateTreeVisitor,
> {
    substate_store: &'s S,
    visitor: &'v mut V,
    max_depth: u32,
}

pub trait StateTreeVisitor {
    fn visit_vault(&mut self, _parent_id: Option<&SubstateId>, _vault_substate: &VaultSubstate) {}
    fn visit_node_id(&mut self, _parent_id: Option<&SubstateId>, _node_id: &RENodeId, _depth: u32) {
    }
}

impl<'s, 'v, S: ReadableSubstateStore + QueryableSubstateStore, V: StateTreeVisitor>
    ComponentStateTreeTraverser<'s, 'v, S, V>
{
    pub fn new(substate_store: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        ComponentStateTreeTraverser {
            substate_store,
            visitor,
            max_depth,
        }
    }

    pub fn traverse_all_descendents(
        &mut self,
        parent_node_id: Option<&SubstateId>,
        node_id: RENodeId,
    ) -> Result<(), StateTreeTraverserError> {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&SubstateId>,
        node_id: RENodeId,
        depth: u32,
    ) -> Result<(), StateTreeTraverserError> {
        if depth > self.max_depth {
            return Err(StateTreeTraverserError::MaxDepthExceeded);
        }
        self.visitor.visit_node_id(parent, &node_id, depth);
        match node_id {
            RENodeId::Global(..) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Global(GlobalOffset::Global),
                    depth,
                )?;
            }
            RENodeId::Vault(..) => {
                let substate_id = SubstateId(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                );
                let substate = self.read_substate(&substate_id)?;
                self.visitor
                    .visit_vault(Some(&substate_id), substate.vault());
            }
            RENodeId::KeyValueStore(kv_store_id) => {
                let map = self.substate_store.get_kv_store_entries(&kv_store_id);
                for (key, substate) in map.into_iter() {
                    let substate_id = SubstateId(
                        node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                    );
                    self.recurse_via_loaded_substate(&substate_id, substate, depth)?;
                }
            }
            RENodeId::Component(..) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Component(ComponentOffset::State),
                    depth,
                )?;
            }
            RENodeId::Account(..) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Account(AccountOffset::Account),
                    depth,
                )?;
            }
            RENodeId::Validator(..) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Validator(ValidatorOffset::Validator),
                    depth,
                )?;
            }
            RENodeId::AccessController(..) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::AccessController(AccessControllerOffset::AccessController),
                    depth,
                )?;
            }
            RENodeId::EpochManager(_) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                    depth,
                )?;
            }
            RENodeId::Clock(_) => {}            // Contains no children
            RENodeId::Identity(_) => {}         // Contains no children
            RENodeId::NonFungibleStore(_) => {} // Contains no children
            RENodeId::ResourceManager(_) => {}  // Contains no children
            RENodeId::Package(_) => {}          // Contains no children
            // TRANSIENT
            RENodeId::Bucket(_)
            | RENodeId::Proof(_)
            | RENodeId::AuthZoneStack
            | RENodeId::FeeReserve(_)
            | RENodeId::Worktop
            | RENodeId::Logger
            | RENodeId::TransactionRuntime => {
                return Err(StateTreeTraverserError::UnexpectedPersistedNode(node_id))
            } // END - NB - we list all types so that we get a compile error if a new type is added
        };

        Ok(())
    }

    fn recurse_via_self_substate(
        &mut self,
        node_id: RENodeId,
        substate_offset: SubstateOffset,
        depth: u32,
    ) -> Result<(), StateTreeTraverserError> {
        let substate_id = SubstateId(node_id, NodeModuleId::SELF, substate_offset);
        let substate = self.read_substate(&substate_id)?;
        self.recurse_via_loaded_substate(&substate_id, substate, depth)
    }

    fn read_substate(
        &self,
        substate_id: &SubstateId,
    ) -> Result<PersistedSubstate, StateTreeTraverserError> {
        let substate = self
            .substate_store
            .get_substate(substate_id)
            .ok_or_else(|| {
                StateTreeTraverserError::ExpectedNodeSubstateNotInStore(substate_id.clone())
            })?
            .substate;
        Ok(substate)
    }

    fn recurse_via_loaded_substate(
        &mut self,
        substate_id: &SubstateId,
        substate: PersistedSubstate,
        depth: u32,
    ) -> Result<(), StateTreeTraverserError> {
        let (_, owned_nodes) = substate.to_runtime().to_ref().references_and_owned_nodes();
        for child_node_id in owned_nodes {
            self.traverse_recursive(Some(substate_id), child_node_id, depth + 1)?;
        }
        Ok(())
    }
}
