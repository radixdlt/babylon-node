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

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.assertj.core.api.Assertions.*;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTCommittedUpdate;
import com.radixdlt.consensus.bft.BFTHighQCUpdate;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.ZeroHasher;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;
import java.util.function.Supplier;
import java.util.stream.Stream;
import org.junit.Before;
import org.junit.Test;

public class VertexStoreTest {
  private VertexWithHash genesisVertex;
  private Supplier<VertexWithHash> nextVertex;
  private Function<Boolean, VertexWithHash> nextSkippableVertex;
  private HashCode genesisHash;
  private QuorumCertificate rootQC;
  private VertexStoreJavaImpl underlyingVertexStore;
  private VertexStoreAdapter vertexStoreAdapter;
  private Ledger ledger;
  private EventDispatcher<BFTInsertUpdate> bftUpdateSender;
  private EventDispatcher<BFTRebuildUpdate> rebuildUpdateEventDispatcher;
  private EventDispatcher<BFTHighQCUpdate> bftHighQCUpdateEventDispatcher;
  private EventDispatcher<BFTCommittedUpdate> committedSender;
  private Hasher hasher = new Sha256Hasher(DefaultSerialization.getInstance());

  private static final LedgerHeader MOCKED_HEADER =
      LedgerHeader.create(
          0,
          Round.genesis(),
          new AccumulatorState(0, HashUtils.zero256()),
          HashUtils.zero256(),
          0,
          0);

