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

package com.radixdlt.integration.steady_state.deterministic.rev2.consensus_ledger_sync;

import static com.radixdlt.environment.deterministic.network.MessageSelector.randomSelector;

import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.invariants.Checkers;
import com.radixdlt.harness.invariants.LedgerLivenessChecker;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.sync.SyncRelayConfig;
import java.util.*;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

@RunWith(Parameterized.class)
public final class MultiNodeRebootTest {
  @Parameterized.Parameters
  public static Collection<Object[]> numNodes() {
    return List.of(new Object[][] {{4}, {10}});
  }

  private static final Logger logger = LogManager.getLogger();

  private final Random random = new Random(12345);
  private final int numValidators;

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  public MultiNodeRebootTest(int numValidators) {
    this.numValidators = numValidators;
  }

  private DeterministicTest createTest() {
    var databaseConfig = REv2DatabaseConfig.rocksDB(folder.getRoot().getAbsolutePath());
    return DeterministicTest.builder()
        .numNodes(numValidators, 0)
        .messageSelector(randomSelector(random))
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                false,
                FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                    StateComputerConfig.rev2(
                        Network.INTEGRATIONTESTNET.getId(),
                        databaseConfig,
                        StateComputerConfig.REV2ProposerConfig.transactionGenerator(
                            new REV2TransactionGenerator(), 1)),
                    SyncRelayConfig.of(5000, 10, 5000L))));
  }

  private void checkSafetyAndLiveness(
      DeterministicTest test,
      Map<Integer, Boolean> nodeLiveStatus,
      LedgerLivenessChecker livenessChecker)
      throws Exception {
    // Bring up down nodes temporarily for checking
    for (var e : nodeLiveStatus.entrySet()) {
      if (!e.getValue()) {
        test.startNode(e.getKey());
      }
    }

    Checkers.assertLedgerTransactionsSafety(test.getNodeInjectors());
    livenessChecker.progressCheck(test.getNodeInjectors());

    // Shutdown nodes which were initially down
    for (var e : nodeLiveStatus.entrySet()) {
      if (!e.getValue()) {
        test.shutdownNode(e.getKey());
      }
    }
  }

  private void runTest(int numRounds, int numDownValidators) throws Exception {
    try (var test = createTest()) {
      test.startAllNodes();

      var nodeLiveStatus =
          IntStream.range(0, numValidators)
              .boxed()
              .collect(Collectors.toMap(i -> i, i -> i >= numDownValidators));

      // Shutdown a subset of validators for the remainder of the test
      for (var e : nodeLiveStatus.entrySet()) {
        if (!e.getValue()) {
          test.shutdownNode(e.getKey());
        }
      }

      var livenessChecker = new LedgerLivenessChecker();

      for (int testRound = 0; testRound < numRounds; testRound++) {
        if (test.getNodeInjectors().stream().anyMatch(Objects::nonNull)) {
          test.runForCount(random.nextInt(numValidators * 50, numValidators * 100));
        }

        if (testRound % 100 == 0) {
          checkSafetyAndLiveness(test, nodeLiveStatus, livenessChecker);
        }

        // Reverse the status (by shutting down or starting up a node) of a random number of
        // validators
        var nodes =
            IntStream.range(numDownValidators, numValidators).boxed().collect(Collectors.toList());
        Collections.shuffle(nodes);
        var numNodesToUpdate = random.nextInt(nodes.size());
        var nodesToUpdate = nodes.subList(0, numNodesToUpdate);
        for (var nodeIndex : nodesToUpdate) {
          var isLive = nodeLiveStatus.get(nodeIndex);
          if (isLive) {
            test.shutdownNode(nodeIndex);
            nodeLiveStatus.put(nodeIndex, false);
          } else {
            test.startNode(nodeIndex);
            nodeLiveStatus.put(nodeIndex, true);
          }
        }
        logger.info(
            "Test_round={}/{}, updated_nodes={}, nodes={}",
            testRound + 1,
            numRounds,
            nodesToUpdate,
            nodeLiveStatus);
      }

      // Post-run assertions
      for (int i = numDownValidators; i < numValidators; i++) {
        test.startNode(i);
      }
      var aliveNodes = test.getNodeInjectors().stream().skip(numDownValidators).toList();
      Checkers.assertLedgerTransactionsSafety(aliveNodes);
    }
  }

  @Test
  public void restart_all_nodes_intermittently() throws Exception {
    var numRounds = 2000 / numValidators;
    var numDownValidators = 0;
    runTest(numRounds, numDownValidators);
  }

  @Test
  public void restart_all_nodes_intermittently_while_f_nodes_down() throws Exception {
    var numRounds = 2000 / numValidators;
    var numDownValidators = (numValidators - 1) / 3;
    runTest(numRounds, numDownValidators);
  }

  // TODO: Architect the test checker in a better way
  @Test(expected = AssertionError.class)
  public void restart_all_nodes_intermittently_while_f_plus_one_nodes_down_should_fail_test()
      throws Exception {
    var numRounds = 2000 / numValidators;
    var numDownValidators = (numValidators - 1) / 3 + 1;
    runTest(numRounds, numDownValidators);
  }
}
