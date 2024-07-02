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

package com.radixdlt.consensus.sync;

import static java.util.function.Predicate.not;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.hash.HashCode;
import com.google.common.util.concurrent.RateLimiter;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.PacemakerReducer;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.consensus.vertexstore.VertexChain;
import com.radixdlt.consensus.vertexstore.VertexStore;
import com.radixdlt.consensus.vertexstore.VertexStoreAdapter;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.*;
import com.radixdlt.ledger.LedgerProofBundle;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.RoundChange.CertificateType;
import com.radixdlt.monitoring.Metrics.RoundChange.HighQcSource;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.sync.messages.local.LocalSyncRequest;
import java.util.*;
import java.util.stream.Stream;
import javax.annotation.Nullable;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Manages keeping the VertexStore and pacemaker in sync for consensus */
public final class BFTSync implements BFTSyncer {
  private enum SyncStage {
    PREPARING,
    GET_COMMITTED_VERTICES,
    LEDGER_SYNC,
    GET_QC_VERTICES
  }

  private static class SyncRequestState {
    private final List<HashCode> syncIds = new ArrayList<>();
    private final ImmutableList<NodeId> authors;
    private final Round round;

    SyncRequestState(ImmutableList<NodeId> authors, Round round) {
      this.authors = Objects.requireNonNull(authors);
      this.round = Objects.requireNonNull(round);
    }
  }

  private static class SyncState {
    private final HashCode localSyncId;
    private final HighQC highQC;
    private final ProcessedQcCommit processedQcCommit;
    private final NodeId author;
    private SyncStage syncStage;
    private final HighQcSource highQcSource;
    private final LinkedList<VertexWithHash> fetched = new LinkedList<>();

    SyncState(HighQC highQC, NodeId author, Hasher hasher, HighQcSource highQcSource) {
      this.localSyncId = highQC.highestQC().getProposedHeader().getVertexId();
      this.processedQcCommit =
          highQC
              .highestCommittedQC()
              .getProcessedCommit(hasher)
              .orElseThrow(() -> new IllegalStateException("committedQC must have a commit"));
      this.highQC = highQC;
      this.author = author;
      this.syncStage = SyncStage.PREPARING;
      this.highQcSource = highQcSource;
    }

    void setSyncStage(SyncStage syncStage) {
      this.syncStage = syncStage;
    }

    @Override
    public String toString() {
      return String.format(
          "%s{%s syncState=%s highQcSource=%s}",
          this.getClass().getSimpleName(), highQC, syncStage, highQcSource);
    }
  }

  private static final Comparator<Map.Entry<GetVerticesRequest, SyncRequestState>> syncPriority =
      Comparator.comparing(
              (Map.Entry<GetVerticesRequest, SyncRequestState> e) -> e.getValue().round)
          .reversed(); // Prioritise by highest round

  private static final Logger log = LogManager.getLogger();
  private final BFTValidatorId self;
  private final VertexStoreAdapter vertexStore;
  private final Hasher hasher;
  private final SafetyRules safetyRules;
  private final PacemakerReducer pacemakerReducer;
  private final Map<HashCode, SyncState> syncing = new HashMap<>();
  private final TreeMap<LedgerHeader, List<HashCode>> ledgerSyncing;
  private final Map<GetVerticesRequest, SyncRequestState> bftSyncing = new HashMap<>();
  private final RemoteEventDispatcher<NodeId, GetVerticesRequest> requestSender;
  private final EventDispatcher<LocalSyncRequest> localSyncRequestEventDispatcher;
  private final ScheduledEventDispatcher<VertexRequestTimeout> timeoutDispatcher;
  private final EventDispatcher<ConsensusByzantineEvent> unexpectedEventEventDispatcher;
  private final Random random;
  private final int bftSyncPatienceMillis;
  private final Metrics metrics;

  private LedgerProofBundle latestProof;

  // FIXME: Remove this once sync is fixed
  private final RateLimiter syncRequestRateLimiter;
  private final RateLimiter loggingRateLimiter = RateLimiter.create(1.0); // 1 per second

