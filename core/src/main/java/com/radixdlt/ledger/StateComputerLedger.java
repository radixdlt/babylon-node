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
import com.google.common.collect.ImmutableMap;
import com.google.common.hash.HashCode;
import com.google.inject.Inject;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.RemoteEventProcessor;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.store.LastProof;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.TimeSupplier;
import java.util.*;

/** Synchronizes execution */
public final class StateComputerLedger implements Ledger, ProposalGenerator {

  public interface ExecutedTransaction {
    RawLedgerTransaction transaction();
  }

  public static class StateComputerResult {
    private final List<ExecutedTransaction> executedTransactions;
    private final Map<RawNotarizedTransaction, Exception> failedTransactions;
    private final NextEpoch nextEpoch;
    private final HashCode stateHash;

    public StateComputerResult(
        List<ExecutedTransaction> executedTransactions,
        Map<RawNotarizedTransaction, Exception> failedTransactions,
        NextEpoch nextEpoch,
        HashCode stateHash) {
      this.executedTransactions = Objects.requireNonNull(executedTransactions);
      this.failedTransactions = Objects.requireNonNull(failedTransactions);
      this.nextEpoch = nextEpoch;
      this.stateHash = stateHash;
    }

    public StateComputerResult(
        List<ExecutedTransaction> executedTransactions,
        Map<RawNotarizedTransaction, Exception> failedTransactions,
        HashCode stateHash) {
      this(executedTransactions, failedTransactions, null, stateHash);
    }

    public Optional<NextEpoch> getNextEpoch() {
      return Optional.ofNullable(nextEpoch);
    }

    public HashCode getStateHash() {
      return stateHash;
    }

    public List<ExecutedTransaction> getSuccessfullyExecutedTransactions() {
      return executedTransactions;
    }

    public Map<RawNotarizedTransaction, Exception> getFailedTransactions() {
      return failedTransactions;
    }
  }

  public interface StateComputer {
    void addToMempool(MempoolAdd mempoolAdd, NodeId origin);

    List<RawNotarizedTransaction> getTransactionsForProposal(
        List<ExecutedTransaction> executedTransactions);

    StateComputerResult prepare(
        HashCode parentAccumulator,
        List<ExecutedVertex> previous,
        List<RawNotarizedTransaction> proposedTransactions,
        RoundDetails roundDetails);

    void commit(
        CommittedTransactionsWithProof committedTransactionsWithProof,
        VertexStoreState vertexStore);
  }

  private final Comparator<LedgerProof> headerComparator;
  private final StateComputer stateComputer;
  private final Metrics metrics;
  private final LedgerAccumulator accumulator;
  private final LedgerAccumulatorVerifier verifier;
  private final Object lock = new Object();
  private final TimeSupplier timeSupplier;

  private LedgerProof currentLedgerHeader;

  @Inject
  public StateComputerLedger(
      TimeSupplier timeSupplier,
      @LastProof LedgerProof initialLedgerState,
      Comparator<LedgerProof> headerComparator,
      StateComputer stateComputer,
      LedgerAccumulator accumulator,
      LedgerAccumulatorVerifier verifier,
      Metrics metrics) {
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.headerComparator = Objects.requireNonNull(headerComparator);
    this.stateComputer = Objects.requireNonNull(stateComputer);
    this.metrics = Objects.requireNonNull(metrics);
    this.accumulator = Objects.requireNonNull(accumulator);
    this.verifier = Objects.requireNonNull(verifier);
    this.currentLedgerHeader = initialLedgerState;
  }

  public RemoteEventProcessor<NodeId, MempoolAdd> mempoolAddRemoteEventProcessor() {
    return (node, mempoolAdd) -> {
      synchronized (lock) {
        stateComputer.addToMempool(mempoolAdd, node);
      }
    };
  }

  public EventProcessor<MempoolAdd> mempoolAddEventProcessor() {
    return mempoolAdd -> {
      synchronized (lock) {
        stateComputer.addToMempool(mempoolAdd, null);
      }
    };
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      Round round, List<ExecutedVertex> prepared) {
    final ImmutableList<ExecutedTransaction> executedTransactions =
        prepared.stream()
            .flatMap(ExecutedVertex::successfulTransactions)
            .collect(ImmutableList.toImmutableList());
    synchronized (lock) {
      return stateComputer.getTransactionsForProposal(executedTransactions);
    }
  }

  @Override
  public Optional<ExecutedVertex> prepare(
      LinkedList<ExecutedVertex> previous, VertexWithHash vertexWithHash) {
    return metrics.ledger().prepare().measure(() -> this.prepareInternal(previous, vertexWithHash));
  }

