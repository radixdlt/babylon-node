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

package com.radixdlt.consensus.bft;

import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.processor.*;
import com.radixdlt.consensus.liveness.Pacemaker;
import com.radixdlt.consensus.liveness.PacemakerTimeoutCalculator;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.utils.TimeSupplier;

/** A helper class to help in constructing a BFT validator state machine */
public final class BFTBuilder {
  // BFT Configuration objects
  private BFTValidatorSet validatorSet;
  private Hasher hasher;
  private HashVerifier verifier;
  private PacemakerTimeoutCalculator timeoutCalculator;
  private ProposerElection proposerElection;

  // BFT Stateful objects
  private Pacemaker pacemaker;
  private VertexStoreAdapter vertexStore;
  private BFTSyncer bftSyncer;
  private EventDispatcher<RoundQuorumReached> roundQuorumReachedEventDispatcher;
  private EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher;
  private EventDispatcher<NoVote> noVoteEventDispatcher;
  private EventDispatcher<RoundLeaderFailure> roundLeaderFailureEventDispatcher;
  private ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutDispatcher;

  // Instance specific objects
  private BFTValidatorId self;

  private RoundUpdate roundUpdate;
  private RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher;
  private SafetyRules safetyRules;

  private TimeSupplier timeSupplier;
  private Metrics metrics;

  private BFTBuilder() {
    // Just making this inaccessible
  }

  public static BFTBuilder create() {
    return new BFTBuilder();
  }

  public BFTBuilder self(BFTValidatorId self) {
    this.self = self;
    return this;
  }

  public BFTBuilder roundUpdate(RoundUpdate roundUpdate) {
    this.roundUpdate = roundUpdate;
    return this;
  }

  public BFTBuilder timeoutCalculator(PacemakerTimeoutCalculator timeoutCalculator) {
    this.timeoutCalculator = timeoutCalculator;
    return this;
  }

  public BFTBuilder timeoutDispatcher(
      ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutDispatcher) {
    this.timeoutDispatcher = timeoutDispatcher;
    return this;
  }

  public BFTBuilder proposerElection(ProposerElection proposerElection) {
    this.proposerElection = proposerElection;
    return this;
  }

  public BFTBuilder voteDispatcher(RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher) {
    this.voteDispatcher = voteDispatcher;
    return this;
  }

  public BFTBuilder roundLeaderFailureEventDispatcher(
      EventDispatcher<RoundLeaderFailure> roundLeaderFailureEventDispatcher) {
    this.roundLeaderFailureEventDispatcher = roundLeaderFailureEventDispatcher;
    return this;
  }

  public BFTBuilder doubleVoteEventDispatcher(
      EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher) {
    this.doubleVoteEventDispatcher = doubleVoteEventDispatcher;
    return this;
  }

  public BFTBuilder safetyRules(SafetyRules safetyRules) {
    this.safetyRules = safetyRules;
    return this;
  }

  public BFTBuilder hasher(Hasher hasher) {
    this.hasher = hasher;
    return this;
  }

  public BFTBuilder verifier(HashVerifier verifier) {
    this.verifier = verifier;
    return this;
  }

  public BFTBuilder validatorSet(BFTValidatorSet validatorSet) {
    this.validatorSet = validatorSet;
    return this;
  }

  public BFTBuilder pacemaker(Pacemaker pacemaker) {
    this.pacemaker = pacemaker;
    return this;
  }

  public BFTBuilder vertexStore(VertexStoreAdapter vertexStore) {
    this.vertexStore = vertexStore;
    return this;
  }

  public BFTBuilder bftSyncer(BFTSyncer bftSyncer) {
    this.bftSyncer = bftSyncer;
    return this;
  }

  public BFTBuilder timeSupplier(TimeSupplier timeSupplier) {
    this.timeSupplier = timeSupplier;
    return this;
  }

  public BFTBuilder roundQuorumReachedEventDispatcher(
      EventDispatcher<RoundQuorumReached> roundQuorumReachedEventDispatcher) {
    this.roundQuorumReachedEventDispatcher = roundQuorumReachedEventDispatcher;
    return this;
  }

  public BFTBuilder noVoteEventDispatcher(EventDispatcher<NoVote> noVoteEventDispatcher) {
    this.noVoteEventDispatcher = noVoteEventDispatcher;
    return this;
  }

  public BFTBuilder metrics(Metrics metrics) {
    this.metrics = metrics;
    return this;
  }

  public BFTEventProcessor build() {
    if (!validatorSet.containsNode(self)) {
      return EmptyBFTEventProcessor.INSTANCE;
    }
    final PendingVotes pendingVotes = new PendingVotes(hasher, doubleVoteEventDispatcher);

    /* Setting up the following BFT event processing pipeline:
    BFTEventStatelessVerifier (verify against stateless parameters [e.g. validator set, round leader] and the signatures)
       -> OneProposalPerRoundVerifier (verify that max 1 genuine proposal is received for each round)
       -> SyncUpPreprocessor (if needed, sync up to match BFT event's round)
       -> BFTEventPostSyncUpVerifier (verifies that we've synced up to a correct round)
       -> RoundTimeoutModerator (may modify round timeouts)
       -> ProposalTimestampVerifier (verify proposal timestamp)
       -> BFTEventReducer (actually process the event) */

    final var bftEventReducer =
        new BFTEventReducer(
            self,
            pacemaker,
            vertexStore,
            roundQuorumReachedEventDispatcher,
            noVoteEventDispatcher,
            voteDispatcher,
            hasher,
            metrics,
            safetyRules,
            validatorSet,
            pendingVotes,
            roundUpdate);

    final var proposalTimestampVerifier =
        new ProposalTimestampVerifier(
            bftEventReducer, timeSupplier, metrics, roundLeaderFailureEventDispatcher);

    final var roundTimeoutModerator =
        new RoundTimeoutModerator(
            proposalTimestampVerifier, timeoutCalculator, timeoutDispatcher, metrics, roundUpdate);

    final var postSyncUpVerifier =
        new BFTEventPostSyncUpVerifier(roundTimeoutModerator, metrics, roundUpdate);

    final var syncUpPreprocessor =
        new SyncUpPreprocessor(postSyncUpVerifier, bftSyncer, metrics, roundUpdate);

    final var oneProposalPerRoundVerifier =
        new OneProposalPerRoundVerifier(syncUpPreprocessor, metrics);

    return new BFTEventStatelessVerifier(
        validatorSet,
        proposerElection,
        oneProposalPerRoundVerifier,
        hasher,
        verifier,
        safetyRules,
        metrics);
  }
}
