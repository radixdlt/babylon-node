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

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

import com.google.common.collect.ImmutableSet;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.VertexStoreAdapter;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.utils.TimeSupplier;
import java.security.SecureRandom;
import java.util.Optional;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;

public class PacemakerTest {

  private static final Hasher hasher = new Sha256Hasher(DefaultSerialization.getInstance());

  private BFTNode self = mock(BFTNode.class);
  private SystemCounters counters = mock(SystemCounters.class);
  private BFTValidatorSet validatorSet = mock(BFTValidatorSet.class);
  private VertexStoreAdapter vertexStore = mock(VertexStoreAdapter.class);
  private SafetyRules safetyRules = mock(SafetyRules.class);
  private PacemakerTimeoutCalculator timeoutCalculator = mock(PacemakerTimeoutCalculator.class);
  private ProposalGenerator proposalGenerator = mock(ProposalGenerator.class);
  private RemoteEventDispatcher<Vote> voteDispatcher = rmock(RemoteEventDispatcher.class);
  private EventDispatcher<RoundLeaderFailure> roundLeaderFailureEventDispatcher =
      rmock(EventDispatcher.class);
  private RemoteEventDispatcher<Proposal> proposalDispatcher = rmock(RemoteEventDispatcher.class);
  private EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher = rmock(EventDispatcher.class);
  private ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender =
      rmock(ScheduledEventDispatcher.class);
  private TimeSupplier timeSupplier = mock(TimeSupplier.class);

  private Pacemaker pacemaker;

  @Before
  public void setUp() {
    HighQC highQC = mock(HighQC.class);
    QuorumCertificate committedQc = mock(QuorumCertificate.class);
    when(committedQc.getRound()).thenReturn(Round.of(0));
    when(highQC.highestCommittedQC()).thenReturn(committedQc);

    RoundUpdate initialRoundUpdate =
        RoundUpdate.create(Round.of(0), highQC, mock(BFTNode.class), mock(BFTNode.class));

    this.pacemaker =
        new Pacemaker(
            this.self,
            this.counters,
            this.validatorSet,
            this.vertexStore,
            this.safetyRules,
            this.timeoutDispatcher,
            this.timeoutSender,
            this.timeoutCalculator,
            this.proposalGenerator,
            this.proposalDispatcher,
            this.voteDispatcher,
            this.roundLeaderFailureEventDispatcher,
            hasher,
            timeSupplier,
            initialRoundUpdate,
            new SystemCountersImpl(),
            new SecureRandom());
  }

  @Test
  public void when_local_timeout__then_resend_previous_vote() {
    Round round = Round.of(0);
    Vote lastVote = mock(Vote.class);
    Vote lastVoteWithTimeout = mock(Vote.class);
    ImmutableSet<BFTNode> validators = rmock(ImmutableSet.class);

    when(this.safetyRules.getLastVote(round)).thenReturn(Optional.of(lastVote));
    when(this.safetyRules.timeoutVote(lastVote)).thenReturn(lastVoteWithTimeout);
    when(this.validatorSet.nodes()).thenReturn(validators);

    RoundUpdate roundUpdate =
        RoundUpdate.create(
            Round.of(0), mock(HighQC.class), mock(BFTNode.class), mock(BFTNode.class));
    this.pacemaker.processLocalTimeout(ScheduledLocalTimeout.create(roundUpdate, 0L));

    verify(this.voteDispatcher, times(1)).dispatch(eq(validators), eq(lastVoteWithTimeout));
    verifyNoMoreInteractions(this.vertexStore);
    verify(this.safetyRules, times(1)).getLastVote(round);
    verify(this.safetyRules, times(1)).timeoutVote(lastVote);
    verifyNoMoreInteractions(this.safetyRules);
  }

