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

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Scopes;
import com.google.inject.Singleton;
import com.google.inject.TypeLiteral;
import com.google.inject.multibindings.Multibinder;
import com.radixdlt.api.system.health.ScheduledStatsCollecting;
import com.radixdlt.consensus.DoubleVote;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTCommittedUpdate;
import com.radixdlt.consensus.bft.BFTHighQCUpdate;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.NoVote;
import com.radixdlt.consensus.bft.RoundLeaderFailure;
import com.radixdlt.consensus.bft.RoundQuorumReached;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.bft.RoundVotingResult;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.epoch.EpochRoundLeaderFailure;
import com.radixdlt.consensus.epoch.EpochRoundUpdate;
import com.radixdlt.consensus.epoch.Epoched;
import com.radixdlt.consensus.liveness.EpochLocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.LocalTimeoutOccurrence;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.sync.GetVerticesErrorResponse;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.consensus.sync.GetVerticesResponse;
import com.radixdlt.consensus.sync.VertexRequestTimeout;
import com.radixdlt.environment.Dispatchers;
import com.radixdlt.environment.Environment;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.EventProcessorOnDispatch;
import com.radixdlt.environment.ProcessOnDispatch;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.mempool.MempoolRelayTrigger;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCounters.CounterType;
import com.radixdlt.p2p.PeerEvent;
import com.radixdlt.p2p.PendingOutboundChannelsManager.PeerOutboundConnectionTimeout;
import com.radixdlt.p2p.discovery.DiscoverPeers;
import com.radixdlt.p2p.discovery.GetPeers;
import com.radixdlt.p2p.discovery.PeersResponse;
import com.radixdlt.p2p.liveness.PeerPingTimeout;
import com.radixdlt.p2p.liveness.PeersLivenessCheckTrigger;
import com.radixdlt.p2p.liveness.Ping;
import com.radixdlt.p2p.liveness.Pong;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import com.radixdlt.sync.messages.local.SyncCheckReceiveStatusTimeout;
import com.radixdlt.sync.messages.local.SyncCheckTrigger;
import com.radixdlt.sync.messages.local.SyncLedgerUpdateTimeout;
import com.radixdlt.sync.messages.local.SyncRequestTimeout;
import com.radixdlt.sync.messages.remote.LedgerStatusUpdate;
import com.radixdlt.sync.messages.remote.StatusRequest;
import com.radixdlt.sync.messages.remote.StatusResponse;
import com.radixdlt.sync.messages.remote.SyncRequest;
import com.radixdlt.sync.messages.remote.SyncResponse;
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
    bind(new TypeLiteral<EventDispatcher<DoubleVote>>() {})
        .toProvider(Dispatchers.dispatcherProvider(DoubleVote.class))
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
            Dispatchers.dispatcherProvider(NoVote.class, v -> CounterType.BFT_NO_VOTES_SENT))
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
    bind(new TypeLiteral<RemoteEventDispatcher<Proposal>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Proposal.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<Vote>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Vote.class))
        .in(Scopes.SINGLETON);

    // BFT Sync
    bind(new TypeLiteral<RemoteEventDispatcher<GetVerticesResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetVerticesResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<GetVerticesErrorResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetVerticesErrorResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<MempoolAdd>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(MempoolAdd.class))
        .in(Scopes.SINGLETON);

    final var doubleVoteKey = new TypeLiteral<EventProcessor<DoubleVote>>() {};
    Multibinder.newSetBinder(binder(), doubleVoteKey, ProcessOnDispatch.class);

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

    final var roundLeaderFailureKey = new TypeLiteral<EventProcessor<RoundLeaderFailure>>() {};
    Multibinder.newSetBinder(binder(), roundLeaderFailureKey, ProcessOnDispatch.class);
    Multibinder.newSetBinder(binder(), roundLeaderFailureKey);

    bind(new TypeLiteral<EventDispatcher<EpochRoundLeaderFailure>>() {})
        .toProvider(Dispatchers.dispatcherProvider(EpochRoundLeaderFailure.class))
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
                v -> {
                  if (v.votingResult() instanceof RoundVotingResult.FormedTC) {
                    return CounterType.BFT_TIMEOUT_QUORUMS;
                  }
                  return CounterType.BFT_VOTE_QUORUMS;
                }));

    Multibinder.newSetBinder(binder(), new TypeLiteral<EventProcessorOnDispatch<?>>() {});

    configureP2p();
    configureSync();
  }

  private void configureP2p() {
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
    bind(new TypeLiteral<RemoteEventDispatcher<Ping>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Ping.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<Pong>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(Pong.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<GetPeers>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(GetPeers.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<PeersResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(PeersResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<EventDispatcher<DiscoverPeers>>() {})
        .toProvider(Dispatchers.dispatcherProvider(DiscoverPeers.class))
        .in(Scopes.SINGLETON);
  }

  private void configureSync() {
    bind(new TypeLiteral<RemoteEventDispatcher<StatusRequest>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(StatusRequest.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<StatusResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(StatusResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<SyncRequest>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(SyncRequest.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<SyncResponse>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(SyncResponse.class))
        .in(Scopes.SINGLETON);
    bind(new TypeLiteral<RemoteEventDispatcher<LedgerStatusUpdate>>() {})
        .toProvider(Dispatchers.remoteDispatcherProvider(LedgerStatusUpdate.class))
        .in(Scopes.SINGLETON);
  }

  @Provides
  private EventDispatcher<LocalSyncRequest> localSyncRequestEventDispatcher(
      @Self BFTNode self,
      @ProcessOnDispatch Set<EventProcessor<LocalSyncRequest>> syncProcessors,
      Environment environment,
      SystemCounters systemCounters) {
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

      long stateVersion = systemCounters.get(CounterType.SYNC_TARGET_STATE_VERSION);
      if (req.getTarget().getStateVersion() > stateVersion) {
        systemCounters.set(
            CounterType.SYNC_TARGET_STATE_VERSION, req.getTarget().getStateVersion());
      }

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
      SystemCounters systemCounters) {
    var dispatcher = environment.getDispatcher(BFTInsertUpdate.class);
    return update -> {
      if (update.getSiblingsCount() > 1) {
        systemCounters.increment(CounterType.BFT_VERTEX_STORE_FORKS);
      }
      if (!update.getInserted().getVertexWithHash().vertex().hasDirectParent()) {
        systemCounters.increment(CounterType.BFT_VERTEX_STORE_INDIRECT_PARENTS);
      }
      systemCounters.set(CounterType.BFT_VERTEX_STORE_SIZE, update.getVertexStoreSize());
      dispatcher.dispatch(update);
      processors.forEach(p -> p.process(update));
    };
  }

  @Provides
  private EventDispatcher<BFTRebuildUpdate> bftRebuildUpdateEventDispatcher(
      Environment environment, SystemCounters systemCounters) {
    var dispatcher = environment.getDispatcher(BFTRebuildUpdate.class);
    return update -> {
      systemCounters.set(
          CounterType.BFT_VERTEX_STORE_SIZE, update.getVertexStoreState().getVertices().size());
      systemCounters.increment(CounterType.BFT_VERTEX_STORE_REBUILDS);
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
      SystemCounters systemCounters) {
    return commit -> {
      systemCounters.add(
          CounterType.SYNC_VALID_RESPONSES_RECEIVED, commit.getTransactions().size());
      processors.forEach(e -> e.process(commit));
    };
  }

  @Provides
  private EventDispatcher<BFTCommittedUpdate> committedUpdateEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<BFTCommittedUpdate>> processors,
      Set<EventProcessor<BFTCommittedUpdate>> asyncProcessors,
      Environment environment,
      SystemCounters systemCounters) {
    if (asyncProcessors.isEmpty()) {
      return commit -> {
        systemCounters.add(CounterType.BFT_COMMITTED_VERTICES, commit.committed().size());
        systemCounters.set(CounterType.BFT_VERTEX_STORE_SIZE, commit.vertexStoreSize());
        processors.forEach(e -> e.process(commit));
      };
    } else {
      var dispatcher = environment.getDispatcher(BFTCommittedUpdate.class);
      return commit -> {
        systemCounters.add(CounterType.BFT_COMMITTED_VERTICES, commit.committed().size());
        systemCounters.set(CounterType.BFT_VERTEX_STORE_SIZE, commit.vertexStoreSize());
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
  private RemoteEventDispatcher<GetVerticesRequest> verticesRequestDispatcher(
      @ProcessOnDispatch Set<EventProcessor<GetVerticesRequest>> processors,
      Environment environment,
      SystemCounters counters) {
    var dispatcher = environment.getRemoteDispatcher(GetVerticesRequest.class);
    return (node, request) -> {
      counters.increment(CounterType.BFT_SYNC_REQUESTS_SENT);
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
  private EventDispatcher<RoundLeaderFailure> roundLeaderFailureEventDispatcher(
      @ProcessOnDispatch Set<EventProcessor<RoundLeaderFailure>> processors,
      Environment environment) {
    final var dispatcher = environment.getDispatcher(RoundLeaderFailure.class);
    return roundLeaderFailure -> {
      processors.forEach(e -> e.process(roundLeaderFailure));
      dispatcher.dispatch(roundLeaderFailure);
    };
  }
}
