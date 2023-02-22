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

import static java.util.Objects.requireNonNull;

import com.google.common.collect.ImmutableSet;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.processor.BFTEventProcessor;
import com.radixdlt.consensus.bft.processor.EmptyBFTEventProcessor;
import com.radixdlt.consensus.liveness.PacemakerFactory;
import com.radixdlt.consensus.liveness.PacemakerStateFactory;
import com.radixdlt.consensus.liveness.PacemakerTimeoutCalculator;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.safety.SafetyState;
import com.radixdlt.consensus.sync.GetVerticesErrorResponse;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.consensus.sync.GetVerticesResponse;
import com.radixdlt.consensus.sync.VertexRequestTimeout;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.RemoteEventProcessor;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.sync.messages.remote.LedgerStatusUpdate;
import java.util.*;
import javax.annotation.concurrent.NotThreadSafe;
import javax.inject.Inject;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Manages Epochs and the BFT instance (which is mostly epoch agnostic) associated with each epoch
 */
@NotThreadSafe
public final class EpochManager {
  private static final Logger log = LogManager.getLogger();
  private final BFTValidatorId self;
  private final PacemakerFactory pacemakerFactory;
  private final VertexStoreFactory vertexStoreFactory;
  private final BFTSyncRequestProcessorFactory bftSyncRequestProcessorFactory;
  private final BFTSyncFactory bftSyncFactory;
  private final Hasher hasher;
  private final HashSigner signer;
  private final HashVerifier hashVerifier;
  private final PacemakerTimeoutCalculator timeoutCalculator;
  private final Metrics metrics;
  private final Map<Long, List<ConsensusEvent>> queuedEvents;
  private final BFTFactory bftFactory;
  private final PacemakerStateFactory pacemakerStateFactory;

  private EpochChange lastEpochChange;

  private ValidationStatus validationStatus;

  private EventProcessor<VertexRequestTimeout> syncTimeoutProcessor;
  private EventProcessor<LedgerUpdate> syncLedgerUpdateProcessor;
  private BFTEventProcessor bftEventProcessor;

  private Set<RemoteEventProcessor<NodeId, GetVerticesRequest>> syncRequestProcessors;
  private Set<RemoteEventProcessor<NodeId, GetVerticesResponse>> syncResponseProcessors;
  private Set<RemoteEventProcessor<NodeId, GetVerticesErrorResponse>> syncErrorResponseProcessors;

  private Set<EventProcessor<BFTInsertUpdate>> bftUpdateProcessors;
  private Set<EventProcessor<BFTRebuildUpdate>> bftRebuildProcessors;

  private final RemoteEventDispatcher<NodeId, LedgerStatusUpdate> ledgerStatusUpdateDispatcher;
  private final PersistentSafetyStateStore persistentSafetyStateStore;

  private final HashSet<NodeId> validatorNodeIds = new HashSet<>();

  @Inject
  public EpochManager(
      @Self BFTValidatorId self,
      RemoteEventDispatcher<NodeId, LedgerStatusUpdate> ledgerStatusUpdateDispatcher,
      EpochChange lastEpochChange,
      PacemakerFactory pacemakerFactory,
      VertexStoreFactory vertexStoreFactory,
      BFTSyncFactory bftSyncFactory,
      BFTSyncRequestProcessorFactory bftSyncRequestProcessorFactory,
      BFTFactory bftFactory,
      Metrics metrics,
      Hasher hasher,
      HashSigner signer,
      HashVerifier hashVerifier,
      PacemakerTimeoutCalculator timeoutCalculator,
      PacemakerStateFactory pacemakerStateFactory,
      SafetyState safetyState,
      PersistentSafetyStateStore persistentSafetyStateStore) {
    this.ledgerStatusUpdateDispatcher = requireNonNull(ledgerStatusUpdateDispatcher);
    this.lastEpochChange = requireNonNull(lastEpochChange);
    this.self = requireNonNull(self);
    this.pacemakerFactory = requireNonNull(pacemakerFactory);
    this.vertexStoreFactory = requireNonNull(vertexStoreFactory);
    this.bftSyncFactory = requireNonNull(bftSyncFactory);
    this.bftSyncRequestProcessorFactory = bftSyncRequestProcessorFactory;
    this.hasher = requireNonNull(hasher);
    this.signer = requireNonNull(signer);
    this.hashVerifier = requireNonNull(hashVerifier);
    this.timeoutCalculator = requireNonNull(timeoutCalculator);
    this.bftFactory = bftFactory;
    this.metrics = requireNonNull(metrics);
    this.pacemakerStateFactory = requireNonNull(pacemakerStateFactory);
    this.persistentSafetyStateStore = requireNonNull(persistentSafetyStateStore);
    this.queuedEvents = new HashMap<>();

    this.updateEpochState(safetyState);
  }

