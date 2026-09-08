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
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.*;

import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.crypto.Blake2b256Hasher;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.protocol.UserTransactionMoratorium;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Optional;
import org.junit.Before;
import org.junit.Test;

public final class PacemakerUserTransactionMoratoriumTest {
  private static final Hasher hasher = new Blake2b256Hasher(DefaultSerialization.getInstance());
  private static final UserTransactionMoratorium MORATORIUM =
      new UserTransactionMoratorium(UInt64.fromNonNegativeLong(3), UInt64.fromNonNegativeLong(4));
  private static final Round CURRENT_ROUND = Round.of(1);

  private final BFTValidatorId self = mock(BFTValidatorId.class);
  private final BFTValidatorSet validatorSet = mock(BFTValidatorSet.class);
  private final VertexStoreAdapter vertexStore = mock(VertexStoreAdapter.class);
  private final SafetyRules safetyRules = mock(SafetyRules.class);
  private final PacemakerTimeoutCalculator timeoutCalculator =
      mock(PacemakerTimeoutCalculator.class);
  private final ProposalGenerator proposalGenerator = mock(ProposalGenerator.class);
  private final RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher =
      rmock(RemoteEventDispatcher.class);
  private final RemoteEventDispatcher<BFTValidatorId, Proposal> proposalDispatcher =
      rmock(RemoteEventDispatcher.class);
  private final EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher =
      rmock(EventDispatcher.class);
  private final EventDispatcher<NoVote> noVoteDispatcher = rmock(EventDispatcher.class);
  private final ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender =
      rmock(ScheduledEventDispatcher.class);
  private final TimeSupplier timeSupplier = mock(TimeSupplier.class);
  private final UserTransactionMoratoriumProvider provider =
      mock(UserTransactionMoratoriumProvider.class);
  private final Metrics metrics = new MetricsInitializer().initialize();

  private HighQC highQC;

  @Before
  public void setUp() {
    this.highQC = mock(HighQC.class);
    final var committedQc = mock(QuorumCertificate.class);
    when(committedQc.getRound()).thenReturn(Round.of(0));
    when(this.highQC.highestCommittedQC()).thenReturn(committedQc);
    when(this.highQC.getHighestRound()).thenReturn(Round.of(0));
    when(this.safetyRules.getLastVote(any())).thenReturn(Optional.empty());
  }

  @Test
  public void withholds_vote_for_vertex_carrying_user_transactions_while_moratorium_is_in_force() {
    // Arrange
    final var pacemaker = createPacemaker(Result.error(MORATORIUM));
    final var insertUpdate =
        insertUpdateOfVertexWith(List.of(RawNotarizedTransaction.create(new byte[] {1, 2, 3})));

    // Act
    pacemaker.processBFTUpdate(insertUpdate);

    // Assert
    verify(this.provider).ensureUserTransactionsAllowed(3);
    verify(this.noVoteDispatcher, times(1)).dispatch(any(NoVote.class));
    verify(this.safetyRules, never()).createVote(any(), any(), anyLong(), any());
    verifyNoInteractions(this.voteDispatcher);
  }

  @Test
  public void votes_for_vertex_without_user_transactions_while_moratorium_is_in_force() {
    // Arrange
    final var pacemaker = createPacemaker(Result.error(MORATORIUM));
    final var insertUpdate = insertUpdateOfVertexWith(List.of());
    final var vote = mockVote();
    when(this.safetyRules.createVote(any(), any(), anyLong(), any())).thenReturn(Optional.of(vote));

    // Act
    pacemaker.processBFTUpdate(insertUpdate);

    // Assert
    verifyNoInteractions(this.provider);
    verify(this.safetyRules, times(1)).createVote(any(), any(), anyLong(), any());
    verify(this.noVoteDispatcher, never()).dispatch(any());
  }

  @Test
  public void votes_for_vertex_carrying_user_transactions_when_no_moratorium_is_in_force() {
    // Arrange
    final var pacemaker = createPacemaker(Result.success(Tuple.tuple()));
    final var insertUpdate =
        insertUpdateOfVertexWith(List.of(RawNotarizedTransaction.create(new byte[] {1, 2, 3})));
    when(insertUpdate.insertedVertex().vertex().getEpoch()).thenReturn(4L);
    final var vote = mockVote();
    when(this.safetyRules.createVote(any(), any(), anyLong(), any())).thenReturn(Optional.of(vote));

    // Act
    pacemaker.processBFTUpdate(insertUpdate);

    // Assert
    verify(this.provider).ensureUserTransactionsAllowed(4);
    verify(this.safetyRules, times(1)).createVote(any(), any(), anyLong(), any());
    verify(this.noVoteDispatcher, never()).dispatch(any());
  }

  private Pacemaker createPacemaker(Result<Tuple.Tuple0, UserTransactionMoratorium> moratorium) {
    when(this.provider.ensureUserTransactionsAllowed(anyLong())).thenReturn(moratorium);
    final var initialRoundUpdate =
        new RoundUpdate(
            CURRENT_ROUND, this.highQC, mock(BFTValidatorId.class), mock(BFTValidatorId.class));
    return new Pacemaker(
        this.self,
        this.validatorSet,
        this.vertexStore,
        this.safetyRules,
        this.timeoutDispatcher,
        this.timeoutSender,
        this.timeoutCalculator,
        this.proposalGenerator,
        provider,
        this.proposalDispatcher,
        this.voteDispatcher,
        this.noVoteDispatcher,
        hasher,
        this.timeSupplier,
        initialRoundUpdate,
        this.metrics);
  }

  private static Vote mockVote() {
    final var proposed = mock(BFTHeader.class);
    when(proposed.getVertexId()).thenReturn(HashCode.fromInt(99));
    final var voteData = mock(VoteData.class);
    when(voteData.getProposed()).thenReturn(proposed);
    final var vote = mock(Vote.class);
    when(vote.getVoteData()).thenReturn(voteData);
    return vote;
  }

  private BFTInsertUpdate insertUpdateOfVertexWith(List<RawNotarizedTransaction> transactions) {
    final var vertex = mock(Vertex.class);
    when(vertex.getEpoch()).thenReturn(3L);
    when(vertex.getTransactions()).thenReturn(transactions);
    final var executedVertex = mock(ExecutedVertex.class);
    when(executedVertex.getRound()).thenReturn(CURRENT_ROUND);
    when(executedVertex.vertex()).thenReturn(vertex);
    when(executedVertex.getVertexWithHash()).thenReturn(mock(VertexWithHash.class));
    when(executedVertex.getVertexHash()).thenReturn(HashCode.fromInt(99));
    when(executedVertex.getLedgerHeader()).thenReturn(mock(LedgerHeader.class));
    final var insertUpdate = mock(BFTInsertUpdate.class);
    when(insertUpdate.insertedVertex()).thenReturn(executedVertex);
    final var header =
        new BFTHeader(CURRENT_ROUND, HashCode.fromInt(99), executedVertex.getLedgerHeader());
    when(insertUpdate.getHeader()).thenReturn(header);
    return insertUpdate;
  }
}
