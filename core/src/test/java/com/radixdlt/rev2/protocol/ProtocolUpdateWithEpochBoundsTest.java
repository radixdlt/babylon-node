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

package com.radixdlt.rev2.protocol;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.*;
import static com.radixdlt.lang.Tuple.tuple;
import static com.radixdlt.rev2.protocol.ProtocolUpdateTestUtils.*;
import static com.radixdlt.rev2.protocol.ProtocolUpdateWithEpochBoundsTest.TestEvent.*;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.radixdlt.environment.DatabaseFlags;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.*;
import com.radixdlt.rev2.*;
import com.radixdlt.utils.UInt192;
import java.util.*;
import java.util.function.Function;
import java.util.stream.Stream;
import org.junit.Ignore;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

@RunWith(Parameterized.class)
public final class ProtocolUpdateWithEpochBoundsTest {
  private static final Decimal STAKE_PER_VALIDATOR = Decimal.ofNonNegative(10000);

  @Parameterized.Parameters
  public static Collection<Object[]> testParameters() {
    return Stream.of(scenariosA(), scenariosB())
        .flatMap(List::stream)
        .map(scenario -> new Object[] {scenario})
        .toList();
  }

  static List<TestScenario> scenariosA() {
    final var numValidators = 4;
    // A list of scenarios using the following protocol update:
    // Enactment bounds: from epoch 5 to epoch 20 (both inclusive)
    // Readiness threshold: required one full epoch at 70%
    final var protocolUpdate =
        new ProtocolUpdate(
            "v2",
            ProtocolUpdateEnactmentCondition.singleReadinessThresholdBetweenEpochs(
                5, 20, Decimal.ofNonNegativeFraction(7, 10), 1));
    final var protocolUpdates = ImmutableList.of(protocolUpdate);

    // Enact at lower bound
    final var scenario1 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(3, signalReadiness(protocolUpdate, 0))
            .atEpoch(3, signalReadiness(protocolUpdate, 1))
            .atEpoch(3, signalReadiness(protocolUpdate, 2))
            // Readiness signals sent while at epoch X count from epoch X+1 onward
            .atEpoch(3, expectNoReportedReadiness())
            .atEpoch(4, expectOneProtocolReadiness(protocolUpdate, totalStakeOfNumValidators(3)))
            .atEpoch(5, expectEnactment(protocolUpdate)) // Enactment at next epoch (lower bound)
            .runUntilEpoch(7); // Run for two more epochs

    // Signal at lower bound (enact later)
    final var scenario2 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(5, signalReadiness(protocolUpdate, 1))
            .atEpoch(5, signalReadiness(protocolUpdate, 3))
            .atEpoch(5, signalReadiness(protocolUpdate, 2))
            .atEpoch(5, expectNoReportedReadiness())
            .atEpoch(6, expectOneProtocolReadiness(protocolUpdate, totalStakeOfNumValidators(3)))
            .atEpoch(7, expectEnactment(protocolUpdate))
            .runUntilEpoch(9);

    // Signals split across multiple epochs; one validator changing their signal
    final var mockReadinessSignalName = "testtesttesttesttesttesttesttest";
    final var scenario3 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(6, signalReadiness(protocolUpdate, 0))
            .atEpoch(6, signalReadiness(protocolUpdate, 1))
            .atEpoch(10, expectOneProtocolReadiness(protocolUpdate, totalStakeOfNumValidators(2)))
            .atEpoch(11, signalReadiness(mockReadinessSignalName, 2))
            .atEpoch(
                12, expectReadinessToInclude(mockReadinessSignalName, totalStakeOfNumValidators(1)))
            .atEpoch(
                12,
                expectReadinessToInclude(
                    protocolUpdate, totalStakeOfNumValidators(2))) // Prev readiness unchanged
            .atEpoch(13, signalReadiness(protocolUpdate, 2)) // Validator 2 changes their signal
            .atEpoch(14, expectOneProtocolReadiness(protocolUpdate, totalStakeOfNumValidators(3)))
            .atEpoch(15, expectEnactment(protocolUpdate))
            .runUntilEpoch(17);

    // Enact at upper bound
    final var scenario4 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(6, signalReadiness(protocolUpdate, 0))
            .atEpoch(6, signalReadiness(protocolUpdate, 1))
            .atEpoch(18, signalReadiness(protocolUpdate, 2))
            .atEpoch(18, signalReadiness(protocolUpdate, 3))
            .atEpoch(19, expectOneProtocolReadiness(protocolUpdate, totalStakeOfNumValidators(4)))
            .atEpoch(20, expectEnactment(protocolUpdate))
            .runUntilEpoch(22);

    return List.of(scenario1, scenario2, scenario3, scenario4);
  }