  public BFTSync(
      BFTValidatorId self,
      RateLimiter syncRequestRateLimiter,
      VertexStoreAdapter vertexStore,
      Hasher hasher,
      SafetyRules safetyRules,
      PacemakerReducer pacemakerReducer,
      Comparator<LedgerHeader> ledgerHeaderComparator,
      RemoteEventDispatcher<NodeId, GetVerticesRequest> requestSender,
      EventDispatcher<LocalSyncRequest> localSyncRequestEventDispatcher,
      ScheduledEventDispatcher<VertexRequestTimeout> timeoutDispatcher,
      EventDispatcher<ConsensusByzantineEvent> unexpectedEventEventDispatcher,
      LedgerProofBundle initialLatestProof,
      Random random,
      int bftSyncPatienceMillis,
      Metrics metrics) {
    this.self = self;
    this.syncRequestRateLimiter = Objects.requireNonNull(syncRequestRateLimiter);
    this.vertexStore = vertexStore;
    this.hasher = Objects.requireNonNull(hasher);
    this.safetyRules = Objects.requireNonNull(safetyRules);
    this.pacemakerReducer = pacemakerReducer;
    this.ledgerSyncing = new TreeMap<>(ledgerHeaderComparator);
    this.requestSender = requestSender;
    this.localSyncRequestEventDispatcher = Objects.requireNonNull(localSyncRequestEventDispatcher);
    this.timeoutDispatcher = Objects.requireNonNull(timeoutDispatcher);
    this.unexpectedEventEventDispatcher = Objects.requireNonNull(unexpectedEventEventDispatcher);
    this.latestProof = Objects.requireNonNull(initialLatestProof);
    this.random = random;
    this.bftSyncPatienceMillis = bftSyncPatienceMillis;
    this.metrics = Objects.requireNonNull(metrics);
  }

  public EventProcessor<RoundQuorumResolution> roundQuorumResolutionEventProcessor() {
    return roundQuorumResolution -> {
      final var highQC =
          switch (roundQuorumResolution.roundQuorum()) {
            case RoundQuorum.RegularRoundQuorum regularRoundQuorum -> this.vertexStore
                .highQC()
                .withHighestQC(regularRoundQuorum.qc());
            case RoundQuorum.TimeoutRoundQuorum timeoutRoundQuorum -> this.vertexStore
                .highQC()
                .withHighestTC(timeoutRoundQuorum.tc());
          };

      final var nodeId =
          NodeId.fromPublicKey(roundQuorumResolution.lastVote().getAuthor().getKey());

      final var highQcSource =
          roundQuorumResolution.lastVote().isTimeout()
              ? HighQcSource.CREATED_ON_RECEIVED_TIMEOUT_VOTE
              : HighQcSource.CREATED_ON_RECEIVED_NON_TIMEOUT_VOTE;

      syncToQC(highQC, nodeId, highQcSource);
    };
  }

  @Override
  public SyncResult syncToQC(HighQC highQC, @Nullable NodeId author, HighQcSource highQcSource) {
    final var qc = highQC.highestQC();

    if (qc.getProposedHeader().getRound().lt(vertexStore.getRoot().vertex().getRound())) {
      return SyncResult.INVALID;
    }

    if (qc.getProposedHeader().getRound().lt(this.latestProof.resultantRound())) {
      return SyncResult.INVALID;
    }

    return switch (vertexStore.insertQuorumCertificate(qc)) {
      case VertexStore.InsertQcResult.Inserted ignored -> {
        // QC was inserted, try TC too (as it can be higher), and then process a new highQC
        highQC.highestTC().map(vertexStore::insertTimeoutCertificate);
        this.pacemakerReducer.processQC(
            vertexStore.highQC(), highQcSource, determineCertificateType(qc, highQC.highestTC()));
        yield SyncResult.SYNCED;
      }
      case VertexStore.InsertQcResult.Ignored ignored -> {
        // QC was ignored, try inserting TC and if that works process a new highQC
        final var insertedTc =
            highQC.highestTC().map(vertexStore::insertTimeoutCertificate).orElse(false);
        if (insertedTc) {
          this.pacemakerReducer.processQC(vertexStore.highQC(), highQcSource, CertificateType.TC);
        }
        yield SyncResult.SYNCED;
      }
      case VertexStore.InsertQcResult.VertexIsMissing ignored -> {
        // QC is missing some vertices, sync up
        // TC (if present) is put aside for now (and reconsidered later, once QC is synced)
        log.trace("SYNC_TO_QC: Need sync: {}", highQC);

        // TODO: Check for other conflicting QCs which a bad validator set may have created
        // Bad genesis QC, ignore...
        if (qc.getRound().isEpochInitial()) {
          this.unexpectedEventEventDispatcher.dispatch(
              new ConsensusByzantineEvent.ConflictingGenesis(qc, author));
          yield SyncResult.INVALID;
        }

        if (syncing.containsKey(qc.getProposedHeader().getVertexId())) {
          yield SyncResult.IN_PROGRESS;
        }

        if (author == null) {
          throw new IllegalStateException("Syncing required but author wasn't provided.");
        }

        startSync(highQC, author, highQcSource);

        yield SyncResult.IN_PROGRESS;
      }
    };
  }

