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

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.BFTHeader;
import com.radixdlt.consensus.HighQC;
import com.radixdlt.consensus.Ledger;
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.TimeoutCertificate;
import com.radixdlt.consensus.VertexWithHash;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.ExecutedVertex;
import com.radixdlt.consensus.bft.MissingParentException;
import com.radixdlt.consensus.bft.VertexChain;
import com.radixdlt.consensus.bft.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.lang.Option;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedList;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import javax.annotation.concurrent.NotThreadSafe;

/** Manages the BFT Vertex chain. TODO: Move this logic into ledger package. */
@NotThreadSafe
public final class VertexStoreJavaImpl implements VertexStore {
  private final Hasher hasher;
  private final Ledger ledger;

  private final Map<HashCode, ExecutedVertex> vertices = new HashMap<>();
  private final Map<HashCode, Set<HashCode>> vertexChildren = new HashMap<>();

  // These should never be null
  private VertexWithHash rootVertex;
  private QuorumCertificate highestQC;
  private QuorumCertificate highestCommittedQC;
  private Optional<TimeoutCertificate> highestTC;

  private VertexStoreJavaImpl(
      Ledger ledger,
      Hasher hasher,
      VertexWithHash rootVertex,
      QuorumCertificate commitQC,
      QuorumCertificate highestQC,
      Optional<TimeoutCertificate> highestTC) {
    this.ledger = Objects.requireNonNull(ledger);
    this.hasher = Objects.requireNonNull(hasher);
    this.rootVertex = Objects.requireNonNull(rootVertex);
    this.highestQC = Objects.requireNonNull(highestQC);
    this.highestCommittedQC = Objects.requireNonNull(commitQC);
    this.vertexChildren.put(rootVertex.getHash(), new HashSet<>());
    this.highestTC = Objects.requireNonNull(highestTC);
  }

  public static VertexStoreJavaImpl create(
      VertexStoreState vertexStoreState, Ledger ledger, Hasher hasher) {
    var vertexStore =
        new VertexStoreJavaImpl(
            ledger,
            hasher,
            vertexStoreState.getRoot(),
            vertexStoreState.getHighQC().highestCommittedQC(),
            vertexStoreState.getHighQC().highestQC(),
            vertexStoreState.getHighQC().highestTC());

    for (var vertex : vertexStoreState.getVertices()) {
      var previous = vertexStore.getPathFromRoot(vertex.getParentVertexId());
      var executedVertexMaybe = ledger.prepare(previous, vertex);
      if (executedVertexMaybe.isEmpty()) {
        // Try pruning to see if that helps catching up to the ledger
        // This can occur if a node crashes between persisting a new QC and committing
        // TODO: Cleanup and remove
        var pruned = vertexStoreState.prune(hasher);
        if (!pruned.equals(vertexStoreState)) {
          return create(pruned, ledger, hasher);
        }

        // FIXME: If this occurs then it means that our highQC may not have an associated vertex
        // FIXME: so should save executedVertex
        break;
      } else {
        var executedVertex = executedVertexMaybe.get();
        vertexStore.vertices.put(executedVertex.getVertexHash(), executedVertex);
        vertexStore.vertexChildren.put(executedVertex.getVertexHash(), new HashSet<>());
        var siblings = vertexStore.vertexChildren.get(executedVertex.getParentId());
        siblings.add(executedVertex.getVertexHash());
      }
    }

    return vertexStore;
  }

  public VertexWithHash getRoot() {
    return rootVertex;
  }

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
    this.highestCommittedQC = vertexStoreState.getHighQC().highestCommittedQC();
    this.highestQC = vertexStoreState.getHighQC().highestQC();
    this.vertices.clear();
    this.vertexChildren.clear();
    this.vertexChildren.put(rootVertex.getHash(), new HashSet<>());

    for (var executedVertex : executedVertices) {
      this.vertices.put(executedVertex.getVertexHash(), executedVertex);
      this.vertexChildren.put(executedVertex.getVertexHash(), new HashSet<>());
      Set<HashCode> siblings = vertexChildren.get(executedVertex.getParentId());
      siblings.add(executedVertex.getVertexHash());
    }

