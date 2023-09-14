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

package com.radixdlt.consensus.bft.processor;

import com.radixdlt.consensus.PendingVotes;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.RoundQuorum;
import com.radixdlt.consensus.bft.RoundQuorumResolution;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.bft.VoteProcessingResult.QuorumReached;
import com.radixdlt.consensus.bft.VoteProcessingResult.VoteAccepted;
import com.radixdlt.consensus.bft.VoteProcessingResult.VoteRejected;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.Bft.IgnoredVote;
import com.radixdlt.monitoring.Metrics.Bft.VoteIgnoreReason;
import java.util.Objects;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Processes BFT Vote events and assembles a quorum (either a regular or timeout quorum). Warning:
 * operates under the assumption that all received events are for the current round.
 */
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public final class BFTQuorumAssembler implements BFTEventProcessorAtCurrentRound {
  private static final Logger log = LogManager.getLogger();

  public record RoundQuorumWithLastVote(RoundQuorum roundQuorum, Vote lastVote) {}

  /** A local event triggering a delayed resolution of a timeout quorum */
  public record TimeoutQuorumDelayedResolution(
      RoundQuorumWithLastVote roundQuorumWithLastVote, long millisecondsWaitTime) {}

  private final BFTEventProcessorAtCurrentRound forwardTo;
  private final BFTValidatorId self;
  private final EventDispatcher<RoundQuorumResolution> roundQuorumResolutionDispatcher;
  private final ScheduledEventDispatcher<TimeoutQuorumDelayedResolution>
      timeoutQuorumDelayedResolutionDispatcher;
  private final Metrics metrics;
  private final PendingVotes pendingVotes;
  private final long timeoutQuorumResolutionDelayMs;

  private RoundUpdate latestRoundUpdate;
  private boolean hasCurrentRoundBeenResolved = false;
  private boolean hasTimeoutQuorumResolutionBeenDelayedInCurrentRound = false;

  public BFTQuorumAssembler(
      BFTEventProcessorAtCurrentRound forwardTo,
      BFTValidatorId self,
      EventDispatcher<RoundQuorumResolution> roundQuorumResolutionDispatcher,
      ScheduledEventDispatcher<TimeoutQuorumDelayedResolution>
          timeoutQuorumDelayedResolutionDispatcher,
      Metrics metrics,
      PendingVotes pendingVotes,
      RoundUpdate initialRoundUpdate,
      long timeoutQuorumResolutionDelayMs) {
    this.forwardTo = Objects.requireNonNull(forwardTo);
    this.self = Objects.requireNonNull(self);
    this.roundQuorumResolutionDispatcher = Objects.requireNonNull(roundQuorumResolutionDispatcher);
    this.timeoutQuorumDelayedResolutionDispatcher =
        Objects.requireNonNull(timeoutQuorumDelayedResolutionDispatcher);
    this.metrics = Objects.requireNonNull(metrics);
    this.pendingVotes = Objects.requireNonNull(pendingVotes);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
    this.timeoutQuorumResolutionDelayMs = timeoutQuorumResolutionDelayMs;
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    this.latestRoundUpdate = roundUpdate;
    this.hasCurrentRoundBeenResolved = false;
    this.hasTimeoutQuorumResolutionBeenDelayedInCurrentRound = false;
    forwardTo.processRoundUpdate(roundUpdate);
  }

  @Override
  public void processVote(Vote vote) {
    log.trace("Vote: Processing {}", vote);
    processVoteInternal(vote);
    forwardTo.processVote(vote);
  }

  private void processVoteInternal(Vote vote) {
    final var currentRound = this.latestRoundUpdate.getCurrentRound();

    if (!this.self.equals(this.latestRoundUpdate.getNextLeader()) && !vote.isTimeout()) {
      metrics.bft().ignoredVotes().label(new IgnoredVote(VoteIgnoreReason.UNEXPECTED_VOTE)).inc();
      log.trace(
          "Vote: Ignoring vote from {} for round {}, unexpected vote",
          vote.getAuthor(),
          currentRound);
      return;
    }

    switch (this.pendingVotes.insertVote(vote)) {
      case VoteAccepted ignored -> log.trace("Vote has been processed but didn't form a quorum");
      case VoteRejected voteRejected -> log.trace(
          "Vote has been rejected because of: {}", voteRejected.reason());
      case QuorumReached quorumReached -> this.processQuorum(quorumReached.roundQuorum(), vote);
    }

    metrics
        .bft()
        .successfullyProcessedVotes()
        .label(new Metrics.Bft.SuccessfullyProcessedVote(vote.isTimeout()))
        .inc();
  }

  private void processQuorum(RoundQuorum roundQuorum, Vote lastVote) {
    if (hasCurrentRoundBeenResolved) {
      // Nothing to do if the round has already been resolved
      return;
    }

    final var roundQuorumWithLastAuthor = new RoundQuorumWithLastVote(roundQuorum, lastVote);

    switch (roundQuorum) {
      case RoundQuorum.RegularRoundQuorum regularRoundQuorum -> {
        // Regular quorum was formed, we can immediately resolve the round
        resolveCurrentRoundWithQuorum(roundQuorum, lastVote);
      }
      case RoundQuorum.TimeoutRoundQuorum timeoutRoundQuorum -> {
        // A timeout quorum has been formed
        if (timeoutQuorumResolutionDelayMs > 0) {
          // A delay has been configured, so we're going to delay its processing,
          // in hope to form a QC (and continue the 3-chain).
          // We might have already formed (and delayed) a timeout quorum in this round, in which
          // case any subsequent quorums (i.e. with additional votes) are ignored -
          // no need to dispatch duplicate delayed resolution events.
          if (!hasTimeoutQuorumResolutionBeenDelayedInCurrentRound) {
            hasTimeoutQuorumResolutionBeenDelayedInCurrentRound = true;
            metrics.bft().timeoutQuorumDelayedResolutions().inc();
            this.timeoutQuorumDelayedResolutionDispatcher.dispatch(
                new TimeoutQuorumDelayedResolution(
                    roundQuorumWithLastAuthor, timeoutQuorumResolutionDelayMs),
                timeoutQuorumResolutionDelayMs);
          }
        } else {
          // No delay was configured, so process the timeout quorum immediately
          resolveCurrentRoundWithQuorum(timeoutRoundQuorum, lastVote);
        }
      }
    }
  }

  @Override
  public void processTimeoutQuorumDelayedResolution(
      TimeoutQuorumDelayedResolution timeoutQuorumDelayedResolution) {
    if (hasCurrentRoundBeenResolved) {
      return; // no-op if current round has already been resolved
    } else {
      final var quorumAndLastVote = timeoutQuorumDelayedResolution.roundQuorumWithLastVote();
      resolveCurrentRoundWithQuorum(quorumAndLastVote.roundQuorum(), quorumAndLastVote.lastVote());
    }

    forwardTo.processTimeoutQuorumDelayedResolution(timeoutQuorumDelayedResolution);
  }

  private void resolveCurrentRoundWithQuorum(RoundQuorum roundQuorum, Vote lastVote) {
    this.hasCurrentRoundBeenResolved = true;
    this.roundQuorumResolutionDispatcher.dispatch(new RoundQuorumResolution(roundQuorum, lastVote));
  }

  @Override
  public Optional<BFTEventProcessorAtCurrentRound> forwardTo() {
    return Optional.of(forwardTo);
  }
}
