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

package com.radixdlt.environment;

import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.*;
import com.google.inject.multibindings.Multibinder;
import com.google.inject.multibindings.OptionalBinder;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.processor.BFTEventProcessor;
import com.radixdlt.consensus.bft.processor.BFTQuorumAssembler.TimeoutQuorumDelayedResolution;
import com.radixdlt.consensus.epoch.EpochManager;
import com.radixdlt.consensus.liveness.*;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.safety.SafetyState;
import com.radixdlt.consensus.sync.*;
import com.radixdlt.consensus.vertexstore.VertexStore;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.consensus.vertexstore.VertexStoreJavaImpl;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.messaging.core.GetVerticesRequestRateLimit;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.LastProof;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import com.radixdlt.utils.TimeSupplier;
import java.util.Comparator;
import java.util.Random;

/** A module used in tests - configures a BFT validator logic for a single epoch. */
public class NoEpochsConsensusModule extends AbstractModule {

  @Override
  public void configure() {
    bind(PacemakerState.class).in(Scopes.SINGLETON);
    bind(PacemakerReducer.class).to(PacemakerState.class);
    bind(ExponentialPacemakerTimeoutCalculator.class).in(Scopes.SINGLETON);
    bind(PacemakerTimeoutCalculator.class).to(ExponentialPacemakerTimeoutCalculator.class);

    OptionalBinder.newOptionalBinder(
        binder(), EpochManager.class); // So that this is consistent with tests
    var eventBinder =
        Multibinder.newSetBinder(binder(), new TypeLiteral<Class<?>>() {}, LocalEvents.class)
            .permitDuplicates();

    eventBinder.addBinding().toInstance(TimeoutQuorumDelayedResolution.class);
    eventBinder.addBinding().toInstance(RoundUpdate.class);
    eventBinder.addBinding().toInstance(BFTRebuildUpdate.class);
    eventBinder.addBinding().toInstance(BFTInsertUpdate.class);
    eventBinder.addBinding().toInstance(Proposal.class);
    eventBinder.addBinding().toInstance(Vote.class);
    eventBinder.addBinding().toInstance(LedgerUpdate.class);
    eventBinder.addBinding().toInstance(ScheduledLocalTimeout.class);
    eventBinder.addBinding().toInstance(VertexRequestTimeout.class);
    eventBinder.addBinding().toInstance(ProposalRejected.class);
  }

  @Provides
  @Singleton
  public VertexStoreBFTSyncRequestProcessor syncRequestProcessor(
      VertexStoreAdapter vertexStore,
      RemoteEventDispatcher<NodeId, GetVerticesErrorResponse> errorResponseDispatcher,
      RemoteEventDispatcher<NodeId, GetVerticesResponse> responseDispatcher,
      Metrics metrics) {
    return new VertexStoreBFTSyncRequestProcessor(
        vertexStore, errorResponseDispatcher, responseDispatcher, metrics);
  }

  @Provides
  @Singleton
  public ProposerElection proposerElection(BFTConfiguration configuration) {
    return configuration.getProposerElection();
  }

