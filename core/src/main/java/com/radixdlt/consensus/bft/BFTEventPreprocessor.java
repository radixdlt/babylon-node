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

package com.radixdlt.consensus.bft;

import com.google.common.hash.HashCode;
import com.radixdlt.consensus.BFTEventProcessor;
import com.radixdlt.consensus.ConsensusEvent;
import com.radixdlt.consensus.HighQC;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTSyncer.SyncResult;
import com.radixdlt.consensus.liveness.ScheduledLocalTimeout;
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
 * Preprocesses consensus events and ensures that the vertexStore is synced to the correct state
 * before they get forwarded to the actual state reducer.
 *
 * <p>This class should not be updating any part of the BFT Safety state besides the VertexStore.
 *
 * <p>A lot of the queue logic could be done more "cleanly" and functionally using lambdas and
 * Functions but the performance impact is too great.
 *
 * <p>This class is NOT thread-safe.
 */
public final class BFTEventPreprocessor implements BFTEventProcessor {
  private static final Logger log = LogManager.getLogger();

  private final BFTEventProcessor forwardTo;
  private final BFTSyncer bftSyncer;
  private final Set<ConsensusEvent> syncingEvents = new HashSet<>();
  private final Map<Round, List<ConsensusEvent>> viewQueues = new HashMap<>();
  private RoundUpdate latestRoundUpdate;

  public BFTEventPreprocessor(
      BFTEventProcessor forwardTo, BFTSyncer bftSyncer, RoundUpdate initialRoundUpdate) {
    this.bftSyncer = Objects.requireNonNull(bftSyncer);
    this.forwardTo = forwardTo;
    this.latestRoundUpdate = Objects.requireNonNull(initialRoundUpdate);
  }

  @Override
  public void processViewUpdate(RoundUpdate roundUpdate) {
    final Round previousRound = this.latestRoundUpdate.getCurrentRound();
    log.trace("Processing viewUpdate {} cur {}", roundUpdate, previousRound);

    // FIXME: Check is required for now since Deterministic tests can randomize local messages
    if (roundUpdate.getCurrentRound().gt(previousRound)) {
      this.latestRoundUpdate = roundUpdate;
      forwardTo.processViewUpdate(roundUpdate);
      viewQueues
          .getOrDefault(roundUpdate.getCurrentRound(), new LinkedList<>())
          .forEach(this::processViewCachedEvent);
      viewQueues.keySet().removeIf(v -> v.lte(roundUpdate.getCurrentRound()));

      syncingEvents.stream()
          .filter(e -> e.getRound().equals(roundUpdate.getCurrentRound()))
          .forEach(this::processQueuedConsensusEvent);

      syncingEvents.removeIf(e -> e.getRound().lte(roundUpdate.getCurrentRound()));
    }
  }

  private void processViewCachedEvent(ConsensusEvent event) {
    if (event instanceof Proposal) {
      log.trace("Processing cached proposal {}", event);
      processProposal((Proposal) event);
    } else if (event instanceof Vote) {
      log.trace("Processing cached vote {}", event);
      processVote((Vote) event);
    } else {
      log.error("Ignoring cached ConsensusEvent {}", event);
    }
  }

  @Override
  public void processBFTUpdate(BFTInsertUpdate update) {
    HashCode vertexId = update.getInserted().getVertexHash();
    log.trace("LOCAL_SYNC: {}", vertexId);

    syncingEvents.stream()
        .filter(e -> e.highQC().highestQC().getProposed().getVertexId().equals(vertexId))
        .forEach(this::processQueuedConsensusEvent);

    syncingEvents.removeIf(
        e -> e.highQC().highestQC().getProposed().getVertexId().equals(vertexId));

    forwardTo.processBFTUpdate(update);
  }

  @Override
  public void processBFTRebuildUpdate(BFTRebuildUpdate rebuildUpdate) {
    rebuildUpdate
        .getVertexStoreState()
        .getVertices()
        .forEach(
            v -> {
              HashCode vertexId = v.getHash();
              syncingEvents.stream()
                  .filter(e -> e.highQC().highestQC().getProposed().getVertexId().equals(vertexId))
                  .forEach(this::processQueuedConsensusEvent);

              syncingEvents.removeIf(
                  e -> e.highQC().highestQC().getProposed().getVertexId().equals(vertexId));
            });
  }

