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

import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.RoundLeaderFailure;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.liveness.PacemakerTimeoutCalculator;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.Metrics;
import java.util.HashSet;
import java.util.Set;

/**
 * Moderates the round timeout. More specifically, extends the round duration if a proposal was
 * received, but takes longer than usual to sync/process.
 */
public final class RoundTimeoutModerator implements BFTEventProcessorAtCurrentRound {
  private final BFTEventProcessor forwardTo;
  private final PacemakerTimeoutCalculator timeoutCalculator;
  private final ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutDispatcher;
  private final Metrics metrics;

  private RoundUpdate latestRoundUpdate;

  private final Set<Round> receivedProposalsForFutureRounds;
  private Round highestReceivedRoundQC;
  private boolean currentRoundTimeoutCanStillBeExtended = true;

  public RoundTimeoutModerator(
      BFTEventProcessor forwardTo,
      PacemakerTimeoutCalculator timeoutCalculator,
      ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutDispatcher,
      Metrics metrics,
      RoundUpdate initialRoundUpdate) {
    this.forwardTo = forwardTo;
    this.timeoutCalculator = timeoutCalculator;
    this.timeoutDispatcher = timeoutDispatcher;
    this.metrics = metrics;
    this.latestRoundUpdate = initialRoundUpdate;
    this.highestReceivedRoundQC = initialRoundUpdate.getCurrentRound();
    this.receivedProposalsForFutureRounds = new HashSet<>();
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    forwardTo.processBFTUpdate(update);
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    this.latestRoundUpdate = roundUpdate;
    if (this.latestRoundUpdate.getCurrentRound().gt(this.highestReceivedRoundQC)) {
      this.highestReceivedRoundQC = roundUpdate.getCurrentRound();
    }
    this.receivedProposalsForFutureRounds.removeIf(p -> p.lt(roundUpdate.getCurrentRound()));
    this.currentRoundTimeoutCanStillBeExtended = true;
    forwardTo.processRoundUpdate(roundUpdate);
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate update) {
    forwardTo.processBFTRebuildUpdate(update);
  }

  @Override
  public void processVote(Vote vote) {
    forwardTo.processVote(vote);
  }

  @Override
  public void processProposal(Proposal proposal) {
    forwardTo.processProposal(proposal);
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledLocalTimeout) {
    /* The timeouts for the current round can be extended in three cases:
    1) we have already received a QC for this round, which hasn't yet been synced-up to:
       this prevents us from sending timeout votes for rounds that already have a QC,
       and potentially avoids creating a competing TC for the same round.
    2) we extend it for round N if we have received a proposal for round N to give us time to process the proposal
    3) we extend it for round N if we have received a proposal for round N + 1 to give us time to sync up the QC for round N (if it exists)
    The 3rd case really duplicates the "received QC" condition (as proposal for round N+1 should contain a QC for round N),
    but adding it explicitly for completeness. */

    final var receivedAnyQcForThisOrHigherRound =
        this.highestReceivedRoundQC.gte(scheduledLocalTimeout.round());

    final var receivedProposalForThisOrNextRound =
        this.receivedProposalsForFutureRounds.contains(scheduledLocalTimeout.round())
            || this.receivedProposalsForFutureRounds.contains(scheduledLocalTimeout.round().next());

    final var additionalTime = timeoutCalculator.additionalRoundTimeIfProposalReceivedMs();

    final var shouldExtendTheTimeout =
        (receivedAnyQcForThisOrHigherRound || receivedProposalForThisOrNextRound)
            && this.currentRoundTimeoutCanStillBeExtended
            && additionalTime > 0;

    // We won't extend the round more than once or if a timeout event has already been processed,
    // so setting a flag regardless of whether this timeout event is being delayed or not
    this.currentRoundTimeoutCanStillBeExtended = false;

    if (shouldExtendTheTimeout) {
      // If proposal was received, extend the round time by re-dispatching the same timeout event,
      // effectively delaying it by additionalRoundTimeIfProposalReceived
      metrics.bft().extendedRoundTimeouts().inc();
      this.timeoutDispatcher.dispatch(scheduledLocalTimeout, additionalTime);
    } else {
      // Nothing we can do. Either there isn't an in-flight proposal for the
      // current round (so there's no reason to extend the round) or it has already been extended.
      this.forwardTo.processLocalTimeout(scheduledLocalTimeout);
    }
  }

  @Override
  public void processRoundLeaderFailure(RoundLeaderFailure roundLeaderFailure) {
    forwardTo.processRoundLeaderFailure(roundLeaderFailure);
  }

  @Override
  public void start() {
    forwardTo.start();
  }

  @Override
  public void preProcessUnsyncedVoteForCurrentOrFutureRound(Vote vote) {
    if (vote.highQC().getHighestRound().gt(this.highestReceivedRoundQC)) {
      this.highestReceivedRoundQC = vote.highQC().getHighestRound();
    }
  }

  @Override
  public void preProcessUnsyncedProposalForCurrentOrFutureRound(Proposal proposal) {
    receivedProposalsForFutureRounds.add(proposal.getRound());
    if (proposal.highQC().getHighestRound().gt(this.highestReceivedRoundQC)) {
      this.highestReceivedRoundQC = proposal.highQC().getHighestRound();
    }
  }
}
