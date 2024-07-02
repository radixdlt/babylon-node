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

package com.radixdlt.ledger;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Iterators;
import com.google.common.collect.Streams;
import com.google.inject.Inject;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.RemoteEventProcessor;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.transactions.NotarizedTransactionHash;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.WrappedByteArray;
import java.util.*;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Synchronizes execution */
public final class StateComputerLedger implements Ledger, ProposalGenerator {

  private static final Logger log = LogManager.getLogger();

  public interface ExecutedTransaction {
    RawLedgerTransaction transaction();

    Option<NotarizedTransactionHash> notarizedTransactionHash();
  }

  public static class StateComputerPrepareResult {
    private final List<ExecutedTransaction> executedTransactions;
    private final int rejectedTransactionCount;
    private final NextEpoch nextEpoch;
    private final String nextProtocolVersion;
    private final LedgerHashes ledgerHashes;

    public StateComputerPrepareResult(
        List<ExecutedTransaction> executedTransactions,
        int rejectedTransactionCount,
        NextEpoch nextEpoch,
        String nextProtocolVersion,
        LedgerHashes ledgerHashes) {
      this.executedTransactions = Objects.requireNonNull(executedTransactions);
      this.rejectedTransactionCount = rejectedTransactionCount;
      this.nextEpoch = nextEpoch;
      this.nextProtocolVersion = nextProtocolVersion;
      this.ledgerHashes = ledgerHashes;
    }

    public StateComputerPrepareResult(
        List<ExecutedTransaction> executedTransactions,
        int rejectedTransactionCount,
        LedgerHashes ledgerHashes) {
      this(executedTransactions, rejectedTransactionCount, null, null, ledgerHashes);
    }

    public Option<NextEpoch> getNextEpoch() {
      return Option.option(nextEpoch);
    }

    public Option<String> getNextProtocolVersion() {
      return Option.option(nextProtocolVersion);
    }

    public LedgerHashes getLedgerHashes() {
      return ledgerHashes;
    }

    public List<ExecutedTransaction> getSuccessfullyExecutedTransactions() {
      return executedTransactions;
    }

    public int getRejectedTransactionCount() {
      return rejectedTransactionCount;
    }
  }

  public interface StateComputer {
    void addToMempool(MempoolAdd mempoolAdd, NodeId origin);

    List<RawNotarizedTransaction> getTransactionsForProposal(
        List<ExecutedTransaction> executedTransactions);

    StateComputerPrepareResult prepare(
        LedgerHashes committedLedgerHashes,
        List<ExecutedVertex> preparedUncommittedVertices,
        LedgerHashes preparedUncommittedLedgerHashes,
        List<RawNotarizedTransaction> proposedTransactions,
        RoundDetails roundDetails);

    LedgerProofBundle commit(
        LedgerExtension ledgerExtension, Option<WrappedByteArray> serializedVertexStoreState);
  }

  private final StateComputer stateComputer;
  private final Hasher hasher;
  private final Metrics metrics;
  private final TimeSupplier timeSupplier;
  private final Object commitAndAdvanceLedgerLock;
  private LedgerProofBundle latestProof;

  @Inject
  public StateComputerLedger(
      TimeSupplier timeSupplier,
      LedgerProofBundle initialLatestProof,
      StateComputer stateComputer,
      Hasher hasher,
      Metrics metrics) {
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.stateComputer = Objects.requireNonNull(stateComputer);
    this.hasher = Objects.requireNonNull(hasher);
    this.metrics = Objects.requireNonNull(metrics);
    this.latestProof = initialLatestProof;
    this.commitAndAdvanceLedgerLock = new Object();
  }

  public RemoteEventProcessor<NodeId, MempoolAdd> mempoolAddRemoteEventProcessor() {
    return (node, mempoolAdd) -> stateComputer.addToMempool(mempoolAdd, node);
  }

  public EventProcessor<MempoolAdd> mempoolAddEventProcessor() {
    return mempoolAdd -> stateComputer.addToMempool(mempoolAdd, null);
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      Round round, List<ExecutedVertex> prepared) {
    final ImmutableList<ExecutedTransaction> executedTransactions =
        prepared.stream()
            .flatMap(ExecutedVertex::successfulTransactions)
            .collect(ImmutableList.toImmutableList());
    return stateComputer.getTransactionsForProposal(executedTransactions);
  }

  @Override
  public Optional<ExecutedVertex> prepare(
      LinkedList<ExecutedVertex> previous, VertexWithHash vertexWithHash) {
    return metrics.ledger().prepare().measure(() -> this.prepareInternal(previous, vertexWithHash));
  }