  private void updateEpochState(SafetyState safetyState) {
    var config = this.lastEpochChange.getBFTConfiguration();
    var validatorSet = config.getValidatorSet();

    if (!validatorSet.containsNode(self)) {
      this.bftRebuildProcessors = Set.of();
      this.bftUpdateProcessors = Set.of();
      this.syncRequestProcessors = Set.of();
      this.syncResponseProcessors = Set.of();
      this.syncErrorResponseProcessors = Set.of();
      this.bftEventProcessor = EmptyBFTEventProcessor.INSTANCE;
      this.syncLedgerUpdateProcessor = update -> {};
      this.syncTimeoutProcessor = timeout -> {};
      if (self.getValidatorAddress() == null) {
        this.validationStatus = ValidationStatus.NOT_CONFIGURED_AS_VALIDATOR;
      } else {
        this.validationStatus = ValidationStatus.NOT_VALIDATING_IN_CURRENT_EPOCH;
      }
      return;
    }
    this.validationStatus = ValidationStatus.VALIDATING_IN_CURRENT_EPOCH;

    // TODO: Move this filterign into a separate network module
    this.validatorNodeIds.clear();
    for (var validator : validatorSet.getValidators()) {
      var nodeId = NodeId.fromPublicKey(validator.getValidatorId().getKey());
      validatorNodeIds.add(nodeId);
    }

    final var nextEpoch = this.lastEpochChange.getNextEpoch();

    // Config
    final var bftConfiguration = this.lastEpochChange.getBFTConfiguration();
    final var proposerElection = bftConfiguration.getProposerElection();
    final var highQC = bftConfiguration.getVertexStoreState().getHighQC();
    final var round = highQC.getHighestRound().next();
    final var leader = proposerElection.getProposer(round);
    final var nextLeader = proposerElection.getProposer(round.next());
    final var initialRoundUpdate = RoundUpdate.create(round, highQC, leader, nextLeader);

    // Mutable Consensus State
    final var vertexStore = vertexStoreFactory.create(bftConfiguration.getVertexStoreState());
    final var pacemakerState =
        pacemakerStateFactory.create(initialRoundUpdate, nextEpoch, proposerElection);

    // Consensus Drivers
    final var safetyRules =
        new SafetyRules(
            self,
            safetyState,
            persistentSafetyStateStore,
            hasher,
            signer,
            hashVerifier,
            validatorSet);
    final var pacemaker =
        pacemakerFactory.create(
            validatorSet,
            vertexStore,
            timeoutCalculator,
            safetyRules,
            initialRoundUpdate,
            nextEpoch);
    final var bftSync =
        bftSyncFactory.create(safetyRules, vertexStore, pacemakerState, bftConfiguration);

    this.syncLedgerUpdateProcessor = bftSync.baseLedgerUpdateEventProcessor();
    this.syncTimeoutProcessor = bftSync.vertexRequestTimeoutEventProcessor();

    this.bftEventProcessor =
        bftFactory.create(
            self,
            pacemaker,
            bftSync,
            bftSync.roundQuorumReachedEventProcessor(),
            validatorSet,
            initialRoundUpdate,
            safetyRules,
            nextEpoch,
            proposerElection);

    this.syncResponseProcessors = Set.of(bftSync.responseProcessor());
    this.syncErrorResponseProcessors = Set.of(bftSync.errorResponseProcessor());

    this.syncRequestProcessors = Set.of(bftSyncRequestProcessorFactory.create(vertexStore));
    this.bftRebuildProcessors = Set.of(bftEventProcessor::processBFTRebuildUpdate);
    this.bftUpdateProcessors = Set.of(bftEventProcessor::processBFTUpdate);
  }