    return Option.present(vertexStoreState);
  }

  public boolean containsVertex(HashCode vertexId) {
    return vertices.containsKey(vertexId) || rootVertex.getHash().equals(vertexId);
  }

  public InsertQcResult insertQc(QuorumCertificate qc) {
    if (!this.containsVertex(qc.getProposedHeader().getVertexId())) {
      return new VertexStore.InsertQcResult.VertexIsMissing(); // false
    }

    final var hasAnyChildren = !vertexChildren.get(qc.getProposedHeader().getVertexId()).isEmpty();
    if (hasAnyChildren) {
      // TODO: Check to see if qc's match in case there's a fault
      return new VertexStore.InsertQcResult.Ignored();
    }

    // proposed vertex doesn't have any children
    boolean isHighQC = qc.getRound().gt(highestQC.getRound());
    boolean isAnythingCommitted = qc.getCommittedAndLedgerStateProof(hasher).isPresent();
    if (!isHighQC && !isAnythingCommitted) {
      return new VertexStore.InsertQcResult.Ignored();
    }

    if (isHighQC) {
      highestQC = qc;
    }

    final var committedUpdate =
        Option.from(qc.getCommittedHeader().flatMap(header -> this.commit(header, qc)));

    return new VertexStore.InsertQcResult.Inserted(highQC(), getState(), committedUpdate);
  }

  private void getChildrenVerticesList(
      VertexWithHash parent, ImmutableList.Builder<VertexWithHash> builder) {
    Set<HashCode> childrenIds = this.vertexChildren.get(parent.getHash());
    if (childrenIds == null) {
      return;
    }

    for (HashCode childId : childrenIds) {
      VertexWithHash v = vertices.get(childId).getVertex();
      builder.add(v);
      getChildrenVerticesList(v, builder);
    }
  }

  private VertexStoreState getState() {
    // TODO: store list dynamically rather than recomputing
    ImmutableList.Builder<VertexWithHash> verticesBuilder = ImmutableList.builder();
    getChildrenVerticesList(this.rootVertex, verticesBuilder);
    return VertexStoreState.create(
        this.highQC(), this.rootVertex, verticesBuilder.build(), this.highestTC, hasher);
  }

  /**
   * Inserts a timeout certificate into the store.
   *
   * @param timeoutCertificate the timeout certificate
   */
  public void insertTimeoutCertificate(TimeoutCertificate timeoutCertificate) {
    if (this.highestTC.isEmpty()
        || this.highestTC.get().getRound().lt(timeoutCertificate.getRound())) {
      this.highestTC = Optional.of(timeoutCertificate);
    }
  }

  /**
   * Returns the vertex with specified id or empty if not exists.
   *
   * @param vertexHash the id of a vertex
   * @return the specified vertex or empty
   */
  // TODO: reimplement in async way
  public Option<ExecutedVertex> getExecutedVertex(HashCode vertexHash) {
    return Option.option(vertices.get(vertexHash));
  }

  public InsertVertexChainResult insertVertexChain(VertexChain vertexChain) {
    final var bftInsertUpdates = new ArrayList<BFTInsertUpdate>();
    final var insertedQcs = new ArrayList<InsertQcResult.Inserted>();
    for (VertexWithHash v : vertexChain.getVertices()) {
      final var insertQcResult = insertQc(v.getParentQC());

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

  /**
   * Inserts a vertex and then attempts to create the next header.
   *
   * @param vertex vertex to insert
   */
  public Option<BFTInsertUpdate> insertVertex(VertexWithHash vertex) {
    ExecutedVertex v = vertices.get(vertex.getHash());
    if (v != null) {
      return Option.empty();
    }

    if (!this.containsVertex(vertex.getParentVertexId())) {
      throw new MissingParentException(vertex.getParentVertexId());
    }

    return insertVertexInternal(vertex);
  }

  private Option<BFTInsertUpdate> insertVertexInternal(VertexWithHash vertex) {
    LinkedList<ExecutedVertex> previous = getPathFromRoot(vertex.getParentVertexId());
    final var executedVertexOption = Option.from(ledger.prepare(previous, vertex));
    return executedVertexOption.map(
        executedVertex -> {
          vertices.put(executedVertex.getVertexHash(), executedVertex);
          vertexChildren.put(executedVertex.getVertexHash(), new HashSet<>());
          Set<HashCode> siblings = vertexChildren.get(executedVertex.getParentId());
          siblings.add(executedVertex.getVertexHash());

          VertexStoreState vertexStoreState = getState();
          return BFTInsertUpdate.insertedVertex(executedVertex, siblings.size(), vertexStoreState);
        });
  }

  private void removeVertexAndPruneInternal(HashCode vertexId, HashCode skip) {
    vertices.remove(vertexId);

    if (this.rootVertex.getHash().equals(vertexId)) {
      return;
    }

    var children = vertexChildren.remove(vertexId);
    for (HashCode child : children) {
      if (!child.equals(skip)) {
        removeVertexAndPruneInternal(child, null);
      }
    }
  }

  /**
   * Commit a vertex. Executes the transactions and prunes the tree.
   *
   * @param header the header to be committed
   * @param commitQC the proof of commit
   */
  private Optional<CommittedUpdate> commit(BFTHeader header, QuorumCertificate commitQC) {
    if (header.getRound().compareTo(this.rootVertex.getRound()) <= 0) {
      return Optional.empty();
    }

    final HashCode vertexId = header.getVertexId();
    final VertexWithHash tipVertex = vertices.get(vertexId).getVertex();

    this.rootVertex = tipVertex;
    this.highestCommittedQC = commitQC;
    final var path = ImmutableList.copyOf(getPathFromRoot(tipVertex.getHash()));
    HashCode prev = null;
    for (int i = path.size() - 1; i >= 0; i--) {
      this.removeVertexAndPruneInternal(path.get(i).getVertexHash(), prev);
      prev = path.get(i).getVertexHash();
    }

    return Optional.of(new CommittedUpdate(path));
  }

  public LinkedList<ExecutedVertex> getPathFromRoot(HashCode vertexId) {
    final LinkedList<ExecutedVertex> path = new LinkedList<>();

    ExecutedVertex vertex = vertices.get(vertexId);
    while (vertex != null) {
      path.addFirst(vertex);
      vertex = vertices.get(vertex.getParentId());
    }

    return path;
  }

  /**
   * Retrieves the highest QC and highest committed QC in the store.
   *
   * @return the highest QCs
   */
  public HighQC highQC() {
    return HighQC.from(this.highestQC, this.highestCommittedQC, this.highestTC);
  }

  /**
   * Retrieves list of vertices starting with the given vertexId and then proceeding to its
   * ancestors.
   *
   * <p>if the store does not contain some vertex then will return an empty list.
   *
   * @param vertexHash the id of the vertex
   * @param count the number of vertices to retrieve
   * @return the list of vertices if all found, otherwise an empty list
   */
  public Option<ImmutableList<VertexWithHash>> getVertices(HashCode vertexHash, int count) {
    HashCode nextId = vertexHash;
    ImmutableList.Builder<VertexWithHash> builder = ImmutableList.builderWithExpectedSize(count);
    for (int i = 0; i < count; i++) {
      final VertexWithHash vertexWithHash;
      if (nextId.equals(rootVertex.getHash())) {
        vertexWithHash = rootVertex;
      } else if (this.vertices.containsKey(nextId)) {
        final ExecutedVertex executedVertex = this.vertices.get(nextId);
        vertexWithHash = executedVertex.getVertex();
      } else {
        return Option.empty();
      }

      builder.add(vertexWithHash);
      nextId = vertexWithHash.getParentVertexId();
    }

    return Option.present(builder.build());
  }
}
