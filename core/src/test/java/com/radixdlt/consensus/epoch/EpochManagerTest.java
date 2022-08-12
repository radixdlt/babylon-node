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

package com.radixdlt.consensus.epoch;

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import com.google.common.collect.ImmutableClassToInstanceMap;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.AbstractModule;
import com.google.inject.Guice;
import com.google.inject.Inject;
import com.google.inject.Module;
import com.google.inject.Provides;
import com.google.inject.TypeLiteral;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.EpochLocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.LocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.consensus.sync.GetVerticesErrorResponse;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.consensus.sync.GetVerticesResponse;
import com.radixdlt.consensus.sync.VertexRequestTimeout;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.StateComputerLedger.ExecutedTransaction;
import com.radixdlt.ledger.StateComputerLedger.StateComputer;
import com.radixdlt.ledger.StateComputerLedger.StateComputerResult;
import com.radixdlt.mempool.Mempool;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.messaging.core.GetVerticesRequestRateLimit;
import com.radixdlt.modules.ConsensusModule;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.modules.LedgerModule;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.rev1.RoundDetails;
import com.radixdlt.store.LastEpochProof;
import com.radixdlt.store.LastProof;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import com.radixdlt.sync.messages.remote.LedgerStatusUpdate;
import com.radixdlt.transactions.RawTransaction;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.UInt256;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.stream.Stream;
import javax.annotation.Nullable;
import org.junit.Before;
import org.junit.Test;

public class EpochManagerTest {
  @Inject private EpochManager epochManager;

  @Inject private Hasher hasher;

  private ECKeyPair ecKeyPair = ECKeyPair.generateNew();

  private ProposalGenerator proposalGenerator = mock(ProposalGenerator.class);
  private ScheduledEventDispatcher<GetVerticesRequest> timeoutScheduler =
      rmock(ScheduledEventDispatcher.class);
  private EventDispatcher<LocalSyncRequest> syncLedgerRequestSender = rmock(EventDispatcher.class);
  private RemoteEventDispatcher<Proposal> proposalDispatcher = rmock(RemoteEventDispatcher.class);
  private RemoteEventDispatcher<Vote> voteDispatcher = rmock(RemoteEventDispatcher.class);
  private Mempool mempool = mock(Mempool.class);
  private StateComputer stateComputer =
      new StateComputer() {
        @Override
        public void addToMempool(MempoolAdd mempoolAdd, @Nullable BFTNode origin) {
          // No-op
        }

        @Override
        public List<RawTransaction> getTransactionsForProposal(
            List<ExecutedTransaction> executedTransactions) {
          return List.of();
        }

        @Override
        public StateComputerResult prepare(
            List<ExecutedTransaction> previous,
            List<RawTransaction> proposedTransactions,
            RoundDetails roundDetails) {
          return new StateComputerResult(List.of(), Map.of());
        }

        @Override
        public void commit(
            CommittedTransactionsWithProof committedTransactionsWithProof,
            VertexStoreState vertexStoreState) {
          // No-op
        }
      };