  @Provides
  @Singleton
  public BFTEventProcessor bftEventProcessor(
      SelfValidatorInfo self,
      BFTConfiguration config,
      Pacemaker pacemaker,
      BFTSync bftSync,
      SafetyRules safetyRules,
      Hasher hasher,
      HashVerifier verifier,
      TimeSupplier timeSupplier,
      ProposerElection proposerElection,
      Metrics metrics,
      EventDispatcher<RoundQuorumResolution> roundQuorumResolutionEventDispatcher,
      ScheduledEventDispatcher<TimeoutQuorumDelayedResolution>
          timeoutQuorumDelayedResolutionDispatcher,
      EventDispatcher<ConsensusByzantineEvent> doubleVoteEventDispatcher,
      EventDispatcher<ProposalRejected> proposalRejectedDispatcher,
      RoundUpdate roundUpdate,
      @TimeoutQuorumResolutionDelayMs long timeoutQuorumResolutionDelayMs) {
    /*
    TODO: consider cleaning this up (but most probably it's not worth it :))
    This is a little hacky.
    We always instantiate NoEpochsConsensusModule (if epochs aren't configured),
    regardless of whether this test node is a validator or a full node
    (because that's how FunctionalRadixNodeModule currently works).
    So for now, if the node is not a validator we're just going to create a dummy
    BFT instance (i.e. Pacemaker, BFTSync, etc). Dummy in this context means
    a complete instance, but configured with "self" validator ID that is not
    present in the current validator set (so it won't be processing any events).
     */
    final var selfValidatorId = self.validatorIdOrFakeForTesting();

    return BFTBuilder.create()
        .self(selfValidatorId)
        .hasher(hasher)
        .verifier(verifier)
        .proposalRejectedDispatcher(proposalRejectedDispatcher)
        .safetyRules(safetyRules)
        .pacemaker(pacemaker)
        .roundQuorumResolutionDispatcher(
            roundQuorumResolution -> {
              // FIXME: a hack for now until replacement of epochmanager factories
              bftSync.roundQuorumResolutionEventProcessor().process(roundQuorumResolution);
              roundQuorumResolutionEventDispatcher.dispatch(roundQuorumResolution);
            })
        .timeoutQuorumDelayedResolutionDispatcher(timeoutQuorumDelayedResolutionDispatcher)
        .timeoutQuorumResolutionDelayMs(timeoutQuorumResolutionDelayMs)
        .doubleVoteDispatcher(doubleVoteEventDispatcher)
        .roundUpdate(roundUpdate)
        .bftSyncer(bftSync)
        .validatorSet(config.getValidatorSet())
        .timeSupplier(timeSupplier)
        .metrics(metrics)
        .proposerElection(proposerElection)
        .build();
  }

  @Provides
  @Singleton
  private SafetyRules safetyRules(
      SelfValidatorInfo self,
      SafetyState initialState,
      PersistentSafetyStateStore persistentSafetyStateStore,
      Hasher hasher,
      HashSigner signer,
      HashVerifier hashVerifier,
      BFTValidatorSet validatorSet) {
    return new SafetyRules(
        self.validatorIdOrFakeForTesting(),
        initialState,
        persistentSafetyStateStore,
        hasher,
        signer,
        hashVerifier,
        validatorSet);
  }

