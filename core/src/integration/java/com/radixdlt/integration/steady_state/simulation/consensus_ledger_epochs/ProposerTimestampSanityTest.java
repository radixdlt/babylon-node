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

package com.radixdlt.integration.steady_state.simulation.consensus_ledger_epochs;

import static org.assertj.core.api.AssertionsForInterfaceTypes.assertThat;
import static org.junit.Assert.assertEquals;

import com.google.common.collect.ImmutableList;
import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.radixdlt.consensus.EpochNodeWeightMapping;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.harness.simulation.NetworkLatencies;
import com.radixdlt.harness.simulation.NetworkOrdering;
import com.radixdlt.harness.simulation.SimulationTest;
import com.radixdlt.harness.simulation.monitors.consensus.ConsensusMonitors;
import com.radixdlt.harness.simulation.monitors.ledger.LedgerMonitors;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.utils.TimeSupplier;
import java.util.concurrent.TimeUnit;
import java.util.stream.IntStream;
import org.junit.Test;

public final class ProposerTimestampSanityTest {
  @Test
  public void test_single_node_clock_delayed_no_timeouts() {
    final var builder =
        SimulationTest.builder()
            .networkModules(NetworkOrdering.inOrder(), NetworkLatencies.fixed())
            .numPhysicalNodes(4)
            .addTestModules(
                ConsensusMonitors.safety(),
                ConsensusMonitors.liveness(5, TimeUnit.SECONDS),
                LedgerMonitors.consensusToLedger(),
                ConsensusMonitors.proposerTimestampChecker(),
                ConsensusMonitors
                    .noTimeouts(), // There should be no timeouts if just a single node is delayed
                ConsensusMonitors.directParents(),
                LedgerMonitors.ordered())
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    NodeStorageConfig.none(),
                    true,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                    ConsensusConfig.of(1000),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerMockedSync(
                        StateComputerConfig.mockedWithEpochs(
                            Round.of(10),
                            EpochNodeWeightMapping.constant(e -> IntStream.range(0, 4)),
                            new StateComputerConfig.MockedMempoolConfig.NoMempool()))));

    /* One node delayed */
    modifyNthNodeTimeSupplier(0, () -> System.currentTimeMillis() - 4000, builder);

    final var results = builder.build().run().awaitCompletion();
    assertThat(results).allSatisfy((name, err) -> assertThat(err).isEmpty());
  }

  @Test
  public void test_two_nodes_clock_slightly_rushing_or_delaying_no_timeouts() {
    final var builder =
        SimulationTest.builder()
            .networkModules(NetworkOrdering.inOrder(), NetworkLatencies.fixed())
            .numPhysicalNodes(4)
            .addTestModules(
                ConsensusMonitors.safety(),
                ConsensusMonitors.liveness(5, TimeUnit.SECONDS),
                ConsensusMonitors.noTimeouts(),
                ConsensusMonitors.directParents(),
                LedgerMonitors.consensusToLedger(),
                ConsensusMonitors.proposerTimestampChecker(),
                LedgerMonitors.ordered())
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    NodeStorageConfig.none(),
                    true,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                    ConsensusConfig.of(1000),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerMockedSync(
                        StateComputerConfig.mockedWithEpochs(
                            Round.of(10),
                            EpochNodeWeightMapping.constant(e -> IntStream.range(0, 4)),
                            new StateComputerConfig.MockedMempoolConfig.NoMempool()))));

    /* One node rushing within acceptable bounds */
    modifyNthNodeTimeSupplier(0, () -> System.currentTimeMillis() + 500, builder);

    /* One node delayed within acceptable bounds */
    modifyNthNodeTimeSupplier(1, () -> System.currentTimeMillis() - 500, builder);

    final var results = builder.build().run().awaitCompletion();
    assertThat(results).allSatisfy((name, err) -> assertThat(err).isEmpty());
  }

  @Test
  public void test_two_nodes_clock_significantly_rushing_or_delaying_safety_but_no_liveness() {
    final var builder =
        SimulationTest.builder()
            .networkModules(NetworkOrdering.inOrder(), NetworkLatencies.fixed())
            .numPhysicalNodes(4)
            .addTestModules(
                ConsensusMonitors.safety(),
                LedgerMonitors.consensusToLedger(),
                ConsensusMonitors.proposerTimestampChecker(),
                LedgerMonitors.ordered())
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    NodeStorageConfig.none(),
                    true,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                    ConsensusConfig.of(1000),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerMockedSync(
                        StateComputerConfig.mockedWithEpochs(
                            Round.of(10),
                            EpochNodeWeightMapping.constant(e -> IntStream.range(0, 4)),
                            new StateComputerConfig.MockedMempoolConfig.NoMempool(),
                            StateComputerConfig.ProposerElectionMode.ONLY_WEIGHTED_BY_STAKE))));

    /* One node rushing */
    modifyNthNodeTimeSupplier(0, () -> System.currentTimeMillis() - 4000, builder);

    /* One node delayed */
    modifyNthNodeTimeSupplier(1, () -> System.currentTimeMillis() + 31000, builder);

    final var runningTest = builder.build().run();
    final var results = runningTest.awaitCompletion();

    assertThat(results).allSatisfy((name, err) -> assertThat(err).isEmpty());

    // In this test scenario there should be no liveness whatsoever.
    // Making sure that not a single transaction went through.
    for (final var nodeCounters : runningTest.getNetwork().getMetrics().values()) {
      assertEquals(0, (long) nodeCounters.ledger().bftTransactionsProcessed().get());
    }
  }

  private void modifyNthNodeTimeSupplier(
      int n, TimeSupplier timeSupplier, SimulationTest.Builder builder) {
    builder.addOverrideModuleToInitialNodes(
        nodes -> ImmutableList.of(nodes.get(n).getPublicKey()),
        () ->
            new AbstractModule() {
              @Provides
              public TimeSupplier timeSupplier() {
                return timeSupplier;
              }
            });
  }
}