  @Test
  public void when_local_timeout__then_send_empty_vote_if_no_previous() {
    final var leader = BFTNode.random();
    HighQC roundUpdateHighQc = mock(HighQC.class);
    QuorumCertificate committedQc = mock(QuorumCertificate.class);
    QuorumCertificate highestQc = mock(QuorumCertificate.class);
    when(roundUpdateHighQc.highestCommittedQC()).thenReturn(committedQc);
    when(roundUpdateHighQc.highestQC()).thenReturn(highestQc);
    BFTHeader highestQcProposed = mock(BFTHeader.class);
    HashCode highQcParentVertexId = mock(HashCode.class);
    when(highestQcProposed.getVertexId()).thenReturn(highQcParentVertexId);
    final var highQcLedgerHeader = mock(LedgerHeader.class);
    when(highQcLedgerHeader.proposerTimestamp()).thenReturn(1L);
    when(highestQcProposed.getLedgerHeader()).thenReturn(highQcLedgerHeader);
    when(highestQc.getProposedHeader()).thenReturn(highestQcProposed);
    when(committedQc.getRound()).thenReturn(Round.of(0));
    RoundUpdate roundUpdate =
        RoundUpdate.create(Round.of(1), roundUpdateHighQc, leader, mock(BFTNode.class));
    this.pacemaker.processRoundUpdate(roundUpdate);
    Round round = Round.of(1);
    Vote emptyVote = mock(Vote.class);
    Vote emptyVoteWithTimeout = mock(Vote.class);
    ImmutableSet<BFTNode> validators = rmock(ImmutableSet.class);
    BFTHeader bftHeader = mock(BFTHeader.class);
    HighQC highQC = mock(HighQC.class);
    BFTInsertUpdate bftInsertUpdate = mock(BFTInsertUpdate.class);
    when(bftInsertUpdate.getHeader()).thenReturn(bftHeader);
    ExecutedVertex executedVertex = mock(ExecutedVertex.class);
    when(executedVertex.getRound()).thenReturn(round);
    when(executedVertex.getLedgerHeader()).thenReturn(mock(LedgerHeader.class));
    VertexStoreState vertexStoreState = mock(VertexStoreState.class);
    when(vertexStoreState.getHighQC()).thenReturn(highQC);
    when(bftInsertUpdate.getInserted()).thenReturn(executedVertex);
    when(bftInsertUpdate.getVertexStoreState()).thenReturn(vertexStoreState);
    final var vertexHash = hasher.hashDsonEncoded(Vertex.createTimeout(highestQc, round, leader));
    when(executedVertex.getVertexHash()).thenReturn(vertexHash);

    when(this.safetyRules.getLastVote(round)).thenReturn(Optional.empty());
    when(this.safetyRules.createVote(any(), any(), anyLong(), any()))
        .thenReturn(Optional.of(emptyVote));
    when(this.safetyRules.timeoutVote(emptyVote)).thenReturn(emptyVoteWithTimeout);
    when(this.validatorSet.nodes()).thenReturn(validators);

    when(this.vertexStore.getExecutedVertex(any())).thenReturn(Optional.empty());

    this.pacemaker.processLocalTimeout(
        ScheduledLocalTimeout.create(
            RoundUpdate.create(Round.of(1), mock(HighQC.class), leader, BFTNode.random()), 0L));

    this.pacemaker.processBFTUpdate(bftInsertUpdate);

    verify(this.voteDispatcher, times(1)).dispatch(eq(validators), eq(emptyVoteWithTimeout));
    verify(this.safetyRules, times(1)).getLastVote(round);
    verify(this.safetyRules, times(1)).createVote(any(), any(), anyLong(), any());
    verify(this.safetyRules, times(1)).timeoutVote(emptyVote);
    verifyNoMoreInteractions(this.safetyRules);

    verify(this.vertexStore, times(1)).getExecutedVertex(any());

    ArgumentCaptor<VertexWithHash> insertVertexCaptor =
        ArgumentCaptor.forClass(VertexWithHash.class);
    verify(this.vertexStore, times(1)).insertVertex(insertVertexCaptor.capture());
    assertEquals(insertVertexCaptor.getValue().vertex().getParentVertexId(), highQcParentVertexId);

    verifyNoMoreInteractions(this.vertexStore);
  }
}
