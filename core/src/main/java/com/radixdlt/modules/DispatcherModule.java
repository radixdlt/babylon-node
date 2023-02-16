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

import com.google.inject.*;
import com.google.inject.multibindings.Multibinder;
import com.radixdlt.api.system.health.ScheduledStatsCollecting;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochProposalRejected;
import com.radixdlt.consensus.epoch.EpochRoundUpdate;
import com.radixdlt.consensus.epoch.Epoched;
import com.radixdlt.consensus.liveness.EpochLocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.LocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.sync.GetVerticesErrorResponse;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.consensus.sync.GetVerticesResponse;
import com.radixdlt.consensus.sync.VertexRequestTimeout;
import com.radixdlt.environment.*;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.mempool.MempoolRelayTrigger;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.PeerEvent;
import com.radixdlt.p2p.PendingOutboundChannelsManager.PeerOutboundConnectionTimeout;
import com.radixdlt.p2p.discovery.DiscoverPeers;
import com.radixdlt.p2p.discovery.GetPeers;
import com.radixdlt.p2p.discovery.PeersResponse;
import com.radixdlt.p2p.liveness.PeerPingTimeout;
import com.radixdlt.p2p.liveness.PeersLivenessCheckTrigger;
import com.radixdlt.p2p.liveness.Ping;
import com.radixdlt.p2p.liveness.Pong;
import com.radixdlt.sync.messages.local.*;
import com.radixdlt.sync.messages.remote.*;
import io.prometheus.client.Gauge;
import java.util.Set;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Manages dispatching of internal events to a given environment TODO: Move all other events into
 * this module
 */
public class DispatcherModule extends AbstractModule {
  private static final Logger logger = LogManager.getLogger();

  @Override
  public void configure() {
    bind(new TypeLiteral<EventDispatcher<ConsensusByzantineEvent>>() {})
        .toProvider(Dispatchers.dispatcherProvider(ConsensusByzantineEvent.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<MempoolAdd>>() {})
        .toProvider(Dispatchers.dispatcherProvider(MempoolAdd.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<MempoolAddSuccess>>() {})
        .toProvider(Dispatchers.dispatcherProvider(MempoolAddSuccess.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<MempoolRelayTrigger>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(MempoolRelayTrigger.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<SyncCheckTrigger>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(SyncCheckTrigger.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<NoVote>>() {})
        .toProvider(
            Dispatchers.dispatcherProvider(
                NoVote.class, (counters, event) -> counters.bft().noVotesSent()))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<Epoched<ScheduledLocalTimeout>>>() {})
        .toProvider(
            Dispatchers.scheduledDispatcherProvider(
                new TypeLiteral<Epoched<ScheduledLocalTimeout>>() {}))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<VertexRequestTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(VertexRequestTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<SyncRequestTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(SyncRequestTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<SyncLedgerUpdateTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(SyncLedgerUpdateTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<SyncCheckReceiveStatusTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(SyncCheckReceiveStatusTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<SyncCheckTrigger>>() {})
        .toProvider(Dispatchers.dispatcherProvider(SyncCheckTrigger.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<ScheduledStatsCollecting>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(ScheduledStatsCollecting.class))
        .in(Scopes.SINGLETON);

    // BFT
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Proposal>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Proposal.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Vote>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Vote.class))
        .in(Scopes.SINGLETON);

    // BFT Sync
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetVerticesResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetVerticesResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetVerticesErrorResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetVerticesErrorResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, MempoolAdd>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(MempoolAdd.class))
        .in(Scopes.SINGLETON);

    final var unexpecedEventKey = new TypeLiteral<EventProcessor<ConsensusByzantineEvent>>() {};
    Multibinder.newSetBinder(binder(), unexpecedEventKey, ProcessOnDispatch.class);

    final var scheduledTimeoutKey = new TypeLiteral<EventProcessor<ScheduledLocalTimeout>>() {};
    Multibinder.newSetBinder(binder(), scheduledTimeoutKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), scheduledTimeoutKey);

    final var syncRequestKey = new TypeLiteral<EventProcessor<LocalSyncRequest>>() {};
    Multibinder.newSetBinder(binder(), syncRequestKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), syncRequestKey);

    final var timeoutOccurrenceKey = new TypeLiteral<EventProcessor<LocalTimeoutOccurrence>>() {};
    Multibinder.newSetBinder(binder(), timeoutOccurrenceKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), timeoutOccurrenceKey);
    bind(new TypeLiteral<EventDispatcher<EpochLocalTimeoutOccurrence>>() {})
        .toProvider(Dispatchers.dispatcherProvider(EpochLocalTimeoutOccurrence.class))
        .in(Scopes.SINGLETON);

    final var proposalRejectedKey = new TypeLiteral<EventProcessor<ProposalRejected>>() {};
    Multibinder.newSetBinder(binder(), proposalRejectedKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), proposalRejectedKey);

    bind(new TypeLiteral<EventDispatcher<EpochProposalRejected>>() {})
        .toProvider(Dispatchers.dispatcherProvider(EpochProposalRejected.class))
        .in(Scopes.SINGLETON);

    final var roundUpdateKey = new TypeLiteral<EventProcessor<RoundUpdate>>() {};
    Multibinder.newSetBinder(binder(), roundUpdateKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), roundUpdateKey);

    bind(new TypeLiteral<EventDispatcher<EpochRoundUpdate>>() {})
        .toProvider(Dispatchers.dispatcherProvider(EpochRoundUpdate.class))
        .in(Scopes.SINGLETON);

    bind(new TypeLiteral<EventDispatcher<LedgerUpdate>>() {})
        .toProvider(Dispatchers.dispatcherProvider(LedgerUpdate.class))
        .in(Scopes.SINGLETON);

    final var insertUpdateKey = new TypeLiteral<EventProcessor<BFTInsertUpdate>>() {};
    Multibinder.newSetBinder(binder(), insertUpdateKey, ProcessOnDispatch.class);
    final var highQcUpdateKey = new TypeLiteral<EventProcessor<BFTHighQCUpdate>>() {};
    Multibinder.newSetBinder(binder(), highQcUpdateKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), highQcUpdateKey);
    final var committedUpdateKey = new TypeLiteral<EventProcessor<BFTCommittedUpdate>>() {};
    Multibinder.newSetBinder(binder(), committedUpdateKey);
    Multibinder.newSetBinder(binder(), committedUpdateKey, ProcessOnDispatch.class);
    final var syncUpdateKey = new TypeLiteral<EventProcessor<CommittedTransactionsWithProof>>() {};
    Multibinder.newSetBinder(binder(), syncUpdateKey, ProcessOnDispatch.class);