  private Module getExternalModule() {
    BFTNode self = BFTNode.create(ecKeyPair.getPublicKey());
    return new AbstractModule() {
      @Override
      protected void configure() {
        bind(HashSigner.class).toInstance(ecKeyPair::sign);
        bind(BFTNode.class).annotatedWith(Self.class).toInstance(self);
        bind(new TypeLiteral<EventDispatcher<LocalTimeoutOccurrence>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTInsertUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTRebuildUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTHighQCUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTCommittedUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<EpochLocalTimeoutOccurrence>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<EpochRound>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<LocalSyncRequest>>() {})
            .toInstance(syncLedgerRequestSender);
        bind(new TypeLiteral<EventDispatcher<RoundQuorumReached>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<EpochRoundUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<RoundUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<NoVote>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<LedgerUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<GetVerticesRequest>>() {})
            .toInstance(timeoutScheduler);
        bind(new TypeLiteral<ScheduledEventDispatcher<ScheduledLocalTimeout>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<Epoched<ScheduledLocalTimeout>>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<VertexRequestTimeout>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<Proposal>>() {}).toInstance(proposalDispatcher);
        bind(new TypeLiteral<RemoteEventDispatcher<Vote>>() {}).toInstance(voteDispatcher);
        bind(new TypeLiteral<RemoteEventDispatcher<GetVerticesRequest>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<GetVerticesResponse>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<GetVerticesErrorResponse>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<LedgerStatusUpdate>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));

        bind(PersistentSafetyStateStore.class).toInstance(mock(PersistentSafetyStateStore.class));
        bind(ProposalGenerator.class).toInstance(proposalGenerator);
        bind(SystemCounters.class).toInstance(new SystemCountersImpl());
        bind(Mempool.class).toInstance(mempool);
        bind(StateComputer.class).toInstance(stateComputer);
        bind(PersistentVertexStore.class).toInstance(mock(PersistentVertexStore.class));
        bind(RateLimiter.class)
            .annotatedWith(GetVerticesRequestRateLimit.class)
            .toInstance(RateLimiter.create(Double.MAX_VALUE));
        bindConstant().annotatedWith(BFTSyncPatienceMillis.class).to(50);
        bindConstant().annotatedWith(PacemakerBaseTimeoutMs.class).to(10L);
        bindConstant().annotatedWith(PacemakerBackoffRate.class).to(2.0);
        bindConstant().annotatedWith(PacemakerMaxExponent.class).to(0);
        bind(TimeSupplier.class).toInstance(System::currentTimeMillis);

        bind(new TypeLiteral<Consumer<EpochRoundUpdate>>() {}).toInstance(rmock(Consumer.class));
      }

      @Provides
      private RoundUpdate initialRoundUpdate(BFTConfiguration bftConfiguration) {
        HighQC highQC = bftConfiguration.getVertexStoreState().getHighQC();
        Round round = highQC.highestQC().getRound().next();
        return RoundUpdate.create(round, highQC, self, self);
      }

      @Provides
      BFTValidatorSet validatorSet() {
        return BFTValidatorSet.from(Stream.of(BFTValidator.from(self, UInt256.ONE)));
      }

      @Provides
      @LastProof
      LedgerProof verifiedLedgerHeaderAndProof(BFTValidatorSet validatorSet) {
        var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
        return LedgerProof.genesis(accumulatorState, validatorSet, 0);
      }

      @Provides
      @LastEpochProof
      LedgerProof lastEpochProof(BFTValidatorSet validatorSet) {
        var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
        return LedgerProof.genesis(accumulatorState, validatorSet, 0);
      }

      @Provides
      BFTConfiguration bftConfiguration(
          @Self BFTNode self, Hasher hasher, BFTValidatorSet validatorSet) {
        var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
        var vertex =
            Vertex.createGenesis(LedgerHeader.genesis(accumulatorState, validatorSet, 0))
                .withId(hasher);
        var qc =
            QuorumCertificate.ofGenesis(
                vertex, LedgerHeader.genesis(accumulatorState, validatorSet, 0));
        var proposerElection = new WeightedRotatingLeaders(validatorSet);
        return new BFTConfiguration(
            proposerElection,
            validatorSet,
            VertexStoreState.create(HighQC.from(qc), vertex, Optional.empty(), hasher));
      }
    };
  }

  @Before
  public void setup() {
    Guice.createInjector(
            new CryptoModule(),
            new ConsensusModule(),
            new EpochsConsensusModule(),
            new LedgerModule(),
            getExternalModule())
        .injectMembers(this);
  }

  @Test
  public void should_not_send_consensus_messages_if_not_part_of_new_epoch() {
    // Arrange
    epochManager.start();
    BFTValidatorSet nextValidatorSet =
        BFTValidatorSet.from(Stream.of(BFTValidator.from(BFTNode.random(), UInt256.ONE)));
    var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
    LedgerHeader header = LedgerHeader.genesis(accumulatorState, nextValidatorSet, 0);
    VertexWithHash verifiedGenesisVertex = Vertex.createGenesis(header).withId(hasher);
    LedgerHeader nextLedgerHeader =
        LedgerHeader.create(
            header.getEpoch() + 1,
            Round.genesis(),
            header.getAccumulatorState(),
            header.roundTimestamp());
    var genesisQC = QuorumCertificate.ofGenesis(verifiedGenesisVertex, nextLedgerHeader);
    var proposerElection = new WeightedRotatingLeaders(nextValidatorSet);
    var bftConfiguration =
        new BFTConfiguration(
            proposerElection,
            nextValidatorSet,
            VertexStoreState.create(
                HighQC.from(genesisQC), verifiedGenesisVertex, Optional.empty(), hasher));
    LedgerProof proof = mock(LedgerProof.class);
    when(proof.getEpoch()).thenReturn(header.getEpoch() + 1);
    when(proof.getNextEpoch()).thenReturn(header.getEpoch() + 2);
    var epochChange = new EpochChange(proof, bftConfiguration);
    var ledgerUpdate =
        new LedgerUpdate(
            mock(CommittedTransactionsWithProof.class),
            ImmutableClassToInstanceMap.of(EpochChange.class, epochChange));

    // Act
    epochManager.epochsLedgerUpdateEventProcessor().process(ledgerUpdate);

    // Assert
    verify(proposalDispatcher, never())
        .dispatch(any(Iterable.class), argThat(p -> p.getEpoch() == epochChange.getNextEpoch()));
    verify(voteDispatcher, never()).dispatch(any(BFTNode.class), any());
  }
}
