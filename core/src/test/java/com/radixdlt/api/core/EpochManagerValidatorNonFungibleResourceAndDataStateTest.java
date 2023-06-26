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

package com.radixdlt.api.core;

import static org.assertj.core.api.Assertions.assertThat;

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import org.junit.Test;

public final class EpochManagerValidatorNonFungibleResourceAndDataStateTest
    extends DeterministicCoreApiTestBase {
  @Test
  public void test_misc_state_endpoints() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // We test all these together because they can bootstrap each other to easily find real
      // addresses from genesis!
      // We use the fact that the default genesis:
      // - Creates a single validator, which creates a validator owner badge
      // - This validator owner badge is a non-fungible resource with a non-fungible deposited into
      // an account
      // - It is configured to use UUID-based ids and have no data (update this test if that
      // changes)
      //
      // Basically - this is just a smoke test to check all these (relatively complex) state
      // endpoints don't panic

      final var stateConsensusManagerResponse =
          getStateApi()
              .stateConsensusManagerPost(
                  new StateConsensusManagerRequest().network(networkLogicalName));

      final var stateSubstate =
          (ConsensusManagerFieldStateSubstate) stateConsensusManagerResponse.getState();
      assertThat(stateSubstate.getEpoch()).isGreaterThanOrEqualTo(0);

      final var validatorAddress =
          ((ConsensusManagerFieldCurrentValidatorSetSubstate)
                  stateConsensusManagerResponse.getCurrentValidatorSet())
              .getValidatorSet()
              .get(0)
              .getAddress();

      final var validatorResponse =
          getStateApi()
              .stateValidatorPost(
                  new StateValidatorRequest()
                      .network(networkLogicalName)
                      .validatorAddress(validatorAddress));

      // TODO(wip) - Re-enable when latest develop from scrypto is pulled
      //      // We extract the "Owner Badge"
      //      final var ownerRoleSubstate = (AccessRulesModuleFieldOwnerRoleSubstate)
      // validatorResponse.getOwnerRole();
      //      final var ownerRole = (FixedOwnerRole) ownerRoleSubstate.getOwnerRole();
      //      final var accessRule = (ProtectedAccessRule) ownerRole.getAccessRule();
      //      final var proofRuleNode = (ProofAccessRuleNode) accessRule.getAccessRule();
      //      final var requireProofRule = (RequireProofRule) proofRuleNode.getProofRule();
      //      final var requirement = (NonFungibleRequirement) requireProofRule.getRequirement();
      //      final var nonFungibleResourceAddress =
      // requirement.getNonFungible().getResourceAddress();
      //      final var nonFungibleLocalId = requirement.getNonFungible().getLocalId();
      //
      //      final var nonFungibleResourceResponse =
      //          getStateApi()
      //              .stateResourcePost(
      //                  new StateResourceRequest()
      //                      .network(networkLogicalName)
      //                      .resourceAddress(nonFungibleResourceAddress));
      //
      //      final var nonFungibleManager =
      //          (StateNonFungibleResourceManager) nonFungibleResourceResponse.getManager();
      //      final var idTypeSubstate =
      //          (NonFungibleResourceManagerFieldIdTypeSubstate) nonFungibleManager.getIdType();
      //      assertThat(idTypeSubstate.getNonFungibleIdType()).isEqualTo(NonFungibleIdType.RUID);
      //
      //      final var nonFungibleDataResponse =
      //          getStateApi()
      //              .stateNonFungiblePost(
      //                  new StateNonFungibleRequest()
      //                      .network(networkLogicalName)
      //                      .resourceAddress(nonFungibleResourceAddress)
      //                      .nonFungibleId(nonFungibleLocalId.getSimpleRep()));
      //
      //      final var dataSubstate =
      //          (NonFungibleResourceManagerDataEntrySubstate)
      // nonFungibleDataResponse.getNonFungible();
      //      assert dataSubstate.getDataStruct() != null;
      //      // Unit tuple
      //      assertThat(dataSubstate.getDataStruct().getStructData().getHex()).isEqualTo("5c2100");
    }
  }
}
