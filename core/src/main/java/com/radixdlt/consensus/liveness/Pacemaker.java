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

package com.radixdlt.consensus.liveness;

import com.google.common.base.Stopwatch;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.processor.BFTEventProcessorAtCurrentRound;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.TimeSupplier;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Manages the pacemaker driver. */
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public final class Pacemaker implements BFTEventProcessorAtCurrentRound {
  private static final Logger log = LogManager.getLogger();

  private enum RoundStatus {
    UNDISTURBED, // All good, still accepting proposals
    PROPOSAL_REJECTED, // A genuine proposal was received, but it has been rejected
    TIMED_OUT // The round has timed out
  }

  private static final int PREVIOUS_ROUND_RUSHING_TIMESTAMP_LOG_THRESHOLD_MS = 1000;

  private final BFTValidatorId self;
  private final BFTValidatorSet validatorSet;
  private final VertexStoreAdapter vertexStore;
  private final SafetyRules safetyRules;
  private final ScheduledEventDispatcher<ScheduledLocalTimeout> scheduledLocalTimeoutDispatcher;
  private final PacemakerTimeoutCalculator timeoutCalculator;
  private final ProposalGenerator proposalGenerator;
  private final Hasher hasher;
  private final RemoteEventDispatcher<BFTValidatorId, Proposal> proposalDispatcher;
  private final RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher;
  private final EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher;
  private final EventDispatcher<NoVote> noVoteDispatcher;
  private final TimeSupplier timeSupplier;
  private final Metrics metrics;

  private RoundUpdate latestRoundUpdate;
  private final Stopwatch currentRoundStopwatch = Stopwatch.createUnstarted();
  private Optional<ExecutedVertex> insertedVertexCarriedOverFromPrevRound = Optional.empty();
  private RoundStatus roundStatus = RoundStatus.UNDISTURBED;

  // For tracking round prolongation
  private Round highestReceivedProposalRound;
  private Round highestKnownQcRound;
  // Whether any scheduled timeout event has already been received
  // set to true even if the result was a prolongation
  private boolean scheduledRoundTimeoutHasOccurred = false;

  public Pacemaker(
      BFTValidatorId self,
      BFTValidatorSet validatorSet,
      VertexStoreAdapter vertexStore,
      SafetyRules safetyRules,
      EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher,
      ScheduledEventDispatcher<ScheduledLocalTimeout> scheduledLocalTimeoutDispatcher,
      PacemakerTimeoutCalculator timeoutCalculator,
      ProposalGenerator proposalGenerator,
      RemoteEventDispatcher<BFTValidatorId, Proposal> proposalDispatcher,
      RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher,
      EventDispatcher<NoVote> noVoteDispatcher,
      Hasher hasher,
      TimeSupplier timeSupplier,
      RoundUpdate initialRoundUpdate,
      Metrics metrics) {
    this.self = Objects.requireNonNull(self);
    this.validatorSet = Objects.requireNonNull(validatorSet);
    this.vertexStore = Objects.requireNonNull(vertexStore);
    this.safetyRules = Objects.requireNonNull(safetyRules);
    this.scheduledLocalTimeoutDispatcher = Objects.requireNonNull(scheduledLocalTimeoutDispatcher);
    this.timeoutDispatcher = Objects.requireNonNull(timeoutDispatcher);
    this.timeoutCalculator = Objects.requireNonNull(timeoutCalculator);
    this.proposalGenerator = Objects.requireNonNull(proposalGenerator);
    this.proposalDispatcher = Objects.requireNonNull(proposalDispatcher);
    this.voteDispatcher = Objects.requireNonNull(voteDispatcher);
    this.noVoteDispatcher = Objects.requireNonNull(noVoteDispatcher);
    this.hasher = Objects.requireNonNull(hasher);
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
    this.metrics = Objects.requireNonNull(metrics);

    final var highestQcRound = initialRoundUpdate.getHighQC().getHighestRound();
    this.highestKnownQcRound = highestQcRound;
    this.highestReceivedProposalRound = highestQcRound;
  }

  @Override
  public void start() {
    log.info("Pacemaker Start: {}", latestRoundUpdate);
    this.startRound();
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    log.trace("Round Update: {}", roundUpdate);
    this.latestRoundUpdate = roundUpdate;
    if (currentRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = roundUpdate.getCurrentRound();
    }
    this.startRound();
  }

  private void startRound() {
    if (currentRoundStopwatch.isRunning()) {
      this.metrics.bft().pacemaker().roundDuration().observe(currentRoundStopwatch.elapsed());
    }
    currentRoundStopwatch.reset();
    currentRoundStopwatch.start();

    this.metrics.bft().pacemaker().round().set(currentRound().number());

    this.roundStatus = RoundStatus.UNDISTURBED;
    this.scheduledRoundTimeoutHasOccurred = false;

    final var timeoutMs =
        timeoutCalculator.calculateTimeoutMs(latestRoundUpdate.consecutiveUncommittedRoundsCount());
    final var scheduledLocalTimeout = ScheduledLocalTimeout.create(latestRoundUpdate, timeoutMs);
    this.scheduledLocalTimeoutDispatcher.dispatch(scheduledLocalTimeout, timeoutMs);

    final var currentRoundProposer = latestRoundUpdate.getLeader();
    if (this.self.equals(currentRoundProposer)) {
      generateProposal()
          .ifPresent(
              proposal -> {
                log.trace("Broadcasting proposal: {}", proposal);
                this.proposalDispatcher.dispatch(this.validatorSet.nodes(), proposal);
                this.metrics.bft().pacemaker().proposalsSent().inc();
              });
    } else {
      // We can immediately vote if there is a vertex for the current round
      // that we've already received while still being at a previous round
      this.insertedVertexCarriedOverFromPrevRound
          .filter(i -> i.getRound().equals(currentRound()))
          .ifPresent(this::attemptVoteOnVertex);
    }
  }

  @Override
  public void processProposal(Proposal proposal) {
    /* A received proposal's vertex is being inserted into the vertex store.
    A vote might be sent once the vertex has been inserted,
    this is done in response to a received BFTInsertUpdate event. */
    final var proposedVertex = proposal.getVertex().withId(hasher);
    this.vertexStore.insertVertex(proposedVertex);
    metrics.bft().successfullyProcessedProposals().inc();
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    log.trace("BFTUpdate: Processing {}", update);

    final var round = update.getHeader().getRound();
    final var vertex = update.getInserted();

    if (round.equals(currentRound())) {
      // A vertex for the current round has been inserted

      // 1. Check if we haven't yet voted for anything (if so, ignore)
      if (this.safetyRules.getLastVote(currentRound()).isPresent()) {
        return;
      }

      /* 2. If we haven't yet voted, try to vote for whatever vertex was inserted
      We only expect three kinds of vertices here:
       a) a vertex received in a proposal for the current round: "normal" scenario
       b) a vertex received in a QC for the next round: a scenario when there's already a QC
          for the current round being processed but the Pacemaker hasn't received it yet.
          In this case we might still want to send our (obsolete) vote
       c) a fallback vertex has been inserted: in this case the Pacemaker's `roundStatus`
          must have already been set to either PROPOSAL_REJECTED or TIMED_OUT

      Since in all 3 cases we want to attempt a vote (if this is our first vote), we're just passing
      over to `attemptVoteOnVertex` which takes care of deciding whether a vote should include
      a timeout flag and whether it should be sent to the next leader or broadcasted to everyone. */
      attemptVoteOnVertex(vertex);
    } else if (round.gt(currentRound())) {
      // A vertex for a future round (from Pacemaker's point of view) has been inserted
      // this might be due to a race condition between this event and a RoundUpdate.
      // We store the vertex so that the Pacemaker can use it once it switches to the next round
      // (see `startRound`).
      this.insertedVertexCarriedOverFromPrevRound = Optional.of(vertex);
    } else {
      log.trace(
          "InsertUpdate: Ignoring insert {} for round {}, current round at {}",
          update,
          round,
          currentRound());
    }
  }

  private void attemptVoteOnVertex(ExecutedVertex executedVertex) {
    final var bftHeader =
        new BFTHeader(
            executedVertex.getRound(),
            executedVertex.getVertexHash(),
            executedVertex.getLedgerHeader());

    final var maybeBaseVote =
        this.safetyRules.createVote(
            executedVertex.getVertexWithHash(),
            bftHeader,
            executedVertex.getTimeOfExecution(),
            this.latestRoundUpdate.getHighQC());

    maybeBaseVote.ifPresentOrElse(
        baseVote -> {
          // A timeout flag is included if any scheduled timeout
          // event was observed (even if in the end the round was prolonged)
          final var vote =
              this.scheduledRoundTimeoutHasOccurred
                  ? this.safetyRules.timeoutVote(baseVote)
                  : baseVote;
          dispatchVote(vote);
        },
        () -> this.noVoteDispatcher.dispatch(NoVote.create(executedVertex.getVertexWithHash())));
  }

  private void dispatchVote(Vote vote) {
    // The vote is sent to all if any timeout has occurred (even if the round was prolonged).
    // Note that a vote might include a timeout flag (f.e. in case of handling proposalRejected
    // before any timeout)
    // and still be sent only to the next leader (rather than being broadcasted).
    if (this.scheduledRoundTimeoutHasOccurred) {
      this.voteDispatcher.dispatch(this.validatorSet.nodes(), vote);
    } else {
      this.voteDispatcher.dispatch(this.latestRoundUpdate.getNextLeader(), vote);
    }
  }

  @Override
  public void processProposalRejected(ProposalRejected proposalRejected) {
    if (this.roundStatus != RoundStatus.UNDISTURBED) {
      // No-op if the round is already disturbed
      return;
    }
    this.roundStatus = RoundStatus.PROPOSAL_REJECTED;

    /* The proposal was rejected, so all we can do is re-send our previous vote with a timeout flag,
    or insert a fallback vertex and await an async BFT update event  */
    resendPreviousVoteWithTimeoutOrVoteForFallbackVertex();
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledTimeout) {
    this.scheduledRoundTimeoutHasOccurred = true;

    if (canRoundTimeoutBeProlonged(scheduledTimeout)) {
      // If the round can be prolonged, then do so.
      // Do not send any timeout votes and/or create fallback vertex yet.
      // Note that even though the round has been prolonged (i.e. a "real" round timeout delayed)
      // some behaviour has already been altered by setting the `scheduledRoundTimeoutHasOccurred`
      // flag to true above.
      // See: `processBFTUpdate` and `dispatchVote`
      prolongRoundTimeout(scheduledTimeout);
    } else {
      // The round can't be prolonged, so timeout for real
      this.roundStatus = RoundStatus.TIMED_OUT;
      updateTimeoutCounters(scheduledTimeout);
      /* Re-send a previous vote (if there is one) or
      insert a fallback vertex and await an async BFT insert update event.
      See: `processBFTUpdate` */
      resendPreviousVoteWithTimeoutOrVoteForFallbackVertex();
      // Dispatch an actual timeout occurrence
      this.timeoutDispatcher.dispatch(new LocalTimeoutOccurrence(scheduledTimeout));
      rescheduleTimeout(scheduledTimeout);
    }
  }

  /**
   * Attempts to re-send a previously sent vote (if present) with a timeout flag included. If no
   * vote has been sent yet, attempts to send a vote for a fallback vertex (if present). If the
   * fallback vertex doesn't exist yet, then attempts to create one and returns. The insertion of a
   * fallback vertex should trigger an async BFT update event and the "vote for a fallback vertex"
   * process should carry on from there (see: `processBFTUpdate`).
   */
  private void resendPreviousVoteWithTimeoutOrVoteForFallbackVertex() {
    this.safetyRules
        .getLastVote(currentRound())
        .map(this.safetyRules::timeoutVote)
        .ifPresentOrElse(this::dispatchVote, this::voteForFallbackVertex);
  }

  /**
   * A helper for `resendPreviousVoteWithTimeoutOrVoteForFallbackVertex`, see its doc for details
   */
  private void voteForFallbackVertex() {
    final var vertex =
        Vertex.createFallback(
                this.latestRoundUpdate.getHighQC().highestQC(),
                this.latestRoundUpdate.getCurrentRound(),
                this.latestRoundUpdate.getLeader())
            .withId(hasher);

    this.vertexStore
        .getExecutedVertex(vertex.hash())
        .ifPresentOrElse(
            this::attemptVoteOnVertex,
            () -> {
              // Fallback vertex doesn't yet exist, so try inserting one.
              // We'll vote for it once received in an async BFTInsertUpdate event (see:
              // `processBFTUpdate`).

              // FIXME: This (try-catch) is a temporary fix so that we can continue
              // if the vertex store is too far ahead of the pacemaker
              try {
                this.vertexStore.insertVertex(vertex);
              } catch (MissingParentException e) {
                log.debug("Could not insert a timeout vertex: {}", e.getMessage());
              }
            });
  }

  private boolean canRoundTimeoutBeProlonged(ScheduledLocalTimeout originalScheduledLocalTimeout) {
    if (originalScheduledLocalTimeout.hasBeenProlonged()) {
      return false;
    }

    if (roundStatus != RoundStatus.UNDISTURBED) {
      return false;
    }

    /* The timeouts for the current round can be prolonged if:
    1) we have already received a QC for this (or higher) round, which hasn't yet been synced-up to:
       this prevents us from sending timeout votes for rounds that already have a QC,
       and potentially avoids creating a competing TC for the same round.
    2) we extend it for round N if we have received a proposal for round N to give us time to process the proposal
    3) we extend it for round N if we have received a proposal for any round M > N to give us time to sync up the QC for round N (if it exists)
    The 3rd case really duplicates the "received QC" condition (as proposal for round M > N should contain a QC for round N),
    but adding it explicitly for completeness. */

    final var receivedAnyQcForThisOrHigherRound = this.highestKnownQcRound.gte(currentRound());

    final var receivedProposalForThisOrFutureRound =
        this.highestReceivedProposalRound.gte(currentRound());

    return (receivedAnyQcForThisOrHigherRound || receivedProposalForThisOrFutureRound)
        && timeoutCalculator.additionalRoundTimeIfProposalReceivedMs() > 0
        && !scheduledRoundTimeoutHasOccurred;
  }

  private void prolongRoundTimeout(ScheduledLocalTimeout originalScheduledLocalTimeout) {
    metrics.bft().prolongedRoundTimeouts().inc();
    final var timeout = timeoutCalculator.additionalRoundTimeIfProposalReceivedMs();
    final var nextTimeout = originalScheduledLocalTimeout.prolong(timeout);
    this.scheduledLocalTimeoutDispatcher.dispatch(nextTimeout, timeout);
  }

  private void updateTimeoutCounters(ScheduledLocalTimeout scheduledTimeout) {
    if (scheduledTimeout.count() == 0) {
      metrics.bft().pacemaker().timedOutRounds().inc();
    }
    metrics.bft().pacemaker().timeoutsSent().inc();
  }

  private void rescheduleTimeout(ScheduledLocalTimeout scheduledTimeout) {
    final var timeout =
        timeoutCalculator.calculateTimeoutMs(latestRoundUpdate.consecutiveUncommittedRoundsCount());
    final var nextTimeout = scheduledTimeout.nextRetry(timeout);
    this.scheduledLocalTimeoutDispatcher.dispatch(nextTimeout, timeout);
  }

  private Optional<Proposal> generateProposal() {
    final var highQC = this.latestRoundUpdate.getHighQC();
    final var highestQC = highQC.highestQC();
    final var nextTransactions = getTransactionsForProposal(currentRound(), highestQC);
    final var proposerTimestamp = determineNextProposalTimestamp(highestQC);
    final var proposedVertex =
        Vertex.create(highestQC, currentRound(), nextTransactions, self, proposerTimestamp)
            .withId(hasher);
    return safetyRules.signProposal(
        proposedVertex, highQC.highestCommittedQC(), highQC.highestTC());
  }

  private long determineNextProposalTimestamp(QuorumCertificate highestQC) {
    final var now = timeSupplier.currentTime();
    final var previousProposerTimestamp =
        highestQC.getProposedHeader().getLedgerHeader().proposerTimestamp();
    if (now >= previousProposerTimestamp) {
      /* All good, previous timestamp is smaller than (or equal to) our system time, so we can just use it  */
      return now;
    } else /* now < previousProposerTimestamp */ {
      /* Our local system time is lagging or the quorum has agreed on a rushing timestamp in the previous round.
       * We can't use our local time because the proposal would be rejected, hence re-using the previous timestamp
       * to get as close to the current time (from this node's perspective) as possible. */
      final var diff = previousProposerTimestamp - now;
      if (diff > PREVIOUS_ROUND_RUSHING_TIMESTAMP_LOG_THRESHOLD_MS) {
        log.warn(
            "Previous round proposer timestamp was greater than the current local system time. "
                + "This may (but doesn't have to) indicate system clock malfunction. "
                + "Consider further investigation if this log message appears on a regular basis.");
      }
      this.metrics.bft().pacemaker().proposalsWithSubstituteTimestamp().inc();
      return previousProposerTimestamp;
    }
  }

  private List<RawNotarizedTransaction> getTransactionsForProposal(
      Round round, QuorumCertificate highestQC) {
    // If we're at the end of an epoch, we need to generate an empty proposal
    // - these transactions will get ignored at vertex preparation time anyway
    // TODO: Remove isEndOfEpoch knowledge from consensus
    if (highestQC.getProposedHeader().getLedgerHeader().isEndOfEpoch()) {
      return List.of();
    }

    final var alreadyExecutedVertices =
        vertexStore.getPathFromRoot(highestQC.getProposedHeader().getVertexId());
    final var nextTransactions =
        proposalGenerator.getTransactionsForProposal(round, alreadyExecutedVertices);
    this.metrics.bft().pacemaker().proposedTransactions().inc(nextTransactions.size());
    return nextTransactions;
  }

  @Override
  public void preProcessUnsyncedProposalForCurrentOrFutureRound(Proposal proposal) {
    // Process highest received proposal / QC
    // Used to determine whether the round timeout should be prolonged
    // See: processLocalTimeout and canRoundTimeoutBeProlonged
    if (proposal.highQC().getHighestRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = proposal.highQC().getHighestRound();
    }
    if (proposal.getRound().gt(this.highestReceivedProposalRound)) {
      this.highestReceivedProposalRound = proposal.getRound();
    }
  }

  @Override
  public void preProcessUnsyncedVoteForCurrentOrFutureRound(Vote vote) {
    // Process highest received proposal / QC
    // Used to determine whether the round timeout should be prolonged
    // See: processLocalTimeout and canRoundTimeoutBeProlonged
    if (vote.highQC().getHighestRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = vote.highQC().getHighestRound();
    }
  }

  private Round currentRound() {
    return this.latestRoundUpdate.getCurrentRound();
  }

  @Override
  public void processVote(Vote vote) {
    // no-op, Pacemaker doesn't process votes
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate update) {
    // no-op, Pacemaker doesn't process BFT rebuilds
  }
}