  private Optional<ExecutedVertex> prepareInternal(
      LinkedList<ExecutedVertex> previous, VertexWithHash vertexWithHash) {
    final var vertex = vertexWithHash.vertex();
    final LedgerHeader parentHeader = vertex.getParentHeader().getLedgerHeader();
    final AccumulatorState parentAccumulatorState = parentHeader.getAccumulatorState();

    synchronized (lock) {
      if (this.currentLedgerHeader.getStateVersion() > parentAccumulatorState.getStateVersion()) {
        return Optional.empty();
      }

      if (parentHeader.isEndOfEpoch()) {
        // Don't execute any transactions and commit to the same LedgerHeader
        // if in the process of an epoch change. Updates to LedgerHeader here
        // may cause a disagreement on the next epoch initial vertex if a TC
        // occurs for example.
        return Optional.of(
            new ExecutedVertex(
                vertexWithHash,
                parentHeader,
                ImmutableList.of(),
                ImmutableMap.of(),
                timeSupplier.currentTime()));
      }

      // It's possible that this function is called with a list of vertices which starts with some
      // committed vertices
      // By matching on the accumulator, we remove the already committed vertices from the
      // "previous" list
      final var committedAccumulatorHash =
          this.currentLedgerHeader.getAccumulatorState().getAccumulatorHash();
      var committedAccumulatorHasMatchedStartOfAVertex =
          committedAccumulatorHash.equals(parentAccumulatorState.getAccumulatorHash());
      var verticesInExtension = new ArrayList<ExecutedVertex>();
      for (var previousVertex : previous) {
        var previousVertexParentAccumulatorHash =
            previousVertex
                .vertex()
                .getParentHeader()
                .getLedgerHeader()
                .getAccumulatorState()
                .getAccumulatorHash();
        if (committedAccumulatorHash.equals(previousVertexParentAccumulatorHash)) {
          committedAccumulatorHasMatchedStartOfAVertex = true;
        }
        if (committedAccumulatorHasMatchedStartOfAVertex) {
          verticesInExtension.add(previousVertex);
        }
      }

      if (!committedAccumulatorHasMatchedStartOfAVertex) {
        // This could trigger if the "previous" vertices don't line up with the committed state.
        // In other words, they don't provide a valid partial path from the committed state to the
        // start of the proposal.
        return Optional.empty();
      }

      // Now we verify the payload hashes of the extension match the start of the proposal
      var extensionMatchesAccumulator =
          this.verifier.verify(
              this.currentLedgerHeader.getAccumulatorState(),
              verticesInExtension.stream()
                  .flatMap(
                      v -> v.successfulTransactions().map(t -> t.transaction().getPayloadHash()))
                  .collect(ImmutableList.toImmutableList()),
              parentAccumulatorState);

      if (!extensionMatchesAccumulator) {
        return Optional.empty();
      }

      final StateComputerResult result =
          stateComputer.prepare(
              committedAccumulatorHash,
              verticesInExtension,
              vertex.getTransactions(),
              RoundDetails.fromVertex(vertexWithHash));

      AccumulatorState accumulatorState = parentHeader.getAccumulatorState();
      for (ExecutedTransaction transaction : result.getSuccessfullyExecutedTransactions()) {
        accumulatorState =
            this.accumulator.accumulate(
                accumulatorState, transaction.transaction().getPayloadHash());
      }

      final LedgerHeader ledgerHeader =
          LedgerHeader.create(
              parentHeader.getEpoch(),
              vertex.getRound(),
              accumulatorState,
              result.getStateHash(),
              vertex.getQCToParent().getWeightedTimestampOfSignatures(),
              vertex.proposerTimestamp(),
              result.getNextEpoch().orElse(null));

      return Optional.of(
          new ExecutedVertex(
              vertexWithHash,
              ledgerHeader,
              result.getSuccessfullyExecutedTransactions(),
              result.getFailedTransactions(),
              timeSupplier.currentTime()));
    }
  }

  public EventProcessor<BFTCommittedUpdate> bftCommittedUpdateEventProcessor() {
    return committedUpdate -> {
      final ImmutableList<RawLedgerTransaction> transactions =
          committedUpdate.committed().stream()
              .flatMap(ExecutedVertex::successfulTransactions)
              .map(ExecutedTransaction::transaction)
              .collect(ImmutableList.toImmutableList());
      var proof = committedUpdate.vertexStoreState().getRootHeader();
      var transactionsWithProof = CommittedTransactionsWithProof.create(transactions, proof);
      metrics
          .ledger()
          .commit()
          .measure(() -> this.commit(transactionsWithProof, committedUpdate.vertexStoreState()));
    };
  }

  public EventProcessor<CommittedTransactionsWithProof> syncEventProcessor() {
    return p -> metrics.ledger().commit().measure(() -> this.commit(p, null));
  }

  private void commit(
      CommittedTransactionsWithProof committedTransactionsWithProof, VertexStoreState vertexStore) {
    synchronized (lock) {
      final LedgerProof nextHeader = committedTransactionsWithProof.getProof();
      if (headerComparator.compare(nextHeader, this.currentLedgerHeader) <= 0) {
        return;
      }

      var verifiedExtension =
          verifier.verifyAndGetExtension(
              this.currentLedgerHeader.getAccumulatorState(),
              committedTransactionsWithProof.getTransactions(),
              RawLedgerTransaction::getPayloadHash,
              committedTransactionsWithProof.getProof().getAccumulatorState());

      if (verifiedExtension.isEmpty()) {
        throw new ByzantineQuorumException(
            "Accumulator failure " + currentLedgerHeader + " " + committedTransactionsWithProof);
      }

      var transactions = verifiedExtension.get();
      if (vertexStore == null) {
        this.metrics.ledger().syncTransactionsProcessed().inc(transactions.size());
      } else {
        this.metrics.ledger().bftTransactionsProcessed().inc(transactions.size());
      }

      var extensionToCommit =
          CommittedTransactionsWithProof.create(
              transactions, committedTransactionsWithProof.getProof());

      // persist
      this.stateComputer.commit(extensionToCommit, vertexStore);

      // TODO: move all of the following to post-persist event handling
      this.currentLedgerHeader = nextHeader;
      this.metrics.ledger().stateVersion().set(this.currentLedgerHeader.getStateVersion());
    }
  }
}