  @SuppressWarnings("OptionalUsedAsFieldOrParameterType")
  private CertificateType determineCertificateType(
      QuorumCertificate highestQc, Optional<TimeoutCertificate> highestTc) {
    if (highestTc.stream().anyMatch(tc -> tc.getRound().gt(highestQc.getRound()))) {
      return CertificateType.TC;
    } else {
      final var isFallbackVertex =
          vertexStore
              .getExecutedVertex(highestQc.getProposedHeader().getVertexId())
              .map(v -> v.vertex().isFallback())
              .orElse(false);
      return isFallbackVertex
          ? CertificateType.QC_ON_FALLBACK_VERTEX
          : CertificateType.QC_ON_REGULAR_VERTEX;
    }
  }

  private boolean requiresLedgerSync(SyncState syncState) {
    return !vertexStore.hasCommittedVertexOrRootAtOrAboveRound(
        syncState.processedQcCommit.committedHeader());
  }

  private void startSync(HighQC highQC, NodeId author, HighQcSource highQcSource) {
    final var syncState = new SyncState(highQC, author, hasher, highQcSource);

    syncing.put(syncState.localSyncId, syncState);

    if (requiresLedgerSync(syncState)) {
      this.doCommittedSync(syncState);
    } else {
      this.doQCSync(syncState);
    }
  }

  private void doQCSync(SyncState syncState) {
    syncState.setSyncStage(SyncStage.GET_QC_VERTICES);
    log.debug("SYNC_VERTICES: QC: Sending initial GetVerticesRequest for sync={}", syncState);

    final var authors =
        Stream.concat(
                Stream.of(syncState.author),
                syncState
                    .highQC
                    .highestQC()
                    .getSigners()
                    .map(v -> NodeId.fromPublicKey(v.getKey()))
                    .filter(n -> !n.equals(syncState.author)))
            .filter(not(n -> n.equals(NodeId.fromPublicKey(this.self.getKey()))))
            .collect(ImmutableList.toImmutableList());

    final var qc = syncState.highQC.highestQC();
    this.sendBFTSyncRequest(
        "QCSync",
        qc.getRound(),
        qc.getProposedHeader().getVertexId(),
        1,
        authors,
        syncState.localSyncId);
  }

  private void doCommittedSync(SyncState syncState) {
    final var committedQCId =
        syncState.highQC.highestCommittedQC().getProposedHeader().getVertexId();
    final var committedRound = syncState.highQC.highestCommittedQC().getRound();

    syncState.setSyncStage(SyncStage.GET_COMMITTED_VERTICES);
    log.debug(
        "SYNC_VERTICES: Committed: Sending initial GetVerticesRequest for sync={}", syncState);
    // Retrieve the 3 vertices preceding the committedQC so we can create a valid committed root

    final var authors =
        Stream.concat(
                Stream.of(syncState.author),
                syncState
                    .highQC
                    .highestCommittedQC()
                    .getSigners()
                    .map(v -> NodeId.fromPublicKey(v.getKey()))
                    .filter(n -> !n.equals(syncState.author)))
            .filter(not(n -> n.equals(NodeId.fromPublicKey(this.self.getKey()))))
            .collect(ImmutableList.toImmutableList());

    this.sendBFTSyncRequest(
        "CommittedSync", committedRound, committedQCId, 3, authors, syncState.localSyncId);
  }