  @Override
  public void processVote(Vote vote) {
    log.trace("Vote: PreProcessing {}", vote);
    if (!processVoteInternal(vote)) {
      log.debug("Vote: Queuing {}, waiting for Sync", vote);
      syncingEvents.add(vote);
    }
  }

  @Override
  public void processProposal(Proposal proposal) {
    log.trace("Proposal: PreProcessing {}", proposal);
    if (!processProposalInternal(proposal)) {
      log.debug("Proposal: Queuing {}, waiting for Sync", proposal);
      syncingEvents.add(proposal);
    }
  }

  @Override
  public void processLocalTimeout(ScheduledLocalTimeout scheduledLocalTimeout) {
    forwardTo.processLocalTimeout(scheduledLocalTimeout);
  }

  @Override
  public void start() {
    forwardTo.start();
  }

  // TODO: rework processQueuedConsensusEvent(), processVoteInternal() and processProposalInternal()
  // avoid code duplication and manual forwarding using instanceof
  // https://radixdlt.atlassian.net/browse/NT-6
  private boolean processQueuedConsensusEvent(ConsensusEvent event) {
    if (event == null) {
      return false;
    }

    // Explicitly using switch case method here rather than functional method
    // to process these events due to much better performance
    if (event instanceof Proposal) {
      final Proposal proposal = (Proposal) event;
      return processProposalInternal(proposal);
    }

    if (event instanceof Vote) {
      final Vote vote = (Vote) event;
      return processVoteInternal(vote);
    }

    throw new IllegalStateException("Unexpected consensus event: " + event);
  }

  private boolean processVoteInternal(Vote vote) {
    final Round currentRound = this.latestRoundUpdate.getCurrentRound();
    if (vote.getRound().gte(currentRound)) {
      log.trace("Vote: PreProcessing {}", vote);
      return syncUp(
          vote.highQC(),
          vote.getAuthor(),
          () -> processOnCurrentViewOrCache(vote, forwardTo::processVote));
    } else {
      log.trace("Vote: Ignoring for past round {}, current round is {}", vote, currentRound);
      return true;
    }
  }

  private boolean processProposalInternal(Proposal proposal) {
    final Round currentRound = this.latestRoundUpdate.getCurrentRound();
    if (proposal.getRound().gte(currentRound)) {
      log.trace("Proposal: PreProcessing {}", proposal);
      return syncUp(
          proposal.highQC(),
          proposal.getAuthor(),
          () -> processOnCurrentViewOrCache(proposal, forwardTo::processProposal));
    } else {
      log.trace(
          "Proposal: Ignoring for past round {}, current round is {}", proposal, currentRound);
      return true;
    }
  }

  private <T extends ConsensusEvent> void processOnCurrentViewOrCache(
      T event, Consumer<T> processFn) {
    if (latestRoundUpdate.getCurrentRound().equals(event.getRound())) {
      processFn.accept(event);
    } else if (latestRoundUpdate.getCurrentRound().lt(event.getRound())) {
      log.trace("Caching {}, current round is {}", event, latestRoundUpdate.getCurrentRound());
      viewQueues.putIfAbsent(event.getRound(), new LinkedList<>());
      viewQueues.get(event.getRound()).add(event);
    } else {
      log.debug("Ignoring {} for past round", event);
    }
  }

  private boolean syncUp(HighQC highQC, BFTNode author, Runnable whenSynced) {
    SyncResult syncResult = this.bftSyncer.syncToQC(highQC, author);

    // TODO: use switch expression and eliminate unnecessary default case
    switch (syncResult) {
      case SYNCED:
        // if already end of epoch then don't need to process
        // TODO: need to do the same checks on pacemaker side
        // TODO: move this to an epoch preprocessor
        final boolean endOfEpoch =
            highQC
                .highestCommittedQC()
                .getCommitted()
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