  static List<TestScenario> scenariosB() {
    final var numValidators = 6;
    // A list of scenarios using the following protocol updates:
    // - two unconditional (formerly known as "fixed-epoch") protocol updates at epochs 5 and 7
    // - followed by a readiness-based protocol update with two thresholds
    final var unconditionalProtocolUpdateAtEpoch5 =
        new ProtocolUpdate("v2", ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(5));
    final var unconditionalProtocolUpdateAtEpoch7 =
        new ProtocolUpdate("v3", ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(7));
    final var readinessThresholdsProtocolUpdate =
        new ProtocolUpdate(
            "v4",
            ProtocolUpdateEnactmentCondition.readinessThresholdsBetweenEpochs(
                5,
                20,
                ImmutableList.of(
                    // 3 epochs @ 4/6 validators
                    tuple(Decimal.ofNonNegativeFraction(4, 6), 3L),
                    // or immediately (i.e. right at the beginning of the first epoch on or above
                    // the threshold)
                    // with 100% support
                    tuple(Decimal.ONE, 0L))));

    final var protocolUpdates =
        ImmutableList.of(
            unconditionalProtocolUpdateAtEpoch5,
            unconditionalProtocolUpdateAtEpoch7,
            readinessThresholdsProtocolUpdate);

    // First threshold matches at epoch 11
    final var scenario1 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(5, expectEnactment(unconditionalProtocolUpdateAtEpoch5))
            .atEpoch(7, expectEnactment(unconditionalProtocolUpdateAtEpoch7))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 0))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 1))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 2))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 3))
            .atEpoch(
                8,
                expectOneProtocolReadiness(
                    readinessThresholdsProtocolUpdate, totalStakeOfNumValidators(4)))
            .atEpoch(11, expectEnactment(readinessThresholdsProtocolUpdate))
            .runUntilEpoch(12);

    // Second threshold matches at epoch 11
    final var scenario2 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(5, expectEnactment(unconditionalProtocolUpdateAtEpoch5))
            .atEpoch(7, expectEnactment(unconditionalProtocolUpdateAtEpoch7))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 0))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 1))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 2))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 3))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 4))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 5))
            .atEpoch(11, expectEnactment(readinessThresholdsProtocolUpdate))
            .runUntilEpoch(12);

    // Both thresholds match at epoch 11
    final var scenario3 =
        new ScenarioBuilder(numValidators, protocolUpdates)
            .atEpoch(5, expectEnactment(unconditionalProtocolUpdateAtEpoch5))
            .atEpoch(7, expectEnactment(unconditionalProtocolUpdateAtEpoch7))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 0))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 1))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 2))
            .atEpoch(7, signalReadiness(readinessThresholdsProtocolUpdate, 3))
            .atEpoch(
                8,
                expectOneProtocolReadiness(
                    readinessThresholdsProtocolUpdate, totalStakeOfNumValidators(4)))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 4))
            .atEpoch(10, signalReadiness(readinessThresholdsProtocolUpdate, 5))
            .atEpoch(11, expectEnactment(readinessThresholdsProtocolUpdate))
            .runUntilEpoch(12);

    return List.of(scenario1, scenario2, scenario3);
  }

  @Rule public TemporaryFolder folder = new TemporaryFolder();
  private final Random random = new Random(1234);
  private final TestScenario scenario;

  public ProtocolUpdateWithEpochBoundsTest(TestScenario scenario) {
    this.scenario = scenario;
  }

  private DeterministicTest createTest(ProtocolConfig protocolConfig) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(scenario.numValidators(), true))
        .messageSelector(firstSelector())
        .addMonitors(
            byzantineBehaviorNotDetected(), consensusLiveness(3000), ledgerTransactionSafety())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.tempFolder(folder),
                true,
                SafetyRecoveryConfig.BERKELEY_DB,
                ConsensusConfig.of(200),
                LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2(
                        Network.INTEGRATIONTESTNET.getId(),
                        GenesisBuilder.createTestGenesisWithNumValidators(
                            scenario.numValidators(),
                            STAKE_PER_VALIDATOR,
                            GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(30)
                                .totalEmissionXrdPerEpoch(Decimal.ZERO)),
                        new DatabaseFlags(true, false),
                        StateComputerConfig.REV2ProposerConfig.Mempool.defaults(),
                        false,
                        true,
                        protocolConfig))));
  }

  @Test
  public void test_protocol_update_scenario() {
    final var protocolConfig = new ProtocolConfig("genesis", scenario.protocolUpdates);
    try (var test = createTest(protocolConfig)) {
      test.startAllNodes();

      long currEpoch = runUntilNextEpoch(test);
      while (currEpoch <= scenario.runUntilEpoch) {
        if (random.nextBoolean()) {
          // Sometimes restart a random node after an epoch change...
          test.restartNode(random.nextInt(test.numNodes()));
        } else {
          // ...and sometimes restart all of them.
          for (int i = 0; i < scenario.numValidators; i++) {
            test.restartNode(i);
          }
        }

        final var events = scenario.eventsByEpoch.getOrDefault(currEpoch, ImmutableList.of());
        boolean expectedEnactment = false;
        for (var event : events) {
          switch (event) {
            case ExpectEnactment expectEnactment -> {
              expectedEnactment = true;
              verifyProtocolUpdateAtEpoch(
                  test, currEpoch, expectEnactment.protocolUpdate.nextProtocolVersion());
            }
            case ExpectReadiness expectReadiness -> {
              verifyCurrentEpochReadiness(test, expectReadiness.verifyFn);
            }
            case SignalReadiness signalReadiness -> {
              signalReadinessAndRunUntilCommit(
                  test, signalReadiness.validatorIdx, signalReadiness.readinessSignalName);
            }
          }
        }
        if (!expectedEnactment) {
          verifyNoProtocolUpdateAtEpoch(test, currEpoch);
        }

        // Restart a random node
        test.restartNode(random.nextInt(test.numNodes()));
        // Run a few more messages and restart another (or the same) node
        test.runForCount(10);
        test.restartNode(random.nextInt(test.numNodes()));

        currEpoch = runUntilNextEpoch(test);
      }
    }
  }

  record TestScenario(
      int numValidators,
      ImmutableList<ProtocolUpdate> protocolUpdates,
      ImmutableMap<Long, ImmutableList<TestEvent>> eventsByEpoch,
      long runUntilEpoch) {}

  @Ignore // No, junit, this is not a test class...
  sealed interface TestEvent {
    record SignalReadiness(String readinessSignalName, int validatorIdx) implements TestEvent {}

    record ExpectReadiness(Function<Map<String, Decimal>, Boolean> verifyFn) implements TestEvent {}

    record ExpectEnactment(ProtocolUpdate protocolUpdate) implements TestEvent {}

    static TestEvent expectNoReportedReadiness() {
      return new ExpectReadiness(Map::isEmpty);
    }

    static TestEvent signalReadiness(ProtocolUpdate protocolUpdate, int validatorIdx) {
      return signalReadiness(ProtocolUpdates.readinessSignalName(protocolUpdate), validatorIdx);
    }

    static TestEvent signalReadiness(String protocolUpdateName, int validatorIdx) {
      return new SignalReadiness(protocolUpdateName, validatorIdx);
    }

    static TestEvent expectOneProtocolReadiness(
        ProtocolUpdate protocolUpdate, Decimal expectedReadiness) {
      return new ExpectReadiness(
          readiness ->
              readiness.size() == 1
                  && readiness
                      .get(ProtocolUpdates.readinessSignalName(protocolUpdate))
                      .equals(expectedReadiness));
    }

    static TestEvent expectReadinessToInclude(
        String readinessSignalName, Decimal expectedReadiness) {
      return new ExpectReadiness(
          readiness -> readiness.get(readinessSignalName).equals(expectedReadiness));
    }

    static TestEvent expectReadinessToInclude(
        ProtocolUpdate protocolUpdate, Decimal expectedReadiness) {
      return expectReadinessToInclude(
          ProtocolUpdates.readinessSignalName(protocolUpdate), expectedReadiness);
    }

    static TestEvent expectEnactment(ProtocolUpdate protocolUpdate) {
      return new ExpectEnactment(protocolUpdate);
    }
  }

  static class ScenarioBuilder {
    int numValidators;
    ImmutableList<ProtocolUpdate> protocolUpdates;
    Map<Long, List<TestEvent>> eventsByEpochBuilder = new HashMap<>();
    long nextEpochMinValue = 3; // No need to handle pre-genesis epochs

    ScenarioBuilder(int numValidators, ImmutableList<ProtocolUpdate> protocolUpdates) {
      this.numValidators = numValidators;
      this.protocolUpdates = protocolUpdates;
    }

    ScenarioBuilder atEpoch(long epoch, TestEvent event) {
      // Enforces ordered builder calls for readability and to protect
      // against silly mistakes.
      if (epoch < nextEpochMinValue) {
        throw new IllegalArgumentException(
            "atEpoch expects an epoch equal to or greater than " + nextEpochMinValue);
      }
      nextEpochMinValue = epoch;

      final var epochEvents =
          eventsByEpochBuilder.computeIfAbsent(epoch, unused -> new ArrayList<>());
      epochEvents.add(event);
      return this;
    }

    TestScenario runUntilEpoch(long epoch) {
      if (epoch < nextEpochMinValue) {
        throw new IllegalArgumentException(
            "The test should run to at least epoch " + nextEpochMinValue);
      }
      return new TestScenario(
          numValidators,
          protocolUpdates,
          eventsByEpochBuilder.entrySet().stream()
              .collect(
                  ImmutableMap.toImmutableMap(
                      Map.Entry::getKey, e -> ImmutableList.copyOf(e.getValue()))),
          epoch);
    }
  }

  static Decimal totalStakeOfNumValidators(int numValidators) {
    return Decimal.fromU192Subunits(
        STAKE_PER_VALIDATOR.toU192Subunits().multiply(UInt192.from(numValidators)));
  }
}