  @Before
  public void setUp() {
    // No type check issues with mocking generic here
    Ledger ssc = mock(Ledger.class);
    this.ledger = ssc;
    // TODO: replace mock with the real thing
    doAnswer(
            invocation -> {
              VertexWithHash vertexWithHash = invocation.getArgument(1);
              return Optional.of(
                  new ExecutedVertex(
                      vertexWithHash, MOCKED_HEADER, ImmutableList.of(), ImmutableMap.of(), 1L));
            })
        .when(ledger)
        .prepare(any(), any());

    this.bftUpdateSender = rmock(EventDispatcher.class);
    this.rebuildUpdateEventDispatcher = rmock(EventDispatcher.class);
    this.bftHighQCUpdateEventDispatcher = rmock(EventDispatcher.class);
    this.committedSender = rmock(EventDispatcher.class);

    this.genesisVertex = Vertex.createInitialEpochVertex(MOCKED_HEADER).withId(ZeroHasher.INSTANCE);
    this.genesisHash = genesisVertex.hash();
    this.rootQC = QuorumCertificate.createInitialEpochQC(genesisVertex, MOCKED_HEADER);
    this.underlyingVertexStore =
        VertexStoreJavaImpl.create(
            VertexStoreState.create(HighQC.ofInitialEpochQc(rootQC), genesisVertex, hasher),
            ledger,
            hasher);
    this.vertexStoreAdapter =
        new VertexStoreAdapter(
            underlyingVertexStore,
            bftHighQCUpdateEventDispatcher,
            bftUpdateSender,
            rebuildUpdateEventDispatcher,
            committedSender);

    AtomicReference<BFTHeader> lastParentHeader =
        new AtomicReference<>(new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER));
    AtomicReference<BFTHeader> lastGrandParentHeader =
        new AtomicReference<>(new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER));
    AtomicReference<BFTHeader> lastGreatGrandParentHeader =
        new AtomicReference<>(new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER));

    this.nextSkippableVertex =
        (skipOne) -> {
          BFTHeader parentHeader = lastParentHeader.get();
          BFTHeader grandParentHeader = lastGrandParentHeader.get();
          BFTHeader greatGrandParentHeader = lastGreatGrandParentHeader.get();
          final QuorumCertificate qc;
          if (!parentHeader.getRound().equals(Round.genesis())) {
            VoteData data =
                new VoteData(
                    parentHeader, grandParentHeader, skipOne ? null : greatGrandParentHeader);
            qc = new QuorumCertificate(data, new TimestampedECDSASignatures());
          } else {
            qc = rootQC;
          }
          Round round = parentHeader.getRound().next();
          if (skipOne) {
            round = round.next();
          }

          var vertex =
              Vertex.create(
                      qc,
                      round,
                      List.of(RawNotarizedTransaction.create(new byte[0])),
                      BFTValidatorId.random(),
                      0)
                  .withId(hasher);
          lastParentHeader.set(new BFTHeader(round, vertex.hash(), MOCKED_HEADER));
          lastGrandParentHeader.set(parentHeader);
          lastGreatGrandParentHeader.set(grandParentHeader);

          return vertex;
        };

    this.nextVertex = () -> nextSkippableVertex.apply(false);
  }

  @Test
  public void adding_a_qc_should_update_highest_qc() {
    // Arrange
    final var vertices = Stream.generate(this.nextVertex).limit(4).toList();
    vertexStoreAdapter.insertVertex(vertices.get(0));

    // Act
    QuorumCertificate qc = vertices.get(1).vertex().getQCToParent();
    vertexStoreAdapter.insertQc(qc);

    // Assert
    assertThat(vertexStoreAdapter.highQC().highestQC()).isEqualTo(qc);
    assertThat(vertexStoreAdapter.highQC().highestCommittedQC()).isEqualTo(rootQC);
    assertTrue(isVertexStoreChildrenMappingTidy(underlyingVertexStore));
  }

  @Test
  public void vertex_store_should_correctly_remove_vertices_when_commit() {
    /* This test checks that when vertex is committed all obsolete vertices are
    removed from the vertex store (including their children).
    Specifically, this scenario includes the following vertices:

      A
      | \
      B  C
      | \
      D  E
      |
      F
      |
      G

    adding a QC for G should result in vertex D being committed, which should result in:
    a) D becoming a new rootVertex,
    b) removal of vertices A, B, C and E */

    // Arrange
    // Three mock vertices for A's ancestors
    final var v1 = new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER);
    final var v2 = new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER);
    final var v3 = new BFTHeader(Round.genesis(), genesisHash, MOCKED_HEADER);

    final var vertexA = createVertex(v3, v2, v1, new byte[] {0});
    final var vertexB = createVertex(v2, v1, mockedHeaderOf(vertexA), new byte[] {0});
    final var vertexC = createVertex(v2, v1, mockedHeaderOf(vertexA), new byte[] {1});
    final var vertexD =
        createVertex(v1, mockedHeaderOf(vertexA), mockedHeaderOf(vertexB), new byte[] {0});
    final var vertexE =
        createVertex(v1, mockedHeaderOf(vertexA), mockedHeaderOf(vertexB), new byte[] {1});
    final var vertexF =
        createVertex(
            mockedHeaderOf(vertexA),
            mockedHeaderOf(vertexB),
            mockedHeaderOf(vertexD),
            new byte[] {0});
    final var vertexG =
        createVertex(
            mockedHeaderOf(vertexB),
            mockedHeaderOf(vertexD),
            mockedHeaderOf(vertexF),
            new byte[] {0});

    for (var v : List.of(vertexA, vertexB, vertexC, vertexD, vertexE, vertexF, vertexG)) {
      vertexStoreAdapter.insertVertex(v);
    }

    final var qcForVertexG =
        createVertex(
                mockedHeaderOf(vertexD),
                mockedHeaderOf(vertexF),
                mockedHeaderOf(vertexG),
                new byte[] {0})
            .vertex()
            .getQCToParent();

    // Act
    final var insertQcResult = vertexStoreAdapter.insertQc(qcForVertexG);

    assertTrue(insertQcResult instanceof VertexStore.InsertQcResult.Inserted);
    assertEquals(vertexStoreAdapter.getRoot().hash(), vertexD.hash());
    assertFalse(vertexStoreAdapter.containsVertex(vertexA.hash()));
    assertFalse(vertexStoreAdapter.containsVertex(vertexB.hash()));
    assertFalse(vertexStoreAdapter.containsVertex(vertexC.hash()));
    assertFalse(vertexStoreAdapter.containsVertex(vertexE.hash()));
    assertTrue(isVertexStoreChildrenMappingTidy(underlyingVertexStore));
  }

  private VertexWithHash createVertex(
      BFTHeader greatGrandParent, BFTHeader grandParent, BFTHeader parent, byte[] tx) {
    final QuorumCertificate qc;
    if (!parent.getRound().equals(Round.genesis())) {
      final var data = new VoteData(parent, grandParent, greatGrandParent);
      qc = new QuorumCertificate(data, new TimestampedECDSASignatures());
    } else {
      qc = rootQC;
    }
    final var round = parent.getRound().next();
    return Vertex.create(
            qc, round, List.of(RawNotarizedTransaction.create(tx)), BFTValidatorId.random(), 0)
        .withId(hasher);
  }

  private BFTHeader mockedHeaderOf(VertexWithHash vertexWithHash) {
    return new BFTHeader(vertexWithHash.vertex().getRound(), vertexWithHash.hash(), MOCKED_HEADER);
  }

  @Test
  public void adding_a_qc_with_commit_should_commit_vertices_to_ledger() {
    // Arrange
    final var vertices = Stream.generate(this.nextVertex).limit(4).toList();
    vertexStoreAdapter.insertVertex(vertices.get(0));
    vertexStoreAdapter.insertVertex(vertices.get(1));
    vertexStoreAdapter.insertVertex(vertices.get(2));

    // Act
    QuorumCertificate qc = vertices.get(3).vertex().getQCToParent();
    final var insertQcResult = vertexStoreAdapter.insertQc(qc);

    // Assert
    assertTrue(insertQcResult instanceof VertexStore.InsertQcResult.Inserted);
    assertThat(vertexStoreAdapter.highQC().highestQC()).isEqualTo(qc);
    assertThat(vertexStoreAdapter.highQC().highestCommittedQC()).isEqualTo(qc);
    assertThat(vertexStoreAdapter.getVertices(vertices.get(2).hash(), 3))
        .hasValue(ImmutableList.of(vertices.get(2), vertices.get(1), vertices.get(0)));
    verify(committedSender, times(1))
        .dispatch(
            argThat(
                u ->
                    u.committed().size() == 1
                        && u.committed().get(0).getVertexWithHash().equals(vertices.get(0))));
    assertTrue(isVertexStoreChildrenMappingTidy(underlyingVertexStore));
  }

  /**
   * Checks that the vertex store is not storing any obsolete child vertices entries for vertices
   * that no longer exist.
   */
  private boolean isVertexStoreChildrenMappingTidy(VertexStoreJavaImpl vertexStore) {
    for (HashCode vertexHash : vertexStore.verticesForWhichChildrenAreBeingStored()) {
      if (!vertexStore.containsVertex(vertexHash)) {
        return false;
      }
    }
    return true;
  }

  @Test
  public void adding_a_qc_which_needs_sync_should_return_a_matching_result() {
    // Arrange
    this.nextVertex.get();

    // Act
    QuorumCertificate qc = this.nextVertex.get().vertex().getQCToParent();
    final var insertQcResult = vertexStoreAdapter.insertQc(qc);

    // Assert
    assertTrue(insertQcResult instanceof VertexStore.InsertQcResult.VertexIsMissing);
  }

  @Test
  public void rebuilding_should_emit_updates() {
    // Arrange
    final var vertices = Stream.generate(this.nextVertex).limit(4).toList();

    final var qc = vertices.get(3).vertex().getQCToParent();
    VertexStoreState vertexStoreState =
        VertexStoreState.create(
            HighQC.from(qc, qc, vertexStoreAdapter.highQC().highestTC()),
            vertices.get(0),
            vertices.stream().skip(1).collect(ImmutableList.toImmutableList()),
            hasher);

    // Act
    vertexStoreAdapter.tryRebuild(vertexStoreState);

    // Assert
    verify(rebuildUpdateEventDispatcher, times(1))
        .dispatch(
            argThat(
                u -> {
                  List<VertexWithHash> sentVertices = u.getVertexStoreState().getVertices();
                  return sentVertices.equals(vertices.subList(1, vertices.size()));
                }));
  }

  @Test
  public void inserting_a_tc_should_only_replace_tcs_for_lower_rounds() {
    TimeoutCertificate initialTC =
        new TimeoutCertificate(1, Round.of(100), mock(TimestampedECDSASignatures.class));
    TimeoutCertificate higherTC =
        new TimeoutCertificate(1, Round.of(101), mock(TimestampedECDSASignatures.class));

    vertexStoreAdapter.insertTimeoutCertificate(initialTC);
    assertEquals(initialTC, vertexStoreAdapter.highQC().highestTC().orElse(null));

    vertexStoreAdapter.insertTimeoutCertificate(higherTC);
    assertEquals(higherTC, vertexStoreAdapter.highQC().highestTC().orElse(null));

    vertexStoreAdapter.insertTimeoutCertificate(initialTC);
    assertEquals(higherTC, vertexStoreAdapter.highQC().highestTC().orElse(null));
  }
}