  private Optional<ExecutedVertex> prepareInternal(
      LinkedList<ExecutedVertex> previousVertices, VertexWithHash vertexWithHash) {
    final var vertex = vertexWithHash.vertex();
    final LedgerHeader parentHeader = vertex.getParentHeader().getLedgerHeader();

    final StateComputerPrepareResult result;
    synchronized (this.commitAndAdvanceLedgerLock) {
      final var latestStateVersion = this.latestProof.resultantStateVersion();
      final var latestLedgerHashes =
          REv2ToConsensus.ledgerHashes(this.latestProof.primaryProof().ledgerHeader().hashes());

      if (latestStateVersion > parentHeader.getStateVersion()) {
        // We have received a stale vertex to prepare - ignore it.
        return Optional.empty();
      }

      if (parentHeader.isEndOfEpoch() || parentHeader.isProtocolUpdate()) {
        // Don't execute any transactions and commit to the same LedgerHeader if in the process of
        // an epoch change or a protocol update.
        // Updates to LedgerHeader here may cause a disagreement on the next epoch
        // initial vertex if a TC occurs for example.
        return Optional.of(
            new ExecutedVertex(
                vertexWithHash, parentHeader, ImmutableList.of(), this.timeSupplier.currentTime()));
      }

      // It's possible that this function is called with a list of vertices which starts with some
      // committed vertices. By matching on the state version, we remove the already committed
      // vertices from the "previous" list.
      final var previousVertexIterator = Iterators.peekingIterator(previousVertices.iterator());
      while (previousVertexIterator.hasNext()) {
        final var previousVertexBaseHeader =
            previousVertexIterator.peek().vertex().getParentHeader().getLedgerHeader();
        if (previousVertexBaseHeader.getStateVersion() == latestStateVersion) {
          if (!previousVertexBaseHeader.getHashes().equals(latestLedgerHashes)) {
            // Some vertex has matched on the state version (which isn't particularly improbable,
            // since only a number of transactions must coincide). However, the ledger hashes did
            // not match, which means that other vertices than ours were committed.
            return Optional.empty();
          }
          break;
        }
        previousVertexIterator.next();
      }
      final var verticesInExtension = Streams.stream(previousVertexIterator).toList();

      if (verticesInExtension.isEmpty()) {
        // None of the previous vertices has matched our current top of ledger. There is still a
        // possibility that the proposed vertex is built right on that top. But if not, then we
        // cannot progress.
        if (!parentHeader.getHashes().equals(latestLedgerHashes)) {
          return Optional.empty();
        }
      }

      result =
          this.stateComputer.prepare(
              latestLedgerHashes,
              verticesInExtension,
              parentHeader.getHashes(),
              vertex.getTransactions(),
              RoundDetails.fromVertex(vertexWithHash));
    }

    final LedgerHeader ledgerHeader =
        LedgerHeader.create(
            parentHeader.getEpoch(),
            vertex.getRound(),
            parentHeader.getStateVersion() + result.getSuccessfullyExecutedTransactions().size(),
            result.getLedgerHashes(),
            vertex.getQCToParent().getWeightedTimestampOfSignatures(),
            vertex.proposerTimestamp(),
            result.getNextEpoch().or((NextEpoch) null),
            result.getNextProtocolVersion().or((String) null));

    return Optional.of(
        new ExecutedVertex(
            vertexWithHash,
            ledgerHeader,
            result.getSuccessfullyExecutedTransactions(),
            this.timeSupplier.currentTime()));
  }

  public EventProcessor<BFTHighQCUpdate> bftHighQcUpdateEventProcessor() {
    return highQcUpdate -> {
      highQcUpdate
          .committedVertices()
          .ifPresent(
              committedVertices -> {
                updateCommittedVerticesMetrics(committedVertices);
                highQcUpdate
                    .newHighQc()
                    .highestCommittedQC()
                    .getProcessedCommit(hasher)
                    .ifPresentOrElse(
                        processedQcCommit -> {
                          final var transactions =
                              committedVertices.stream()
                                  .flatMap(ExecutedVertex::successfulTransactions)
                                  .map(ExecutedTransaction::transaction)
                                  .collect(ImmutableList.toImmutableList());
                          processQcCommit(
                              processedQcCommit,
                              transactions,
                              highQcUpdate.serializedVertexStoreState());
                        },
                        () -> {
                          // no-op, ignore
                          this.metrics.ledger().ignoredBftCommittedUpdates().inc();
                        });
              });
    };
  }

