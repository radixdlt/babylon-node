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

import com.google.inject.*;
import com.google.inject.Module;
import com.radixdlt.consensus.LedgerHashes;
import com.radixdlt.consensus.NextEpoch;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreConfig;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.*;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.rev2.REv2StateComputer;
import com.radixdlt.rev2.REv2TransactionsAndProofReader;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.List;
import java.util.Random;
import java.util.function.Consumer;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

// spotless:off
/**
 * Test a scenario when there's a prolonged liveness break (for the purpose of this test caused by a
 * forced/faked divergent vertex execution). We verify that:
 * 1) The situation is kept under control, vertex store size-wise. I.e. that it doesn't grow uncontrollably,
 * which would ultimately result in node crash due to SBOR overflow.
 * 2) Liveness can be restored by:
 * a) fixing the underlying issue (here: reverting the mocked divergence) and
 * b) increasing the vertex store size limit (so that it can accept a few more vertices,
 *    while staying below the critical SBOR limit)
 *
 * The test consists of 3 phases:
 * - Phase 1 (rounds 0 to LIVENESS_BREAK_START_ROUND): initial liveness (to verify that the test setup is correct)
 * - Phase 2 (from LIVENESS_BREAK_START_ROUND): liveness break due to divergent vertex execution (different hashes)
 * - Phase 3 Recovery. The nodes reboot with a new configuration (slightly increased vertex store size limit)
*            and the forced divergent vertex execution in `prepare` is reverted.
 */
// spotless:on
public final class DivergentExecutionLivenessBreakTest {
  private static final Random random = new Random(123456);

  private static final int NUM_VALIDATORS = 4;

  // The test starts with 30 regular rounds, as a sanity check for our test setup
  private static final Round LIVENESS_BREAK_START_ROUND = Round.of(30);

  // Vertex store config used in the first two phases of the test: initial liveness followed by a
  // liveness break
  private static final VertexStoreConfig INITIAL_VERTEX_STORE_CONFIG =
      new VertexStoreConfig(100_000 /* 100 KiB */);
  private static final ConsensusConfig INITIAL_CONSENSUS_CONFIG =
      new ConsensusConfig(1000, 200L, 2.0, 0L, 0L, INITIAL_VERTEX_STORE_CONFIG);

  // Vertex store config used in the third phase of the test: liveness "fix" and recovery
  private static final VertexStoreConfig RECOVERY_VERTEX_STORE_CONFIG =
      new VertexStoreConfig(150_000 /* 150 KiB */);
  private static final ConsensusConfig RECOVERY_CONSENSUS_CONFIG =
      new ConsensusConfig(1000, 200L, 2.0, 0L, 0L, RECOVERY_VERTEX_STORE_CONFIG);

  // A module that overrides StateComputer in the first and second phase of a test:
  // It runs normally until LIVENESS_BREAK_START_ROUND, at which point it switches
  // its implementation to produce a liveness break by altering the resultant ledger hashes in
  // `prepare`.
  private static final Module INITIAL_OVERRIDE_MODULE =
      new AbstractModule() {
        @Provides
        @Singleton
        private StateComputerLedger.StateComputer stateComputer(
            REv2StateComputer underlyingStateComputer) {
          return new StateComputerLedger.StateComputer() {
            @Override
            public void addToMempool(MempoolAdd mempoolAdd, NodeId origin) {
              underlyingStateComputer.addToMempool(mempoolAdd, origin);
            }

            @Override
            public List<RawNotarizedTransaction> getTransactionsForProposal(
                List<StateComputerLedger.ExecutedTransaction> executedTransactions) {
              return underlyingStateComputer.getTransactionsForProposal(executedTransactions);
            }

            @Override
            public StateComputerLedger.StateComputerPrepareResult prepare(
                LedgerHashes committedLedgerHashes,
                List<ExecutedVertex> preparedUncommittedVertices,
                LedgerHashes preparedUncommittedLedgerHashes,
                List<RawNotarizedTransaction> proposedTransactions,
                RoundDetails roundDetails) {
              final var baseResult =
                  underlyingStateComputer.prepare(
                      committedLedgerHashes,
                      preparedUncommittedVertices,
                      preparedUncommittedLedgerHashes,
                      proposedTransactions,
                      roundDetails);

              if (roundDetails.roundNumber() < LIVENESS_BREAK_START_ROUND.number()) {
                return baseResult;
              } else {
                return new StateComputerLedger.StateComputerPrepareResult(
                    baseResult.getSuccessfullyExecutedTransactions(),
                    baseResult.getRejectedTransactionCount(),
                    baseResult.getNextEpoch().or((NextEpoch) null),
                    baseResult.getNextProtocolVersion().or((String) null),
                    // Random hashes to produce a liveness break
                    LedgerHashes.create(
                        HashUtils.random256(), HashUtils.random256(), HashUtils.random256()));
              }
            }

            @Override
            public LedgerProofBundle commit(
                LedgerExtension ledgerExtension, Option<byte[]> serializedVertexStoreState) {
              return underlyingStateComputer.commit(ledgerExtension, serializedVertexStoreState);
            }
          };
        }
      };

