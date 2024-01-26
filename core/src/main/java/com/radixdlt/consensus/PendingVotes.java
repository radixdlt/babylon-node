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

package com.radixdlt.consensus;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.Maps;
import com.google.common.hash.HashCode;
import com.radixdlt.SecurityCritical;
import com.radixdlt.SecurityCritical.SecurityKind;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.ValidationState;
import com.radixdlt.consensus.bft.VoteProcessingResult;
import com.radixdlt.consensus.bft.VoteProcessingResult.VoteRejected.VoteRejectedReason;
import com.radixdlt.consensus.liveness.VoteTimeout;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.monitoring.Metrics;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import javax.annotation.concurrent.NotThreadSafe;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Manages pending votes for various vertices.
 *
 * <p>This class is NOT thread-safe.
 *
 * <p>This class is security critical (signature checks, validator set membership checks).
 */
@NotThreadSafe
@SecurityCritical({SecurityKind.SIG_VERIFY, SecurityKind.GENERAL})
public final class PendingVotes {
  private static final Logger log = LogManager.getLogger();

  private final Map<HashCode, ValidationState> voteState = Maps.newHashMap();
  private final Map<HashCode, ValidationState> timeoutVoteState = Maps.newHashMap();
  private final Map<BFTValidatorId, PreviousVote> previousVotes = Maps.newHashMap();
  private final Hasher hasher;
  private final EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher;
  private final BFTValidatorSet validatorSet;
  private final Metrics metrics;

  public PendingVotes(
      Hasher hasher,
      EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher,
      BFTValidatorSet validatorSet,
      Metrics metrics) {
    this.hasher = Objects.requireNonNull(hasher);
    this.doubleVoteEventDispatcher = Objects.requireNonNull(doubleVoteEventDispatcher);
    this.validatorSet = Objects.requireNonNull(validatorSet);
    this.metrics = metrics;
  }

  private void checkForDivergentVertexExecution(Vote vote) {
    final var voteVertexId = vote.getVoteData().getProposed().getVertexId();
    final var voteLedgerHeader = vote.getVoteData().getProposed().getLedgerHeader();
    for (var otherVote : this.previousVotes.entrySet()) {
      final var otherVertexId = otherVote.getValue().proposedHeader().getVertexId();
      final var otherLedgerHeader = otherVote.getValue().proposedHeader().getLedgerHeader();
      if (voteVertexId.equals(otherVertexId) && !voteLedgerHeader.equals(otherLedgerHeader)) {
        if (!voteLedgerHeader
            .nextProtocolVersion()
            .equals(otherLedgerHeader.nextProtocolVersion())) {
          log.info(
              """
              Received BFT votes for conflicting protocol update headers.
              Validator {} enacts {}, while validator {} enacts {}.
              This indicates that one or more validators run an outdated node version.
              This node is unaffected, unless other error messages follow.""",
              vote.getAuthor(),
              voteLedgerHeader.nextProtocolVersion(),
              otherVote.getKey(),
              otherLedgerHeader.nextProtocolVersion());
        } else {
          log.warn(
              "Divergent vertex execution detected! An incoming vote (from {}) for vertex {} claims"
                  + " the following resultant ledger header: {}, while validator {} thinks that the"
                  + " resultant ledger header is {}. [this_vote={}, other_vote={}]",
              vote.getAuthor(),
              voteVertexId,
              voteLedgerHeader,
              otherVote.getKey(),
              otherLedgerHeader,
              vote,
              otherVote);
        }
        this.metrics.bft().divergentVertexExecutions().inc();
      }
    }
  }

  /**
   * Inserts a vote for a given vertex, attempting to form either a quorum certificate for that
   * vertex or a timeout certificate. A quorum will only be formed if permitted by the {@link
   * BFTValidatorSet}.
   *
   * @param vote The vote to be inserted
   * @return The result of vote processing
   */
  public VoteProcessingResult insertVote(Vote vote) {
    final BFTValidatorId author = vote.getAuthor();
    final VoteData voteData = vote.getVoteData();
    final HashCode voteDataHash = this.hasher.hashDsonEncoded(voteData);

    if (!validatorSet.containsValidator(author)) {
      return VoteProcessingResult.rejected(VoteRejectedReason.INVALID_AUTHOR);
    }

    // This doesn't do anything, other than logging and bumping the metrics,
    // when divergent execution is detected (which should hopefully never happen).
    checkForDivergentVertexExecution(vote);

    if (!replacePreviousVote(author, vote, voteDataHash)) {
      return VoteProcessingResult.rejected(VoteRejectedReason.DUPLICATE_VOTE);
    }

    return processVoteForQC(vote)
        .<VoteProcessingResult>map(VoteProcessingResult::regularQuorum)
        .or(() -> processVoteForTC(vote).map(VoteProcessingResult::timeoutQuorum))
        .orElseGet(VoteProcessingResult::accepted);
  }

