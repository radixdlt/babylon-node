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

import static org.assertj.core.api.Assertions.assertThat;

import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.modules.StateComputerConfig.MockedMempoolConfig;
import com.radixdlt.monitoring.Metrics;
import java.util.Random;
import org.junit.Test;

public class OneProposalTimeoutResponsiveTest {
  private final Random random = new Random(123456);

  private void run(int numValidatorNodes, int numRounds, long dropPeriod) {
    var test =
        DeterministicTest.builder()
            .addPhysicalNodes(PhysicalNodeConfig.createBasicBatch(numValidatorNodes))
            .messageSelector(MessageSelector.randomSelector(random))
            .messageMutator(dropSomeProposals(dropPeriod))
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    NodeStorageConfig.none(),
                    false,
                    SafetyRecoveryConfig.MOCKED,
                    ConsensusConfig.of(200L, 0L),
                    LedgerConfig.stateComputerNoSync(
                        StateComputerConfig.mockedNoEpochs(
                            numValidatorNodes, MockedMempoolConfig.noMempool()))));

    test.startAllNodes();
    test.runUntilMessage(
        DeterministicTest.hasReachedRound(Round.of(numRounds)),
        numRounds * numValidatorNodes * numValidatorNodes * 10);

    long maxIndirectParents =
        numValidatorNodes <= 3
            ? 0 // there are no indirect parents for 3 nodes (QC is always formed)
            : (numRounds - 1) / dropPeriod; // Edge case if dropPeriod a factor of numRounds

    long requiredTimeouts = numRounds / dropPeriod * 2;

    long expectedTimeoutQuorumsOrQCsOnFallbackVertices = requiredTimeouts / 2;

    for (int nodeIndex = 0; nodeIndex < numValidatorNodes; ++nodeIndex) {
      Metrics metrics = test.getInstance(nodeIndex, Metrics.class);
      long numberOfIndirectParents = (long) metrics.bft().vertexStore().indirectParents().get();
      long totalNumberOfTimeouts = (long) metrics.bft().pacemaker().timeoutsSent().get();
      long totalNumberOfTimeoutQuorums =
          (long)
              metrics.bft().quorumResolutions().label(new Metrics.Bft.QuorumResolution(true)).get();
      long numOfCommittedFallbackVertices =
          (long)
              metrics.bft().committedVertices().label(new Metrics.Bft.CommittedVertex(true)).get();
      assertThat(totalNumberOfTimeouts).isEqualTo(requiredTimeouts);

      // We expect the correct num of either QCs on fallback (timeout) vertices or TCs
      final var timeoutQuorumsAndQCsOnFallbackVertices =
          numOfCommittedFallbackVertices + totalNumberOfTimeoutQuorums;
      assertThat(timeoutQuorumsAndQCsOnFallbackVertices)
          .isBetween(
              expectedTimeoutQuorumsOrQCsOnFallbackVertices - 1,
              expectedTimeoutQuorumsOrQCsOnFallbackVertices);
      assertThat(numberOfIndirectParents).isLessThanOrEqualTo(maxIndirectParents);
    }
  }

  private static MessageMutator dropSomeProposals(long dropPeriod) {
    return (message, queue) -> {
      Object msg = message.message();
      if (msg instanceof Proposal) {
        final Proposal proposal = (Proposal) msg;
        final Round round = proposal.getVertex().getRound();
        final long roundNumber = round.number();

        return roundNumber % dropPeriod == 0;
      }
      return false;
    };
  }

  @Test
  public void when_run_3_correct_nodes_with_1_timeout__then_bft_should_be_responsive() {
    this.run(3, 50_000, 100);
  }

  @Test
  public void when_run_4_correct_nodes_with_1_timeout__then_bft_should_be_responsive() {
    this.run(4, 50_000, 100);
  }

  @Test
  public void when_run_100_correct_nodes_with_1_timeout__then_bft_should_be_responsive() {
    this.run(100, 1_000, 100);
  }
}
