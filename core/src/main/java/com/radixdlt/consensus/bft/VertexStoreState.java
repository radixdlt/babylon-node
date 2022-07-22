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

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.serialization.*;
import java.util.HashMap;
import java.util.Objects;
import java.util.Optional;
import javax.annotation.concurrent.Immutable;

/**
 * The current overall state of the vertex store.
 *
 * <p>A snapshot of this is serialized and saved at various times (e.g. atomically at commit), to
 * allow recovery when the node crashes, to prevent important data being lost (e.g. the content of
 * vertices which have been voted upon).
 *
 * <p>In future, we'd like to move to having a separate vertex store, responsible for maintaining
 * its own state.
 */
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
@Immutable
public final class VertexStoreState {
  private final VertexWithHash root;
  private final LedgerProof rootHeader;
  private final HighQC highQC;
  // TODO: collapse the following two
  private final ImmutableList<VertexWithHash> vertices;
  private final ImmutableMap<HashCode, VertexWithHash> idToVertex;
  private Optional<TimeoutCertificate> highestTC;

  private VertexStoreState(
      HighQC highQC,
      LedgerProof rootHeader,
      VertexWithHash root,
      ImmutableMap<HashCode, VertexWithHash> idToVertex,
      ImmutableList<VertexWithHash> vertices,
      Optional<TimeoutCertificate> highestTC) {
    this.highQC = highQC;
    this.rootHeader = rootHeader;
    this.root = root;
    this.idToVertex = idToVertex;
    this.vertices = vertices;
    this.highestTC = highestTC;
  }

  public static VertexStoreState create(
      HighQC highQC, VertexWithHash root, Optional<TimeoutCertificate> highestTC, Hasher hasher) {
    return create(highQC, root, ImmutableList.of(), highestTC, hasher);
  }

  public static VertexStoreState create(
      HighQC highQC,
      VertexWithHash root,
      ImmutableList<VertexWithHash> vertices,
      Optional<TimeoutCertificate> highestTC,
      Hasher hasher) {
    final var headers =
        highQC
            .highestCommittedQC()
            .getCommittedAndLedgerStateProof(hasher)
            .orElseThrow(
                () ->
                    new IllegalStateException(
                        String.format("highQC=%s does not have commit", highQC)));
    var bftHeader = headers.getFirst();

    if (!bftHeader.getVertexId().equals(root.getHash())) {
      throw new IllegalStateException(
          String.format("committedHeader=%s does not match rootVertex=%s", bftHeader, root));
    }

    var seen = new HashMap<HashCode, VertexWithHash>();
    seen.put(root.getHash(), root);

    for (var vertex : vertices) {
      if (!seen.containsKey(vertex.getParentVertexId())) {
        throw new IllegalStateException(
            String.format(
                "Missing qc=%s {root=%s vertices=%s}", vertex.getQCToParent(), root, vertices));
      }
      seen.put(vertex.getHash(), vertex);
    }

    if (seen.keySet().stream()
        .noneMatch(highQC.highestCommittedQC().getProposedHeader().getVertexId()::equals)) {
      throw new IllegalStateException(
          String.format(
              "highQC=%s highCommitted proposed missing {root=%s vertices=%s}",
              highQC, root, vertices));
    }

    if (seen.keySet().stream()
        .noneMatch(highQC.highestCommittedQC().getParentHeader().getVertexId()::equals)) {
      throw new IllegalStateException(
          String.format(
              "highQC=%s highCommitted parent does not have a corresponding vertex", highQC));
    }

    if (seen.keySet().stream()
        .noneMatch(highQC.highestQC().getParentHeader().getVertexId()::equals)) {
      throw new IllegalStateException(
          String.format("highQC=%s highQC parent does not have a corresponding vertex", highQC));
    }

    if (seen.keySet().stream()
        .noneMatch(highQC.highestQC().getProposedHeader().getVertexId()::equals)) {
      throw new IllegalStateException(
          String.format("highQC=%s highQC proposed does not have a corresponding vertex", highQC));
    }

    return new VertexStoreState(
        highQC, headers.getSecond(), root, ImmutableMap.copyOf(seen), vertices, highestTC);
  }

  public VertexStoreState prune(Hasher hasher) {
    var stateProof = highQC.highestQC().getCommittedAndLedgerStateProof(hasher);
    if (stateProof.isPresent()) {
      var newHeaders = stateProof.get();
      var header = newHeaders.getFirst();

      if (header.getRound().gt(root.getRound())) {
        var newRoot = idToVertex.get(header.getVertexId());
        var newVertices =
            ImmutableList.of(
                idToVertex.get(highQC.highestQC().getParentHeader().getVertexId()),
                idToVertex.get(highQC.highestQC().getProposedHeader().getVertexId()));
        var idToVertexMap =
            ImmutableMap.of(
                highQC.highestQC().getParentHeader().getVertexId(), newVertices.get(0),
                highQC.highestQC().getProposedHeader().getVertexId(), newVertices.get(1));
        var newHighQC = HighQC.from(highQC.highestQC());
        var proof = newHeaders.getSecond();
        return new VertexStoreState(
            newHighQC, proof, newRoot, idToVertexMap, newVertices, highestTC);
      }
    }

    return this;
  }

