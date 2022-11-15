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

import com.google.common.hash.HashCode;
import com.radixdlt.SecurityCritical;
import com.radixdlt.SecurityCritical.SecurityKind;
import com.radixdlt.consensus.BFTEventProcessor;
import com.radixdlt.consensus.ConsensusEvent;
import com.radixdlt.consensus.HashVerifier;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.liveness.VoteTimeout;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.utils.TimeSupplier;
import java.util.Objects;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Verifies signatures of BFT messages then forwards to the next processor */
@SecurityCritical({SecurityKind.SIG_VERIFY})
public final class BFTEventVerifier implements BFTEventProcessor {
  private static final Logger log = LogManager.getLogger();

  /* These two constants specify the bounds for acceptable proposal timestamps in relation to
  the current time (id est, the system time when the proposal message starts being processed).
  Proposals that fall out of bounds are rejected (ignored).
  The acceptable delay is slightly higher to account for network latency.
  In addition to the delay/rush bounds, the proposal timestamp must be strictly
  greater than the previous one (see `isProposalTimestampAcceptable`). */
  private static final long MAX_ACCEPTABLE_PROPOSAL_TIMESTAMP_DELAY_MS = 3000;
  private static final long MAX_ACCEPTABLE_PROPOSAL_TIMESTAMP_RUSH_MS = 2000;

  private final BFTValidatorSet validatorSet;
  private final BFTEventProcessor forwardTo;
  private final Hasher hasher;
  private final HashVerifier verifier;
  private final SafetyRules safetyRules;
  private final TimeSupplier timeSupplier;
  private final SystemCounters systemCounters;

  public BFTEventVerifier(
      BFTValidatorSet validatorSet,
      BFTEventProcessor forwardTo,
      Hasher hasher,
      HashVerifier verifier,
      SafetyRules safetyRules,
      TimeSupplier timeSupplier,
      SystemCounters systemCounters) {
    this.validatorSet = Objects.requireNonNull(validatorSet);
    this.hasher = Objects.requireNonNull(hasher);
    this.verifier = Objects.requireNonNull(verifier);
    this.forwardTo = Objects.requireNonNull(forwardTo);
    this.safetyRules = Objects.requireNonNull(safetyRules);
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.systemCounters = Objects.requireNonNull(systemCounters);
  }

  @Override
  public void start() {
    forwardTo.start();
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    forwardTo.processRoundUpdate(roundUpdate);
  }

  @Override
  public void processVote(Vote vote) {
    if (!isAuthorInValidatorSet(vote)) {
      systemCounters.increment(SystemCounters.CounterType.BFT_VERIFIER_INVALID_VOTE_AUTHORS);
      log.warn("Ignoring a vote from {}: not a validator", vote.getAuthor());
      return;
    }

    final var verifiedVoteData =
        verifyHashSignature(
            vote.getAuthor(), vote.getHashOfData(hasher), vote.getSignature(), vote);
    if (!verifiedVoteData) {
      log.warn("Ignoring invalid vote data {}", vote);
      return;
    }

    final var verifiedTimeoutData =
        vote.getTimeoutSignature()
            .map(
                timeoutSignature ->
                    verifyObjectSignature(
                        vote.getAuthor(), VoteTimeout.of(vote), timeoutSignature, vote))
            .orElse(true);

    if (!verifiedTimeoutData) {
      log.warn("Ignoring invalid timeout data {}", vote);
      return;
    }

    if (!safetyRules.verifyHighQcAgainstTheValidatorSet(vote.highQC())) {
      log.warn("Ignoring a vote {} with invalid high QC", vote);
      return;
    }

    forwardTo.processVote(vote);
  }

  @Override
  public void processProposal(Proposal proposal) {
    if (!isAuthorInValidatorSet(proposal)) {
      systemCounters.increment(SystemCounters.CounterType.BFT_VERIFIER_INVALID_PROPOSAL_AUTHORS);
      log.warn("Ignoring a proposal from {}: not a validator", proposal.getAuthor());
      return;
    }

    final var now = timeSupplier.currentTime();
    if (!isProposalTimestampAcceptable(proposal, now)) {
      systemCounters.increment(SystemCounters.CounterType.BFT_VERIFIER_INVALID_PROPOSAL_TIMESTAMPS);
      log.warn(
          "Ignoring a proposal from {}: invalid timestamp (received = {}, local time = {})",
          proposal.getAuthor(),
          proposal.getVertex().proposerTimestamp(),
          now);
      return;
    }

    if (!verifyObjectSignature(
        proposal.getAuthor(), proposal.getVertex(), proposal.getSignature(), proposal)) {
      log.warn("Ignoring a proposal {} with invalid signature", proposal);
      return;
    }

    if (!safetyRules.verifyHighQcAgainstTheValidatorSet(proposal.highQC())) {
      log.warn("Ignoring a proposal {} with invalid high QC", proposal);
      return;
    }

    forwardTo.processProposal(proposal);
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout localTimeout) {
    forwardTo.processLocalTimeout(localTimeout);
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    forwardTo.processBFTUpdate(update);
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate update) {
    forwardTo.processBFTRebuildUpdate(update);
  }

  private boolean isAuthorInValidatorSet(ConsensusEvent event) {
    return validatorSet.containsNode(event.getAuthor());
  }

  private boolean isProposalTimestampAcceptable(Proposal proposal, long now) {
    final var lowerBoundInclusive = now - MAX_ACCEPTABLE_PROPOSAL_TIMESTAMP_DELAY_MS;
    final var upperBoundInclusive = now + MAX_ACCEPTABLE_PROPOSAL_TIMESTAMP_RUSH_MS;

    final var prevTimestamp = proposal.getVertex().parentLedgerHeader().proposerTimestamp();

    final var ts = proposal.getVertex().proposerTimestamp();
    return ts >= lowerBoundInclusive && ts <= upperBoundInclusive && ts > prevTimestamp;
  }

  private boolean verifyHashSignature(
      BFTNode author, HashCode hash, ECDSASecp256k1Signature signature, Object what) {
    boolean verified = this.verifier.verify(author.getKey(), hash, signature);
    if (!verified) {
      log.info("Ignoring invalid signature from {} for {}", author, what);
    }
    return verified;
  }

  private boolean verifyObjectSignature(
      BFTNode author, Object hashable, ECDSASecp256k1Signature signature, Object what) {
    return verifyHashSignature(author, this.hasher.hashDsonEncoded(hashable), signature, what);
  }
}
