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

package com.radixdlt.modules;

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

import com.google.common.collect.ImmutableList;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.*;
import com.google.inject.Module;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.LocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.consensus.sync.*;
import com.radixdlt.consensus.vertexstore.PersistentVertexStore;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.messaging.core.GetVerticesRequestRateLimit;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.RoundChange.HighQcSource;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.store.LastProof;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.*;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Stream;
import org.junit.Before;
import org.junit.Test;

public class ConsensusModuleTest {
  @Inject private BFTSync bftSync;

  @Inject private VertexStoreAdapter vertexStore;

  private Hasher hasher = new Sha256Hasher(DefaultSerialization.getInstance());

  private ECKeyPair validatorKeyPair;

  private BFTValidatorId validatorId;
  private BFTConfiguration bftConfiguration;

  private ECKeyPair ecKeyPair;
  private RemoteEventDispatcher<NodeId, GetVerticesRequest> requestSender;
  private RemoteEventDispatcher<NodeId, GetVerticesResponse> responseSender;
  private RemoteEventDispatcher<NodeId, GetVerticesErrorResponse> errorResponseSender;

  @Before
  public void setup() {
    var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
    var genesisVertex =
        Vertex.createInitialEpochVertex(
                LedgerHeader.genesis(accumulatorState, HashUtils.zero256(), null, 0, 0))
            .withId(ZeroHasher.INSTANCE);
    var qc =
        QuorumCertificate.createInitialEpochQC(
            genesisVertex, LedgerHeader.genesis(accumulatorState, HashUtils.zero256(), null, 0, 0));
    this.validatorKeyPair = ECKeyPair.generateNew();
    this.validatorId = BFTValidatorId.create(this.validatorKeyPair.getPublicKey());
    var validatorSet =
        BFTValidatorSet.from(Stream.of(BFTValidator.from(this.validatorId, UInt256.ONE)));
    var vertexStoreState =
        VertexStoreState.create(HighQC.ofInitialEpochQc(qc), genesisVertex, hasher);
    var proposerElection = new WeightedRotatingLeaders(validatorSet);
    this.bftConfiguration = new BFTConfiguration(proposerElection, validatorSet, vertexStoreState);
    this.ecKeyPair = ECKeyPair.generateNew();
    this.requestSender = rmock(RemoteEventDispatcher.class);
    this.responseSender = rmock(RemoteEventDispatcher.class);
    this.errorResponseSender = rmock(RemoteEventDispatcher.class);

    Guice.createInjector(new ConsensusModule(), new CryptoModule(), getExternalModule())
        .injectMembers(this);
  }

  private Module getExternalModule() {
    return new AbstractModule() {
      @Override
      protected void configure() {
        bind(Ledger.class).toInstance(mock(Ledger.class));

        bind(new TypeLiteral<EventDispatcher<LocalTimeoutOccurrence>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<RoundUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTInsertUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTRebuildUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTHighQCUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<BFTCommittedUpdate>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<LocalSyncRequest>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<GetVerticesRequest>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<ScheduledLocalTimeout>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<RoundQuorumReached>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<ProposalRejected>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Vote>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Proposal>>() {})
            .toInstance(rmock(RemoteEventDispatcher.class));
        bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetVerticesRequest>>() {})
            .toInstance(requestSender);
        bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetVerticesResponse>>() {})
            .toInstance(responseSender);
        bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetVerticesErrorResponse>>() {})
            .toInstance(errorResponseSender);
        bind(new TypeLiteral<EventDispatcher<NoVote>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<ConsensusByzantineEvent.DoubleVote>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<ConsensusByzantineEvent>>() {})
            .toInstance(rmock(EventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<Round>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));
        bind(new TypeLiteral<ScheduledEventDispatcher<VertexRequestTimeout>>() {})
            .toInstance(rmock(ScheduledEventDispatcher.class));

