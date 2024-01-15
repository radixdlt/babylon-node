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

package com.radixdlt.api.browse;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.Iterables;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;
import com.radixdlt.api.DeterministicBrowseApiTestBase;
import com.radixdlt.api.browse.generated.models.*;
import java.util.List;
import java.util.Set;
import org.junit.Test;

public final class BrowseBlueprintInfoTest extends DeterministicBrowseApiTestBase {

  @Test
  public void browse_api_returns_blueprint_info() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();
      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // fetch ConsensusManager's blueprint, since it has nice fields and collections to assert on
      final var blueprintInfo =
          getTypesApi()
              .blueprintInfoPost(
                  new BrowseBlueprintInfoRequest()
                      .packageAddress(wellKnownAddresses.getConsensusManagerPackage())
                      .blueprintName("ConsensusManager"))
              .getInfo();

      assertThat(blueprintInfo.getOuterBlueprintName()).isNull();
      assertThat(blueprintInfo.getIsTransient()).isFalse();
      assertThat(blueprintInfo.getGenericTypeParameters()).isEmpty();
      assertThat(blueprintInfo.getAvailableFeatures()).isEmpty();
      assertThat(blueprintInfo.getNamedTypes()).isEmpty();

      final var fieldNames =
          Lists.transform(blueprintInfo.getFields(), BlueprintFieldInfo::getName);
      assertThat(fieldNames)
          .isEqualTo(
              List.of(
                  "configuration",
                  "state",
                  "validator_rewards",
                  "current_validator_set",
                  "current_proposal_statistic",
                  "proposer_minute_timestamp",
                  "proposer_milli_timestamp"));

      final var collectionNames =
          Lists.transform(blueprintInfo.getCollections(), BlueprintCollectionInfo::getName);
      assertThat(collectionNames).isEqualTo(List.of("registered_validator_by_stake"));

      // the only function there is convenient to assert the access rule on:
      final var createFunction = Iterables.getOnlyElement(blueprintInfo.getFunctions());
      assertThat(createFunction.getName()).isEqualTo("create");
      assertThat(createFunction.getAuthorization())
          .isEqualTo(
              new ByAccessRuleBlueprintFunctionAuthorization()
                  .rule(
                      new ProtectedAccessRule()
                          .accessRule(
                              new ProofAccessRuleNode()
                                  .proofRule(
                                      new RequireProofRule()
                                          .requirement(
                                              new NonFungibleRequirement()
                                                  .nonFungible(
                                                      new NonFungibleGlobalId()
                                                          .resourceAddress(
                                                              "resource_test1nfxxxxxxxxxxsystxnxxxxxxxxx002683325037xxxxxxxxx39ajmy")
                                                          .localId("#0#"))
                                                  .type(RequirementType.NONFUNGIBLE))
                                          .type(ProofRuleType.REQUIRE))
                                  .type(AccessRuleNodeType.PROOFRULE))
                          .type(AccessRuleType.PROTECTED))
                  .type(BlueprintFunctionAuthorizationType.BYACCESSRULE));

      // assert on all methods, and some example authorization configs:
      final var methods =
          Maps.uniqueIndex(blueprintInfo.getMethods(), BlueprintMethodInfo::getName);
      assertThat(methods.keySet())
          .isEqualTo(
              Set.of(
                  "get_current_epoch",
                  "start",
                  "get_current_time",
                  "compare_current_time",
                  "next_round",
                  "create_validator"));
      assertThat(methods.get("get_current_epoch").getAuthorization())
          .isEqualTo(
              new PublicBlueprintMethodAuthorization()
                  .type(BlueprintMethodAuthorizationType.PUBLIC));
      assertThat(methods.get("next_round").getAuthorization())
          .isEqualTo(
              new ByRolesBlueprintMethodAuthorization()
                  .roleKeys(List.of("validator"))
                  .type(BlueprintMethodAuthorizationType.BYROLES));

      final var roles = (LocalBlueprintRolesDefinition) blueprintInfo.getRoles();
      final var roleKeys = Lists.transform(roles.getDefinitions(), BlueprintRoleInfo::getKey);
      assertThat(roleKeys).isEqualTo(List.of("validator"));

      final var eventNames =
          Lists.transform(blueprintInfo.getEvents(), BlueprintEventInfo::getName);
      assertThat(eventNames).isEqualTo(List.of("RoundChangeEvent", "EpochChangeEvent"));
    }
  }
}
