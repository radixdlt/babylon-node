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

use radix_engine::blueprints::resource::VaultInfoSubstate;
use radix_engine::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::node_substates::PersistedSubstate;

use radix_engine::types::{
    AccessControllerOffset, Address, ComponentOffset, EpochManagerOffset, KeyValueStoreOffset,
    RENodeId, SubstateId, SubstateOffset, ValidatorOffset, VaultOffset,
};
use radix_engine_interface::api::types::{AccountOffset, Blueprint, NodeModuleId, TypeInfoOffset};
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_BLUEPRINT;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::VALIDATOR_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType, VAULT_BLUEPRINT,
};
use radix_engine_interface::constants::*;
use radix_engine_interface::data::scrypto::model::ComponentAddress;

#[derive(Debug)]
pub enum StateTreeTraverserError {
    ExpectedNodeSubstateNotInStore(SubstateId),
    UnexpectedPersistedNode(RENodeId),
    UnexpectedTypeIdForObject(RENodeId),
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
    fn visit_fungible_vault(
        &mut self,
        _parent_id: Option<&SubstateId>,
        _info: &VaultInfoSubstate,
        _liquid: &LiquidFungibleResource,
    ) {
    }
    fn visit_non_fungible_vault(
        &mut self,
        _parent_id: Option<&SubstateId>,
        _info: &VaultInfoSubstate,
        _liquid: &LiquidNonFungibleResource,
    ) {
    }
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
            RENodeId::Object(..) => {
                let info_substate = self.read_substate(&SubstateId(
                    node_id,
                    NodeModuleId::TypeInfo,
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                ))?;
                let type_info: TypeInfoSubstate = info_substate.to_runtime().into();

                match type_info {
                    TypeInfoSubstate::Object {
                        blueprint:
                            Blueprint {
                                package_address,
                                blueprint_name,
                            },
                        ..
                    } => match (package_address, blueprint_name.as_str()) {
                        (RESOURCE_MANAGER_PACKAGE, VAULT_BLUEPRINT) => {
                            let info_substate = self.read_substate(&SubstateId(
                                node_id,
                                NodeModuleId::SELF,
                                SubstateOffset::Vault(VaultOffset::Info),
                            ))?;
                            let vault_info: VaultInfoSubstate = info_substate.to_runtime().into();
                            match vault_info.resource_type {
                                ResourceType::Fungible { .. } => {
                                    let liquid_substate = self.read_substate(&SubstateId(
                                        node_id,
                                        NodeModuleId::SELF,
                                        SubstateOffset::Vault(VaultOffset::LiquidFungible),
                                    ))?;

                                    self.visitor.visit_fungible_vault(
                                        parent,
                                        &vault_info,
                                        &liquid_substate.into(),
                                    );
                                }
                                ResourceType::NonFungible { .. } => {
                                    let liquid_substate = self.read_substate(&SubstateId(
                                        node_id,
                                        NodeModuleId::SELF,
                                        SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                                    ))?;

                                    self.visitor.visit_non_fungible_vault(
                                        parent,
                                        &vault_info,
                                        &liquid_substate.into(),
                                    );
                                }
                            }
                        }
                        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => {
                            self.recurse_via_self_substate(
                                node_id,
                                SubstateOffset::Account(AccountOffset::Account),
                                depth,
                            )?;
                        }
                        (EPOCH_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => {
                            self.recurse_via_self_substate(
                                node_id,
                                SubstateOffset::Validator(ValidatorOffset::Validator),
                                depth,
                            )?;
                        }
                        (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                            self.recurse_via_self_substate(
                                node_id,
                                SubstateOffset::AccessController(
                                    AccessControllerOffset::AccessController,
                                ),
                                depth,
                            )?;
                        }
                        _ => {
                            self.recurse_via_self_substate(
                                node_id,
                                SubstateOffset::Component(ComponentOffset::State0),
                                depth,
                            )?;
                        }
                    },
                    TypeInfoSubstate::KeyValueStore(_) => {
                        return Err(StateTreeTraverserError::UnexpectedTypeIdForObject(node_id))
                    }
                }
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
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Normal(..))) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Component(ComponentOffset::State0),
                    depth,
                )?;
            }
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Account(..)
                | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                | ComponentAddress::EddsaEd25519VirtualAccount(..),
            )) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Account(AccountOffset::Account),
                    depth,
                )?;
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Validator(..))) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::Validator(ValidatorOffset::Validator),
                    depth,
                )?;
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::AccessController(..))) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::AccessController(AccessControllerOffset::AccessController),
                    depth,
                )?;
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::EpochManager(..))) => {
                self.recurse_via_self_substate(
                    node_id,
                    SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
                    depth,
                )?;
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Clock(..))) => {} // Contains no children
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Identity(..)
                | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                | ComponentAddress::EddsaEd25519VirtualIdentity(..),
            )) => {} // Contains no children
            RENodeId::GlobalObject(Address::Resource(..)) => {} // Contains no children
            RENodeId::GlobalObject(Address::Package(..)) => {}  // Contains no children
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
