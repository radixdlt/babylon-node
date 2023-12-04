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

package com.radixdlt.integration.steady_state.deterministic.consensus;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Singleton;
import com.radixdlt.consensus.LedgerHashes;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.ledger.LedgerExtension;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.RoundDetails;
import com.radixdlt.ledger.StateComputerLedger;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.statecomputer.StatelessComputer;
import com.radixdlt.statecomputer.StatelessTransactionVerifier;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.KeyComparator;
import java.util.List;
import java.util.Random;
import java.util.function.Consumer;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

/**
 * Tests that a liveness break caused by divergent vertex execution (timeout quorums can still be
 * formed) doesn't cause the vertex store to grow uncontrollably.
 */
public final class DivergentExecutionLivenessBreakTest {
  private static final int NUM_NODES = 4;
  private static final int VERTEX_STORE_MAX_SIZE = 50;

  private static final long FIRST_DIVERGENT_ROUND = 20;
  private static final long LAST_DIVERGENT_ROUND = 200;

  private static final Random random = new Random(123456);

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(
            PhysicalNodeConfig.createSortedBatchWithFakeAddresses(
                NUM_NODES, KeyComparator.instance()))
        .messageSelector(MessageSelector.randomSelector(random))
        .overrideWithIncorrectModule(
            new AbstractModule() {
              @Provides
              @Singleton
              private StateComputerLedger.StateComputer stateComputer(
                  StatelessTransactionVerifier verifier,
                  EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
                  Hasher hasher) {
                return new CustomStateComputer(verifier, ledgerUpdateDispatcher, hasher);
              }
            })
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.none(),
                false,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                new ConsensusConfig(1000, 100L, 2.0, 0L, 0L, VERTEX_STORE_MAX_SIZE),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.mockedNoEpochs(
                        NUM_NODES, new StateComputerConfig.MockedMempoolConfig.NoMempool()))));
  }

  @Test
  public void test_divergent_execution_liveness_break_and_recovery() {
    try (final var test = createTest()) {
      test.startAllNodes();

      // First, let's run a few regular rounds, just to verify that our test
      // setup is correct.
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.bftAtOrOverRound(Round.of(FIRST_DIVERGENT_ROUND - 1))));
      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // No divergent resolutions
            assertEquals(0, (int) metrics.bft().divergentVertexExecutions().get());
            // No timeouts quorums
            assertEquals(
                0,
                (int)
                    metrics
                        .bft()
                        .quorumResolutions()
                        .label(new Metrics.Bft.QuorumResolution(true))
                        .get());
          });

      // Now let's run through the divergent execution period
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.bftAtOrOverRound(Round.of(LAST_DIVERGENT_ROUND))));
      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // Vertex store should grow to its max size
            assertEquals(VERTEX_STORE_MAX_SIZE, (int) metrics.bft().vertexStore().size().get());
            // We're getting some divergent executions
            // Just a sanity check that the test does what it's supposed to do
            assertTrue(metrics.bft().divergentVertexExecutions().get() >= 10);
            // Check expected number of timeout quorum rounds
            assertEquals(
                LAST_DIVERGENT_ROUND - FIRST_DIVERGENT_ROUND + 1,
                (int)
                    metrics
                        .bft()
                        .quorumResolutions()
                        .label(new Metrics.Bft.QuorumResolution(true))
                        .get());
          });

      // Run 100 more rounds
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.bftAtOrOverRound(Round.of(LAST_DIVERGENT_ROUND + 100))));
      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // No more timeout quorums (same value as before)
            assertEquals(
                LAST_DIVERGENT_ROUND - FIRST_DIVERGENT_ROUND + 1,
                (int)
                    metrics
                        .bft()
                        .quorumResolutions()
                        .label(new Metrics.Bft.QuorumResolution(true))
                        .get());
            // Vertex store is back to a healthy <= 3 vertices
            assertTrue(metrics.bft().vertexStore().size().get() <= 3);
          });
    }
  }

  private void verifyMetricsOnAllNodes(DeterministicTest test, Consumer<Metrics> testFn) {
    for (final var i : test.getNodeInjectors()) {
      testFn.accept(i.getInstance(Metrics.class));
    }
  }

  // spotless:off

    /**
     * A test StateComputer that uses two underlying StatelessComputer implementations:
     * - one that produces the same ledger hashes (zero):
     *  for rounds 0 - FIRST_DIVERGENT_ROUND-1 and LAST_DIVERGENT_ROUND+1 - ∞
     * - another one that produces different ledger hashes (random):
     *  for rounds FIRST_DIVERGENT_ROUND - LAST_DIVERGENT_ROUND
     */
    // spotless:on
  private static final class CustomStateComputer implements StateComputerLedger.StateComputer {
    private final StateComputerLedger.StateComputer underlyingDivergent;
    private final StateComputerLedger.StateComputer underlyingConvergent;

    public CustomStateComputer(
        StatelessTransactionVerifier verifier,
        EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
        Hasher hasher) {
      this.underlyingDivergent =
          new StatelessComputer(
              verifier,
              ledgerUpdateDispatcher,
              hasher,
              LedgerHashes.create(
                  HashUtils.random256(), HashUtils.random256(), HashUtils.random256()));
      this.underlyingConvergent =
          new StatelessComputer(verifier, ledgerUpdateDispatcher, hasher, LedgerHashes.zero());
    }

    @Override
    public void addToMempool(MempoolAdd mempoolAdd, NodeId origin) {}

    @Override
    public List<RawNotarizedTransaction> getTransactionsForProposal(
        List<StateComputerLedger.ExecutedTransaction> executedTransactions) {
      return List.of();
    }

    @Override
    public StateComputerLedger.StateComputerResult prepare(
        LedgerHashes committedLedgerHashes,
        List<ExecutedVertex> preparedUncommittedVertices,
        LedgerHashes preparedUncommittedLedgerHashes,
        List<RawNotarizedTransaction> proposedTransactions,
        RoundDetails roundDetails) {
      final StateComputerLedger.StateComputer underlyingToUse;
      if (roundDetails.roundNumber() < FIRST_DIVERGENT_ROUND
          || roundDetails.roundNumber() > LAST_DIVERGENT_ROUND) {
        underlyingToUse = this.underlyingConvergent;
      } else {
        underlyingToUse = this.underlyingDivergent;
      }
      return underlyingToUse.prepare(
          committedLedgerHashes,
          preparedUncommittedVertices,
          preparedUncommittedLedgerHashes,
          proposedTransactions,
          roundDetails);
    }

    @Override
    public void commit(LedgerExtension ledgerExtension, VertexStoreState vertexStore) {
      // Doesn't matter which one we use here
      this.underlyingConvergent.commit(ledgerExtension, vertexStore);
    }
  }
}