  private Optional<QuorumCertificate> processVoteForQC(Vote vote) {
    final VoteData voteData = vote.getVoteData();
    final HashCode voteDataHash = this.hasher.hashDsonEncoded(voteData);
    final BFTValidatorId author = vote.getAuthor();

    final ValidationState validationState =
        this.voteState.computeIfAbsent(voteDataHash, k -> validatorSet.newValidationState());

    final boolean signatureAdded =
        validationState.addSignature(author, vote.getTimestamp(), vote.getSignature());

    if (signatureAdded && validationState.complete()) {
      return Optional.of(new QuorumCertificate(voteData, validationState.signatures()));
    } else {
      return Optional.empty();
    }
  }

  private Optional<TimeoutCertificate> processVoteForTC(Vote vote) {
    if (!vote.isTimeout()) {
      return Optional.empty(); // TC can't be formed if vote is not timed out
    }

    final ECDSASecp256k1Signature timeoutSignature = vote.getTimeoutSignature().orElseThrow();

    final VoteTimeout voteTimeout = VoteTimeout.of(vote);
    final HashCode voteTimeoutHash = this.hasher.hashDsonEncoded(voteTimeout);
    final BFTValidatorId author = vote.getAuthor();

    final ValidationState validationState =
        this.timeoutVoteState.computeIfAbsent(
            voteTimeoutHash, k -> validatorSet.newValidationState());

    final boolean signatureAdded =
        validationState.addSignature(author, vote.getTimestamp(), timeoutSignature);

    if (signatureAdded && validationState.complete()) {
      return Optional.of(
          new TimeoutCertificate(
              voteTimeout.getEpoch(), voteTimeout.getRound(), validationState.signatures()));
    } else {
      return Optional.empty();
    }
  }

  // TODO: Need to rethink whether we should be removing previous signature
  // TODO: Could be causing quorum formation to slow down
  private boolean replacePreviousVote(BFTValidatorId author, Vote vote, HashCode voteHash) {
    final PreviousVote thisVote =
        new PreviousVote(
            vote.getRound(),
            vote.getEpoch(),
            voteHash,
            vote.isTimeout(),
            vote.getVoteData().getProposed());
    final PreviousVote previousVote = this.previousVotes.put(author, thisVote);
    if (previousVote == null) {
      // No previous vote for this author, all good here
      return true;
    }

    if (thisVote.equals(previousVote)) {
      // Just going to ignore this duplicate vote for now.
      // However, we can't count duplicate votes multiple times.
      return false;
    }

    // Prune last pending vote from the pending votes.
    // This limits the number of pending vertices that are in the pipeline.
    var validationState = this.voteState.get(previousVote.hash());
    if (validationState != null) {
      validationState.removeSignature(author);
      if (validationState.isEmpty()) {
        this.voteState.remove(previousVote.hash());
      }
    }

    if (previousVote.isTimeout()) {
      final var voteTimeout = new VoteTimeout(previousVote.round(), previousVote.epoch());
      final var voteTimeoutHash = this.hasher.hashDsonEncoded(voteTimeout);

      var timeoutValidationState = this.timeoutVoteState.get(voteTimeoutHash);
      if (timeoutValidationState != null) {
        timeoutValidationState.removeSignature(author);
        if (timeoutValidationState.isEmpty()) {
          this.timeoutVoteState.remove(voteTimeoutHash);
        }
      }
    }

    if (vote.getRound().equals(previousVote.round())) {
      // If the validator already voted in this round for something else,
      // then the only valid possibility is a non-timeout vote being replaced by a timeout vote
      // on the same vote data, or a byzantine validator

      var isValidVote = thisVote.hash().equals(previousVote.hash());
      if (!isValidVote) {
        this.doubleVoteEventDispatcher.dispatch(
            new ConsensusByzantineEvent.DoubleVote(author, previousVote, vote, thisVote.hash()));
      }
      return isValidVote && !previousVote.isTimeout() && vote.isTimeout();
    } else {
      // all good if vote is for a different round
      return true;
    }
  }

  @VisibleForTesting
  // Greybox stuff for testing
  int voteStateSize() {
    return this.voteState.size();
  }

  @VisibleForTesting
  // Greybox stuff for testing
  int timeoutVoteStateSize() {
    return this.timeoutVoteState.size();
  }

  @VisibleForTesting
  // Greybox stuff for testing
  int previousVotesSize() {
    return this.previousVotes.size();
  }
}