  public EventProcessor<VertexRequestTimeout> vertexRequestTimeoutEventProcessor() {
    return this::processGetVerticesLocalTimeout;
  }

  private void processGetVerticesLocalTimeout(VertexRequestTimeout timeout) {
    final var request = highestQCRequest(this.bftSyncing.entrySet());
    var syncRequestState = bftSyncing.remove(request);

    if (syncRequestState == null) {
      return;
    }

    if (syncRequestState.authors.isEmpty()) {
      throw new IllegalStateException("Request contains no authors except ourselves");
    }

    var syncIds =
        syncRequestState.syncIds.stream().filter(syncing::containsKey).distinct().toList();

    for (var syncId : syncIds) {
      metrics.bft().sync().requestTimeouts().inc();
      var syncState = syncing.remove(syncId);
      syncToQC(syncState.highQC, randomFrom(syncRequestState.authors), syncState.highQcSource);
    }
  }

  private GetVerticesRequest highestQCRequest(
      Collection<Map.Entry<GetVerticesRequest, SyncRequestState>> requests) {
    return requests.stream().sorted(syncPriority).findFirst().map(Map.Entry::getKey).orElse(null);
  }

  private <T> T randomFrom(List<T> elements) {
    final var size = elements.size();

    if (size <= 0) {
      return null;
    }

    return elements.get(random.nextInt(size));
  }

  private void sendBFTSyncRequest(
      String reason,
      Round round,
      HashCode vertexId,
      int count,
      ImmutableList<NodeId> authors,
      HashCode syncId) {
    var request = new GetVerticesRequest(vertexId, count);
    var syncRequestState = bftSyncing.getOrDefault(request, new SyncRequestState(authors, round));

    if (syncRequestState.syncIds.isEmpty()) {
      var author = authors.get(0);

      if (this.syncRequestRateLimiter.tryAcquire()) {
        this.timeoutDispatcher.dispatch(new VertexRequestTimeout(request), bftSyncPatienceMillis);
        this.requestSender.dispatch(author, request);
      } else {
        // Report issue. Once per second as info-level message, rest as debug
        if (loggingRateLimiter.tryAcquire() && log.isInfoEnabled()) {
          log.info(outboundRateLimitLogMessage(reason, round, author, request));
        } else if (log.isDebugEnabled()) {
          log.debug(outboundRateLimitLogMessage(reason, round, author, request));
        }
      }
      this.bftSyncing.put(request, syncRequestState);
    }
    syncRequestState.syncIds.add(syncId);
  }

  private String outboundRateLimitLogMessage(
      String reason, Round round, NodeId author, GetVerticesRequest request) {
    return String.format(
        """
			RATE_LIMIT: Outbound BFT Sync request %s for round %s due to %s to %s was not sent\
			 because we're over our %s/second rate limit on sync requests.
			This can happen if a node has a temporarily flaky internet connection.""",
        request, round, reason, author, this.syncRequestRateLimiter.getRate());
  }

