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

package com.radixdlt.integration.targeted.rev2.recovery;

import static com.radixdlt.environment.deterministic.network.MessageSelector.randomSelector;
import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.byzantineBehaviorNotDetected;
import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.ledgerTransactionSafety;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.PacemakerState;
import com.radixdlt.environment.deterministic.network.MessageMutator;
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
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.utils.UInt64;
import java.util.Random;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

/** Tests a scenario when a node is rebooted while its highest round was due to a timeout quorum. */
public final class RecoveryAfterTimeoutQuorumTest {
  private static final int NUM_VALIDATORS = 4;
  private static final Round TIMEOUT_QUORUM_ROUND = Round.of(5);
  private static final int LEADER_INDEX_AT_TIMEOUT_ROUND = 1;

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  @Test
  public void recovery_after_timeout_quorum_test() {
    final var builder =
        DeterministicTest.builder()
            .addPhysicalNodes(PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true))
            .messageSelector(randomSelector(new Random(12345)))
            .messageMutator(dropProposalForHalfNodesAtRound(TIMEOUT_QUORUM_ROUND))
            .addMonitors(byzantineBehaviorNotDetected(), ledgerTransactionSafety());

    final var functionalNodeModule =
        new FunctionalRadixNodeModule(
            NodeStorageConfig.tempFolder(folder),
            false,
            SafetyRecoveryConfig.BERKELEY_DB,
            ConsensusConfig.of(1000),
            LedgerConfig.stateComputerWithSyncRelay(
                StateComputerConfig.rev2(
                    Network.INTEGRATIONTESTNET.getId(),
                    TransactionBuilder.createGenesisWithNumValidators(
                        NUM_VALIDATORS, Decimal.of(1), UInt64.fromNonNegativeLong(10)),
                    REv2StateManagerModule.DatabaseType.ROCKS_DB,
                    StateComputerConfig.REV2ProposerConfig.transactionGenerator(
                        new REV2TransactionGenerator(), 1)),
                SyncRelayConfig.of(5000, 10, 5000L)));

    try (var test = builder.functionalNodeModule(functionalNodeModule)) {
      test.startAllNodes();

      // Run until the round that's expected to form a timeout quorum
      test.runUntilState(
          NodesPredicate.allNodesMatch(NodePredicate.bftAtOrOverRound(TIMEOUT_QUORUM_ROUND)), 1000);

      // Make sure that round was indeed due to a timeout quorum
      for (var node : test.getNodeInjectors()) {
        final var pacemakerState = node.getInstance(PacemakerState.class);
        assertEquals(
            pacemakerState.highQC().highestQC().getRound(), TIMEOUT_QUORUM_ROUND.previous());
        assertTrue(
            pacemakerState.highQC().highestTC().stream()
                .anyMatch(tc -> tc.getRound().gte(TIMEOUT_QUORUM_ROUND)));
      }

      // Lock some votes for the next round
      final var nextRound = TIMEOUT_QUORUM_ROUND.next();
      for (var i = 0; i < NUM_VALIDATORS; i++) {
        if (i != LEADER_INDEX_AT_TIMEOUT_ROUND) {
          test.runUntilState(NodesPredicate.nodeAt(i, NodePredicate.votedAtRound(nextRound)), 100);
        }
      }

      // Shutdown the leader for a timeout round, so that after restart other nodes don't receive a
      // proposal which would re-trigger QC/TC processing and advance them to the correct round
      test.shutdownNode(LEADER_INDEX_AT_TIMEOUT_ROUND);

      // Before restarting, drop any queued votes that could be received (and processed) after
      // restart (and again, re-trigger QC/TC sync up)
      test.getNetwork().dropMessages(msg -> msg.message() instanceof Vote);

      // --- at this point 3 out of 4 nodes are live (so network should be live) and the votes
      //     have been dropped one-off (again, shouldn't affect liveness) --

      // Restart the nodes (the leader for prev round remains down)
      for (var i = 0; i < NUM_VALIDATORS; i++) {
        if (i != LEADER_INDEX_AT_TIMEOUT_ROUND) {
          test.restartNode(i);
        }
      }

      // After restart, the network should be live
      for (var i = 0; i < NUM_VALIDATORS; i++) {
        if (i != LEADER_INDEX_AT_TIMEOUT_ROUND) {
          test.runUntilState(
              NodesPredicate.nodeAt(i, NodePredicate.bftAtOrOverRound(Round.of(10))), 1000);
        }
      }

      // The leader for the timeout round, once started, should catch up too
      test.startNode(LEADER_INDEX_AT_TIMEOUT_ROUND);
      test.runUntilState(
          NodesPredicate.nodeAt(
              LEADER_INDEX_AT_TIMEOUT_ROUND, NodePredicate.bftAtOrOverRound(Round.of(10))),
          1000);
    }
  }

  private static MessageMutator dropProposalForHalfNodesAtRound(Round round) {
    return (message, queue) -> {
      final var msg = message.message();
      if (msg instanceof Proposal proposal) {
        return proposal.getRound().equals(round)
            && message.channelId().receiverIndex() >= (NUM_VALIDATORS / 2);
      }
      return false;
    };
  }
}
