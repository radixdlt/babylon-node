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
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.TimeSupplier;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Manages the pacemaker driver. */
public final class Pacemaker {
  private static final Logger log = LogManager.getLogger();

  private enum RoundStatus {
    UNDISTURBED, // All good, still accepting proposals
    PROPOSAL_REJECTED, // A genuine proposal was received, but it has been rejected
    TIMED_OUT // The round has timed out
  }

  private static final int PREVIOUS_ROUND_RUSHING_TIMESTAMP_LOG_THRESHOLD_MS = 1000;

  private final BFTNode self;
  private final BFTValidatorSet validatorSet;
  private final VertexStoreAdapter vertexStore;
  private final SafetyRules safetyRules;
  private final ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender;
  private final PacemakerTimeoutCalculator timeoutCalculator;
  private final ProposalGenerator proposalGenerator;
  private final Hasher hasher;
  private final RemoteEventDispatcher<Proposal> proposalDispatcher;
  private final RemoteEventDispatcher<Vote> voteDispatcher;
  private final EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher;
  private final EventDispatcher<RoundLeaderFailure> roundLeaderFailureDispatcher;
  private final TimeSupplier timeSupplier;
  private final SystemCounters systemCounters;

  private RoundUpdate latestRoundUpdate;
  private RoundStatus roundStatus = RoundStatus.UNDISTURBED;

  // ID of the vertex to be used as a substitute for the vertex proposed by
  // the current leader (either because we didn't receive it or it was rejected).
  // The vertex isn't created until it's actually needed (hence Optional).
  private Optional<HashCode> vertexIdForLeaderFailure = Optional.empty();

  public Pacemaker(
      BFTNode self,
      BFTValidatorSet validatorSet,
      VertexStoreAdapter vertexStore,
      SafetyRules safetyRules,
      EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher,
      ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender,
      PacemakerTimeoutCalculator timeoutCalculator,
      ProposalGenerator proposalGenerator,
      RemoteEventDispatcher<Proposal> proposalDispatcher,
      RemoteEventDispatcher<Vote> voteDispatcher,
      EventDispatcher<RoundLeaderFailure> roundLeaderFailureDispatcher,
      Hasher hasher,
      TimeSupplier timeSupplier,
      RoundUpdate initialRoundUpdate,
      SystemCounters systemCounters) {
    this.self = Objects.requireNonNull(self);
    this.validatorSet = Objects.requireNonNull(validatorSet);
    this.vertexStore = Objects.requireNonNull(vertexStore);
    this.safetyRules = Objects.requireNonNull(safetyRules);
    this.timeoutSender = Objects.requireNonNull(timeoutSender);
    this.timeoutDispatcher = Objects.requireNonNull(timeoutDispatcher);
    this.timeoutCalculator = Objects.requireNonNull(timeoutCalculator);
    this.proposalGenerator = Objects.requireNonNull(proposalGenerator);
    this.proposalDispatcher = Objects.requireNonNull(proposalDispatcher);
    this.voteDispatcher = Objects.requireNonNull(voteDispatcher);
    this.roundLeaderFailureDispatcher = Objects.requireNonNull(roundLeaderFailureDispatcher);
    this.hasher = Objects.requireNonNull(hasher);
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
    this.systemCounters = Objects.requireNonNull(systemCounters);
  }

  public void start() {
    log.info("Pacemaker Start: {}", latestRoundUpdate);
    this.startRound();
  }

  /** Processes a local round update message * */
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    log.trace("Round Update: {}", roundUpdate);

    final var previousRound = this.latestRoundUpdate.getCurrentRound();
    if (roundUpdate.getCurrentRound().lte(previousRound)) {
      // This shouldn't really happen but ignore any outdated updates
      systemCounters.bft().preconditionViolations().inc();
      return;
    }

    this.latestRoundUpdate = roundUpdate;
    this.systemCounters.bft().pacemaker().round().set(roundUpdate.getCurrentRound().number());

