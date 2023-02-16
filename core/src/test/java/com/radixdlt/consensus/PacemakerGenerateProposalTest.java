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

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.mockito.ArgumentMatchers.*;
import static org.mockito.Mockito.*;

import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.*;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.UInt256;
import java.util.List;
import java.util.stream.Stream;
import org.junit.Before;
import org.junit.Test;

public final class PacemakerGenerateProposalTest {
  private BFTValidatorId self = BFTValidatorId.create(PrivateKeys.ofNumeric(1).getPublicKey());
  private BFTValidatorId validator1 =
      BFTValidatorId.create(PrivateKeys.ofNumeric(2).getPublicKey());
  private BFTValidatorId validator2 =
      BFTValidatorId.create(PrivateKeys.ofNumeric(3).getPublicKey());
  private BFTValidatorSet validatorSet;
  private VertexStoreAdapter vertexStore;
  private SafetyRules safetyRules;
  private EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher;
  private ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender;
  private PacemakerTimeoutCalculator timeoutCalculator;
  private ProposalGenerator proposalGenerator;
  private RemoteEventDispatcher<BFTValidatorId, Proposal> proposalDispatcher;
  private RemoteEventDispatcher<BFTValidatorId, Vote> voteDispatcher;
  private EventDispatcher<ProposalRejected> proposalRejectedDispatcher;
  private EventDispatcher<NoVote> noVoteDispatcher;
  private Hasher hasher;
  private TimeSupplier timeSupplier;
  private RoundUpdate initialRoundUpdate;
  private Metrics metrics;

  private Pacemaker sut;

  @Before
  public void setup() {
    this.metrics = new MetricsInitializer().initialize();
    this.validatorSet =
        BFTValidatorSet.from(
            Stream.of(
                BFTValidator.from(validator1, UInt256.ONE),
                BFTValidator.from(validator2, UInt256.ONE),
                BFTValidator.from(self, UInt256.ONE)));
    this.vertexStore = mock(VertexStoreAdapter.class);
    this.safetyRules = mock(SafetyRules.class);
    this.timeoutDispatcher = rmock(EventDispatcher.class);
    this.timeoutSender = rmock(ScheduledEventDispatcher.class);
    this.timeoutCalculator = mock(PacemakerTimeoutCalculator.class);
    this.proposalGenerator = mock(ProposalGenerator.class);
    this.proposalDispatcher = rmock(RemoteEventDispatcher.class);
    this.voteDispatcher = rmock(RemoteEventDispatcher.class);
    this.noVoteDispatcher = rmock(EventDispatcher.class);
    this.hasher = new Sha256Hasher(DefaultSerialization.getInstance());
    this.timeSupplier = mock(TimeSupplier.class);
    final var initialHighQc = mock(HighQC.class);
    when(initialHighQc.getHighestRound()).thenReturn(Round.of(0L));
    this.initialRoundUpdate =
        RoundUpdate.create(Round.of(1L), initialHighQc, validator1, validator2);

    this.sut =
        new Pacemaker(
            self,
            validatorSet,
            vertexStore,
            safetyRules,
            timeoutDispatcher,
            timeoutSender,
            timeoutCalculator,
            proposalGenerator,
            proposalDispatcher,
            voteDispatcher,
            noVoteDispatcher,
            hasher,
            timeSupplier,
            initialRoundUpdate,
            metrics);
  }

  @Test
  public void when_previous_vertex_in_the_past_then_should_use_current_time_for_proposal() {
    when(timeoutCalculator.calculateTimeoutMs(anyLong())).thenReturn(0L);
    when(vertexStore.getPathFromRoot(any())).thenReturn(List.of());
    when(proposalGenerator.getTransactionsForProposal(any(), any())).thenReturn(List.of());

    final var previousTimestamp = 100L;
    final var currentTimestamp = 101L;
    final var highQc = createMockHighQc(previousTimestamp);
    when(timeSupplier.currentTime()).thenReturn(currentTimestamp);

    sut.processRoundUpdate(RoundUpdate.create(Round.of(2L), highQc, self, validator1));

    verify(this.safetyRules, times(1))
        .signProposal(
            argThat(v -> v.vertex().proposerTimestamp() == currentTimestamp), any(), any());
  }

  @Test
  public void when_previous_vertex_in_the_future_then_should_reuse_its_timestamp() {
    when(timeoutCalculator.calculateTimeoutMs(anyLong())).thenReturn(0L);
    when(vertexStore.getPathFromRoot(any())).thenReturn(List.of());
    when(proposalGenerator.getTransactionsForProposal(any(), any())).thenReturn(List.of());

    final var previousTimestamp = 100L;
    final var currentTimestamp = 90L;
    final var highQc = createMockHighQc(previousTimestamp);
    when(timeSupplier.currentTime()).thenReturn(currentTimestamp);

    sut.processRoundUpdate(RoundUpdate.create(Round.of(2L), highQc, self, validator1));

    verify(this.safetyRules, times(1))
        .signProposal(
            argThat(v -> v.vertex().proposerTimestamp() == previousTimestamp), any(), any());
  }

  private HighQC createMockHighQc(long previousTimestamp) {
    final var highQc = mock(HighQC.class);
    final var highestQc = mock(QuorumCertificate.class);
    final var highestCommittedQc = mock(QuorumCertificate.class);
    final var highestQcProposedHeader = mock(BFTHeader.class);
    final var highestQcLedgerHeader = mock(LedgerHeader.class);

    when(highestCommittedQc.getRound()).thenReturn(Round.of(1));
    when(highQc.highestCommittedQC()).thenReturn(highestCommittedQc);

    when(highestQc.getProposedHeader()).thenReturn(highestQcProposedHeader);
    when(highestQcProposedHeader.getLedgerHeader()).thenReturn(highestQcLedgerHeader);
    when(highestQcLedgerHeader.isEndOfEpoch()).thenReturn(false);
    when(highestQcLedgerHeader.proposerTimestamp()).thenReturn(previousTimestamp);
    when(highQc.highestQC()).thenReturn(highestQc);

    return highQc;
  }
}
