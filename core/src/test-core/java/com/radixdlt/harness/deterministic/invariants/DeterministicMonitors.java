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

package com.radixdlt.harness.deterministic.invariants;

import com.google.inject.AbstractModule;
import com.google.inject.Module;
import com.google.inject.Singleton;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.bft.BFTHighQCUpdate;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochRound;
import com.radixdlt.consensus.liveness.EpochLocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.LocalTimeoutOccurrence;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.harness.invariants.Checkers;

public final class DeterministicMonitors {
  private DeterministicMonitors() {
    throw new IllegalStateException("Cannot instantiate");
  }

  public static class TimeoutOccurred extends IllegalStateException {
    TimeoutOccurred(String nodeName, String round) {
      super("Timeout on leader " + nodeName + ": " + round);
    }
  }

  public static class ByzantineBehaviorDetected extends IllegalStateException {
    ByzantineBehaviorDetected(ConsensusByzantineEvent event) {
      super("Byzantine Behavior detected: " + event);
    }
  }

  public static Module noTimeouts() {
    return new AbstractModule() {
      @ProvidesIntoSet
      private MessageMonitor byzantineDetection() {
        return (m, t) -> {
          if (m.message() instanceof LocalTimeoutOccurrence event) {
            throw new TimeoutOccurred(event.leader().toString(), event.round().toString());
          }

          if (m.message() instanceof EpochLocalTimeoutOccurrence event) {
            throw new TimeoutOccurred(
                event.getLeader().toString(), event.getEpochRound().toString());
          }
        };
      }
    };
  }

  public static Module byzantineBehaviorNotDetected() {
    return new AbstractModule() {
      @ProvidesIntoSet
      private MessageMonitor byzantineDetection() {
        return (m, t) -> {
          if (m.message() instanceof ConsensusByzantineEvent event) {
            throw new ByzantineBehaviorDetected(event);
          }
        };
      }
    };
  }

  public static class ConsensusLivenessException extends RuntimeException {
    private ConsensusLivenessException(
        long duration, EpochRound lastSeen, long timeSeen, long currentTime) {
      super(
          String.format(
              "Liveness Increasing QC invariant of %s ms broken. LastSeen: %s %s CurrentTime: %s",
              duration, lastSeen, timeSeen, currentTime));
    }
  }

  private static class LivenessMessageMonitor implements MessageMonitor {
    private EpochRound highestEpochRoundSeen = EpochRound.of(1, Round.epochInitial());
    private long timeOfHighestEpochRoundSeen = 0;
    private final long duration;

    private LivenessMessageMonitor(long duration) {
      this.duration = duration;
    }

    @Override
    public void next(ControlledMessage message, long currentTime) {
      if (timeOfHighestEpochRoundSeen != 0) {
        if (currentTime - timeOfHighestEpochRoundSeen > duration) {
          throw new ConsensusLivenessException(
              duration, this.highestEpochRoundSeen, this.timeOfHighestEpochRoundSeen, currentTime);
        }
      }

      final QuorumCertificate highQC;
      if (message.message() instanceof BFTHighQCUpdate update) {
        highQC = update.newHighQc().highestQC();
      } else {
        return;
      }

      var header = highQC.getProposedHeader();
      var epochRound = EpochRound.of(header.getLedgerHeader().getEpoch(), header.getRound());
      if (epochRound.compareTo(this.highestEpochRoundSeen) > 0) {
        this.highestEpochRoundSeen = epochRound;
        this.timeOfHighestEpochRoundSeen = message.arrivalTime();
      }
    }
  }

  public static Module consensusLiveness(long duration) {
    return new AbstractModule() {
      @ProvidesIntoSet
      @Singleton
      private MessageMonitor byzantineDetection() {
        return new LivenessMessageMonitor(duration);
      }
    };
  }

  public static Module ledgerTransactionSafety() {
    return new AbstractModule() {
      @ProvidesIntoSet
      private StateMonitor ledgerTransactionSafety() {
        return (nodes, time, msgs) -> {
          // Execute state monitor checks after some arbitrary number of messages
          // so we don't kill our CPUs
          // TODO: Better would be to only execute on ledger transaction events
          var numMessagesBeforeStateCheck = 89 * nodes.size();
          if (msgs % numMessagesBeforeStateCheck != 0) {
            return;
          }

          Checkers.assertLedgerTransactionsSafety(nodes);
        };
      }
    };
  }
}
