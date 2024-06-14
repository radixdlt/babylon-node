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

import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.*;
import com.google.inject.multibindings.Multibinder;
import com.google.inject.multibindings.OptionalBinder;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.processor.BFTQuorumAssembler.TimeoutQuorumDelayedResolution;
import com.radixdlt.consensus.liveness.*;
import com.radixdlt.consensus.sync.*;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.consensus.vertexstore.VertexStoreConfig;
import com.radixdlt.consensus.vertexstore.VertexStoreJavaImpl;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.*;
import com.radixdlt.ledger.LedgerProofBundle;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.messaging.core.GetVerticesRequestRateLimit;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import com.radixdlt.utils.TimeSupplier;
import java.util.Comparator;
import java.util.Random;

/** Module which allows for consensus to have multiple epochs */
public class EpochsConsensusModule extends AbstractModule {
  @Override
  protected void configure() {
    bind(MultiFactorPacemakerTimeoutCalculator.class).in(Scopes.SINGLETON);
    bind(PacemakerTimeoutCalculator.class).to(MultiFactorPacemakerTimeoutCalculator.class);

    OptionalBinder.newOptionalBinder(
        binder(), EpochManager.class); // So that this is consistent with tests
    bind(EpochManager.class).in(Scopes.SINGLETON);
    var eventBinder =
        Multibinder.newSetBinder(binder(), new TypeLiteral<Class<?>>() {}, LocalEvents.class)
            .permitDuplicates();

    // Non-epoched local events
    eventBinder.addBinding().toInstance(BFTRebuildUpdate.class);
    eventBinder.addBinding().toInstance(BFTInsertUpdate.class);
    eventBinder.addBinding().toInstance(BFTHighQCUpdate.class);
    eventBinder.addBinding().toInstance(Proposal.class);
    eventBinder.addBinding().toInstance(Vote.class);
    eventBinder.addBinding().toInstance(LedgerUpdate.class);
    eventBinder.addBinding().toInstance(VertexRequestTimeout.class);

    // Epoched local events
    eventBinder.addBinding().toInstance(EpochRoundUpdate.class);
    eventBinder.addBinding().toInstance(EpochProposalRejected.class);
    eventBinder.addBinding().toInstance(EpochLocalTimeoutOccurrence.class);
    eventBinder.addBinding().toInstance(Epoched.class);
  }

