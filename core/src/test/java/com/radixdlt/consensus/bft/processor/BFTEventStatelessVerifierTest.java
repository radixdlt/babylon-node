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

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.utils.TimeSupplier;
import java.util.Optional;
import org.junit.Before;
import org.junit.Test;

public class BFTEventStatelessVerifierTest {

  private BFTValidatorSet validatorSet;
  private BFTEventProcessor forwardTo;
  private Hasher hasher;
  private HashVerifier verifier;
  private BFTEventStatelessVerifier eventVerifier;
  private SafetyRules safetyRules;
  private TimeSupplier timeSupplier;
  private ProposerElection proposerElection;
  private Metrics metrics;

  @Before
  public void setup() {
    this.validatorSet = mock(BFTValidatorSet.class);
    this.forwardTo = mock(BFTEventProcessor.class);
    this.hasher = mock(Hasher.class);
    this.verifier = mock(HashVerifier.class);
    this.safetyRules = mock(SafetyRules.class);
    this.timeSupplier = mock(TimeSupplier.class);
    this.proposerElection = mock(ProposerElection.class);
    this.metrics = new MetricsInitializer().initialize();
    this.eventVerifier =
        new BFTEventStatelessVerifier(
            validatorSet, proposerElection, forwardTo, hasher, verifier, safetyRules, metrics);
  }

  @Test
  public void when_start__then_should_be_forwarded() {
    eventVerifier.start();
    verify(forwardTo, times(1)).start();
  }

  @Test
  public void when_process_local_timeout__then_should_be_forwarded() {
    ScheduledLocalTimeout timeout = mock(ScheduledLocalTimeout.class);
    eventVerifier.processLocalTimeout(timeout);
    verify(forwardTo, times(1)).processLocalTimeout(eq(timeout));
  }

  @Test
  public void when_process_local_sync__then_should_be_forwarded() {
    BFTInsertUpdate update = mock(BFTInsertUpdate.class);
    eventVerifier.processBFTUpdate(update);
    verify(forwardTo, times(1)).processBFTUpdate(update);
  }