        bind(PersistentVertexStore.class).toInstance(mock(PersistentVertexStore.class));
        bind(PersistentSafetyStateStore.class).toInstance(mock(PersistentSafetyStateStore.class));
        bind(ProposalGenerator.class).toInstance(mock(ProposalGenerator.class));
        bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
        bind(TimeSupplier.class).toInstance(mock(TimeSupplier.class));
        bind(BFTConfiguration.class).toInstance(bftConfiguration);
        bind(BFTValidatorSet.class).toInstance(bftConfiguration.getValidatorSet());
        LedgerProof proof = mock(LedgerProof.class);
        when(proof.getRound()).thenReturn(Round.genesis());
        bind(LedgerProof.class).annotatedWith(LastProof.class).toInstance(proof);
        bind(RateLimiter.class)
            .annotatedWith(GetVerticesRequestRateLimit.class)
            .toInstance(RateLimiter.create(Double.MAX_VALUE));
        bind(Addressing.class).toInstance(Addressing.ofNetwork(Network.LOCALNET));
        bindConstant().annotatedWith(BFTSyncPatienceMillis.class).to(200);
        bindConstant().annotatedWith(PacemakerBaseTimeoutMs.class).to(1000L);
        bindConstant().annotatedWith(PacemakerBackoffRate.class).to(2.0);
        bindConstant().annotatedWith(PacemakerMaxExponent.class).to(6);
        bindConstant().annotatedWith(AdditionalRoundTimeIfProposalReceivedMs.class).to(1000L);