  public void start() {
    this.bftEventProcessor.start();
  }

  public ValidationStatus validationStatus() {
    return this.validationStatus;
  }

  private long currentEpoch() {
    return this.lastEpochChange.getNextEpoch();
  }

  public EventProcessor<LedgerUpdate> epochsLedgerUpdateEventProcessor() {
    return this::processLedgerUpdate;
  }

  private void processLedgerUpdate(LedgerUpdate ledgerUpdate) {
    var epochChange = ledgerUpdate.getStateComputerOutput().getInstance(EpochChange.class);

    if (epochChange != null) {
      this.processEpochChange(epochChange);
    } else {
      this.syncLedgerUpdateProcessor.process(ledgerUpdate);
    }
  }

  private void processEpochChange(EpochChange epochChange) {
    // Sanity check
    if (epochChange.getNextEpoch() != this.currentEpoch() + 1) {
      // safe, as message is internal
      throw new IllegalStateException(
          "Bad Epoch change: " + epochChange + " current epoch: " + this.lastEpochChange);
    }

    if (this.lastEpochChange.getBFTConfiguration().getValidatorSet().containsNode(this.self)) {
      final var currentAndNextValidators =
          ImmutableSet.<BFTValidator>builder()
              .addAll(epochChange.getBFTConfiguration().getValidatorSet().getValidators())
              .addAll(this.lastEpochChange.getBFTConfiguration().getValidatorSet().getValidators())
              .build();

      final var ledgerStatusUpdate = LedgerStatusUpdate.create(epochChange.getGenesisHeader());
      for (var validator : currentAndNextValidators) {
        if (!validator.getValidatorId().equals(self)) {
          var nodeId = NodeId.fromPublicKey(validator.getValidatorId().getKey());
          this.ledgerStatusUpdateDispatcher.dispatch(nodeId, ledgerStatusUpdate);
        }
      }
    }

    this.lastEpochChange = epochChange;
    this.updateEpochState(SafetyState.initialState());
    this.bftEventProcessor.start();

    // Execute any queued up consensus events
    final List<ConsensusEvent> queuedEventsForEpoch =
        queuedEvents.getOrDefault(epochChange.getNextEpoch(), Collections.emptyList());
    var highestSeenRound =
        queuedEventsForEpoch.stream()
            .map(ConsensusEvent::getRound)
            .max(Comparator.naturalOrder())
            .orElse(Round.genesis());

    queuedEventsForEpoch.stream()
        .filter(e -> e.getRound().equals(highestSeenRound))
        .forEach(this::processConsensusEventInternal);

    queuedEvents.remove(epochChange.getNextEpoch());
  }

  private void processConsensusEventInternal(ConsensusEvent consensusEvent) {
    this.metrics.bft().eventsReceived().inc();

    switch (consensusEvent) {
      case Proposal proposal -> bftEventProcessor.processProposal(proposal);
      case Vote vote -> bftEventProcessor.processVote(vote);
    }
  }

  public void processConsensusEvent(ConsensusEvent consensusEvent) {
    if (consensusEvent.getEpoch() > this.currentEpoch()) {
      log.debug(
          "{}: CONSENSUS_EVENT: Received higher epoch event: {} current epoch: {}",
          this.self::toString,
          () -> consensusEvent,
          this::currentEpoch);

      // queue higher epoch events for later processing
      // TODO: need to clear this by some rule (e.g. timeout or max size) or else memory leak attack
      // possible
      queuedEvents
          .computeIfAbsent(consensusEvent.getEpoch(), e -> new ArrayList<>())
          .add(consensusEvent);
      metrics.epochManager().enqueuedConsensusEvents().inc();
      return;
    }

    if (consensusEvent.getEpoch() < this.currentEpoch()) {
      log.debug(
          "{}: CONSENSUS_EVENT: Ignoring lower epoch event: {} current epoch: {}",
          this.self::toString,
          () -> consensusEvent,
          this::currentEpoch);
      return;
    }

    this.processConsensusEventInternal(consensusEvent);
  }

