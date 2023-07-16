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

import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.byzantineBehaviorNotDetected;

import com.radixdlt.consensus.EpochNodeWeightMapping;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import io.reactivex.rxjava3.schedulers.Timed;
import java.util.function.Predicate;
import org.junit.Test;

/**
 * This test tests a specific case where if a TC occurs on the epoch boundary that votes on
 * additional Consensus Vertices past the epoch round does NOT cause any disagreement amongst
 * validators for the initial vertex in the next epoch.
 */
public final class EpochTimeoutCertTest {

  private static final long ROUNDS_PER_EPOCH = 100;
  private static final int NUM_NODES = 4;

  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatchWithFakeAddresses(4))
        .messageSelector(MessageSelector.firstSelector())
        .messageMutator(
            (m, q) -> {
              // Drop single proposal at end of epoch to cause a TC
              if (m.message() instanceof Proposal proposal) {
                if (proposal.getRound().number() == ROUNDS_PER_EPOCH + 2) {
                  return m.channelId().receiverIndex() == (m.channelId().senderIndex() + 1) % 4;
                }
              }
              return false;
            })
        .addMonitors(byzantineBehaviorNotDetected())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                NodeStorageConfig.none(),
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                FunctionalRadixNodeModule.ConsensusConfig.of(),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.mockedWithEpochs(
                        Round.of(ROUNDS_PER_EPOCH),
                        EpochNodeWeightMapping.constant(NUM_NODES),
                        new StateComputerConfig.MockedMempoolConfig.NoMempool()))));
  }

  public static Predicate<Timed<ControlledMessage>> proposalAtRound(long round) {
    return m -> {
      if (m.value().message() instanceof Proposal proposal) {
        return proposal.getRound().number() == round;
      }
      return false;
    };
  }

  @Test
  public void no_byzantine_event_occurs_on_epoch_tc_event() {
    try (var test = createTest()) {
      test.startAllNodes();
      // Run until end of first epoch
      test.runUntilMessage(
          proposalAtRound(ROUNDS_PER_EPOCH + 2),
          true,
          10 * NUM_NODES * NUM_NODES * ((int) ROUNDS_PER_EPOCH));
      // Run for a while more and verify that no byzantine issues occur
      test.runForCount(40000);
    }
  }
}