  @ProvidesIntoSet
  private StartProcessorOnRunner startProcessor(EpochManager epochManager) {
    return new StartProcessorOnRunner(Runners.CONSENSUS, epochManager::start);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> localVoteProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, Vote.class, epochManager::processConsensusEvent);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> localProposalProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, Proposal.class, epochManager::processConsensusEvent);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> epochTimeoutProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS,
        new TypeLiteral<Epoched<ScheduledLocalTimeout>>() {},
        epochManager::processLocalTimeout);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> timeoutQuorumDelayedResolutionProcessor(
      EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS,
        new TypeLiteral<Epoched<TimeoutQuorumDelayedResolution>>() {},
        epochManager::processTimeoutQuorumDelayedResolution);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> epochsLedgerUpdateEventProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, LedgerUpdate.class, epochManager.epochsLedgerUpdateEventProcessor());
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> bftUpdateProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, BFTInsertUpdate.class, epochManager::processBFTUpdate);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> bftRebuildUpdateEventProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, BFTRebuildUpdate.class, epochManager.bftRebuildUpdateEventProcessor());
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> bftSyncTimeoutProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, VertexRequestTimeout.class, epochManager.timeoutEventProcessor());
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> epochRoundUpdateEventProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, EpochRoundUpdate.class, epochManager.epochRoundUpdateEventProcessor());
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> epochProposalRejectedEventProcessor(EpochManager epochManager) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS,
        EpochProposalRejected.class,
        epochManager.epochProposalRejectedEventProcessor());
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> remoteVoteProcessor(EpochManager epochManager) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        Vote.class,
        (node, vote) -> epochManager.processConsensusEvent(vote));
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> remoteProposalProcessor(EpochManager epochManager) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        Proposal.class,
        (node, proposal) -> epochManager.processConsensusEvent(proposal));
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> getVerticesRequestRemoteEventProcessor(
      EpochManager epochManager) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        GetVerticesRequest.class,
        epochManager.bftSyncRequestProcessor());
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> responseRemoteEventProcessor(
      EpochManager epochManager) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        GetVerticesResponse.class,
        epochManager.bftSyncResponseProcessor());
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> errorResponseRemoteEventProcessor(
      EpochManager epochManager) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        GetVerticesErrorResponse.class,
        epochManager.bftSyncErrorResponseProcessor());
  }

  @Provides
  private EpochChange initialEpoch(
      LedgerProofBundle latestProof, BFTConfiguration initialBFTConfig) {
    return new EpochChange(latestProof, initialBFTConfig);
  }

  @ProvidesIntoSet
  @ProcessOnDispatch
  private EventProcessor<ScheduledLocalTimeout> initialEpochsTimeoutSender(
      ScheduledEventDispatcher<Epoched<ScheduledLocalTimeout>> localTimeoutSender,
      EpochChange initialEpoch) {
    return localTimeout -> {
      var epochTimeout = new Epoched(initialEpoch.nextEpoch(), localTimeout);
      localTimeoutSender.dispatch(epochTimeout, localTimeout.millisecondsWaitTime());
    };
  }

  @ProvidesIntoSet
  @ProcessOnDispatch
  private EventProcessor<TimeoutQuorumDelayedResolution>
      initialEpochtimeoutQuorumDelayedResolutionDispatcher(
          ScheduledEventDispatcher<Epoched<TimeoutQuorumDelayedResolution>>
              epochedtimeoutQuorumDelayedResolutionDispatcher,
          EpochChange initialEpoch) {
    return timeoutQuorumDelayedResolution -> {
      final var epochedTimeoutQuorumDelayedResolution =
          new Epoched(initialEpoch.nextEpoch(), timeoutQuorumDelayedResolution);
      epochedtimeoutQuorumDelayedResolutionDispatcher.dispatch(
          epochedTimeoutQuorumDelayedResolution,
          timeoutQuorumDelayedResolution.millisecondsWaitTime());
    };
  }

  @ProvidesIntoSet
  @ProcessOnDispatch
  private EventProcessor<RoundUpdate> initialRoundUpdateToEpochRoundUpdateConverter(
      EventDispatcher<EpochRoundUpdate> epochRoundUpdateEventDispatcher, EpochChange initialEpoch) {
    return roundUpdate -> {
      EpochRoundUpdate epochRoundUpdate =
          new EpochRoundUpdate(initialEpoch.nextEpoch(), roundUpdate);
      epochRoundUpdateEventDispatcher.dispatch(epochRoundUpdate);
    };
  }

  @Provides
  private PacemakerStateFactory pacemakerStateFactory(
      EventDispatcher<EpochRoundUpdate> epochRoundUpdateEventDispatcher,
      Metrics metrics,
      Addressing addressing) {
    return (initialRound, epoch, proposerElection) ->
        new PacemakerState(
            initialRound,
            proposerElection,
            roundUpdate -> {
              EpochRoundUpdate epochRoundUpdate = new EpochRoundUpdate(epoch, roundUpdate);
              epochRoundUpdateEventDispatcher.dispatch(epochRoundUpdate);
            },
            metrics,
            addressing);
  }

  @ProvidesIntoSet
  @ProcessOnDispatch
  EventProcessor<LocalTimeoutOccurrence> initialEpochsTimeoutProcessor(
      EpochChange initialEpoch, EventDispatcher<EpochLocalTimeoutOccurrence> timeoutDispatcher) {
    return timeoutOccurrence ->
        timeoutDispatcher.dispatch(
            new EpochLocalTimeoutOccurrence(initialEpoch.nextEpoch(), timeoutOccurrence));
  }

  @Provides
  private PacemakerFactory pacemakerFactory(
      Metrics metrics,
      ProposalGenerator proposalGenerator,
      Hasher hasher,
      EventDispatcher<EpochLocalTimeoutOccurrence> timeoutEventDispatcher,
      ScheduledEventDispatcher<Epoched<ScheduledLocalTimeout>> localTimeoutSender,
      RemoteEventDispatcher<NodeId, Proposal> proposalDispatcher,
      RemoteEventDispatcher<NodeId, Vote> voteDispatcher,
      EventDispatcher<NoVote> noVoteDispatcher,
      TimeSupplier timeSupplier) {
    return (selfValidatorId,
        validatorSet,
        vertexStore,
        timeoutCalculator,
        safetyRules,
        initialRoundUpdate,
        epoch) ->
        new Pacemaker(
            selfValidatorId,
            validatorSet,
            vertexStore,
            safetyRules,
            timeout ->
                timeoutEventDispatcher.dispatch(new EpochLocalTimeoutOccurrence(epoch, timeout)),
            (scheduledTimeout, ms) ->
                localTimeoutSender.dispatch(new Epoched(epoch, scheduledTimeout), ms),
            timeoutCalculator,
            proposalGenerator,
            (n, m) -> {
              var nodeId = NodeId.fromPublicKey(n.getKey());
              proposalDispatcher.dispatch(nodeId, m);
            },
            (n, m) -> {
              var nodeId = NodeId.fromPublicKey(n.getKey());
              voteDispatcher.dispatch(nodeId, m);
            },
            noVoteDispatcher,
            hasher,
            timeSupplier,
            initialRoundUpdate,
            metrics);
  }

  @Provides
  private BFTFactory bftFactory(
      Hasher hasher,
      HashVerifier verifier,
      TimeSupplier timeSupplier,
      Metrics metrics,
      Addressing addressing,
      EventDispatcher<RoundQuorumResolution> roundQuorumResolutionEventDispatcher,
      ScheduledEventDispatcher<Epoched<TimeoutQuorumDelayedResolution>>
          timeoutQuorumDelayedResolutionDispatcher,
      EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher,
      EventDispatcher<EpochProposalRejected> proposalRejectedDispatcher,
      @TimeoutQuorumResolutionDelayMs long timeoutQuorumResolutionDelayMs) {
    return (self,
        pacemaker,
        bftSyncer,
        roundQuorumResolutionEventProcessor,
        validatorSet,
        roundUpdate,
        safetyRules,
        epoch,
        proposerElection) ->
        BFTBuilder.create()
            .self(self)
            .hasher(hasher)
            .verifier(verifier)
            .proposalRejectedDispatcher(
                proposalRejected ->
                    proposalRejectedDispatcher.dispatch(
                        new EpochProposalRejected(epoch, proposalRejected)))
            .safetyRules(safetyRules)
            .pacemaker(pacemaker)
            .roundQuorumResolutionDispatcher(
                roundQuorumResolution -> {
                  // FIXME: a hack for now until replacement of epochmanager factories
                  roundQuorumResolutionEventProcessor.process(roundQuorumResolution);
                  roundQuorumResolutionEventDispatcher.dispatch(roundQuorumResolution);
                })
            .timeoutQuorumDelayedResolutionDispatcher(
                (timeoutQuorumDelayedResolution, ms) -> {
                  timeoutQuorumDelayedResolutionDispatcher.dispatch(
                      new Epoched(epoch, timeoutQuorumDelayedResolution), ms);
                })
            .timeoutQuorumResolutionDelayMs(timeoutQuorumResolutionDelayMs)
            .doubleVoteDispatcher(doubleVoteEventDispatcher)
            .roundUpdate(roundUpdate)
            .bftSyncer(bftSyncer)
            .validatorSet(validatorSet)
            .timeSupplier(timeSupplier)
            .proposerElection(proposerElection)
            .metrics(metrics)
            .addressing(addressing)
            .build();
  }

  @Provides
  private BFTSyncRequestProcessorFactory vertexStoreSyncVerticesRequestProcessorFactory(
      RemoteEventDispatcher<NodeId, GetVerticesErrorResponse> errorResponseRemoteEventDispatcher,
      RemoteEventDispatcher<NodeId, GetVerticesResponse> responseRemoteEventDispatcher,
      Metrics metrics) {
    return (vertexStore) ->
        new VertexStoreBFTSyncRequestProcessor(
            vertexStore,
            errorResponseRemoteEventDispatcher,
            responseRemoteEventDispatcher,
            metrics);
  }

  @Provides
  private BFTSyncFactory bftSyncFactory(
      @GetVerticesRequestRateLimit RateLimiter syncRequestRateLimiter,
      EventDispatcher<LocalSyncRequest> syncLedgerRequestSender,
      ScheduledEventDispatcher<VertexRequestTimeout> timeoutDispatcher,
      EventDispatcher<ConsensusByzantineEvent> unexpectedEventEventDispatcher,
      RemoteEventDispatcher<NodeId, GetVerticesRequest> verticesRequestRemoteEventDispatcher,
      Random random,
      @BFTSyncPatienceMillis int bftSyncPatienceMillis,
      Metrics metrics,
      Hasher hasher) {
    return (selfValidatorId, safetyRules, vertexStore, pacemakerState, latestProof) ->
        new BFTSync(
            selfValidatorId,
            syncRequestRateLimiter,
            vertexStore,
            hasher,
            safetyRules,
            pacemakerState,
            Comparator.comparingLong(LedgerHeader::getStateVersion),
            verticesRequestRemoteEventDispatcher,
            syncLedgerRequestSender,
            timeoutDispatcher,
            unexpectedEventEventDispatcher,
            latestProof,
            random,
            bftSyncPatienceMillis,
            metrics);
  }

  @Provides
  private VertexStoreFactory vertexStoreFactory(
      EventDispatcher<BFTInsertUpdate> updateSender,
      EventDispatcher<BFTRebuildUpdate> rebuildUpdateDispatcher,
      EventDispatcher<BFTHighQCUpdate> highQCUpdateEventDispatcher,
      Ledger ledger,
      Hasher hasher,
      Serialization serialization,
      Metrics metrics,
      VertexStoreConfig vertexStoreConfig) {
    return vertexStoreState ->
        new VertexStoreAdapter(
            new VertexStoreJavaImpl(
                ledger, hasher, serialization, metrics, vertexStoreConfig, vertexStoreState),
            highQCUpdateEventDispatcher,
            updateSender,
            rebuildUpdateDispatcher);
  }
}
