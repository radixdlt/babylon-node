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
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.NoVote;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.RoundLeaderFailure;
import com.radixdlt.consensus.bft.RoundQuorumReached;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.bft.VertexStoreAdapter;
import com.radixdlt.consensus.bft.VoteProcessingResult.*;
import com.radixdlt.consensus.liveness.Pacemaker;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.monitoring.SystemCounters;
import java.util.Objects;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Processes and reduces BFT events to the BFT state based on core BFT validation logic, any
 * messages which must be sent to other nodes are then forwarded to the BFT sender.
 */
// TODO: cleanup TODOs https://radixdlt.atlassian.net/browse/NT-7
public final class BFTEventReducer implements BFTEventProcessor {
  private static final Logger log = LogManager.getLogger();

  private final BFTNode self;
  private final VertexStoreAdapter vertexStore;
  private final Pacemaker pacemaker;
  private final EventDispatcher<RoundQuorumReached> roundQuorumReachedEventDispatcher;
  private final EventDispatcher<NoVote> noVoteDispatcher;
  private final RemoteEventDispatcher<Vote> voteDispatcher;
  private final Hasher hasher;
  private final SystemCounters systemCounters;
  private final SafetyRules safetyRules;
  private final BFTValidatorSet validatorSet;
  private final PendingVotes pendingVotes;

  private BFTInsertUpdate latestInsertUpdate;
  private RoundUpdate latestRoundUpdate;

  /* Indicates whether the quorum (QC or TC) has already been formed for the current round.
   * If the quorum has been reached (but round hasn't yet been updated), subsequent votes are ignored.
   * TODO: consider moving it to PendingVotes or elsewhere.
   */
  private boolean hasReachedQuorum = false;

  private boolean hasLeaderFailedTheRound = false;

  public BFTEventReducer(
      BFTNode self,
      Pacemaker pacemaker,
      VertexStoreAdapter vertexStore,
      EventDispatcher<RoundQuorumReached> roundQuorumReachedEventDispatcher,
      EventDispatcher<NoVote> noVoteDispatcher,
      RemoteEventDispatcher<Vote> voteDispatcher,
      Hasher hasher,
      SystemCounters systemCounters,
      SafetyRules safetyRules,
      BFTValidatorSet validatorSet,
      PendingVotes pendingVotes,
      RoundUpdate initialRoundUpdate) {
    this.self = Objects.requireNonNull(self);
    this.pacemaker = Objects.requireNonNull(pacemaker);
    this.vertexStore = Objects.requireNonNull(vertexStore);
    this.roundQuorumReachedEventDispatcher =
        Objects.requireNonNull(roundQuorumReachedEventDispatcher);
    this.noVoteDispatcher = Objects.requireNonNull(noVoteDispatcher);
    this.voteDispatcher = Objects.requireNonNull(voteDispatcher);
    this.hasher = Objects.requireNonNull(hasher);
    this.systemCounters = Objects.requireNonNull(systemCounters);
    this.safetyRules = Objects.requireNonNull(safetyRules);
    this.validatorSet = Objects.requireNonNull(validatorSet);
    this.pendingVotes = Objects.requireNonNull(pendingVotes);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    log.trace("BFTUpdate: Processing {}", update);

    final Round round = update.getHeader().getRound();
    if (round.lt(this.latestRoundUpdate.getCurrentRound())) {
      log.trace(
          "InsertUpdate: Ignoring insert {} for round {}, current round at {}",
          update,
          round,
          this.latestRoundUpdate.getCurrentRound());
      return;
    }

    this.latestInsertUpdate = update;
    this.tryVote();

    this.pacemaker.processBFTUpdate(update);
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    this.hasReachedQuorum = false;
    this.hasLeaderFailedTheRound = false;
    this.latestRoundUpdate = roundUpdate;
    this.pacemaker.processRoundUpdate(roundUpdate);
    this.tryVote();
  }

  private void tryVote() {
    BFTInsertUpdate update = this.latestInsertUpdate;
    if (update == null) {
      return;
    }

    if (!Objects.equals(update.getHeader().getRound(), this.latestRoundUpdate.getCurrentRound())) {
      return;
    }

    // Check if already voted in this round
    if (this.safetyRules.getLastVote(this.latestRoundUpdate.getCurrentRound()).isPresent()) {
      return;
    }

    // Don't vote if the leader has already failed their round
    if (this.hasLeaderFailedTheRound) {
      return;
    }

    // TODO: what if insertUpdate occurs before roundUpdate
    final BFTNode nextLeader = this.latestRoundUpdate.getNextLeader();
    final Optional<Vote> maybeVote =
        this.safetyRules.createVote(
            update.getInserted().getVertexWithHash(),
            update.getHeader(),
            update.getInserted().getTimeOfExecution(),
            this.latestRoundUpdate.getHighQC());
    maybeVote.ifPresentOrElse(
        vote -> this.voteDispatcher.dispatch(nextLeader, vote),
        () ->
            this.noVoteDispatcher.dispatch(
                NoVote.create(update.getInserted().getVertexWithHash())));
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate update) {
    // No-op
  }

  @Override
  public void processVote(Vote vote) {
    log.trace("Vote: Processing {}", vote);

    final Round round = vote.getRound();

    if (this.hasReachedQuorum) {
      log.trace(
          "Vote: Ignoring vote from {} for round {}, quorum has already been reached",
          vote.getAuthor(),
          round);
      return;
    }

    if (!this.self.equals(this.latestRoundUpdate.getNextLeader()) && !vote.isTimeout()) {
      log.trace(
          "Vote: Ignoring vote from {} for round {}, unexpected vote", vote.getAuthor(), round);
      return;
    }

    switch (this.pendingVotes.insertVote(vote, this.validatorSet)) {
      case VoteAccepted ignored -> log.trace("Vote has been processed but didn't form a quorum");
      case VoteRejected voteRejected -> log.trace(
          "Vote has been rejected because of: {}", voteRejected.getReason());
      case QuorumReached quorumReached -> {
        this.hasReachedQuorum = true;
        roundQuorumReachedEventDispatcher.dispatch(
            new RoundQuorumReached(quorumReached.getRoundVotingResult(), vote.getAuthor()));
      }
    }

    systemCounters.increment(SystemCounters.CounterType.BFT_SUCCESSFULLY_PROCESSED_VOTES);
  }

  @Override
  public void processProposal(Proposal proposal) {
    log.trace("Proposal: Processing {}", proposal);

    // TODO: Move insertion and maybe check into BFTSync
    final var proposedVertex = proposal.getVertex().withId(hasher);
    this.vertexStore.insertVertex(proposedVertex);

    systemCounters.increment(SystemCounters.CounterType.BFT_SUCCESSFULLY_PROCESSED_PROPOSALS);
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledLocalTimeout) {
    this.hasLeaderFailedTheRound = true;
    this.pacemaker.processLocalTimeout(scheduledLocalTimeout);
  }

  @Override
  public void processRoundLeaderFailure(RoundLeaderFailure roundLeaderFailure) {
    this.hasLeaderFailedTheRound = true;
    this.pacemaker.processRoundLeaderFailure(roundLeaderFailure);
  }

  @Override
  public void start() {
    this.pacemaker.start();
  }
}
