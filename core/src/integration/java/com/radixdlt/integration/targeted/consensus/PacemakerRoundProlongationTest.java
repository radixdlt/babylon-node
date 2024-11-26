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

import static org.junit.Assert.assertEquals;

import com.radixdlt.consensus.EpochNodeWeightMapping;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
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
import com.radixdlt.monitoring.Metrics;
import org.junit.Test;

public final class PacemakerRoundProlongationTest {
  private static final long PACEMAKER_BASE_TIMEOUT_MS = 1000;
  private static final long ADDITIONAL_TIME_MS = 1000;

  private DeterministicTest createTest(MessageMutator messageMutator) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatchWithFakeAddresses(1))
        .messageSelector(MessageSelector.firstSelector())
        .messageMutators(messageMutator)
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.none(),
                true,
                SafetyRecoveryConfig.MOCKED,
                ConsensusConfig.of(PACEMAKER_BASE_TIMEOUT_MS, ADDITIONAL_TIME_MS),
                LedgerConfig.stateComputerMockedSync(
                    StateComputerConfig.mockedWithEpochs(
                        10000, EpochNodeWeightMapping.constant(1)))));
  }

  // spotless:off
  /** Scenario 1: Round prolonged because Proposal took to long to process
   * - Local BFTInsertUpdate delayed: simulated long Proposal execution (vertex prepare)
   * - Local timeout kicks in
   * - Proposal received and no vote sent yet in the current round,
   *    so it can be prolonged
   * - (delayed) BFTInsertUpdate finally arrives and the vote is sent
   * Result: 1 prolonged round, 0 actual timeout occurrences
   */
  // spotless:on
  @Test
  public void round_prolonged_when_proposal_took_too_long() {
    // Delay BFTInsertUpdate (simulated long execution)
    final var messageMutator =
        delayMessagesOfType(BFTInsertUpdate.class, PACEMAKER_BASE_TIMEOUT_MS + 200L);
    try (var test = createTest(messageMutator)) {
      test.startAllNodes();
      test.runUntilMessage(m -> m.value().message() instanceof Vote, 1000);
      final var bftMetrics = test.getInstance(0, Metrics.class).bft();
      assertEquals(1, (int) bftMetrics.prolongedRoundTimeouts().get());
      assertEquals(0, (int) bftMetrics.pacemaker().timedOutRounds().get());
    }
  }

  // spotless:off
  /** Scenario 2: Round not prolonged because proposal not received
   * - Proposal reception delayed
   * - Local timeout kicks in
   * - Round not prolonged, because Proposal not received
   * - (later) A vote is sent
   * Result: 0 prolonged rounds, 1 actual timeout occurrences
   */
  // spotless:on
  @Test
  public void round_not_prolonged_when_proposal_not_received() {
    // Delay Proposal reception
    final var messageMutator =
        delayMessagesOfType(Proposal.class, PACEMAKER_BASE_TIMEOUT_MS + 200L);
    try (var test = createTest(messageMutator)) {
      test.startAllNodes();
      test.runUntilMessage(m -> m.value().message() instanceof Vote, 1000);
      final var bftMetrics = test.getInstance(0, Metrics.class).bft();
      assertEquals(0, (int) bftMetrics.prolongedRoundTimeouts().get());
      assertEquals(1, (int) bftMetrics.pacemaker().timedOutRounds().get());
    }
  }

  // spotless:off
  /** Scenario 3: Round not prolonged because vote already sent
   * - Proposal processed normally (no delay for BFTInsertUpdate)
   * - Vote sent immediately, but its reception is delayed, which
   *    simulates "next leader" failure
   * - QC not formed on time
   * - Local timeout kicks in
   * - A vote was sent earlier, so round can't be prolonged
   * Result: 0 prolonged rounds, 1 actual timeout occurrence
   */
  // spotless:on
  @Test
  public void round_not_prolonged_when_vote_already_sent() {
    // Delay Vote reception
    final var messageMutator = delayMessagesOfType(Vote.class, PACEMAKER_BASE_TIMEOUT_MS + 200L);
    try (var test = createTest(messageMutator)) {
      test.startAllNodes();
      test.runUntilMessage(m -> m.value().message() instanceof Vote, 1000);
      final var bftMetrics = test.getInstance(0, Metrics.class).bft();
      assertEquals(0, (int) bftMetrics.prolongedRoundTimeouts().get());
      assertEquals(1, (int) bftMetrics.pacemaker().timedOutRounds().get());
    }
  }

  // spotless:off
  /** Scenario 4: Round prolonged and then timeout for real
   * - Local BFTInsertUpdate delayed so much that even a prolonged round times out
   * - Local timeout kicks in
   * - Proposal received and no vote sent yet in the current round,
   *    so it can be prolonged
   * - Prolonged timeout kicks in
   * - Can't prolong no more, so timeout for real
   * - (delayed) BFTInsertUpdate finally arrives and the vote is sent
   * Result: 1 prolonged round, 1 actual timeout occurrences
   */
  // spotless:on
  @Test
  public void round_prolonged_at_most_once() {
    // Delay BFTInsertUpdate by more than (initial timeout + additional time)
    final var messageMutator =
        delayMessagesOfType(
            BFTInsertUpdate.class, PACEMAKER_BASE_TIMEOUT_MS + ADDITIONAL_TIME_MS + 500L);
    try (var test = createTest(messageMutator)) {
      test.startAllNodes();
      test.runUntilMessage(m -> m.value().message() instanceof Vote, 1000);
      final var bftMetrics = test.getInstance(0, Metrics.class).bft();
      assertEquals(1, (int) bftMetrics.prolongedRoundTimeouts().get());
      assertEquals(1, (int) bftMetrics.pacemaker().timedOutRounds().get());
    }
  }

  private MessageMutator delayMessagesOfType(Class<?> cls, long additionalDelay) {
    return (message, queue) -> {
      if (cls.isAssignableFrom(message.message().getClass())) {
        queue.add(message.withAdditionalDelay(additionalDelay));
        return true;
      }
      return false;
    };
  }
}