  private void rebuildAndSyncQC(SyncState syncState) {
    log.debug(
        "SYNC_STATE: Rebuilding and syncing QC: sync={} curRoot={}",
        syncState,
        vertexStore.getRoot());

    // TODO: check if there are any vertices which haven't been local sync processed yet
    if (requiresLedgerSync(syncState)) {
      syncState.fetched.sort(Comparator.comparing(v -> v.vertex().getRound()));
      ImmutableSet<VertexWithHash> nonRootVertices =
          syncState.fetched.stream().skip(1).collect(ImmutableSet.toImmutableSet());

      final var syncStateHighestCommittedQc = syncState.highQC.highestCommittedQC();
      final var syncStateHighestTc = syncState.highQC.highestTC();
      final var currentHighestTc = vertexStore.highQC().highestTC();
      final var highestKnownTc =
          Stream.of(currentHighestTc, syncStateHighestTc)
              .flatMap(Optional::stream)
              .max(Comparator.comparing(TimeoutCertificate::getRound));

      var vertexStoreState =
          VertexStoreState.create(
              HighQC.from(syncStateHighestCommittedQc, syncStateHighestCommittedQc, highestKnownTc),
              syncState.fetched.get(0),
              nonRootVertices,
              hasher);
      if (vertexStore.tryRebuild(vertexStoreState)) {
        // TODO: Move pacemaker outside of sync
        pacemakerReducer.processQC(
            vertexStoreState.getHighQC(),
            syncState.highQcSource,
            determineCertificateType(syncStateHighestCommittedQc, highestKnownTc));
      }
    } else {
      log.debug("SYNC_STATE: skipping rebuild");
    }

    // At this point we are guaranteed to be in sync with the committed state
    // Retry sync
    this.syncing.remove(syncState.localSyncId);
    this.syncToQC(syncState.highQC, syncState.author, syncState.highQcSource);
  }

  private void processVerticesResponseForCommittedSync(
      SyncState syncState, NodeId sender, GetVerticesResponse response) {
    log.debug(
        "SYNC_STATE: Processing vertices {} Round {} From {} LatestProof {}",
        syncState,
        response.vertices().get(0).vertex().getRound(),
        sender,
        this.latestProof);

    syncState.fetched.addAll(response.vertices());

    final var commitHeader = syncState.processedQcCommit.committedHeader().getLedgerHeader();
    // TODO: verify actually extends rather than just state version comparison
    if (commitHeader.getStateVersion() <= this.latestProof.resultantStateVersion()) {
      rebuildAndSyncQC(syncState);
    } else {
      syncState.setSyncStage(SyncStage.LEDGER_SYNC);
      ledgerSyncing.compute(
          commitHeader,
          (header, existingList) -> {
            var list = (existingList == null) ? new ArrayList<HashCode>() : existingList;
            list.add(syncState.localSyncId);
            return list;
          });
      switch (syncState.processedQcCommit) {
        case ProcessedQcCommit.OfConensusQc ofConensusQc -> {
          final var signersWithoutSelf =
              ofConensusQc.origin().signatures().stream()
                  .filter(
                      not(s -> BFTValidatorId.create(s.validatorAddress(), s.key()).equals(self)))
                  .collect(ImmutableList.toImmutableList());
          final var nodeIds =
              signersWithoutSelf.stream()
                  .map(n -> NodeId.fromPublicKey(n.key()))
                  .collect(ImmutableList.toImmutableList());
          localSyncRequestEventDispatcher.dispatch(
              new LocalSyncRequest(ofConensusQc.ledgerProof(), nodeIds));
        }
        case ProcessedQcCommit.OfInitialEpochQc ignored -> {
          // BFTSync somehow decided it needs to ledger-sync
          // to a header that should already be committed (epoch initial).
          // This should never happen, but if by any chance it does,
          // this is not terribly wrong. We can just ignore this request
          // and wait for a timeout or another event.
          this.metrics.bft().sync().invalidEpochInitialQcSyncStates().inc();
        }
      }
    }
  }

  private void processVerticesResponseForQCSync(SyncState syncState, GetVerticesResponse response) {
    final var vertexWithHash = response.vertices().get(0);
    final var vertex = vertexWithHash.vertex();
    syncState.fetched.addFirst(vertexWithHash);

    var parentId = vertex.getParentVertexId();

    if (vertexStore.containsVertex(parentId)) {
      vertexStore.insertVertexChain(VertexChain.create(syncState.fetched));
      // Finish it off
      this.syncing.remove(syncState.localSyncId);
      this.syncToQC(syncState.highQC, syncState.author, syncState.highQcSource);
    } else {
      log.debug(
          "SYNC_VERTICES: Sending further GetVerticesRequest for {} fetched={} root={}",
          syncState.highQC,
          syncState.fetched.size(),
          vertexStore.getRoot());

      final var authors =
          Stream.concat(
                  Stream.of(syncState.author),
                  vertex
                      .getQCToParent()
                      .getSigners()
                      .map(v -> NodeId.fromPublicKey(v.getKey()))
                      .filter(n -> !n.equals(syncState.author)))
              .filter(not(n -> n.equals(NodeId.fromPublicKey(this.self.getKey()))))
              .collect(ImmutableList.toImmutableList());

      this.sendBFTSyncRequest(
          "VertexResponseMissingParent",
          syncState.highQC.highestQC().getRound(),
          parentId,
          1,
          authors,
          syncState.localSyncId);
    }
  }

