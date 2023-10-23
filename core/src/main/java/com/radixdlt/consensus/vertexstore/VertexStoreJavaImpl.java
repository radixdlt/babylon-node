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

package com.radixdlt.consensus.vertexstore;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.MissingParentException;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.lang.Option;
import com.radixdlt.monitoring.Metrics;
import java.util.*;
import javax.annotation.concurrent.NotThreadSafe;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

@NotThreadSafe
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public final class VertexStoreJavaImpl implements VertexStore {
  private static final Logger logger = LogManager.getLogger();

  private final Ledger ledger;
  private final Hasher hasher;
  private final Metrics metrics;
  private final VertexStoreConfig config;

  private final Map<HashCode, VertexWithHash> vertices = new HashMap<>();
  private final Map<HashCode, Set<HashCode>> vertexChildren = new HashMap<>();
  private final Map<HashCode, ExecutedVertex> executedVertices = new HashMap<>();

  // These should never be null
  private VertexWithHash rootVertex;
  private HighQC highQC;

  private VertexStoreJavaImpl(
      Ledger ledger,
      Hasher hasher,
      Metrics metrics,
      VertexStoreConfig config,
      VertexWithHash rootVertex,
      HighQC highQC) {
    this.ledger = Objects.requireNonNull(ledger);
    this.hasher = Objects.requireNonNull(hasher);
    this.metrics = Objects.requireNonNull(metrics);
    this.config = Objects.requireNonNull(config);
    this.rootVertex = Objects.requireNonNull(rootVertex);
    this.highQC = Objects.requireNonNull(highQC);
    this.vertexChildren.put(rootVertex.hash(), new HashSet<>());
  }

  public static VertexStoreJavaImpl create(
      VertexStoreState vertexStoreState,
      Ledger ledger,
      Hasher hasher,
      Metrics metrics,
      VertexStoreConfig config) {
    final var vertexStore =
        new VertexStoreJavaImpl(
            ledger,
            hasher,
            metrics,
            config,
            vertexStoreState.getRoot(),
            vertexStoreState.getHighQC());

    for (var vertexWithHash : vertexStoreState.getVertices()) {
      vertexStore.vertices.put(vertexWithHash.hash(), vertexWithHash);
      vertexStore.vertexChildren.put(vertexWithHash.hash(), new HashSet<>());
      var siblings = vertexStore.vertexChildren.get(vertexWithHash.vertex().getParentVertexId());
      siblings.add(vertexWithHash.hash());
    }

    return vertexStore;
  }

  @Override
  public VertexWithHash getRoot() {
    return rootVertex;
  }

  @Override
  public Option<VertexStoreState> tryRebuild(VertexStoreState vertexStoreState) {
    // FIXME: Currently this assumes vertexStoreState is a chain with no forks which is our only use
    // case at the moment.
    var executedVertices = new LinkedList<ExecutedVertex>();
    for (VertexWithHash vertex : vertexStoreState.getVertices()) {
      var executedVertexMaybe = ledger.prepare(executedVertices, vertex);

      // If any vertex couldn't be executed successfully, our saved state is invalid
      if (executedVertexMaybe.isEmpty()) {
        return Option.empty();
      }

      executedVertices.add(executedVertexMaybe.get());
    }

    this.rootVertex = vertexStoreState.getRoot();
    this.highQC = vertexStoreState.getHighQC();
    this.vertices.clear();
    this.executedVertices.clear();
    this.vertexChildren.clear();
    this.vertexChildren.put(rootVertex.hash(), new HashSet<>());

    // Note that the vertices aren't re-executed at boot. See comment at `getExecutedVertex`
    for (var executedVertex : executedVertices) {
      this.vertices.put(executedVertex.getVertexHash(), executedVertex.getVertexWithHash());
      this.executedVertices.put(executedVertex.getVertexHash(), executedVertex);
      this.vertexChildren.put(executedVertex.getVertexHash(), new HashSet<>());
      Set<HashCode> siblings = vertexChildren.get(executedVertex.getParentId());
      siblings.add(executedVertex.getVertexHash());
    }

    return Option.present(vertexStoreState);
  }

  @Override
  public boolean containsVertex(HashCode vertexId) {
    return vertices.containsKey(vertexId) || rootVertex.hash().equals(vertexId);
  }

  @Override
  public InsertQcResult insertQc(QuorumCertificate qc) {
    if (!this.containsVertex(qc.getProposedHeader().getVertexId())) {
      return new VertexStore.InsertQcResult.VertexIsMissing();
    }

    final var hasAnyChildren = !vertexChildren.get(qc.getProposedHeader().getVertexId()).isEmpty();
    if (hasAnyChildren) {
      // TODO: Check to see if qc's match in case there's a fault
      return new VertexStore.InsertQcResult.Ignored();
    }

    // proposed vertex doesn't have any children
    boolean isHighQC = qc.getRound().gt(highQC.highestQC().getRound());
    if (isHighQC) {
      this.highQC = this.highQC.withHighestQC(qc);
    }

    final Option<CommittedUpdate> committedUpdate;
    if (qc.getCommittedHeader().isPresent()) {
      final var committedHeader = qc.getCommittedHeader().orElseThrow();
      if (committedHeader.getRound().gt(this.rootVertex.vertex().getRound())) {
        // QC has a valid committed header
        committedUpdate = Option.some(this.commit(committedHeader, qc));
      } else {
        // QC has committed header for an older round, ignore
        committedUpdate = Option.empty();
      }
    } else {
      // No committed header in QC
      committedUpdate = Option.empty();
    }

    if (isHighQC || committedUpdate.isPresent()) {
      // We have either received a new highQc, or some vertices
      // were committed, or both.
      return new VertexStore.InsertQcResult.Inserted(highQC(), getState(), committedUpdate);
    } else {
      // This wasn't our new high QC and nothing has been committed
      return new VertexStore.InsertQcResult.Ignored();
    }
  }

  private void getChildrenVerticesList(
      VertexWithHash parent, ImmutableList.Builder<VertexWithHash> builder) {
    Set<HashCode> childrenIds = this.vertexChildren.get(parent.hash());
    if (childrenIds == null) {
      return;
    }

    for (HashCode childId : childrenIds) {
      final var v = vertices.get(childId);
      builder.add(v);
      getChildrenVerticesList(v, builder);
    }
  }

  private VertexStoreState getState() {
    // TODO: store list dynamically rather than recomputing
    ImmutableList.Builder<VertexWithHash> verticesBuilder = ImmutableList.builder();
    getChildrenVerticesList(this.rootVertex, verticesBuilder);
    return VertexStoreState.create(this.highQC(), this.rootVertex, verticesBuilder.build(), hasher);
  }

  @Override
  public boolean insertTimeoutCertificate(TimeoutCertificate timeoutCertificate) {
    if (timeoutCertificate.getRound().gt(highQC().getHighestRound())) {
      this.highQC = this.highQC.withHighestTC(timeoutCertificate);
      return true;
    }
    return false;
  }

  // TODO: reimplement in async way
  /**
   * Returns an existing ExecutedVertex or executes a vertex that hasn't yet been executed and
   * returns it. This lazy-execution model was introduced to speed up node recovery after it has
   * been restarted while storing a significant number of vertices - for example timeout vertices,
   * if network liveness was lost before the restart. In this scenario, without the lazy-execution
   * mechanism, the node would have to re-execute all those vertices at boot. This could take a
   * significant amount of time and might be pretty wasteful (f.e. if the vertices themselves are
   * empty but their root "path" contains some heavy transactions). Note that the vertices inserted
   * after boot are always executed immediately (including the vertices they depend on),
   * lazy-loading only applies to the initial vertices.
   */
  @Override
  public Option<ExecutedVertex> getExecutedVertex(HashCode vertexHash) {
    final var existingExecutedVertex = this.executedVertices.get(vertexHash);
    if (existingExecutedVertex != null) {
      return Option.some(existingExecutedVertex);
    } else {
      final var vertex = this.vertices.get(vertexHash);
      if (vertex != null) {
        final var previous =
            vertex.hash().equals(vertex.vertex().getParentVertexId())
                ? new LinkedList<ExecutedVertex>()
                : getPathFromRoot(vertex.vertex().getParentVertexId());
        final var executedVertexMaybe = ledger.prepare(previous, vertex);
        if (executedVertexMaybe.isEmpty()) {
          logger.warn("VertexStore contains a vertex {} but it couldn't be executed", vertex);
          return Option.none();
        } else {
          this.executedVertices.put(vertexHash, executedVertexMaybe.get());
          return Option.some(executedVertexMaybe.get());
        }
      } else {
        return Option.empty();
      }
    }
  }

  @Override
  public InsertVertexChainResult insertVertexChain(VertexChain vertexChain) {
    final var bftInsertUpdates = new ArrayList<BFTInsertUpdate>();
    final var insertedQcs = new ArrayList<InsertQcResult.Inserted>();
    for (VertexWithHash v : vertexChain.getVertices()) {
      final var insertQcResult = insertQc(v.vertex().getQCToParent());

      switch (insertQcResult) {
        case InsertQcResult.VertexIsMissing missing -> {
          return new InsertVertexChainResult(insertedQcs, bftInsertUpdates);
        }
        case InsertQcResult.Inserted inserted -> insertedQcs.add(inserted);
        case InsertQcResult.Ignored ignored -> {
          // no-op, but continue processing remaining vertices
        }
      }

      insertVertex(v).onPresent(bftInsertUpdates::add);
    }

    return new InsertVertexChainResult(insertedQcs, bftInsertUpdates);
  }

  @Override
  public Option<BFTInsertUpdate> insertVertex(VertexWithHash vertexWithHash) {
    if (vertices.containsKey(vertexWithHash.hash())) {
      return Option.empty();
    }

    final var vertex = vertexWithHash.vertex();
    if (!this.containsVertex(vertex.getParentVertexId())) {
      throw new MissingParentException(vertex.getParentVertexId());
    }

    final var previous = getPathFromRoot(vertexWithHash.vertex().getParentVertexId());
    final var executedVertexOption = Option.from(ledger.prepare(previous, vertexWithHash));
    return executedVertexOption.flatMap(
        executedVertex -> {
          vertices.put(executedVertex.getVertexHash(), executedVertex.getVertexWithHash());
          executedVertices.put(executedVertex.getVertexHash(), executedVertex);
          vertexChildren.put(executedVertex.getVertexHash(), new HashSet<>());
          final var siblings = vertexChildren.get(executedVertex.getParentId());
          siblings.add(executedVertex.getVertexHash());

          if (vertices.size() <= config.maxNumVertices()) {
            // All good, we're under (or at) the limit
            return Option.some(
                BFTInsertUpdate.insertedVertex(executedVertex, siblings.size(), getState()));
          } else {
            /*
            We're above the capacity and some vertices must be removed!
            Warning: this technically breaks the protocol and can lead to inconsistencies.
            E.g. it may lead to a situation where a QC has been formed but no one (not even its signers!)
            have the corresponding vertex, because it has been removed before the QC was formed.
            While this isn't 100% correct from protocol's point of view, the node can handle
            this situation (specifically BFTSync can).
            This has been introduced to prevent vertex store growing so large that it couldn't be serialized,
            which would result in a hard-to-recover liveness break.
            This could happen e.g. during a prolonged liveness break (e.g. due to execution divergence),
            where there are many TC-based round changes and many vertices are added
            to the vertex store, but none can get a quorum.
            We believe this is a safer alternative to an uncontrolled vertex store growth.
            */
            final var droppedVertex = dropLeastImportantVertex();
            if (droppedVertex.hash().equals(vertexWithHash.hash())) {
              // The vertex we've just tried to insert was considered the least important,
              // and effectively wasn't inserted.
              this.metrics.bft().vertexStore().verticesNotInsertedDueToSizeLimit().inc();
              return Option.empty();
            } else {
              // Some other vertex was removed
              this.metrics.bft().vertexStore().verticesDroppedDueToSizeLimit().inc();
              return Option.some(
                  BFTInsertUpdate.insertedVertex(executedVertex, siblings.size(), getState()));
            }
          }
        });
  }

  private void removeVertexAndPruneInternal(HashCode vertexId, HashCode skip) {
    final var removedVertexOrNull = vertices.remove(vertexId);
    executedVertices.remove(vertexId);

    if (this.rootVertex.hash().equals(vertexId)) {
      return;
    }

    var children = vertexChildren.remove(vertexId);
    if (children != null) {
      for (HashCode child : children) {
        if (!child.equals(skip)) {
          removeVertexAndPruneInternal(child, null);
        }
      }
    }

    // Remove this vertex from its parent's children list
    if (removedVertexOrNull != null) {
      final var parentVertexId = removedVertexOrNull.vertex().getParentVertexId();
      if (this.vertexChildren.containsKey(parentVertexId)) {
        final var parentVertexChildren = this.vertexChildren.get(parentVertexId);
        parentVertexChildren.remove(vertexId);
      }
    }
  }

  /**
   * Commit a vertex. Executes the transactions and prunes the tree.
   *
   * @param header the header to be committed
   * @param commitQC the proof of commit
   */
  private CommittedUpdate commit(BFTHeader header, QuorumCertificate commitQC) {
    final HashCode vertexId = header.getVertexId();
    final VertexWithHash tipVertex = vertices.get(vertexId);

    /* removeVertexAndPruneInternal skips children removal for the rootVertex, so we need to
    keep a reference to the previous root and prune it *after* new rootVertex is set.
    This isn't particularly easy to reason about and should be refactored at some point
    (i.e. the logic should be moved out of removeVertexAndPruneInternal). */
    final var prevRootVertex = this.rootVertex;
    this.rootVertex = tipVertex;
    this.highQC = this.highQC.withHighestCommittedQC(commitQC);
    final var path = ImmutableList.copyOf(getPathFromRoot(tipVertex.hash()));
    HashCode prev = null;
    for (int i = path.size() - 1; i >= 0; i--) {
      this.removeVertexAndPruneInternal(path.get(i).getVertexHash(), prev);
      prev = path.get(i).getVertexHash();
    }
    removeVertexAndPruneInternal(prevRootVertex.hash(), null);

    return new CommittedUpdate(path);
  }

  /** Returns a path of vertices up to the root vertex (excluding the root itself) */
  @Override
  public LinkedList<ExecutedVertex> getPathFromRoot(HashCode vertexId) {
    final LinkedList<ExecutedVertex> path = new LinkedList<>();

    /* TODO: consider throwing an exception if some vertices on path couldn't be executed
       There might be a corner case when ledger is ahead of vertex store and not-throwing
       actually allows to still be able to get a path (the issue was more likely when vertex execution happened
       at init - with async execution bft/ledger sync has a chance to catch up).
    */
    var executedVertexOpt = getExecutedVertex(vertexId);
    while (executedVertexOpt.isPresent()) {
      final var v = executedVertexOpt.unwrap();
      path.addFirst(v);
      if (v.getVertexHash().equals(v.vertex().getParentVertexId())) {
        break;
      }
      executedVertexOpt = getExecutedVertex(v.vertex().getParentVertexId());
    }

    return path;
  }

  @Override
  public HighQC highQC() {
    return this.highQC;
  }

  @Override
  public Option<ImmutableList<VertexWithHash>> getVertices(HashCode vertexHash, int count) {
    HashCode nextId = vertexHash;
    ImmutableList.Builder<VertexWithHash> builder = ImmutableList.builderWithExpectedSize(count);
    for (int i = 0; i < count; i++) {
      final VertexWithHash vertexWithHash;
      if (nextId.equals(rootVertex.hash())) {
        vertexWithHash = rootVertex;
      } else if (this.vertices.containsKey(nextId)) {
        vertexWithHash = this.vertices.get(nextId);
      } else {
        return Option.empty();
      }

      builder.add(vertexWithHash);
      nextId = vertexWithHash.vertex().getParentVertexId();
    }

    return Option.present(builder.build());
  }

  @VisibleForTesting
  Set<HashCode> verticesForWhichChildrenAreBeingStored() {
    return this.vertexChildren.keySet();
  }

  /** Returns the number of vertices (excluding the root vertex) stored in this vertex store. */
  @VisibleForTesting
  int numVertices() {
    return vertices.size();
  }

  /**
   * Removes the least important (as determined by `findLeastImportantVertexToDrop`) vertex from
   * this vertex store. Throws if no vertices could be removed.
   */
  @VisibleForTesting
  VertexWithHash dropLeastImportantVertex() {
    final var vertexToRemove =
        findLeastImportantVertexToDrop()
            .orElseThrow(
                () ->
                    new RuntimeException(
                        "VertexStore has reached its maximum size of "
                            + config.maxNumVertices()
                            + " and no vertices could be removed."));
    removeVertexAndPruneInternal(vertexToRemove.hash(), null);
    return vertexToRemove;
  }

  /**
   * Finds the next vertex to remove if vertex store determines that some vertex *must* be removed
   * (e.g. due to reaching its maximum size). The vertex to remove is the oldest (by round) leaf
   * vertex on the shortest QC path (by QC round). See
   * VertexStoreTest.test_vertex_store_gc_removal_order for details.
   */
  private Optional<VertexWithHash> findLeastImportantVertexToDrop() {
    return this.vertices.entrySet().stream()
        .filter(
            e -> {
              final var hasChildren =
                  vertexChildren.containsKey(e.getKey())
                      && !vertexChildren.get(e.getKey()).isEmpty();
              // Don't remove any vertices that still have children
              return !hasChildren;
            })
        .map(Map.Entry::getValue)
        .min(
            /* Sort by: QC round (lowest first) and then proposed round (lowest first) */
            Comparator.<VertexWithHash, Round>comparing(v -> v.vertex().getQCToParent().getRound())
                .thenComparing(v -> v.vertex().getRound()));
  }
}
