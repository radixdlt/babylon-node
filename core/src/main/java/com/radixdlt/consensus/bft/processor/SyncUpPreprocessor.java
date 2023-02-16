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

package com.radixdlt.consensus.bft.processor;

import com.google.common.base.Stopwatch;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.ConsensusEvent;
import com.radixdlt.consensus.HighQC;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.BFTSyncer;
import com.radixdlt.consensus.bft.BFTSyncer.SyncResult;
import com.radixdlt.consensus.bft.ProposalRejected;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.RoundChange.HighQcSource;
import com.radixdlt.p2p.NodeId;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.function.Consumer;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Preprocesses consensus events and ensures that the vertexStore is synced up before forwarding the
 * event to the next processor. This class should not be updating any part of the BFT Safety state
 * besides the VertexStore.
 *
 * <p>This class is NOT thread-safe.
 */
public final class SyncUpPreprocessor implements BFTEventProcessor {
  private record QueuedConsensusEvent(ConsensusEvent event, Stopwatch stopwatch) {}

  private static final Logger log = LogManager.getLogger();

  private final BFTEventProcessorAtCurrentRound forwardTo;
  private final BFTSyncer bftSyncer;
  private final Metrics metrics;
  private final Set<QueuedConsensusEvent> syncingEvents = new HashSet<>();
  private final Map<Round, List<QueuedConsensusEvent>> roundQueues = new HashMap<>();
  private RoundUpdate latestRoundUpdate;

  public SyncUpPreprocessor(
      BFTEventProcessorAtCurrentRound forwardTo,
      BFTSyncer bftSyncer,
      Metrics metrics,
      RoundUpdate initialRoundUpdate) {
    this.forwardTo = Objects.requireNonNull(forwardTo);
    this.bftSyncer = Objects.requireNonNull(bftSyncer);
    this.metrics = Objects.requireNonNull(metrics);
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
  }

  @Override
  public void start() {
    forwardTo.start();
  }

  @Override
  public void processRoundUpdate(RoundUpdate roundUpdate) {
    final Round previousRound = this.latestRoundUpdate.getCurrentRound();
    log.trace("Processing roundUpdate {} cur {}", roundUpdate, previousRound);

    this.latestRoundUpdate = roundUpdate;
    forwardTo.processRoundUpdate(roundUpdate);
    roundQueues
        .getOrDefault(roundUpdate.getCurrentRound(), new LinkedList<>())
        .forEach(this::processRoundCachedEvent);
    roundQueues.keySet().removeIf(v -> v.lte(roundUpdate.getCurrentRound()));

    syncingEvents.stream()
        .filter(e -> e.event.getRound().equals(roundUpdate.getCurrentRound()))
        .forEach(this::processQueuedConsensusEvent);

    syncingEvents.removeIf(e -> e.event.getRound().lte(roundUpdate.getCurrentRound()));
  }

  private void processRoundCachedEvent(QueuedConsensusEvent queuedEvent) {
    metrics.bft().consensusEventsQueueWait().observe(queuedEvent.stopwatch.elapsed());
    switch (queuedEvent.event) {
      case Proposal proposal -> syncUpAndProcess(proposal, forwardTo::processProposal);
      case Vote vote -> syncUpAndProcess(vote, forwardTo::processVote);
    }
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    final var vertexId = update.getInserted().getVertexHash();
    log.trace("LOCAL_SYNC: {}", vertexId);

    syncingEvents.stream()
        .filter(
            e -> e.event.highQC().highestQC().getProposedHeader().getVertexId().equals(vertexId))
        .forEach(this::processQueuedConsensusEvent);

    syncingEvents.removeIf(
        e -> e.event.highQC().highestQC().getProposedHeader().getVertexId().equals(vertexId));

    forwardTo.processBFTUpdate(update);
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate rebuildUpdate) {
    rebuildUpdate
        .getVertexStoreState()
        .getVertices()
        .forEach(
            v -> {
              HashCode vertexId = v.hash();
              syncingEvents.stream()
                  .filter(
                      e ->
                          e.event
                              .highQC()
                              .highestQC()
                              .getProposedHeader()
                              .getVertexId()
                              .equals(vertexId))
                  .forEach(this::processQueuedConsensusEvent);

              syncingEvents.removeIf(
                  e ->
                      e.event
                          .highQC()
                          .highestQC()
                          .getProposedHeader()
                          .getVertexId()
                          .equals(vertexId));
            });
  }

