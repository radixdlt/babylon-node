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

package com.radixdlt.integration.steady_state.deterministic.rev2.consensus_ledger;

import static com.radixdlt.environment.deterministic.network.MessageSelector.randomSelector;
import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.*;

import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.NodesReader;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.invariants.Checkers;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Random;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

/**
 * Randomly reboots nodes immediately (no dropped messages except local messages) and verifies that
 * ledger safety and liveness is not broken.
 */
@RunWith(Parameterized.class)
public final class MultiNodeRecoveryTest {
  @Parameterized.Parameters
  public static Collection<Object[]> parameters() {
    return List.of(
        new Object[][] {
          {false, 100000},
          {true, 10},
          {true, 100},
        });
  }

  private static final Logger logger = LogManager.getLogger();
  private static final int NUM_VALIDATORS = 4;

  private final Random random = new Random(12345);
  @Rule public TemporaryFolder folder = new TemporaryFolder();
  private final boolean epochs;
  private final long roundsPerEpoch;

  public MultiNodeRecoveryTest(boolean epochs, long roundsPerEpoch) {
    this.epochs = epochs;
    this.roundsPerEpoch = roundsPerEpoch;
  }

  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true))
        .messageSelector(randomSelector(random))
        .addMonitors(byzantineBehaviorNotDetected(), ledgerTransactionSafety())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.tempFolder(folder),
                this.epochs,
                SafetyRecoveryConfig.BERKELEY_DB,
                ConsensusConfig.of(1000),
                LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2(
                        Network.INTEGRATIONTESTNET.getId(),
                        GenesisBuilder.createTestGenesisWithNumValidators(
                            NUM_VALIDATORS,
                            Decimal.ONE,
                            GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(
                                this.roundsPerEpoch)),
                        REv2StateManagerModule.DatabaseType.ROCKS_DB,
                        StateComputerConfig.REV2ProposerConfig.transactionGenerator(
                            new REV2TransactionGenerator(), 1)))));
  }

  @Test
  public void rebooting_nodes_with_persistent_store_should_not_cause_safety_or_liveness_issues() {
    try (var test = createTest()) {
      test.startAllNodes();

      for (int i = 0; i < 50; i++) {
        long stateVersion = NodesReader.getHighestStateVersion(test.getNodeInjectors());
        test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(stateVersion + 5), 1000000);

        // Reboot some count of random nodes
        var validatorIndices =
            IntStream.range(0, NUM_VALIDATORS).boxed().collect(Collectors.toList());
        Collections.shuffle(validatorIndices, random);
        var nodesToReboot = validatorIndices.subList(0, random.nextInt(NUM_VALIDATORS));
        logger.info("Rebooting round {} nodes: {}", i, nodesToReboot);
        for (var index : nodesToReboot) {
          test.restartNode(index);
        }
      }

      // Post-run assertions
      Checkers.assertNodesSyncedToVersionAtleast(test.getNodeInjectors(), 100);
    }
  }
}