  private void processGetVerticesErrorResponse(NodeId sender, GetVerticesErrorResponse response) {
    if (!safetyRules.verifyHighQcAgainstTheValidatorSet(response.highQC())) {
      // If the response is invalid we just ignore it and wait for the timeout event
      log.warn("Received an invalid BFT sync error response. Ignoring.");
      return;
    }

    // TODO: check response
    final var request = response.request();
    final var syncRequestState = bftSyncing.get(request);
    if (syncRequestState != null) {
      log.debug(
          "SYNC_VERTICES: Received GetVerticesErrorResponse: {} highQC: {}",
          response,
          vertexStore.highQC());
      if (response
              .highQC()
              .highestQC()
              .getRound()
              .compareTo(vertexStore.highQC().highestQC().getRound())
          > 0) {
        // error response indicates that the node has moved on from last sync so try and sync to a
        // new qc
        syncToQC(
            response.highQC(), sender, HighQcSource.RECEIVED_IN_BFT_SYNC_VERTICES_ERROR_RESPONSE);
      }
    }
  }

  public RemoteEventProcessor<NodeId, GetVerticesErrorResponse> errorResponseProcessor() {
    return this::processGetVerticesErrorResponse;
  }

  public RemoteEventProcessor<NodeId, GetVerticesResponse> responseProcessor() {
    return this::processGetVerticesResponse;
  }

  private void processGetVerticesResponse(NodeId sender, GetVerticesResponse response) {
    final var allVerticesHaveValidQc =
        response.vertices().stream()
            .allMatch(v -> safetyRules.verifyQcAgainstTheValidatorSet(v.vertex().getQCToParent()));

    if (!allVerticesHaveValidQc) {
      // If the response is invalid we just ignore it and wait for the timeout event
      log.warn("Received an invalid BFT sync response. Ignoring.");
      return;
    }

    log.debug("SYNC_VERTICES: Received GetVerticesResponse {}", response);

    var firstVertex = response.vertices().get(0);
    var requestInfo = new GetVerticesRequest(firstVertex.hash(), response.vertices().size());
    var syncRequestState = bftSyncing.remove(requestInfo);

    if (syncRequestState != null) {
      for (var syncTo : syncRequestState.syncIds) {
        var syncState = syncing.get(syncTo);
        if (syncState == null) {
          continue; // sync requirements already satisfied by another sync
        }

        switch (syncState.syncStage) {
          case GET_COMMITTED_VERTICES -> processVerticesResponseForCommittedSync(
              syncState, sender, response);
          case GET_QC_VERTICES -> processVerticesResponseForQCSync(syncState, response);
          default -> throw new IllegalStateException("Unknown sync stage: " + syncState.syncStage);
        }
      }
    }
  }

  public EventProcessor<LedgerUpdate> baseLedgerUpdateEventProcessor() {
    return this::processLedgerUpdate;
  }

  // TODO: Verify headers match
  private void processLedgerUpdate(LedgerUpdate ledgerUpdate) {
    this.latestProof = ledgerUpdate.committedProof();
    final var header = REv2ToConsensus.ledgerHeader(latestProof.primaryProof().ledgerHeader());
    var listeners = this.ledgerSyncing.headMap(header, true).values();
    var listenersIterator = listeners.iterator();

    while (listenersIterator.hasNext()) {
      var syncs = listenersIterator.next();
      for (var syncTo : syncs) {

        var syncState = syncing.get(syncTo);
        if (syncState != null) {
          rebuildAndSyncQC(syncState);
        }
      }
      listenersIterator.remove();
    }

    syncing
        .entrySet()
        .removeIf(e -> e.getValue().highQC.highestQC().getRound().lte(header.getRound()));
  }
}