  public void processLocalTimeout(Epoched<ScheduledLocalTimeout> localTimeout) {
    if (localTimeout.epoch() != this.currentEpoch()) {
      return;
    }

    bftEventProcessor.processLocalTimeout(localTimeout.event());
  }

  public EventProcessor<EpochRoundUpdate> epochRoundUpdateEventProcessor() {
    return epochRoundUpdate -> {
      if (epochRoundUpdate.getEpoch() != this.currentEpoch()) {
        return;
      }

      log.trace("Processing RoundUpdate: {}", epochRoundUpdate);
      bftEventProcessor.processRoundUpdate(epochRoundUpdate.getRoundUpdate());
    };
  }

  public EventProcessor<EpochProposalRejected> epochProposalRejectedEventProcessor() {
    return epochProposalRejected -> {
      if (epochProposalRejected.epoch() != this.currentEpoch()) {
        return;
      }
      bftEventProcessor.processProposalRejected(epochProposalRejected.proposalRejected());
    };
  }

  public void processBFTUpdate(BFTInsertUpdate update) {
    bftUpdateProcessors.forEach(p -> p.process(update));
  }

  public EventProcessor<BFTRebuildUpdate> bftRebuildUpdateEventProcessor() {
    return update -> {
      if (update
              .getVertexStoreState()
              .getRoot()
              .vertex()
              .getParentHeader()
              .getLedgerHeader()
              .getEpoch()
          != this.currentEpoch()) {
        return;
      }

      bftRebuildProcessors.forEach(p -> p.process(update));
    };
  }

  public RemoteEventProcessor<NodeId, GetVerticesRequest> bftSyncRequestProcessor() {
    return (nodeId, request) -> {
      if (this.validatorNodeIds.contains(nodeId)) {
        syncRequestProcessors.forEach(p -> p.process(nodeId, request));
      }
    };
  }

  public RemoteEventProcessor<NodeId, GetVerticesResponse> bftSyncResponseProcessor() {
    return (nodeId, resp) -> {
      if (this.validatorNodeIds.contains(nodeId)) {
        syncResponseProcessors.forEach(p -> p.process(nodeId, resp));
      }
    };
  }

  public RemoteEventProcessor<NodeId, GetVerticesErrorResponse> bftSyncErrorResponseProcessor() {
    return (nodeId, err) -> {
      if (log.isDebugEnabled()) {
        log.debug("SYNC_ERROR: Received GetVerticesErrorResponse {}", err);
      }

      final var responseEpoch = err.highQC().highestQC().getEpoch();

      if (responseEpoch < this.currentEpoch()) {
        if (log.isDebugEnabled()) {
          log.debug(
              "SYNC_ERROR: Ignoring lower epoch error response: {} current epoch: {}",
              err,
              this.currentEpoch());
        }
        return;
      }
      if (responseEpoch > this.currentEpoch()) {
        if (log.isDebugEnabled()) {
          log.debug(
              "SYNC_ERROR: Received higher epoch error response: {} current epoch: {}",
              err,
              this.currentEpoch());
        }
      } else {
        // Current epoch
        if (this.validatorNodeIds.contains(nodeId)) {
          syncErrorResponseProcessors.forEach(p -> p.process(nodeId, err));
        }
      }
    };
  }

  public EventProcessor<VertexRequestTimeout> timeoutEventProcessor() {
    // Return reference to method rather than syncTimeoutProcessor directly,
    // since syncTimeoutProcessor will change over the time
    return this::processGetVerticesLocalTimeout;
  }

  private void processGetVerticesLocalTimeout(VertexRequestTimeout timeout) {
    syncTimeoutProcessor.process(timeout);
  }
}
