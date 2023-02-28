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

package com.radixdlt.integration.targeted.consensus;

import static com.radixdlt.harness.predicates.NodesPredicate.anyAtExactlyStateVersion;
import static org.junit.Assert.assertEquals;

import com.google.common.collect.ImmutableList;
import com.radixdlt.consensus.EpochNodeWeightMapping;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import org.junit.Test;

// spotless:off
/**
 * Test a scenario where a timeout cert is formed first, but its
 * processing is delayed, then a QC is formed
 * which results in a QC output (RoundQuorumReached event with a QC).
 *
 * The test works by dropping a proposal to 3 out of 4 nodes,
 * which results in:
 * a) 1 node receives a proposal, sends a regular vote and then a timeout vote
 * b) 3 nodes don't receive a proposal and they send a timeout vote for a fallback vertex
 *
 * If the order of received votes is mixed, and it happens to be when
 * using the "first" message selector (it's deterministic, but just a coincidence),
 * then the node is able to create a TC (by using the timeout signatures) before
 * it can create a QC (on a fallback vertex, voted on by 3 nodes).
 *
 * Example order of received votes:
 *  1) vote for fallback vertex + timeout
 *  2) vote for proposal vertex + timeout
 *  3) vote for fallback vertex + timeout
 *  4) vote for fallback vertex + timeout
 *
 *  Note that if the votes for fallback vertices were received first,
 *  then the node could form a QC straight away and the test wouldn't
 *  actually test what it's supposed to. So there's another assertion
 *  that makes sure that the nodes have actually created (and ignored) a TC.
 */
// spotless:on
public final class DelayAfterFormingTcAndFormAQcTest {
  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBasicBatch(4))
        .messageSelector(MessageSelector.firstSelector())
        .messageMutators(dropProposalToNodes(Round.of(3), ImmutableList.of(0, 1, 2)))
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.mocked(),
                FunctionalRadixNodeModule.ConsensusConfig.of(),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerMockedSync(
                    StateComputerConfig.mockedWithEpochs(
                        Round.of(10000),
                        EpochNodeWeightMapping.constant(4),
                        new StateComputerConfig.MockedMempoolConfig.NoMempool()))));
  }

  @Test
  public void delay_tc_processing_and_form_a_qc_test() {
    try (var test = createTest()) {
      test.startAllNodes();
      test.runUntilState(anyAtExactlyStateVersion(10));

      for (var injector : test.getNodeInjectors()) {
        final var metrics = injector.getInstance(Metrics.class);
        // Each node should have created and postponed a timeout cert...
        assertEquals(1, (int) metrics.bft().postponedRoundQuorums().get());
        // ...and then form a QC, resulting in no TCs actually processed.
        assertEquals(0, (int) metrics.bft().timeoutQuorums().get());
      }
    }
  }

  private static MessageMutator dropProposalToNodes(Round round, ImmutableList<Integer> nodes) {
    return (message, queue) -> {
      final var msg = message.message();
      if (msg instanceof final Proposal proposal) {
        return proposal.getRound().equals(round)
            && nodes.contains(message.channelId().receiverIndex());
      }
      return false;
    };
  }
}
