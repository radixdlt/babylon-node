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

package com.radixdlt.rev2;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;

import com.radixdlt.environment.DatabaseConfig;
import com.radixdlt.environment.LedgerProofsGcConfig;
import com.radixdlt.environment.StateHashTreeGcConfig;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.modules.StateComputerConfig.REV2ProposerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Predicate;
import org.awaitility.Awaitility;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class StateHashTreeGcTest {

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(
      long stateVersionHistoryLength, boolean storeHistoricalSubstates) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
        .messageSelector(firstSelector())
        .messageMutator(MessageMutator.dropTimeouts())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.tempFolder(folder),
                true,
                SafetyRecoveryConfig.MOCKED,
                ConsensusConfig.of(1000),
                LedgerConfig.stateComputerNoSync(
                    new StateComputerConfig.REv2StateComputerConfig(
                        Network.INTEGRATIONTESTNET.getId(),
                        GenesisBuilder.createTestGenesisWithNumValidators(
                            1,
                            Decimal.ONE,
                            GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(100)),
                        new DatabaseConfig(false, false, storeHistoricalSubstates, false),
                        REV2ProposerConfig.noUserTransactions(),
                        false,
                        new StateHashTreeGcConfig(
                            UInt32.fromNonNegativeInt(1),
                            UInt64.fromNonNegativeLong(stateVersionHistoryLength)),
                        LedgerProofsGcConfig.forTesting(),
                        LedgerSyncLimitsConfig.defaults(),
                        ProtocolConfig.testingDefault(),
                        false))));
  }

  @Test
  public void node_keeps_exactly_the_configured_number_of_stale_state_hash_tree_versions() {
    // Arrange: configure 37 historical state versions to be kept in the state hash tree
    try (var test = createTest(37, false)) {
      test.startAllNodes();

      // Act: Advance at least that many versions, so we are sure about the `current - leastStale`
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atExactlyStateVersion(43)), 10000);

      // Assert: Wait until an async GC executes and deletes old state hash tree versions (until 43
      // - 37 = 6):
      Awaitility.await()
          .until(
              test.getInstance(0, TestStateReader.class)::getLeastStaleStateHashTreeVersion,
              Predicate.isEqual(6L));
    }
  }

  @Test
  public void node_keeps_historical_substate_values_only_for_configured_number_of_versions() {
    // Configure a small number of historical versions to be kept together with values
    try (var test = createTest(5, true)) {
      test.startAllNodes();
      final var testReader = test.getInstance(0, TestStateReader.class);

      // Keep progressing the state versions, and assert that at some point the count of history
      // table rows will *decrease*, which proves the existence of some GC process:
      final var previousCount = new AtomicLong();
      test.runUntilState(
          injector -> {
            final var currentCount = testReader.getHistoricalSubstateCount();
            return currentCount < previousCount.getAndSet(currentCount);
          },
          10000);
    }
  }
}
