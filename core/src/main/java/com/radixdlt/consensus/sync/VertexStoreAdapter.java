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
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.TimeoutCertificate;
import com.radixdlt.consensus.bft.BFTCommittedUpdate;
import com.radixdlt.consensus.bft.BFTHighQCUpdate;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.BFTRebuildUpdate;
import com.radixdlt.consensus.bft.PreparedVertex;
import com.radixdlt.consensus.bft.VerifiedVertex;
import com.radixdlt.consensus.bft.VerifiedVertexChain;
import com.radixdlt.consensus.bft.VerifiedVertexStoreState;
import com.radixdlt.environment.EventDispatcher;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import javax.inject.Inject;

public final class VertexStoreAdapter {
  private final VertexStore vertexStore;

  private final EventDispatcher<BFTHighQCUpdate> highQCUpdateDispatcher;
  private final EventDispatcher<BFTInsertUpdate> bftUpdateDispatcher;
  private final EventDispatcher<BFTRebuildUpdate> bftRebuildDispatcher;
  private final EventDispatcher<BFTCommittedUpdate> bftCommittedDispatcher;

  @Inject
  public VertexStoreAdapter(
      VertexStore vertexStore,
      EventDispatcher<BFTHighQCUpdate> highQCUpdateDispatcher,
      EventDispatcher<BFTInsertUpdate> bftUpdateDispatcher,
      EventDispatcher<BFTRebuildUpdate> bftRebuildDispatcher,
      EventDispatcher<BFTCommittedUpdate> bftCommittedDispatcher) {
    this.vertexStore = Objects.requireNonNull(vertexStore);
    this.highQCUpdateDispatcher = Objects.requireNonNull(highQCUpdateDispatcher);
    this.bftUpdateDispatcher = Objects.requireNonNull(bftUpdateDispatcher);
    this.bftRebuildDispatcher = Objects.requireNonNull(bftRebuildDispatcher);
    this.bftCommittedDispatcher = Objects.requireNonNull(bftCommittedDispatcher);
  }

  public boolean tryRebuild(VerifiedVertexStoreState vertexStoreState) {
    final var result = vertexStore.tryRebuild(vertexStoreState);

    result.onPresent(
        newVertexStoreState ->
            bftRebuildDispatcher.dispatch(BFTRebuildUpdate.create(newVertexStoreState)));

    return result.isPresent();
  }

  public void insertTimeoutCertificate(TimeoutCertificate timeoutCertificate) {
    vertexStore.insertTimeoutCertificate(timeoutCertificate);
  }

  public boolean insertQc(QuorumCertificate qc) {
    return switch (vertexStore.insertQc(qc)) {
      case VertexStore.InsertQcResult.Inserted inserted -> {
        // TODO: why is this if statement needed?
        if (inserted.committedUpdate().isEmpty()) {
          this.highQCUpdateDispatcher.dispatch(
              BFTHighQCUpdate.create(inserted.verifiedVertexStoreState()));
        }
        inserted
            .committedUpdate()
            .onPresent(
                committedUpdate ->
                    this.bftCommittedDispatcher.dispatch(
                        new BFTCommittedUpdate(
                            committedUpdate.committedVertices(),
                            inserted.verifiedVertexStoreState())));
        yield true;
      }
      case VertexStore.InsertQcResult.Ignored ignored -> true;
      case VertexStore.InsertQcResult.VertexIsMissing vertexIsMissing -> false;
    };
  }

  public Optional<PreparedVertex> getPreparedVertex(HashCode id) {
    return vertexStore.getPreparedVertex(id).toOptional();
  }

  public List<PreparedVertex> getPathFromRoot(HashCode vertexId) {
    return vertexStore.getPathFromRoot(vertexId);
  }

  public void insertVertexChain(VerifiedVertexChain verifiedVertexChain) {
    final var result = vertexStore.insertVertexChain(verifiedVertexChain);
    result
        .insertedQcs()
        .forEach(
            insertedQc -> {
              this.highQCUpdateDispatcher.dispatch(
                  BFTHighQCUpdate.create(insertedQc.verifiedVertexStoreState()));
              insertedQc
                  .committedUpdate()
                  .onPresent(
                      committedUpdate ->
                          this.bftCommittedDispatcher.dispatch(
                              new BFTCommittedUpdate(
                                  committedUpdate.committedVertices(),
                                  insertedQc.verifiedVertexStoreState())));
            });
    result.insertUpdates().forEach(bftUpdateDispatcher::dispatch);
  }

  public Optional<ImmutableList<VerifiedVertex>> getVertices(HashCode vertexId, int count) {
    return vertexStore.getVertices(vertexId, count).toOptional();
  }

  public void insertVertex(VerifiedVertex vertex) {
    final var result = vertexStore.insertVertex(vertex);
    result.onPresent(bftUpdateDispatcher::dispatch);
  }

  public boolean containsVertex(HashCode vertexId) {
    return vertexStore.containsVertex(vertexId);
  }

  public boolean hasCommittedVertexOrRootAtOrAboveView(BFTHeader committedHeader) {
    if (vertexStore.containsVertex(committedHeader.getVertexId())) {
      return true;
    } else {
      final var rootView = vertexStore.getRoot().getView();
      return rootView.gte(committedHeader.getView());
    }
  }

  public HighQC highQC() {
    return vertexStore.highQC();
  }

  public VerifiedVertex getRoot() {
    return vertexStore.getRoot();
  }
}
