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
import static org.junit.Assert.assertTrue;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.environment.Runners;
import com.radixdlt.harness.simulation.NetworkLatencies;
import com.radixdlt.harness.simulation.NetworkOrdering;
import com.radixdlt.harness.simulation.SimulationTest;
import com.radixdlt.harness.simulation.monitors.consensus.ConsensusMonitors;
import com.radixdlt.harness.simulation.monitors.ledger.LedgerMonitors;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.RejectedConsensusEvent;
import com.radixdlt.monitoring.Metrics.RejectedConsensusEvent.TimestampIssue;
import com.radixdlt.monitoring.Metrics.RejectedConsensusEvent.Type;
import com.radixdlt.utils.TimeSupplier;
import java.time.Duration;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.IntStream;
import org.junit.Test;

/**
 * This tests a specific scenario where there are 7 nodes, a leader is down, and the previous leader
 * has an inaccurate clock. Quorum should still manage to time out both the proposals and make
 * progress, and we should avoid a liveness break. In particular, this tests that timeout votes can
 * be sent eventually to all nodes even if the next leader is down.
 */
public final class ProposerTimestampInaccurateClockAndLeaderDownTest {
  private static final int NUM_VALIDATORS = 7;

  // Two consecutive leaders (verified below with an assertion)
  private static final int NODE_WITH_INACCURATE_CLOCK_INDEX = 6;
  private static final int DOWN_NODE_INDEX = 0;

  @Test
  public void test_liveness_when_one_node_has_inaccurate_clock_and_next_leader_is_down() {
    final var builder =
        SimulationTest.builder()
            .networkModules(NetworkOrdering.inOrder(), NetworkLatencies.fixed())
            .numNodes(NUM_VALIDATORS)
            .addTestModules(
                ConsensusMonitors.safety(),
                ConsensusMonitors.liveness(5, TimeUnit.SECONDS),
                LedgerMonitors.consensusToLedger(),
                ConsensusMonitors.proposerTimestampChecker(),
                LedgerMonitors.ordered())
            .ledgerAndEpochs(
                ConsensusConfig.of(1000), Round.of(10), e -> IntStream.range(0, NUM_VALIDATORS));

    // One node has an inaccurate clock: 10s rushing
    // A little "hack" with AtomicReference to get the lucky node's key out of the closure
    final var rushingNode = new AtomicReference<BFTNode>();
    builder.addOverrideModuleToInitialNodes(
        nodes -> {
          final var nodeWithInaccurateClock =
              BFTNode.create(nodes.get(NODE_WITH_INACCURATE_CLOCK_INDEX).getPublicKey());
          rushingNode.set(nodeWithInaccurateClock);
          return ImmutableList.of(nodeWithInaccurateClock.getKey());
        },
        nodes ->
            new AbstractModule() {
              @Provides
              public TimeSupplier timeSupplier() {
                return () -> System.currentTimeMillis() + 10000;
              }
            });

    final var simulationTest = builder.build();

    // Simulating a down node by disabling the consensus module runner
    final var downNodeKey = simulationTest.getInitialNodes().get(DOWN_NODE_INDEX);
    final var downNode = BFTNode.create(downNodeKey.getPublicKey());
    final var runningTest =
        simulationTest.run(
            Duration.ofSeconds(15), ImmutableMap.of(downNode, ImmutableSet.of(Runners.CONSENSUS)));

    // Making sure that the altered nodes are in fact consecutive leaders
    final var proposerElection =
        runningTest.getNodeInjectors().get(0).getInstance(ProposerElection.class);
    final var firstLeader = proposerElection.getProposer(Round.of(1L));
    final var nextLeader = proposerElection.getProposer(Round.of(2L));
    assertEquals(firstLeader, rushingNode.get());
    assertEquals(nextLeader, downNode);

    final var results = runningTest.awaitCompletion();

    assertThat(results).allSatisfy((name, err) -> assertThat(err).isEmpty());

    final var network = runningTest.getNetwork();
    for (final var node : network.getNodes()) {
      final var counters = network.getInstance(Metrics.class, node);
      if (node.equals(rushingNode.get())) {
        // There are some invalid timestamp proposals reported (delayed from this node's point of
        // view)
        assertTrue(
            counters
                    .bft()
                    .rejectedConsensusEvents()
                    .label(new RejectedConsensusEvent(Type.PROPOSAL, TimestampIssue.TOO_FAR_PAST))
                    .get()
                >= 1);
      } else if (node.equals(downNode)) {
        // The down node shouldn't process any consensus events
        assertEquals(0, (long) counters.bft().successfullyProcessedVotes().get());
        assertEquals(0, (long) counters.bft().successfullyProcessedProposals().get());
      } else {
        // A healthy node:
        // There are some invalid timestamp proposals reported
        assertTrue(
            counters
                    .bft()
                    .rejectedConsensusEvents()
                    .label(new RejectedConsensusEvent(Type.PROPOSAL, TimestampIssue.TOO_FAR_FUTURE))
                    .get()
                >= 1);
        // And some timed out rounds
        assertTrue(counters.bft().pacemaker().timedOutRounds().get() >= 1);
        // And all timed out rounds only required a single timeout event to proceed
        assertEquals(
            (long) counters.bft().pacemaker().timedOutRounds().get(),
            (long) counters.bft().pacemaker().timeoutsSent().get());
      }
    }
  }
}