    this.startRound();
  }

  /** Processes a local BFTInsertUpdate message */
  public void processBFTUpdate(BFTInsertUpdate update) {
    final var updateIsInsertionOfLeaderFailureVertex =
        this.vertexIdForLeaderFailure
            .filter(update.getInserted().getVertexHash()::equals)
            .isPresent();

    // The Pacemaker only processes the insertion of a leader failure vertex,
    // which should have been (asynchronously) initialized earlier.
    // No other vertices are of interest here, so they're ignored.
    if (!updateIsInsertionOfLeaderFailureVertex) {
      return;
    }

    // Continue the "leader failure" process from where it left off when requesting an (async)
    // vertex insertion
    switch (this.roundStatus) {
      case UNDISTURBED -> {} // no-op
      case PROPOSAL_REJECTED ->
      // We have received a rejected proposal, send a timeout vote to the next leader only
      createAndSendTimeoutVote(
          update.getInserted(), ImmutableSet.of(this.latestRoundUpdate.getNextLeader()));
      case TIMED_OUT ->
      // The round has timed out for real, send a timeout vote to everyone
      this.createAndSendTimeoutVoteToAll(update.getInserted());
    }
  }

  private void startRound() {
    this.roundStatus = RoundStatus.UNDISTURBED;
    this.vertexIdForLeaderFailure = Optional.empty();

    final var timeoutMs =
        timeoutCalculator.calculateTimeoutMs(latestRoundUpdate.consecutiveUncommittedRoundsCount());
    final var timeoutEvent = ScheduledLocalTimeout.create(latestRoundUpdate, timeoutMs);
    this.timeoutSender.dispatch(timeoutEvent, timeoutMs);

    final var currentRoundProposer = latestRoundUpdate.getLeader();
    if (this.self.equals(currentRoundProposer)) {
      final var maybeProposal = generateProposal(latestRoundUpdate.getCurrentRound());
      maybeProposal.ifPresent(
          proposal -> {
            log.trace("Broadcasting proposal: {}", proposal);
            this.proposalDispatcher.dispatch(this.validatorSet.nodes(), proposal);
            this.systemCounters.bft().pacemaker().proposalsSent().inc();
          });
    }
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
      this.systemCounters.bft().pacemaker().proposalsWithSubstituteTimestamp().inc();
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
    this.systemCounters.bft().pacemaker().proposedTransactions().inc(nextTransactions.size());
    return nextTransactions;
  }

  /**
   * Processes a local timeout, causing the pacemaker to either broadcast previously sent vote to
   * all nodes or broadcast a new vote for a "null" proposal. In either case, the sent vote includes
   * a timeout signature, which can later be used to form a timeout certificate.
   */
  public void processLocalTimeout(ScheduledLocalTimeout scheduledTimeout) {
    final var prevStatus = this.roundStatus;
    this.roundStatus = RoundStatus.TIMED_OUT;

    // Dispatch RoundLeaderFailure event if the round hasn't failed so far
    if (prevStatus == RoundStatus.UNDISTURBED) {
      roundLeaderFailureDispatcher.dispatch(
          new RoundLeaderFailure(
              this.latestRoundUpdate.getCurrentRound(), RoundLeaderFailureReason.ROUND_TIMEOUT));
    }

    updateTimeoutCounters(scheduledTimeout);

    resendPreviousOrNewVoteWithTimeout(this.validatorSet.nodes());

    // Dispatch an actual timeout occurrence
    this.timeoutDispatcher.dispatch(new LocalTimeoutOccurrence(scheduledTimeout));

    rescheduleTimeout(scheduledTimeout);
  }

  private void resendPreviousOrNewVoteWithTimeout(ImmutableSet<BFTNode> receivers) {
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

  public void processRoundLeaderFailure(RoundLeaderFailure roundLeaderFailure) {
    switch (roundLeaderFailure.reason()) {
      case ROUND_TIMEOUT -> {
        /* nothing to do here, timeouts are handled within their own events */
      }
      case PROPOSED_TIMESTAMP_UNACCEPTABLE -> {
        if (this.roundStatus != RoundStatus.UNDISTURBED) {
          // No-op if the round is already disturbed
          return;
        }

        this.roundStatus = RoundStatus.PROPOSAL_REJECTED;

        // Send a single timeout vote to the next leader
        resendPreviousOrNewVoteWithTimeout(ImmutableSet.of(this.latestRoundUpdate.getNextLeader()));
      }
    }
  }

  private void createTimeoutVertexAndSendVote(
      RoundUpdate roundUpdate, ImmutableSet<BFTNode> receivers) {
    if (this.vertexIdForLeaderFailure.isPresent()) {
      // The "leader failure" vertex for this round is already being inserted (or has already been
      // inserted)
      return;
    }

    final var highQC = this.latestRoundUpdate.getHighQC();
    final var vertex =
        Vertex.createTimeout(
                highQC.highestQC(), roundUpdate.getCurrentRound(), roundUpdate.getLeader())
            .withId(hasher);
    this.vertexIdForLeaderFailure = Optional.of(vertex.hash());

    // TODO: reimplement in async way
    this.vertexStore
        .getExecutedVertex(vertex.hash())
        .ifPresentOrElse(
            v ->
                createAndSendTimeoutVote(
                    v,
                    receivers), // the vertex was already present in the vertex store, send the vote
            // immediately
            () -> maybeInsertVertex(vertex) // otherwise insert and wait for async bft update event
            );
  }

  // FIXME: This is a temporary fix so that we can continue
  // if the vertex store is too far ahead of the pacemaker
  private void maybeInsertVertex(VertexWithHash vertexWithHash) {
    try {
      this.vertexStore.insertVertex(vertexWithHash);
    } catch (MissingParentException e) {
      log.debug("Could not insert timeout vertex: {}", e.getMessage());
    }
  }

  private void createAndSendTimeoutVoteToAll(ExecutedVertex executedVertex) {
    createAndSendTimeoutVote(executedVertex, this.validatorSet.nodes());
  }

  private void createAndSendTimeoutVote(
      ExecutedVertex executedVertex, ImmutableSet<BFTNode> receivers) {
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
      systemCounters.bft().pacemaker().timedOutRounds().inc();
    }
    systemCounters.bft().pacemaker().timeoutsSent().inc();
  }

  private void rescheduleTimeout(ScheduledLocalTimeout scheduledTimeout) {
    final var timeout =
        timeoutCalculator.calculateTimeoutMs(latestRoundUpdate.consecutiveUncommittedRoundsCount());
    final var nextTimeout = scheduledTimeout.nextRetry(timeout);
    this.timeoutSender.dispatch(nextTimeout, timeout);
  }
}
