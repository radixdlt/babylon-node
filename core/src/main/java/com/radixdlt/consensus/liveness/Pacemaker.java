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

import com.google.common.collect.ImmutableSet;
import com.google.common.hash.HashCode;
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
  private Optional<BFTInsertUpdate> maybeLatestInsertUpdate = Optional.empty();
  private RoundStatus roundStatus = RoundStatus.UNDISTURBED;

  // ID of the vertex to be used as a substitute for the vertex proposed by
  // the current leader (either because we didn't receive it or it was rejected).
  // The vertex isn't created until it's actually needed (hence Optional).
  private Optional<HashCode> timeoutVertexIdForCurrentRound = Optional.empty();

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
    if (this.latestRoundUpdate.getCurrentRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = roundUpdate.getCurrentRound();
    }
    this.startRound();
  }

  private void startRound() {
    this.metrics.bft().pacemaker().round().set(this.latestRoundUpdate.getCurrentRound().number());

    this.roundStatus = RoundStatus.UNDISTURBED;
    this.timeoutVertexIdForCurrentRound = Optional.empty();
    this.scheduledRoundTimeoutHasOccurred = false;

    final var timeoutMs =
        timeoutCalculator.calculateTimeoutMs(latestRoundUpdate.consecutiveUncommittedRoundsCount());
    final var scheduledLocalTimeout = ScheduledLocalTimeout.create(latestRoundUpdate, timeoutMs);
    this.scheduledLocalTimeoutDispatcher.dispatch(scheduledLocalTimeout, timeoutMs);

    final var currentRoundProposer = latestRoundUpdate.getLeader();
    if (this.self.equals(currentRoundProposer)) {
      final var maybeProposal = generateProposal(latestRoundUpdate.getCurrentRound());
      maybeProposal.ifPresent(
          proposal -> {
            log.trace("Broadcasting proposal: {}", proposal);
            this.proposalDispatcher.dispatch(this.validatorSet.nodes(), proposal);
            this.metrics.bft().pacemaker().proposalsSent().inc();
          });
    } else {
      // Try sending a vote if by any chance we already can at the start of the round.
      // Since this is the beginning of the round, it's never a timeout.
      this.tryToVoteForTheProposalForTheFirstTime(false);
    }
  }

  private void tryToVoteForTheProposalForTheFirstTime(boolean broadcastToEveryoneAsTimeout) {
    if (this.maybeLatestInsertUpdate.isEmpty()) {
      // Nothing to vote for, haven't received any inserted vertices
      return;
    }
    final var latestInsertUpdate = this.maybeLatestInsertUpdate.get();

    if (!latestInsertUpdate
        .getHeader()
        .getRound()
        .equals(this.latestRoundUpdate.getCurrentRound())) {
      // Nothing to vote for, the latest received inserted vertex isn't for the current round
      return;
    }

    if (this.safetyRules.getLastVote(this.latestRoundUpdate.getCurrentRound()).isPresent()) {
      // We've already sent our initial vote
      return;
    }

    /* Assuming that the vertex we're about to vote for has either been:
     a) received in a proposal for the current round
     b) received in a QC (f.e. along with the next proposal, or during sync)
    in either case, if the vertex is valid (and at this point we know it is, as it has been inserted),
    we send our vote for it - if safety rules allow. */

    // TODO: what if insertUpdate occurs before roundUpdate
    final var maybeVote =
        this.safetyRules.createVote(
            latestInsertUpdate.getInserted().getVertexWithHash(),
            latestInsertUpdate.getHeader(),
            latestInsertUpdate.getInserted().getTimeOfExecution(),
            this.latestRoundUpdate.getHighQC());

    maybeVote.ifPresentOrElse(
        vote -> {
          if (broadcastToEveryoneAsTimeout) {
            final var timeoutVote = this.safetyRules.timeoutVote(vote);
            this.voteDispatcher.dispatch(this.validatorSet.nodes(), timeoutVote);
          } else {
            this.voteDispatcher.dispatch(this.latestRoundUpdate.getNextLeader(), vote);
          }
        },
        () ->
            this.noVoteDispatcher.dispatch(
                NoVote.create(latestInsertUpdate.getInserted().getVertexWithHash())));
  }

  @Override
  public void preProcessUnsyncedProposalForCurrentOrFutureRound(Proposal proposal) {
    if (proposal.highQC().getHighestRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = proposal.highQC().getHighestRound();
    }
    if (proposal.getRound().gt(this.highestReceivedProposalRound)) {
      this.highestReceivedProposalRound = proposal.getRound();
    }
  }

  @Override
  public void processProposal(Proposal proposal) {
    final var proposedVertex = proposal.getVertex().withId(hasher);
    this.vertexStore.insertVertex(proposedVertex);
    metrics.bft().successfullyProcessedProposals().inc();
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    log.trace("BFTUpdate: Processing {}", update);

    final var round = update.getHeader().getRound();
    if (round.lt(this.latestRoundUpdate.getCurrentRound())) {
      log.trace(
          "InsertUpdate: Ignoring insert {} for round {}, current round at {}",
          update,
          round,
          this.latestRoundUpdate.getCurrentRound());
      return;
    }

    this.maybeLatestInsertUpdate = Optional.of(update);

    final var updateIsInsertionOfTimeoutVertex =
        this.timeoutVertexIdForCurrentRound
            .filter(update.getInserted().getVertexHash()::equals)
            .isPresent();

    switch (this.roundStatus) {
      case UNDISTURBED -> {
        // The round is undisturbed but if the timeout has already
        // occurred (even if it was prolonged), we'll add a timeout flag to
        // our initial vote and broadcast it to everyone.
        this.tryToVoteForTheProposalForTheFirstTime(this.scheduledRoundTimeoutHasOccurred);
      }
      case PROPOSAL_REJECTED -> {
        if (updateIsInsertionOfTimeoutVertex) {
          // Continue the timeout vote process from where it left off
          // when requesting an (async) vertex insertion.
          // So far we've only rejected the proposal, but the actual
          // round timeout hasn't yet occurred, so we send our timeout
          // vote only to the next leader.
          createAndSendTimeoutVote(
              update.getInserted(), ImmutableSet.of(this.latestRoundUpdate.getNextLeader()));
        }
      }
      case TIMED_OUT -> {
        if (updateIsInsertionOfTimeoutVertex) {
          // Continue the timeout vote process from where it left off
          // when requesting an (async) vertex insertion.
          // The round has timed out for real, send a timeout vote to everyone.
          this.createAndSendTimeoutVoteToAll(update.getInserted());
        } else {
          // The round has timed out, but the insertion could still be a non-timeout vertex we can
          // vote on. In this case we'll add a timeout flag and broadcast our initial vote to
          // everyone.
          this.tryToVoteForTheProposalForTheFirstTime(true);
        }
      }
    }
  }

  /**
   * Processes a local timeout, causing the pacemaker to either broadcast previously sent vote to
   * all nodes or broadcast a new vote for a "null" proposal. In either case, the sent vote includes
   * a timeout signature, which can later be used to form a timeout certificate.
   */
  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledTimeout) {
    this.scheduledRoundTimeoutHasOccurred = true;

    if (canRoundTimeoutBeProlonged(scheduledTimeout)) {
      prolongRoundTimeout(scheduledTimeout);
    } else {
      this.roundStatus = RoundStatus.TIMED_OUT;
      updateTimeoutCounters(scheduledTimeout);
      resendPreviousOrNewVoteWithTimeout(this.validatorSet.nodes());
      // Dispatch an actual timeout occurrence
      this.timeoutDispatcher.dispatch(new LocalTimeoutOccurrence(scheduledTimeout));
      rescheduleTimeout(scheduledTimeout);
    }
  }

  private void prolongRoundTimeout(ScheduledLocalTimeout originalScheduledLocalTimeout) {
    metrics.bft().prolongedRoundTimeouts().inc();
    final var timeout = timeoutCalculator.additionalRoundTimeIfProposalReceivedMs();
    final var nextTimeout = originalScheduledLocalTimeout.prolong(timeout);
    this.scheduledLocalTimeoutDispatcher.dispatch(nextTimeout, timeout);
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

    final var receivedAnyQcForThisOrHigherRound =
        this.highestKnownQcRound.gte(latestRoundUpdate.getCurrentRound());

    final var receivedProposalForThisOrFutureRound =
        this.highestReceivedProposalRound.gte(latestRoundUpdate.getCurrentRound());

    return (receivedAnyQcForThisOrHigherRound || receivedProposalForThisOrFutureRound)
        && timeoutCalculator.additionalRoundTimeIfProposalReceivedMs() > 0
        && !scheduledRoundTimeoutHasOccurred;
  }

  private void resendPreviousOrNewVoteWithTimeout(ImmutableSet<BFTValidatorId> receivers) {
    this.safetyRules
        .getLastVote(this.latestRoundUpdate.getCurrentRound())
        .map(this.safetyRules::timeoutVote)
        .ifPresentOrElse(
            /* if there is a previously sent vote: time it out and send */
            vote -> this.voteDispatcher.dispatch(receivers, vote),
            /* otherwise: asynchronously insert a "leader failure" (timeout) vertex and, when done,
            send a timeout vote for it (see processBFTUpdate) */
            () -> createTimeoutVertexAndSendVote(this.latestRoundUpdate, receivers));
  }

  @Override
  public void processProposalRejected(ProposalRejected proposalRejected) {
    if (this.roundStatus != RoundStatus.UNDISTURBED) {
      // No-op if the round is already disturbed
      return;
    }

    this.roundStatus = RoundStatus.PROPOSAL_REJECTED;

    // Send a single timeout vote to the next leader
    resendPreviousOrNewVoteWithTimeout(ImmutableSet.of(this.latestRoundUpdate.getNextLeader()));
  }

  private void createTimeoutVertexAndSendVote(
      RoundUpdate roundUpdate, ImmutableSet<BFTValidatorId> receivers) {
    if (this.timeoutVertexIdForCurrentRound.isPresent()) {
      // The "leader failure" vertex for this round is already being
      // inserted (or has already been inserted)
      return;
    }

    final var highQC = this.latestRoundUpdate.getHighQC();
    final var vertex =
        Vertex.createTimeout(
                highQC.highestQC(), roundUpdate.getCurrentRound(), roundUpdate.getLeader())
            .withId(hasher);
    this.timeoutVertexIdForCurrentRound = Optional.of(vertex.hash());

    // TODO: reimplement in async way
    this.vertexStore
        .getExecutedVertex(vertex.hash())
        .ifPresentOrElse(
            v ->
                createAndSendTimeoutVote(
                    v,
                    receivers), // the vertex was already present in the vertex store, send the vote
            // immediately
            () ->
                insertTimeoutVertexAndIgnoreMissingParentException(
                    vertex) // otherwise insert and wait for async bft update event
            );
  }

  // FIXME: This is a temporary fix so that we can continue
  // if the vertex store is too far ahead of the pacemaker
  private void insertTimeoutVertexAndIgnoreMissingParentException(VertexWithHash vertexWithHash) {
    if (!vertexWithHash.vertex().isTimeout()) {
      throw new IllegalArgumentException(
          "insertTimeoutVertexAndIgnoreMissingParentException should only be used "
              + "for inserting timeout vertices!");
    }
    try {
      this.vertexStore.insertVertex(vertexWithHash);
    } catch (MissingParentException e) {
      log.debug("Could not insert a timeout vertex: {}", e.getMessage());
    }
  }

  private void createAndSendTimeoutVoteToAll(ExecutedVertex executedVertex) {
    createAndSendTimeoutVote(executedVertex, this.validatorSet.nodes());
  }

  private void createAndSendTimeoutVote(
      ExecutedVertex executedVertex, ImmutableSet<BFTValidatorId> receivers) {
    final var bftHeader =
        new BFTHeader(
            executedVertex.getRound(),
            executedVertex.getVertexHash(),
            executedVertex.getLedgerHeader());

    // TODO: It is possible that an empty vote may be returned here if
    // TODO: we are missing the vertex which we voted for is missing.
    // TODO: This would occur if the liveness bug in VertexStoreJavaImpl:150
    // TODO: occurs. Once liveness bug is fixed we should never hit this state.
    final var maybeBaseVote =
        this.safetyRules.createVote(
            executedVertex.getVertexWithHash(),
            bftHeader,
            this.timeSupplier.currentTime(),
            this.latestRoundUpdate.getHighQC());

    maybeBaseVote.ifPresent(
        baseVote -> {
          final var timeoutVote = this.safetyRules.timeoutVote(baseVote);
          this.voteDispatcher.dispatch(receivers, timeoutVote);
        });
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

  private Optional<Proposal> generateProposal(Round round) {
    final var highQC = this.latestRoundUpdate.getHighQC();
    final var highestQC = highQC.highestQC();
    final var nextTransactions = getTransactionsForProposal(round, highestQC);
    final var proposerTimestamp = determineNextProposalTimestamp(highestQC);
    final var proposedVertex =
        Vertex.create(highestQC, round, nextTransactions, self, proposerTimestamp).withId(hasher);
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
  public void preProcessUnsyncedVoteForCurrentOrFutureRound(Vote vote) {
    if (vote.highQC().getHighestRound().gt(this.highestKnownQcRound)) {
      this.highestKnownQcRound = vote.highQC().getHighestRound();
    }
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