        ECKeyPair ecKeyPair = ECKeyPair.generateNew();
        bind(HashSigner.class).toInstance(ecKeyPair::sign);
      }

      @Provides
      RoundUpdate initialRoundUpdate(@Self BFTValidatorId node) {
        return RoundUpdate.create(Round.of(1), mock(HighQC.class), node, node);
      }

      @Provides
      @Self
      private BFTValidatorId bftNode() {
        return BFTValidatorId.create(ecKeyPair.getPublicKey());
      }
    };
  }

  private Pair<QuorumCertificate, VertexWithHash> createNextVertex(
      QuorumCertificate parent, ECKeyPair proposerKeyPair) {
    return createNextVertex(
        parent, proposerKeyPair, RawNotarizedTransaction.create(new byte[] {0}));
  }

  private Pair<QuorumCertificate, VertexWithHash> createNextVertex(
      QuorumCertificate parent, ECKeyPair proposerKeyPair, RawNotarizedTransaction txn) {
    final var proposerBftNode = BFTValidatorId.create(proposerKeyPair.getPublicKey());
    var vertex =
        Vertex.create(parent, Round.of(1), List.of(txn), proposerBftNode, 0).withId(hasher);
    var next =
        new BFTHeader(
            Round.of(1),
            vertex.hash(),
            LedgerHeader.create(
                1,
                Round.of(1),
                new AccumulatorState(1, HashUtils.zero256()),
                HashUtils.zero256(),
                1,
                1));
    final var voteData = new VoteData(next, parent.getProposedHeader(), parent.getParentHeader());
    final var timestamp = 1;
    final var voteDataHash = Vote.getHashOfData(hasher, voteData, timestamp);
    final var qcSignature = proposerKeyPair.sign(voteDataHash);
    var unsyncedQC =
        new QuorumCertificate(
            voteData,
            new TimestampedECDSASignatures(
                Map.of(proposerBftNode, TimestampedECDSASignature.from(timestamp, qcSignature))));

    return Pair.of(unsyncedQC, vertex);
  }

  @Test
  public void on_sync_request_timeout_should_retry() {
    // Arrange
    QuorumCertificate parent = vertexStore.highQC().highestQC();
    Pair<QuorumCertificate, VertexWithHash> nextVertex = createNextVertex(parent, validatorKeyPair);
    HighQC unsyncedHighQC =
        HighQC.from(nextVertex.getFirst(), nextVertex.getFirst(), Optional.empty());
    bftSync.syncToQC(
        unsyncedHighQC,
        NodeId.fromPublicKey(validatorId.getKey()),
        HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL);
    GetVerticesRequest request = new GetVerticesRequest(nextVertex.getSecond().hash(), 1);
    VertexRequestTimeout timeout = VertexRequestTimeout.create(request);

    // Act
    nothrowSleep(100); // FIXME: Remove when rate limit on send removed
    bftSync.vertexRequestTimeoutEventProcessor().process(timeout);

    // Assert
    verify(requestSender, times(2))
        .dispatch(
            eq(NodeId.fromPublicKey(validatorId.getKey())),
            argThat(
                r -> r.getCount() == 1 && r.getVertexId().equals(nextVertex.getSecond().hash())));
  }

  @Test
  public void on_synced_to_vertex_should_request_for_parent() {
    // Arrange
    var nodeId = NodeId.fromPublicKey(PrivateKeys.ofNumeric(1).getPublicKey());
    QuorumCertificate parent = vertexStore.highQC().highestQC();
    Pair<QuorumCertificate, VertexWithHash> nextVertex = createNextVertex(parent, validatorKeyPair);
    Pair<QuorumCertificate, VertexWithHash> nextNextVertex =
        createNextVertex(nextVertex.getFirst(), validatorKeyPair);
    HighQC unsyncedHighQC =
        HighQC.from(nextNextVertex.getFirst(), nextNextVertex.getFirst(), Optional.empty());
    bftSync.syncToQC(unsyncedHighQC, nodeId, HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL);

    // Act
    nothrowSleep(100); // FIXME: Remove when rate limit on send removed
    GetVerticesResponse response =
        new GetVerticesResponse(ImmutableList.of(nextNextVertex.getSecond()));
    bftSync.responseProcessor().process(nodeId, response);

    // Assert
    verify(requestSender, times(1))
        .dispatch(
            eq(nodeId),
            argThat(
                r -> r.getCount() == 1 && r.getVertexId().equals(nextVertex.getSecond().hash())));
  }

  @Test
  public void bft_sync_should_sync_two_different_QCs_with_the_same_parent() {

    var nodeId = NodeId.fromPublicKey(validatorId.getKey());
    final var parentQc = vertexStore.highQC().highestQC();
    final var parentVertex = createNextVertex(parentQc, validatorKeyPair);
    final var proposedVertex1 =
        createNextVertex(
            parentVertex.getFirst(),
            validatorKeyPair,
            RawNotarizedTransaction.create(new byte[] {1}));
    final var proposedVertex2 =
        createNextVertex(
            parentVertex.getFirst(),
            validatorKeyPair,
            RawNotarizedTransaction.create(new byte[] {2}));
    final var unsyncedHighQC1 =
        HighQC.from(proposedVertex1.getFirst(), proposedVertex1.getFirst(), Optional.empty());
    final var unsyncedHighQC2 =
        HighQC.from(proposedVertex2.getFirst(), proposedVertex2.getFirst(), Optional.empty());

    bftSync.syncToQC(unsyncedHighQC1, nodeId, HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL);
    bftSync.syncToQC(unsyncedHighQC2, nodeId, HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL);

    nothrowSleep(100);
    final var response1 = new GetVerticesResponse(ImmutableList.of(proposedVertex1.getSecond()));
    bftSync.responseProcessor().process(nodeId, response1);

    final var response2 = new GetVerticesResponse(ImmutableList.of(proposedVertex2.getSecond()));
    bftSync.responseProcessor().process(nodeId, response2);

    verify(requestSender, times(1))
        .dispatch(
            eq(nodeId),
            argThat(
                r ->
                    r.getCount() == 1
                        && r.getVertexId().equals(proposedVertex1.getSecond().hash())));

    verify(requestSender, times(1))
        .dispatch(
            eq(nodeId),
            argThat(
                r ->
                    r.getCount() == 1
                        && r.getVertexId().equals(proposedVertex2.getSecond().hash())));
  }

  private void nothrowSleep(long milliseconds) {
    try {
      Thread.sleep(milliseconds);
    } catch (InterruptedException e) {
      // Ignore
      Thread.currentThread().interrupt();
    }
  }
}