  @Test
  public void when_process_correct_proposal_then_should_be_forwarded() {
    Proposal proposal = mock(Proposal.class);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(proposal.getRound()).thenReturn(Round.of(2L));
    when(proposal.getAuthor()).thenReturn(author);
    when(proposal.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(timeSupplier.currentTime()).thenReturn(5L);
    final var vertex =
        vertexWithProposerTimestamps(timeSupplier.currentTime() - 1, timeSupplier.currentTime());
    when(proposal.getVertex()).thenReturn(vertex);
    when(proposal.getRound()).thenReturn(Round.of(3));
    final var highQc = mock(HighQC.class);
    when(highQc.getHighestRound()).thenReturn(Round.of(2));
    when(proposal.highQC()).thenReturn(highQc);
    when(validatorSet.containsValidator(eq(author))).thenReturn(true);
    when(verifier.verify(any(), any(), any())).thenReturn(true);
    when(safetyRules.verifyHighQcAgainstTheValidatorSet(any())).thenReturn(true);
    when(proposerElection.getProposer(proposal.getRound())).thenReturn(author);
    eventVerifier.processProposal(proposal);
    verify(forwardTo, times(1)).processProposal(eq(proposal));
  }

  @Test
  public void when_process_bad_author_proposal_then_should_not_be_forwarded() {
    Proposal proposal = mock(Proposal.class);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(proposal.getAuthor()).thenReturn(author);
    when(proposal.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(proposal.getRound()).thenReturn(Round.of(2));
    final var highQc = mock(HighQC.class);
    when(highQc.getHighestRound()).thenReturn(Round.of(1));
    when(proposal.highQC()).thenReturn(highQc);
    when(validatorSet.containsValidator(eq(author))).thenReturn(false);
    when(verifier.verify(any(), any(), any())).thenReturn(true);
    eventVerifier.processProposal(proposal);
    verify(forwardTo, never()).processProposal(any());
  }

  @Test
  public void when_process_bad_signature_proposal_then_should_not_be_forwarded() {
    Proposal proposal = mock(Proposal.class);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(proposal.getAuthor()).thenReturn(author);
    when(proposerElection.getProposer(Round.of(2))).thenReturn(author);
    when(proposal.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(timeSupplier.currentTime()).thenReturn(5L);
    final var vertex =
        vertexWithProposerTimestamps(timeSupplier.currentTime() - 1, timeSupplier.currentTime());
    when(proposal.getVertex()).thenReturn(vertex);
    when(vertex.getRound()).thenReturn(Round.of(2));
    when(proposal.getRound()).thenReturn(Round.of(2));
    final var highQc = mock(HighQC.class);
    when(highQc.getHighestRound()).thenReturn(Round.of(1));
    when(proposal.highQC()).thenReturn(highQc);
    when(validatorSet.containsValidator(eq(author))).thenReturn(true);
    when(verifier.verify(any(), any(), any())).thenReturn(false);
    eventVerifier.processProposal(proposal);
    verify(forwardTo, never()).processProposal(any());
  }

  @Test
  public void when_process_proposal_with_high_qc_for_a_wrong_round_then_should_not_be_forwarded() {
    final var proposal = mock(Proposal.class);
    final var author = mock(BFTValidatorId.class);
    when(proposerElection.getProposer(Round.of(2))).thenReturn(author);
    when(proposal.getAuthor()).thenReturn(author);
    when(proposal.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(timeSupplier.currentTime()).thenReturn(5L);
    final var vertex =
        vertexWithProposerTimestamps(timeSupplier.currentTime() - 1, timeSupplier.currentTime());
    when(proposal.getVertex()).thenReturn(vertex);
    when(vertex.getRound()).thenReturn(Round.of(2));
    when(proposal.getRound()).thenReturn(Round.of(2));
    final var highQc = mock(HighQC.class);
    when(highQc.getHighestRound()).thenReturn(Round.of(0));
    when(proposal.highQC()).thenReturn(highQc);
    eventVerifier.processProposal(proposal);
    verify(forwardTo, never()).processProposal(any());
  }

  private Vertex vertexWithProposerTimestamps(long prevTimestamp, long currentTimestamp) {
    final var vertex = mock(Vertex.class);
    final var qcToParent = mock(QuorumCertificate.class);
    final var bftHeader = mock(BFTHeader.class);
    final var ledgerHeader = mock(LedgerHeader.class);
    when(vertex.parentBFTHeader()).thenReturn(bftHeader);
    when(vertex.parentLedgerHeader()).thenReturn(ledgerHeader);
    when(vertex.getQCToParent()).thenReturn(qcToParent);
    when(vertex.proposerTimestamp()).thenReturn(currentTimestamp);
    when(qcToParent.getProposedHeader()).thenReturn(bftHeader);
    when(bftHeader.getLedgerHeader()).thenReturn(ledgerHeader);
    when(ledgerHeader.proposerTimestamp()).thenReturn(prevTimestamp);
    return vertex;
  }

  @Test
  public void when_process_correct_vote_then_should_be_forwarded() {
    Vote vote = mock(Vote.class);
    when(vote.getRound()).thenReturn(Round.of(1));
    when(vote.getEpoch()).thenReturn(0L);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(vote.getAuthor()).thenReturn(author);
    ECDSASecp256k1Signature voteSignature = mock(ECDSASecp256k1Signature.class);
    ECDSASecp256k1Signature timeoutSignature = mock(ECDSASecp256k1Signature.class);
    when(vote.getSignature()).thenReturn(voteSignature);
    when(vote.getTimeoutSignature()).thenReturn(Optional.of(timeoutSignature));
    when(validatorSet.containsValidator(eq(author))).thenReturn(true);
    when(verifier.verify(any(), any(), eq(voteSignature))).thenReturn(true);
    when(verifier.verify(any(), any(), eq(timeoutSignature))).thenReturn(true);
    when(safetyRules.verifyHighQcAgainstTheValidatorSet(any())).thenReturn(true);
    eventVerifier.processVote(vote);
    verify(forwardTo, times(1)).processVote(eq(vote));
  }

  @Test
  public void when_process_bad_author_vote_then_should_not_be_forwarded() {
    Vote vote = mock(Vote.class);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(vote.getAuthor()).thenReturn(author);
    when(vote.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(validatorSet.containsValidator(eq(author))).thenReturn(false);
    when(verifier.verify(any(), any(), any())).thenReturn(true);
    eventVerifier.processVote(vote);
    verify(forwardTo, never()).processVote(any());
  }

  @Test
  public void when_process_bad_signature_vote_then_should_not_be_forwarded() {
    Vote vote = mock(Vote.class);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(vote.getAuthor()).thenReturn(author);
    when(vote.getSignature()).thenReturn(mock(ECDSASecp256k1Signature.class));
    when(validatorSet.containsValidator(eq(author))).thenReturn(true);
    when(verifier.verify(any(), any(), any())).thenReturn(false);
    eventVerifier.processVote(vote);
    verify(forwardTo, never()).processVote(any());
  }

  @Test
  public void when_process_bad_timeout_signature_vote_then_should_not_be_forwarded() {
    Vote vote = mock(Vote.class);
    when(vote.getRound()).thenReturn(Round.of(1));
    when(vote.getEpoch()).thenReturn(0L);
    BFTValidatorId author = mock(BFTValidatorId.class);
    when(vote.getAuthor()).thenReturn(author);
    ECDSASecp256k1Signature voteSignature = mock(ECDSASecp256k1Signature.class);
    ECDSASecp256k1Signature timeoutSignature = mock(ECDSASecp256k1Signature.class);
    when(vote.getSignature()).thenReturn(voteSignature);
    when(vote.getTimeoutSignature()).thenReturn(Optional.of(timeoutSignature));
    when(validatorSet.containsValidator(eq(author))).thenReturn(true);
    when(verifier.verify(any(), any(), eq(voteSignature))).thenReturn(true);
    when(verifier.verify(any(), any(), eq(timeoutSignature))).thenReturn(false);
    eventVerifier.processVote(vote);
    verify(forwardTo, never()).processVote(any());
  }
}