  private void processQcCommit(
      ProcessedQcCommit processedQcCommit,
      ImmutableList<RawLedgerTransaction> transactions,
      WrappedByteArray serializedVertexStoreState) {
    switch (processedQcCommit) {
      case ProcessedQcCommit.OfConensusQc ofConensusQc -> {
        final var ledgerExtension = new LedgerExtension(transactions, ofConensusQc.ledgerProof());
        metrics
            .ledger()
            .commit()
            .measure(() -> this.commit(ledgerExtension, Option.some(serializedVertexStoreState)));
      }
      case ProcessedQcCommit.OfInitialEpochQc ofInitialEpochQc -> {
        // no-op, ignore
        this.metrics.ledger().ignoredBftCommittedUpdates().inc();
      }
    }
  }

  private void updateCommittedVerticesMetrics(ImmutableList<ExecutedVertex> committedVertices) {
    final var numCommittedFallbackVertices =
        committedVertices.stream().filter(v -> v.vertex().isFallback()).count();
    final var numCommittedNonFallbackVertices =
        committedVertices.size() - numCommittedFallbackVertices;

    metrics
        .bft()
        .committedVertices()
        .label(new Metrics.Bft.CommittedVertex(true))
        .inc(numCommittedFallbackVertices);

    metrics
        .bft()
        .committedVertices()
        .label(new Metrics.Bft.CommittedVertex(false))
        .inc(numCommittedNonFallbackVertices);
  }

  public EventProcessor<LedgerExtension> syncEventProcessor() {
    return p -> metrics.ledger().commit().measure(() -> this.commit(p, Option.empty()));
  }

  /**
   * Appends a useful suffix of the given {@link LedgerExtension} to the persistent ledger (while
   * also writing the new serialized {@link VertexStoreState}, if present).
   *
   * <p>A "useful suffix" is a sub-list of transactions that are not yet persisted in the local
   * ledger. The implementation will resolve this sub-list based on the end state version from the
   * {@link LedgerExtension#proof() proof}. If the useful suffix is empty, then this method will
   * no-op and return successfully (with just a trace log, since this kind of situation is most
   * likely a harmless "stale response").
   *
   * <p>This method <b>silently assumes</b> (i.e. does not check) the following facts:
   *
   * <ul>
   *   <li>The proof is signed by a quorum of validators from the current validator set.
   *   <li>The validators' signatures are valid.
   * </ul>
   *
   * <p>This method <b>verifies</b> the following preconditions:
   *
   * <ul>
   *   <li>The useful suffix does not start with the immediate next state version of the local
   *       ledger's end - in case of such "gap", an {@link IllegalArgumentException} is thrown.
   *   <li>Any of the transactions within useful suffix is non-parseable - in this case, a {@link
   *       InvalidCommitRequestException} is propagated from the Rust state computer.
   *   <li>A transaction root hash computed while applying the useful suffix differs from the one
   *       specified by ny of the {@link LedgerExtension#proof() proof} - in this case, a {@link
   *       InvalidCommitRequestException} is propagated from the Rust state computer.
   * </ul>
   */
  private void commit(
      LedgerExtension ledgerExtension, Option<WrappedByteArray> serializedVertexStoreState) {
    final var proofToCommit = ledgerExtension.proof();

    final int extensionTransactionCount; // for metrics purposes only
    synchronized (this.commitAndAdvanceLedgerLock) {
      final var latestStateVersion = this.latestProof.resultantStateVersion();

      if (proofToCommit.stateVersion() <= latestStateVersion) {
        log.trace(
            "Ignoring the ledger extension {} which would not progress the current ledger {}",
            proofToCommit,
            latestProof);
        return;
      }

      final var extensionToCommit = ledgerExtension.getExtensionFrom(latestStateVersion);

      // persist
      this.latestProof = this.stateComputer.commit(extensionToCommit, serializedVertexStoreState);

      extensionTransactionCount = extensionToCommit.transactions().size();
    }

    // TODO: move all of the following to post-persist event handling (while considering the
    // synchronization theoretically needed here).

    this.metrics.ledger().stateVersion().set(this.latestProof.resultantStateVersion());
    if (serializedVertexStoreState.isPresent()) {
      this.metrics.ledger().syncTransactionsProcessed().inc(extensionTransactionCount);
    } else {
      this.metrics.ledger().bftTransactionsProcessed().inc(extensionTransactionCount);
    }
  }
}
