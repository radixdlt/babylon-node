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

import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.harness.deterministic.TestProtocolConfig;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import java.util.List;
import java.util.stream.LongStream;
import org.junit.Test;

public class NetworkScenariosTest extends DeterministicCoreApiTestBase {
  @Test
  public void test_network_scenarios() throws Exception {
    // pick a custom subset/permutation (different than "all scenarios"):
    final var protocolConfig =
        new TestProtocolConfig()
            .withGenesisScenarios(ImmutableList.of("radiswap", "transfer_xrd", "royalties"))
            .with(
                TestProtocolConfig.updateTo(
                    ProtocolUpdateTrigger.ANEMONE,
                    ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(3L)))
            .with(
                TestProtocolConfig.updateTo(
                        ProtocolUpdateTrigger.BOTTLENOSE,
                        ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(4L))
                    .withScenarios(ImmutableList.of("maya_router", "access-controller-v2")));
    try (var test = buildRunningServerTestWithProtocolConfig(30, protocolConfig)) {
      test.suppressUnusedWarning();

      // Query scenarios right after genesis
      final var genesisScenarios =
          getStatusApi()
              .statusScenariosPost(new ScenariosRequest().network(networkLogicalName))
              .getExecutedScenarios();
      assertThat(genesisScenarios).hasSize(3); // there is 3 of them

      // Wait for all protocol updates:
      test.runUntilState(allAtOrOverEpoch(protocolConfig.lastProtocolUpdateEnactmentEpoch()));

      // query all scenarios
      final var allScenarios =
          getStatusApi()
              .statusScenariosPost(new ScenariosRequest().network(networkLogicalName))
              .getExecutedScenarios();
      assertThat(allScenarios).hasSize(5); // there is 3 genesis + 2 bottlenose

      // assert some selected properties of the known scenarios
      assertScenario(
          allScenarios,
          0,
          "radiswap",
          ImmutableList.of(
              "radiswap-create-new-resources",
              "radiswap-create-owner-badge-and-dapp-definition-account",
              "radiswap-publish-and-create-pools",
              "radiswap-add-liquidity",
              "radiswap-distribute-tokens",
              "radiswap-swap-tokens",
              "radiswap-remove-tokens",
              "radiswap-set-two-way-linking"),
          ImmutableList.of(
              "radiswap_dapp_definition_account",
              "radiswap_dapp_owner_badge",
              "storing_account",
              "user_account_1",
              "user_account_2",
              "user_account_3",
              "radiswap_package",
              "pool_1_radiswap",
              "pool_1_pool",
              "pool_1_resource_1",
              "pool_1_resource_2",
              "pool_1_pool_unit",
              "pool_2_radiswap",
              "pool_2_pool",
              "pool_2_resource_1",
              "pool_2_resource_2",
              "pool_2_pool_unit"));
      assertScenario(
          allScenarios,
          1,
          "transfer_xrd",
          ImmutableList.of(
              "faucet-top-up",
              "transfer--try_deposit_or_abort",
              "transfer--try_deposit_or_refund",
              "transfer--try_deposit_batch_or_abort",
              "transfer--try_deposit_batch_or_refund",
              "self-transfer--deposit_batch",
              "multi-transfer--deposit_batch"),
          ImmutableList.of("from_account", "to_account_1", "to_account_2"));
      assertScenario(
          allScenarios,
          2,
          "royalties",
          ImmutableList.of(
              "royalties--publish-package",
              "royalties--instantiate-components",
              "royalties--set-components-royalty",
              "royalties--call_all_components_all_methods"),
          ImmutableList.of(
              "royalty_package_address",
              "no_royalty_component_address",
              "xrd_royalty_component_address",
              "usd_royalty_component_address"));
      assertScenario(
          allScenarios,
          3,
          "maya_router",
          ImmutableList.of(
              "maya-router-create-accounts",
              "faucet-top-up",
              "maya-router-create-resources",
              "maya-router-publish-and-instantiate"),
          ImmutableList.of(
              "owner_account",
              "swapper_account",
              "maya_router_package",
              "maya_router_address",
              "XRD",
              "resource_1",
              "resource_2"));
      assertScenario(
          allScenarios,
          4,
          "access-controller-v2",
          ImmutableList.of(
              "access-controller-v2-instantiate",
              "access-controller-v2-deposit-fees-xrd",
              "access-controller-v2-lock-fee-and-recover"),
          ImmutableList.of("access_controller_v2_component_address"));
    }
  }

  private static void assertScenario(
      List<ExecutedScenario> scenarios,
      int index,
      String scenarioName,
      ImmutableList<String> expectedTransactionNames,
      ImmutableList<String> expectedAddressNames) {
    final var scenario = scenarios.get(index);
    assertThat(scenario.getSequenceNumber()).isEqualTo(index);
    assertThat(scenario.getLogicalName()).isEqualTo(scenarioName);

    final var transactionNames =
        Lists.transform(
            scenario.getCommittedTransactions(), ExecutedScenarioTransaction::getLogicalName);
    assertThat(transactionNames).hasSameElementsAs(expectedTransactionNames);
    final var transactionStateVersions =
        Lists.transform(
            scenario.getCommittedTransactions(), ExecutedScenarioTransaction::getStateVersion);
    assertThat(transactionStateVersions).isNotEmpty();
    assertThat(transactionStateVersions).doesNotHaveDuplicates();
    assertThat(transactionStateVersions).isSorted();
    final var consecutiveVersions =
        LongStream.rangeClosed(
            transactionStateVersions.get(0),
            transactionStateVersions.get(transactionStateVersions.size() - 1));
    assertThat(transactionStateVersions).hasSameElementsAs(consecutiveVersions.boxed().toList());

    assertThat(scenario.getAddresses().keySet()).hasSameElementsAs(expectedAddressNames);
  }
}