  @Provides
  @Singleton
  private Pacemaker pacemaker(
      SelfValidatorInfo self,
      SafetyRules safetyRules,
      BFTConfiguration configuration,
      VertexStoreAdapter vertexStore,
      EventDispatcher<LocalTimeoutOccurrence> timeoutDispatcher,
      ScheduledEventDispatcher<ScheduledLocalTimeout> timeoutSender,
      PacemakerTimeoutCalculator timeoutCalculator,
      ProposalGenerator proposalGenerator,
      Hasher hasher,
      RemoteEventDispatcher<NodeId, Proposal> proposalDispatcher,
      RemoteEventDispatcher<NodeId, Vote> voteDispatcher,
      EventDispatcher<NoVote> noVoteDispatcher,
      TimeSupplier timeSupplier,
      RoundUpdate initialRoundUpdate,
      Metrics metrics) {
    BFTValidatorSet validatorSet = configuration.getValidatorSet();
    return new Pacemaker(
        self.validatorIdOrFakeForTesting(),
        validatorSet,
        vertexStore,
        safetyRules,
        timeoutDispatcher,
        timeoutSender,
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
  @Singleton
  private BFTSync bftSync(
      SelfValidatorInfo self,
      @GetVerticesRequestRateLimit RateLimiter syncRequestRateLimiter,
      VertexStoreAdapter vertexStore,
      PacemakerReducer pacemakerReducer,
      RemoteEventDispatcher<NodeId, GetVerticesRequest> requestSender,
      EventDispatcher<LocalSyncRequest> syncLedgerRequestSender,
      ScheduledEventDispatcher<VertexRequestTimeout> timeoutDispatcher,
      EventDispatcher<ConsensusByzantineEvent> unexpectedEventEventDispatcher,
      @LastProof LedgerProof ledgerLastProof,
      Random random,
      @BFTSyncPatienceMillis int bftSyncPatienceMillis,
      Hasher hasher,
      SafetyRules safetyRules,
      Metrics metrics) {
    return new BFTSync(
        self.validatorIdOrFakeForTesting(),
        syncRequestRateLimiter,
        vertexStore,
        hasher,
        safetyRules,
        pacemakerReducer,
        Comparator.comparingLong(LedgerHeader::getStateVersion),
        requestSender,
        syncLedgerRequestSender,
        timeoutDispatcher,
        unexpectedEventEventDispatcher,
        ledgerLastProof.getHeader(),
        random,
        bftSyncPatienceMillis,
        metrics);
  }

  @Provides
  @Singleton
  private VertexStore vertexStore(BFTConfiguration bftConfiguration, Ledger ledger, Hasher hasher) {
    return VertexStoreJavaImpl.create(bftConfiguration.getVertexStoreState(), ledger, hasher);
  }

  @Provides
  @Singleton
  private VertexStoreAdapter vertexStoreAdapter(
      VertexStore vertexStore,
      EventDispatcher<BFTInsertUpdate> updateSender,
      EventDispatcher<BFTRebuildUpdate> rebuildUpdateDispatcher,
      EventDispatcher<BFTHighQCUpdate> highQCUpdateEventDispatcher,
      EventDispatcher<BFTCommittedUpdate> committedSender) {
    return new VertexStoreAdapter(
        vertexStore,
        highQCUpdateEventDispatcher,
        updateSender,
        rebuildUpdateDispatcher,
        committedSender);
  }

  @ProvidesIntoSet
  private StartProcessorOnRunner startProcessor(BFTEventProcessor processor) {
    return new StartProcessorOnRunner(Runners.CONSENSUS, processor::start);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> proposalProcessor(BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, Proposal.class, processor::processProposal);
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> remoteProposalProcessor(BFTEventProcessor processor) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        Proposal.class,
        (node, proposal) -> processor.processProposal(proposal));
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> voteProcessor(BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(Runners.CONSENSUS, Vote.class, processor::processVote);
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> remoteVoteProcessor(BFTEventProcessor processor) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS, NodeId.class, Vote.class, (node, vote) -> processor.processVote(vote));
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> timeoutProcessor(BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, ScheduledLocalTimeout.class, processor::processLocalTimeout);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> timeoutQuorumDelayedResolutionProcessor(
      BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS,
        TimeoutQuorumDelayedResolution.class,
        processor::processTimeoutQuorumDelayedResolution);
  }

  @ProvidesIntoSet
  public EventProcessorOnRunner<?> bftSyncTimeoutProcessor(BFTSync bftSync) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS,
        VertexRequestTimeout.class,
        bftSync.vertexRequestTimeoutEventProcessor());
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> roundUpdateProcessor(BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, RoundUpdate.class, processor::processRoundUpdate);
  }

  @ProvidesIntoSet
  private EventProcessorOnRunner<?> proposalRejectedProcessor(BFTEventProcessor processor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, ProposalRejected.class, processor::processProposalRejected);
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> bftSyncResponseProcessor(BFTSync bftSync) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS, NodeId.class, GetVerticesResponse.class, bftSync.responseProcessor());
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> bftSyncErrorResponseProcessor(BFTSync bftSync) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS,
        NodeId.class,
        GetVerticesErrorResponse.class,
        bftSync.errorResponseProcessor());
  }

  @ProvidesIntoSet
  private RemoteEventProcessorOnRunner<?, ?> bftSyncRequestProcessor(
      VertexStoreBFTSyncRequestProcessor processor) {
    return new RemoteEventProcessorOnRunner<>(
        Runners.CONSENSUS, NodeId.class, GetVerticesRequest.class, processor);
  }

  @ProvidesIntoSet
  public EventProcessorOnRunner<?> bftRebuildUpdateEventProcessor(
      BFTEventProcessor eventProcessor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, BFTRebuildUpdate.class, eventProcessor::processBFTRebuildUpdate);
  }

  @ProvidesIntoSet
  public EventProcessorOnRunner<?> bftUpdateEventProcessor(BFTEventProcessor eventProcessor) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, BFTInsertUpdate.class, eventProcessor::processBFTUpdate);
  }

  @ProvidesIntoSet
  public EventProcessorOnRunner<?> baseLedgerUpdateEventProcessor(BFTSync bftSync) {
    return new EventProcessorOnRunner<>(
        Runners.CONSENSUS, LedgerUpdate.class, bftSync.baseLedgerUpdateEventProcessor());
  }
}