  @Override
  public void processVote(Vote vote) {
    log.trace("SyncUpPreprocessor: processing vote {}", vote);
    this.forwardTo.preProcessUnsyncedVoteForCurrentOrFutureRound(vote);
    syncUpAndProcess(vote, forwardTo::processVote);
  }

  @Override
  public void processProposal(Proposal proposal) {
    log.trace("SyncUpPreprocessor: processing proposal {}", proposal);
    this.forwardTo.preProcessUnsyncedProposalForCurrentOrFutureRound(proposal);
    syncUpAndProcess(proposal, forwardTo::processProposal);
  }

  private <T extends ConsensusEvent> void syncUpAndProcess(T event, Consumer<T> processFn) {
    final Round currentRound = this.latestRoundUpdate.getCurrentRound();
    if (event.getRound().gte(currentRound)) {
      final var highQcSource =
          switch (event) {
            case Vote vote -> HighQcSource.RECEIVED_ALONG_WITH_VOTE;
            case Proposal proposal -> HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL;
            default -> throw new IllegalStateException(
                "T is a sealed ConsensusEvent, this shouldn't be needed, but Java...");
          };
      final var isSynced =
          syncUp(
              event.highQC(),
              NodeId.fromPublicKey(event.getAuthor().getKey()),
              () -> processOnCurrentRoundOrCache(event, processFn),
              highQcSource);
      if (!isSynced) {
        log.debug("Queuing {}, waiting for Sync", event);
        syncingEvents.add(new QueuedConsensusEvent(event, Stopwatch.createStarted()));
      }
    } else {
      log.trace("Ignoring BFT event {} for past round, current round is {}", event, currentRound);
    }
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledLocalTimeout) {
    forwardTo.processLocalTimeout(scheduledLocalTimeout);
  }

  @Override
  public void processProposalRejected(ProposalRejected proposalRejected) {
    forwardTo.processProposalRejected(proposalRejected);
  }

  private void processQueuedConsensusEvent(QueuedConsensusEvent queuedEvent) {
    metrics.bft().consensusEventsQueueWait().observe(queuedEvent.stopwatch.elapsed());
    switch (queuedEvent.event) {
      case Proposal proposal -> syncUp(
          proposal.highQC(),
          NodeId.fromPublicKey(proposal.getAuthor().getKey()),
          () -> processOnCurrentRoundOrCache(proposal, forwardTo::processProposal),
          HighQcSource.RECEIVED_ALONG_WITH_PROPOSAL);
      case Vote vote -> syncUp(
          vote.highQC(),
          NodeId.fromPublicKey(vote.getAuthor().getKey()),
          () -> processOnCurrentRoundOrCache(vote, forwardTo::processVote),
          HighQcSource.RECEIVED_ALONG_WITH_VOTE);
    }
  }

  private <T extends ConsensusEvent> void processOnCurrentRoundOrCache(
      T event, Consumer<T> processFn) {
    if (latestRoundUpdate.getCurrentRound().equals(event.getRound())) {
      processFn.accept(event);
    } else if (latestRoundUpdate.getCurrentRound().lt(event.getRound())) {
      log.trace("Caching {}, current round is {}", event, latestRoundUpdate.getCurrentRound());
      roundQueues.putIfAbsent(event.getRound(), new LinkedList<>());
      roundQueues
          .get(event.getRound())
          .add(new QueuedConsensusEvent(event, Stopwatch.createStarted()));
    } else {
      log.debug("Ignoring {} for past round", event);
    }
  }

  private boolean syncUp(
      HighQC highQC, NodeId author, Runnable whenSynced, HighQcSource highQcSource) {
    SyncResult syncResult = this.bftSyncer.syncToQC(highQC, author, highQcSource);

    // TODO: use switch expression and eliminate unnecessary default case
    switch (syncResult) {
      case SYNCED:
        // if already end of epoch then don't need to process
        // TODO: need to do the same checks on pacemaker side
        // TODO: move this to an epoch preprocessor
        final boolean endOfEpoch =
            highQC
                .highestCommittedQC()
                .getCommittedHeader()
                .orElseThrow(() -> new IllegalStateException("Invalid High QC"))
                .getLedgerHeader()
                .isEndOfEpoch();
        if (!endOfEpoch) {
          whenSynced.run();
        }

        return true;
      case INVALID:
        return true;
      case IN_PROGRESS:
        return false;
      default:
        throw new IllegalStateException("Unknown syncResult " + syncResult);
    }
  }
}