    final var verticesRequestKey = new TypeLiteral<EventProcessor<GetVerticesRequest>>() {};
    Multibinder.newSetBinder(binder(), verticesRequestKey, ProcessOnDispatch.class);

    bind(new TypeLiteral<EventDispatcher<RoundQuorumReached>>() {})
        .toProvider(
            Dispatchers.dispatcherProvider(
                RoundQuorumReached.class,
                (counters, event) -> {
                  if (event.votingResult() instanceof RoundVotingResult.FormedTC) {
                    counters.bft().timeoutQuorums().inc();
                  } else {
                    counters.bft().voteQuorums().inc();
                  }
                }));

    Multibinder.newSetBinder(binder(), new TypeLiteral<EventProcessorOnDispatch<?>>() {});

    configureP2p();
    configureSync();
  }

  private void configureP2p() {
    // Local Dispatchers
    bind(new TypeLiteral<EventDispatcher<PeerEvent>>() {})
        .toProvider(Dispatchers.dispatcherProvider(PeerEvent.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<PeersLivenessCheckTrigger>>() {})
        .toProvider(Dispatchers.dispatcherProvider(PeersLivenessCheckTrigger.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<PeerPingTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(PeerPingTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<ScheduledEventDispatcher<PeerOutboundConnectionTimeout>>() {})
        .toProvider(Dispatchers.scheduledDispatcherProvider(PeerOutboundConnectionTimeout.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<DiscoverPeers>>() {})
        .toProvider(Dispatchers.dispatcherProvider(DiscoverPeers.class))
        .in(Scopes.SINGLETON);

    // Remote Dispatchers
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Ping>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Ping.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, Pong>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Pong.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, GetPeers>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetPeers.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, PeersResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(PeersResponse.class))
        .in(Scopes.SINGLETON);
  }

  private void configureSync() {
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, StatusRequest>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(StatusRequest.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, StatusResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(StatusResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, SyncRequest>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(SyncRequest.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, SyncResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(SyncResponse.class))
        .in(Scopes.SINGLETON);

    bind(new TypeLiteral<RemoteEventDispatcher<NodeId, LedgerStatusUpdate>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(LedgerStatusUpdate.class))
        .in(Scopes.SINGLETON);
  }

  @Provides
  private EventDispatcher<LocalSyncRequest> localSyncRequestEventDispatcher(
      @Self NodeId self,
      @ProcessOnDispatch Set<EventProcessor<LocalSyncRequest>> syncProcessors,
      Environment environment,
      Metrics metrics) {
    var envDispatcher = environment.getDispatcher(LocalSyncRequest.class);
    return req -> {
      if (logger.isTraceEnabled()) {
        var callingClass =
            StackWalker.getInstance(StackWalker.Option.RETAIN_CLASS_REFERENCE).getCallerClass();
        logger.trace("LOCAL_SYNC_REQUEST dispatched by {}", callingClass);
      }

      if (req.getTargetNodes().contains(self)) {
        throw new IllegalStateException("Should not be targeting self.");
      }

      Gauge syncTargetStateVersion = metrics.sync().targetStateVersion();
      syncTargetStateVersion.set(
          Math.max(syncTargetStateVersion.labels().get(), req.getTarget().getStateVersion()));

      syncProcessors.forEach(e -> e.process(req));
      envDispatcher.dispatch(req);
    };
  }

  @Provides
  private ScheduledEventDispatcher<ScheduledLocalTimeout> scheduledTimeoutDispatcher(
      @ProcessOnDispatch Set<EventProcessor<ScheduledLocalTimeout>> processors,
      Environment environment) {
    var dispatcher = environment.getScheduledDispatcher(ScheduledLocalTimeout.class);
    return (timeout, ms) -> {
      dispatcher.dispatch(timeout, ms);
      processors.forEach(e -> e.process(timeout));
    };
  }

  @Provides
  private EventDispatcher<BFTInsertUpdate> bftInsertUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<BFTInsertUpdate>> processors,
      Environment environment,
      Metrics metrics) {
    var dispatcher = environment.getDispatcher(BFTInsertUpdate.class);
    return update -> {
      if (update.getSiblingsCount() > 1) {
        metrics.bft().vertexStore().forks().inc();
      }
      if (!update.getInserted().getVertexWithHash().vertex().hasDirectParent()) {
        metrics.bft().vertexStore().indirectParents().inc();
      }
      metrics.bft().vertexStore().size().set(update.getVertexStoreSize());
      dispatcher.dispatch(update);
      processors.forEach(p -> p.process(update));
    };
  }

  @Provides
  private EventDispatcher<BFTRebuildUpdate> bftRebuildUpdateEventDispatcher(
      Environment environment, Metrics metrics) {
    var dispatcher = environment.getDispatcher(BFTRebuildUpdate.class);
    return update -> {
      metrics.bft().vertexStore().size().set(update.getVertexStoreState().getVertices().size());
      metrics.bft().vertexStore().rebuilds().inc();
      dispatcher.dispatch(update);
    };
  }

  @Provides
  private EventDispatcher<BFTHighQCUpdate> bftHighQCUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<BFTHighQCUpdate>> processors, Environment environment) {
    var dispatcher = environment.getDispatcher(BFTHighQCUpdate.class);
    return update -> {
      dispatcher.dispatch(update);
      processors.forEach(p -> p.process(update));
    };
  }

  @Provides
  private EventDispatcher<CommittedTransactionsWithProof> syncUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<CommittedTransactionsWithProof>> processors,
      Metrics metrics) {
    return commit -> {
      metrics.sync().validResponsesReceived().inc(commit.getTransactions().size());
      processors.forEach(e -> e.process(commit));
    };
  }

  @Provides
  private EventDispatcher<BFTCommittedUpdate> committedUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<BFTCommittedUpdate>> processors,
      Set<EventProcessor<BFTCommittedUpdate>> asyncProcessors,
      Environment environment,
      Metrics metrics) {
    if (asyncProcessors.isEmpty()) {
      return commit -> {
        metrics.bft().committedVertices().inc(commit.committed().size());
        metrics.bft().vertexStore().size().set(commit.vertexStoreSize());
        processors.forEach(e -> e.process(commit));
      };
    } else {
      var dispatcher = environment.getDispatcher(BFTCommittedUpdate.class);
      return commit -> {
        metrics.bft().committedVertices().inc(commit.committed().size());
        metrics.bft().vertexStore().size().set(commit.vertexStoreSize());
        processors.forEach(e -> e.process(commit));
        dispatcher.dispatch(commit);
      };
    }
  }

  @Provides
  private EventDispatcher<LocalTimeoutOccurrence> localConsensusTimeoutDispatcher(
      @ProcessOnDispatch Set<EventProcessor<LocalTimeoutOccurrence>> syncProcessors,
      Set<EventProcessor<LocalTimeoutOccurrence>> asyncProcessors,
      Environment environment) {
    if (asyncProcessors.isEmpty()) {
      return roundTimeout -> syncProcessors.forEach(e -> e.process(roundTimeout));
    } else {
      var dispatcher = environment.getDispatcher(LocalTimeoutOccurrence.class);
      return timeout -> {
        syncProcessors.forEach(e -> e.process(timeout));
        dispatcher.dispatch(timeout);
      };
    }
  }

  @Provides
  private RemoteEventDispatcher<NodeId, GetVerticesRequest> verticesRequestDispatcher(
      @ProcessOnDispatch Set<EventProcessor<GetVerticesRequest>> processors,
      Environment environment,
      Metrics metrics) {
    var dispatcher = environment.getRemoteDispatcher(GetVerticesRequest.class);
    return (node, request) -> {
      metrics.bft().sync().requestsSent().inc();
      dispatcher.dispatch(node, request);
      processors.forEach(e -> e.process(request));
    };
  }

  @Provides
  @Singleton
  private EventDispatcher<RoundUpdate> roundUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<RoundUpdate>> processors, Environment environment) {
    var dispatcher = environment.getDispatcher(RoundUpdate.class);
    return roundUpdate -> {
      processors.forEach(e -> e.process(roundUpdate));
      dispatcher.dispatch(roundUpdate);
    };
  }

  @Provides
  @Singleton
  private EventDispatcher<ProposalRejected> proposalRejectedDispatcher(
      @ProcessOnDispatch Set<EventProcessor<ProposalRejected>> processors,
      Environment environment) {
    final var dispatcher = environment.getDispatcher(ProposalRejected.class);
    return proposalRejected -> {
      processors.forEach(e -> e.process(proposalRejected));
      dispatcher.dispatch(proposalRejected);
    };
  }
}
