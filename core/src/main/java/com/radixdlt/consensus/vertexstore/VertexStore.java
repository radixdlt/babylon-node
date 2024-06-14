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

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Result;
import com.radixdlt.utils.WrappedByteArray;
import java.util.List;

/** Manages the BFT Vertex chain. TODO: Move this logic into ledger package. */
public interface VertexStore {
  record CommittedUpdate(ImmutableList<ExecutedVertex> committedVertices, HighQC newHighQc) {}

  sealed interface InsertQcResult {
    record Inserted(
        HighQC newHighQc,
        WrappedByteArray serializedVertexStoreState,
        Option<CommittedUpdate> committedUpdate)
        implements InsertQcResult {}

    record Ignored() implements InsertQcResult {}

    record VertexIsMissing() implements InsertQcResult {}
  }

  sealed interface InsertTcResult {
    record Inserted(HighQC newHighQc, WrappedByteArray serializedVertexStoreState)
        implements InsertTcResult {}

    record Ignored() implements InsertTcResult {}
  }

  record InsertVertexChainResult(
      List<InsertQcResult.Inserted> insertedQcs, List<BFTInsertUpdate> insertUpdates) {}

  record RebuildSummary(
      VertexStoreState resultantState, WrappedByteArray serializedVertexStoreState) {}

  enum RebuildError {
    VERTEX_STORE_SIZE_EXCEEDED,
    VERTEX_EXECUTION_ERROR
  }

  InsertQcResult insertQc(QuorumCertificate qc);

  /**
   * Inserts a timeout certificate into the store.
   *
   * @param timeoutCertificate the timeout certificate
   */
  InsertTcResult insertTimeoutCertificate(TimeoutCertificate timeoutCertificate);

  /**
   * Inserts a vertex and then attempts to create the next header.
   *
   * @param vertexWithHash vertex to insert
   */
  Option<BFTInsertUpdate> insertVertex(VertexWithHash vertexWithHash);

  InsertVertexChainResult insertVertexChain(VertexChain vertexChain);

  Result<RebuildSummary, RebuildError> tryRebuild(VertexStoreState vertexStoreState);

  boolean containsVertex(HashCode vertexId);

  HighQC highQC();

  VertexWithHash getRoot();

  List<ExecutedVertex> getPathFromRoot(HashCode vertexId);

  /**
   * Returns the vertex with specified id or empty if not exists.
   *
   * @param vertexHash the id of a vertex
   * @return the specified vertex or empty
   */
  Option<ExecutedVertex> getExecutedVertex(HashCode vertexHash);

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
  Option<ImmutableList<VertexWithHash>> getVertices(HashCode vertexHash, int count);

  /** Returns the current size of the serialized vertex store state (in bytes). */
  int getCurrentSerializedSizeBytes();

  /**
   * Returns a value indicating the utilization of vertex store capacity. Returns a percentage value
   * between 0 and 1 (both inclusive). In practice the size is always above 0.
   */
  double getCurrentUtilizationRatio();
}