  public SerializedVertexStoreState toSerialized() {
    return new SerializedVertexStoreState(
        this.highQC,
        this.root.toSerializable(),
        this.vertices.stream()
            .map(VertexWithHash::toSerializable)
            .collect(ImmutableList.toImmutableList()),
        this.highestTC.orElse(null));
  }

  public HighQC getHighQC() {
    return highQC;
  }

  public VertexWithHash getRoot() {
    return root;
  }

  public ImmutableList<VertexWithHash> getVertices() {
    return vertices;
  }

  public LedgerProof getRootHeader() {
    return rootHeader;
  }

  @Override
  public int hashCode() {
    return Objects.hash(root, rootHeader, highQC, idToVertex, vertices, highestTC);
  }

  @Override
  public boolean equals(Object o) {
    if (o == this) {
      return true;
    }

    return o instanceof VertexStoreState other
        && Objects.equals(this.root, other.root)
        && Objects.equals(this.rootHeader, other.rootHeader)
        && Objects.equals(this.highQC, other.highQC)
        && Objects.equals(this.vertices, other.vertices)
        && Objects.equals(this.idToVertex, other.idToVertex)
        && Objects.equals(this.highestTC, other.highestTC);
  }

  /** Vertex Store State version which can be serialized. */
  @SerializerId2("store.vertices")
  public static final class SerializedVertexStoreState {

    @JsonProperty(SerializerConstants.SERIALIZER_NAME)
    @DsonOutput(DsonOutput.Output.ALL)
    SerializerDummy serializer = SerializerDummy.DUMMY;

    @JsonProperty("root")
    @DsonOutput(DsonOutput.Output.ALL)
    private final Vertex root;

    @JsonProperty("vertices")
    @DsonOutput(DsonOutput.Output.ALL)
    private final ImmutableList<Vertex> vertices;

    @JsonProperty("high_qc")
    @DsonOutput(DsonOutput.Output.ALL)
    private final HighQC highQC;

    @JsonProperty("highest_tc")
    @DsonOutput(DsonOutput.Output.ALL)
    private final TimeoutCertificate highestTC;

    @JsonCreator
    public SerializedVertexStoreState(
        @JsonProperty(value = "high_qc", required = true) HighQC highQC,
        @JsonProperty(value = "root", required = true) Vertex root,
        @JsonProperty(value = "vertices", required = true) ImmutableList<Vertex> vertices,
        @JsonProperty("highest_tc") TimeoutCertificate highestTC) {
      this.root = Objects.requireNonNull(root);
      this.vertices = Objects.requireNonNull(vertices);
      this.highQC = Objects.requireNonNull(highQC);
      this.highestTC = highestTC;
    }

    public Vertex getRoot() {
      return root;
    }

    public ImmutableList<Vertex> getVertices() {
      return vertices;
    }

    public HighQC getHighQC() {
      return highQC;
    }

    public Optional<TimeoutCertificate> getHighestTC() {
      return Optional.ofNullable(highestTC);
    }

    @Override
    public int hashCode() {
      return Objects.hash(root, vertices, highQC, highestTC);
    }

    public boolean isForEpoch(long epoch) {
      return getHighQC().highestQC().getEpoch() == epoch;
    }

    public VertexStoreState toVertexStoreState(Hasher hasher) {
      var rootVertex = getRoot().withId(hasher);

      var vertices =
          getVertices().stream()
              .map(v -> v.withId(hasher))
              .collect(ImmutableList.toImmutableList());

      return VertexStoreState.create(getHighQC(), rootVertex, vertices, getHighestTC(), hasher);
    }

    @Override
    public boolean equals(Object o) {
      if (o == this) {
        return true;
      }

      return (o instanceof SerializedVertexStoreState other)
          && Objects.equals(this.root, other.root)
          && Objects.equals(this.vertices, other.vertices)
          && Objects.equals(this.highQC, other.highQC)
          && Objects.equals(this.highestTC, other.highestTC);
    }

    @Override
    public String toString() {
      return String.format(
          "%s{highQC=%s root=%s vertices=%s highestTc=%s}",
          this.getClass().getSimpleName(), this.highQC, this.root, this.vertices, this.highestTC);
    }
  }
}
