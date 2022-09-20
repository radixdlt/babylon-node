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

use radix_engine::engine::Substate;
use radix_engine::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::model::{ComponentState, KeyValueStoreEntryWrapper, Vault};

use scrypto::engine::types::{RENodeId, SubstateId};
use scrypto::values::ScryptoValue;

#[derive(Debug)]
pub enum StateTreeVisitorError {
    RENodeNotFound(RENodeId),
    MaxDepthExceeded,
}

pub struct StateTreeTraversor<
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
    fn visit_vault(&mut self, _parent_id: Option<&SubstateId>, _vault: &Vault) {}
    fn visit_node_id(&mut self, _parent_id: Option<&SubstateId>, _node_id: &RENodeId, _depth: u32) {
    }
}

impl<'s, 'v, S: ReadableSubstateStore + QueryableSubstateStore, V: StateTreeVisitor>
    StateTreeTraversor<'s, 'v, S, V>
{
    pub fn new(substate_store: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        StateTreeTraversor {
            substate_store,
            visitor,
            max_depth,
        }
    }

    pub fn traverse_all_descendents(
        &mut self,
        parent_node_id: Option<&SubstateId>,
        node_id: RENodeId,
    ) -> Result<(), StateTreeVisitorError> {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&SubstateId>,
        node_id: RENodeId,
        depth: u32,
    ) -> Result<(), StateTreeVisitorError> {
        if depth > self.max_depth {
            return Err(StateTreeVisitorError::MaxDepthExceeded);
        }
        self.visitor.visit_node_id(parent, &node_id, depth);
        match node_id {
            RENodeId::Vault(vault_id) => {
                let substate_id = SubstateId::Vault(vault_id);
                if let Some(output_value) = self.substate_store.get_substate(&substate_id) {
                    let vault: Vault = output_value.substate.into();

                    self.visitor.visit_vault(Some(&substate_id), &vault);
                } else {
                    return Err(StateTreeVisitorError::RENodeNotFound(node_id));
                }
            }
            RENodeId::KeyValueStore(kv_store_id) => {
                let map = self.substate_store.get_kv_store_entries(&kv_store_id);
                for (key, v) in map.iter() {
                    let substate_id = SubstateId::KeyValueStoreEntry(kv_store_id, key.clone());
                    if let Substate::KeyValueStoreEntry(KeyValueStoreEntryWrapper(Some(entry))) = v
                    {
                        let value = ScryptoValue::from_slice(entry)
                            .expect("Key Value Store Entry should be parseable.");
                        for child_node_id in value.stored_node_ids() {
                            self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                                .expect("Broken Node Store");
                        }
                    }
                }
            }
            RENodeId::Component(component_address) => {
                let substate_id = SubstateId::ComponentState(component_address);
                if let Some(output_value) = self.substate_store.get_substate(&substate_id) {
                    let component_state: ComponentState = output_value.substate.into();
                    let value = ScryptoValue::from_slice(component_state.state())
                        .expect("Component state should be parseable.");
                    for child_node_id in value.stored_node_ids() {
                        self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                            .expect("Broken Node Store");
                    }
                } else {
                    return Err(StateTreeVisitorError::RENodeNotFound(node_id));
                }
            }
            _ => {}
        };

        Ok(())
    }
}