  // A module used in the third phase of the test that:
  // - reverts StateComputer override from INITIAL_OVERRIDE_MODULE (bringing back liveness)
  // - updates ConsensusConfig to RECOVERY_CONSENSUS_CONFIG (which increases vertex store size
  // limit)
  private static final Module RECOVERY_OVERRIDE_MODULE =
      new AbstractModule() {
        @Override
        public void configure() {
          install(RECOVERY_CONSENSUS_CONFIG.asModule());
        }

        @Provides
        @Singleton
        private StateComputerLedger.StateComputer stateComputer(REv2StateComputer underlying) {
          return underlying;
        }
      };

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true))
        .messageSelector(MessageSelector.randomSelector(random))
        .overrideWithIncorrectModule(INITIAL_OVERRIDE_MODULE)
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.tempFolder(folder),
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.BERKELEY_DB,
                INITIAL_CONSENSUS_CONFIG,
                FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2()
                        .withGenesis(
                            GenesisBuilder.createTestGenesisWithNumValidators(
                                NUM_VALIDATORS,
                                Decimal.ONE,
                                GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(
                                    100000)))
                        .withProposerConfig(
                            StateComputerConfig.REV2ProposerConfig.transactionGenerator(
                                new REV2TransactionGenerator(), 1)))));
  }

  @Test
  public void test_divergent_execution_liveness_break_and_recovery() {
    try (final var test = createTest()) {
      // Phase 1: Initial liveness
      test.startAllNodes();
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.atOrOverRound(LIVENESS_BREAK_START_ROUND.previous())));

      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // No divergent executions
            assertEquals(0, (int) metrics.bft().divergentVertexExecutions().getSum());
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

      // Phase 2: Liveness break
      // Run until we observe that vertex store hits its size limit
      // 100 occurrences is chosen arbitrarily, just to make sure that the issue is not transient
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.metricsPredicate(
                  metrics -> metrics.bft().vertexStore().errorsDueToSizeLimit().get() > 100)));

      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // Verify that the cause is what we expect: a divergent execution
            assertTrue(metrics.bft().divergentVertexExecutions().getSum() > 1);
            // Cross-check another metric to verify that vertex store
            // indeed holds more vertices than expected in a healthy scenario.
            assertTrue(metrics.bft().vertexStore().vertexCount().get() >= 20);
          });

      // Another verification that we're in a liveness break
      final var stateVersionA =
          test.getInstance(0, REv2TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .orElseThrow()
              .primaryProof()
              .stateVersion();
      test.runForCount(1000);
      final var stateVersionB =
          test.getInstance(0, REv2TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .orElseThrow()
              .primaryProof()
              .stateVersion();
      assertEquals(stateVersionA, stateVersionB);

      // Phase 3: Recovery
      for (var i = 0; i < test.numNodes(); i++) {
        // Restart all nodes with recovery settings (provided via RECOVERY_OVERRIDE_MODULE)
        test.restartNodeWithOverrideModule(i, RECOVERY_OVERRIDE_MODULE);
      }

      // Verify liveness: run until each node resolves at least 20 (chosen arbitrarily) non-timeout
      // quorums
      // Note: nodes start with fresh metrics post-restart
      test.runUntilState(
          NodesPredicate.allNodesMatch(
              NodePredicate.metricsPredicate(
                  metrics ->
                      metrics
                              .bft()
                              .quorumResolutions()
                              .label(new Metrics.Bft.QuorumResolution(false))
                              .get()
                          > 20)));

      verifyMetricsOnAllNodes(
          test,
          metrics -> {
            // Assert that vertex store is well below its max size
            assertTrue(
                (int) metrics.bft().vertexStore().byteSize().get()
                    < INITIAL_VERTEX_STORE_CONFIG.maxSerializedSizeBytes() / 2);
            // No more errors due to size limit
            assertEquals(0, (int) metrics.bft().vertexStore().errorsDueToSizeLimit().get());
            // There are no more than 3 vertices (as expected in a healthy network)
            assertTrue(metrics.bft().vertexStore().vertexCount().get() <= 3);
            // No timeouts
            assertEquals(
                0,
                (int)
                    metrics
                        .bft()
                        .quorumResolutions()
                        .label(new Metrics.Bft.QuorumResolution(true))
                        .get());
          });
    }
  }

  private void verifyMetricsOnAllNodes(DeterministicTest test, Consumer<Metrics> testFn) {
    for (final var i : test.getNodeInjectors()) {
      testFn.accept(i.getInstance(Metrics.class));
    }
  }
}
