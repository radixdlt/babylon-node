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
import com.radixdlt.consensus.bft.RoundQuorumReached;
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

  /** An event indicating that processing a round quorum has been postponed */
  public record PostponedRoundQuorum(
      RoundQuorumWithLastVote roundQuorumWithLastVote, long millisecondsWaitTime) {}

  private final BFTEventProcessorAtCurrentRound forwardTo;
  private final BFTValidatorId self;
  private final EventDispatcher<RoundQuorumReached> roundQuorumReachedDispatcher;
  private final ScheduledEventDispatcher<PostponedRoundQuorum> postponedRoundQuorumDispatcher;
  private final Metrics metrics;
  private final PendingVotes pendingVotes;
  private final long timeoutQuorumProcessingDelayMs;

  private RoundUpdate latestRoundUpdate;
  private Optional<RoundQuorumWithLastVote> bestQuorumFormedSoFarInCurrentRound = Optional.empty();

  public BFTQuorumAssembler(
      BFTEventProcessorAtCurrentRound forwardTo,
      BFTValidatorId self,
      EventDispatcher<RoundQuorumReached> roundQuorumReachedDispatcher,
      ScheduledEventDispatcher<PostponedRoundQuorum> postponedRoundQuorumDispatcher,
      Metrics metrics,
      PendingVotes pendingVotes,
      RoundUpdate initialRoundUpdate,
      long timeoutQuorumProcessingDelayMs) {
    this.forwardTo = Objects.requireNonNull(forwardTo);
    this.self = Objects.requireNonNull(self);
    this.roundQuorumReachedDispatcher = Objects.requireNonNull(roundQuorumReachedDispatcher);
    this.postponedRoundQuorumDispatcher = Objects.requireNonNull(postponedRoundQuorumDispatcher);
    this.metrics = Objects.requireNonNull(metrics);
    this.pendingVotes = Objects.requireNonNull(pendingVotes);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
    this.timeoutQuorumProcessingDelayMs = timeoutQuorumProcessingDelayMs;
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    this.latestRoundUpdate = roundUpdate;
    this.bestQuorumFormedSoFarInCurrentRound = Optional.empty();
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

    metrics.bft().successfullyProcessedVotes().inc();
  }

  private void processQuorum(RoundQuorum roundQuorum, Vote lastVote) {
    if (hasRegularQuorumBeenFormedForCurrentRound()) {
      // Nothing to do if regular form has already been formed
      return;
    }

    final var roundQuorumWithLastAuthor = new RoundQuorumWithLastVote(roundQuorum, lastVote);

    switch (roundQuorum) {
      case RoundQuorum.RegularRoundQuorum regularRoundQuorum -> {
        // Regular quorum was formed, it's always "better" than any timeout quorum,
        // so we dispatch it immediately.
        this.bestQuorumFormedSoFarInCurrentRound = Optional.of(roundQuorumWithLastAuthor);
        this.roundQuorumReachedDispatcher.dispatch(new RoundQuorumReached(roundQuorum, lastVote));
      }
      case RoundQuorum.TimeoutRoundQuorum timeoutRoundQuorum -> {
        // A timeout quorum has been formed.
        // If this is the first quorum we've formed in this round, we're
        // going to postpone its processing, in hope to form a QC (and continue the 3-chain).
        // This can also be a subsequent timeout quorum (i.e. with another vote/signature included).
        // In this case we don't want to dispatch a postponed quorum event
        // (because it has already been dispatched when processing the previous TC),
        // but we still replace our `bestQuorumFormedSoFarInCurrentRound` because
        // (arguably) more signatures == a more trustworthy certificate.
        final var isThisTheFirstQuorumFormedInThisRound =
            this.bestQuorumFormedSoFarInCurrentRound.isEmpty();
        this.bestQuorumFormedSoFarInCurrentRound = Optional.of(roundQuorumWithLastAuthor);
        if (isThisTheFirstQuorumFormedInThisRound) {
          metrics.bft().postponedRoundQuorums().inc();
          this.postponedRoundQuorumDispatcher.dispatch(
              new PostponedRoundQuorum(roundQuorumWithLastAuthor, timeoutQuorumProcessingDelayMs),
              timeoutQuorumProcessingDelayMs);
        }
      }
    }
  }

  @Override
  public void processPostponedRoundQuorum(PostponedRoundQuorum postponedRoundQuorum) {
    this.bestQuorumFormedSoFarInCurrentRound.ifPresent(
        bestQuorum -> {
          switch (bestQuorum.roundQuorum) {
            case RoundQuorum.RegularRoundQuorum regularRoundQuorum -> {
              // Regular round quorums are dispatched immediately, so nothing to do here
            }
            case RoundQuorum.TimeoutRoundQuorum timeoutRoundQuorum -> {
              // We've failed to form a QC while the TC processing was being postponed,
              // dispatch our current best TC then (note that the quorum we've received
              // in the event is ignored).
              this.roundQuorumReachedDispatcher.dispatch(
                  new RoundQuorumReached(timeoutRoundQuorum, bestQuorum.lastVote));
            }
          }
        });

    forwardTo.processPostponedRoundQuorum(postponedRoundQuorum);
  }

  private boolean hasRegularQuorumBeenFormedForCurrentRound() {
    return this.bestQuorumFormedSoFarInCurrentRound.stream()
        .anyMatch(q -> q.roundQuorum instanceof RoundQuorum.RegularRoundQuorum);
  }

  @Override
  public Optional<BFTEventProcessorAtCurrentRound> forwardTo() {
    return Optional.of(forwardTo);
  }
}
