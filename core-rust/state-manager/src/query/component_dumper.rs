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

use std::collections::BTreeSet;
use radix_engine::blueprints::resource::{VaultInfoSubstate};
use radix_engine::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use radix_engine::types::{ComponentAddress, RENodeId, SubstateId};
use radix_engine_interface::blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource, NonFungibleLocalId, ResourceAddress};
use radix_engine_interface::math::Decimal;

use super::component_state_tree_traverser::*;


pub enum VaultData {
    Fungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    },
}

pub struct ComponentStateDump {
    pub vaults: Vec<VaultData>,
    pub descendents: Vec<(Option<SubstateId>, RENodeId, u32)>,
}

impl StateTreeVisitor for ComponentStateDump {
    fn visit_fungible_vault(&mut self, _parent_id: Option<&SubstateId>, info: &VaultInfoSubstate, liquid: &LiquidFungibleResource) {
        self.vaults.push(VaultData::Fungible {
            resource_address: info.resource_address,
            amount: liquid.amount(),
        });
    }

    fn visit_non_fungible_vault(&mut self, _parent_id: Option<&SubstateId>, info: &VaultInfoSubstate, liquid: &LiquidNonFungibleResource) {
        self.vaults.push(VaultData::NonFungible {
            resource_address: info.resource_address,
            ids: liquid.ids().clone(),
        });
    }

    fn visit_node_id(&mut self, _parent_id: Option<&SubstateId>, node_id: &RENodeId, depth: u32) {
        self.descendents
            .push((_parent_id.cloned(), *node_id, depth));
    }
}

pub fn dump_component_state<S>(
    substate_store: &S,
    component: ComponentAddress,
) -> Result<ComponentStateDump, StateTreeTraverserError>
where
    S: ReadableSubstateStore + QueryableSubstateStore,
{
    let node_id = RENodeId::GlobalComponent(component);
    let mut component_dump = ComponentStateDump {
        vaults: Vec::new(),
        descendents: Vec::new(),
    };
    let mut state_tree_visitor =
        ComponentStateTreeTraverser::new(substate_store, &mut component_dump, 100);
    state_tree_visitor.traverse_all_descendents(None, node_id)?;

    Ok(component_dump)
}
