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
import static org.assertj.core.api.Assertions.assertThat;

import com.radixdlt.environment.DatabaseFlags;
import com.radixdlt.environment.LedgerProofsGcConfig;
import com.radixdlt.environment.StateHashTreeGcConfig;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.harness.simulation.application.TransactionGenerator;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.modules.StateComputerConfig.REV2ProposerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.awaitility.Awaitility;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class LedgerProofsGcTest {

  private static final int GC_INTERVAL_SEC = 1;

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  /**
   * A common test set-up. The following hardcoded properties are important to the test asserts:
   *
   * <ul>
   *   <li>every round contains 2 transactions: 1 from the generator, and a round change;
   *   <li>node 0 is a validator, node 1 is a full node.
   * </ul>
   */
  private DeterministicTest createTest(
      long mostRecentFullResolutionEpochCount,
      long roundsPerEpoch,
      int txnSize,
      int maxTxnCountUnderProof,
      int maxTxnPayloadSizeUnderProof) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(2, true))
        .messageSelector(firstSelector())
        .messageMutator(MessageMutator.dropTimeouts())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.tempFolder(folder),
                true,
                SafetyRecoveryConfig.MOCKED,
                ConsensusConfig.of(1000),
                LedgerConfig.stateComputerWithSyncRelay(
                    new StateComputerConfig.REv2StateComputerConfig(
                        Network.INTEGRATIONTESTNET.getId(),
                        GenesisBuilder.createTestGenesisWithNumValidators(
                            1,
                            Decimal.ONE,
                            GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(
                                roundsPerEpoch)),
                        new DatabaseFlags(false, false),
                        REV2ProposerConfig.transactionGenerator(
                            new SizedTransactionGenerator(NetworkDefinition.INT_TEST_NET, txnSize),
                            1),
                        false,
                        StateHashTreeGcConfig.forTesting(),
                        new LedgerProofsGcConfig(
                            UInt32.fromNonNegativeInt(GC_INTERVAL_SEC),
                            UInt64.fromNonNegativeLong(mostRecentFullResolutionEpochCount)),
                        new LedgerSyncLimitsConfig(
                            UInt32.fromNonNegativeInt(maxTxnCountUnderProof),
                            UInt32.fromNonNegativeInt(maxTxnPayloadSizeUnderProof)),
                        ProtocolConfig.testingDefault(),
                        false),
                    SyncRelayConfig.of(100, 2, 200L))));
  }

  @Test
  public void node_retains_enough_proofs_to_cover_max_transaction_size_in_old_enough_epochs() {
    /// 6 rounds * 1 KB = 6KB
    var roundsPerEpoch = 6;
    var txnSize = 1024;
    /// Limit is 4 KB - so we require 2 proofs to cover 6 KB
    var maxTxnPayloadSizeUnderProof = 4 * 1024; // the genesis is larger than that, but gets skipped
    try (var test = createTest(2, roundsPerEpoch, txnSize, 1000, maxTxnPayloadSizeUnderProof)) {
      test.startNode(0);

      // Act: advance to epoch 5
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atOrOverEpoch(5)));

      // Assert: after an async GC, we expect certain number of proofs in each epoch:
      Awaitility.await()
          .atMost(2 * GC_INTERVAL_SEC, TimeUnit.SECONDS)
          .untilAsserted(
              () -> {
                var stateReader = test.getInstance(0, TestStateReader.class);
                // - The epoch 5 has just started, hence it has no proofs
                assertThat(stateReader.countProofsWithinEpoch(5)).isEqualTo(0);
                // - The 2 most recent completed epochs contain all their proofs (i.e. not pruned)
                assertThat(stateReader.countProofsWithinEpoch(4)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(3)).isEqualTo(roundsPerEpoch);
                // - This epoch was pruned, and it has 2 proofs (to fit the size limit)
                assertThat(stateReader.countProofsWithinEpoch(2)).isEqualTo(2);
              });

      // Follow-up: Advance one more epoch
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atOrOverEpoch(6)));

      // Assert: after an async GC...
      Awaitility.await()
          .atMost(2 * GC_INTERVAL_SEC, TimeUnit.SECONDS)
          .untilAsserted(
              () -> {
                // ... the "pruning window" has progressed (i.e. now epoch 3 got pruned)
                var stateReader = test.getInstance(0, TestStateReader.class);
                assertThat(stateReader.countProofsWithinEpoch(6)).isEqualTo(0);
                assertThat(stateReader.countProofsWithinEpoch(5)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(4)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(3)).isEqualTo(2);
                assertThat(stateReader.countProofsWithinEpoch(2)).isEqualTo(2);
              });
    }
  }

  @Test
  public void node_retains_enough_proofs_to_cover_max_transaction_count_in_old_enough_epochs() {
    // 37 rounds, each contains a generated transaction + a round change = 74 transactions
    var roundsPerEpoch = 37;
    var maxTxnCountUnderProof = 20; // we will require 4 proofs
    try (var test = createTest(2, roundsPerEpoch, 1024, maxTxnCountUnderProof, 16 * 1024 * 1024)) {
      test.startNode(0);

      // Act: advance to epoch 5
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atOrOverEpoch(5)));

      // Act: after an async GC, we expect certain number of proofs in each epoch:
      Awaitility.await()
          .atMost(2 * GC_INTERVAL_SEC, TimeUnit.SECONDS)
          .untilAsserted(
              () -> {
                var stateReader = test.getInstance(0, TestStateReader.class);
                // - The epoch 5 has just started, hence it has no proofs
                assertThat(stateReader.countProofsWithinEpoch(5)).isEqualTo(0);
                // - The 2 most recent completed epochs contain all their proofs (i.e. not pruned)
                assertThat(stateReader.countProofsWithinEpoch(4)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(3)).isEqualTo(roundsPerEpoch);
                // - This epoch was pruned, and it has 4 proofs (to fit the count limit)
                assertThat(stateReader.countProofsWithinEpoch(2)).isEqualTo(4);
              });

      // Follow-up: Advance one more epoch
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atOrOverEpoch(6)));

      // Assert: after an async GC...
      Awaitility.await()
          .atMost(2 * GC_INTERVAL_SEC, TimeUnit.SECONDS)
          .untilAsserted(
              () -> {
                // ... the "pruning window" has progressed (i.e. now epoch 3 got pruned)
                var stateReader = test.getInstance(0, TestStateReader.class);
                assertThat(stateReader.countProofsWithinEpoch(6)).isEqualTo(0);
                assertThat(stateReader.countProofsWithinEpoch(5)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(4)).isEqualTo(roundsPerEpoch);
                assertThat(stateReader.countProofsWithinEpoch(3)).isEqualTo(4);
                assertThat(stateReader.countProofsWithinEpoch(2)).isEqualTo(4);
              });
    }
  }

  @Test
  public void new_full_node_can_fully_sync_from_node_which_executed_gc() {
    var roundsPerEpoch = 13;
    var maxTxnCountUnderProof = 10; // the set-up is count-bounded (and requires 3 proofs per epoch)
    try (var test = createTest(0, roundsPerEpoch, 256, maxTxnCountUnderProof, 16 * 1024 * 1024)) {
      test.startNode(0);

      // Act: advance to epoch 9
      test.runUntilState(NodesPredicate.nodeAt(0, NodePredicate.atOrOverEpoch(9)));

      // Act: wait for (potentially >1) GC, expecting only the critical proofs in epoch 8:
      Awaitility.await()
          .atMost(3 * GC_INTERVAL_SEC, TimeUnit.SECONDS)
          .untilAsserted(
              () -> {
                var stateReader = test.getInstance(0, TestStateReader.class);
                assertThat(stateReader.countProofsWithinEpoch(8)).isEqualTo(3);
              });

      // Act: spin-up a full-node
      test.startNode(1);

      // Assert: it finally ledger-syncs up to epoch 9 too:
      test.runUntilState(NodesPredicate.nodeAt(1, NodePredicate.atOrOverEpoch(9)));
    }
  }

  private static final class SizedTransactionGenerator
      implements TransactionGenerator<RawNotarizedTransaction> {

    private final NetworkDefinition networkDefinition;
    private final int size;

    public SizedTransactionGenerator(NetworkDefinition networkDefinition, int size) {
      this.networkDefinition = networkDefinition;
      this.size = size;
    }

    @Override
    public RawNotarizedTransaction nextTransaction() {
      return TransactionBuilder.forNetwork(networkDefinition)
          .manifest(Manifest.valid())
          .blobs(List.of(new byte[size - 240])) // account for headers, etc.
          .prepare()
          .raw();
    }
  }
}
