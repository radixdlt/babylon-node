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
import com.google.inject.Inject;
import com.radixdlt.consensus.Ledger;
import com.radixdlt.consensus.LedgerHeader;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.consensus.VertexWithHash;
import com.radixdlt.consensus.bft.BFTCommittedUpdate;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.ExecutedVertex;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.VertexStoreState;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.RemoteEventProcessor;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCounters.CounterType;
import com.radixdlt.store.LastProof;
import com.radixdlt.transactions.Transaction;
import com.radixdlt.utils.TimeSupplier;
import java.util.Comparator;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

/** Synchronizes execution */
public final class StateComputerLedger implements Ledger, ProposalGenerator {

  public interface ExecutedTransaction {
    Transaction transaction();
  }

  public static class StateComputerResult {
    private final List<ExecutedTransaction> executedTransactions;
    private final Map<Transaction, Exception> failedTransactions;
    private final BFTValidatorSet nextValidatorSet;

    public StateComputerResult(
        List<ExecutedTransaction> executedTransactions,
        Map<Transaction, Exception> failedTransactions,
        BFTValidatorSet nextValidatorSet) {
      this.executedTransactions = Objects.requireNonNull(executedTransactions);
      this.failedTransactions = Objects.requireNonNull(failedTransactions);
      this.nextValidatorSet = nextValidatorSet;
    }

    public StateComputerResult(
        List<ExecutedTransaction> executedTransactions,
        Map<Transaction, Exception> failedTransactions) {
      this(executedTransactions, failedTransactions, null);
    }

    public Optional<BFTValidatorSet> getNextValidatorSet() {
      return Optional.ofNullable(nextValidatorSet);
    }

    public List<ExecutedTransaction> getSuccessfullyExecutedTransactions() {
      return executedTransactions;
    }

    public Map<Transaction, Exception> getFailedTransactions() {
      return failedTransactions;
    }
  }

  public interface StateComputer {
    void addToMempool(MempoolAdd mempoolAdd, BFTNode origin);

    List<Transaction> getTransactionsForProposal(List<ExecutedTransaction> executedTransactions);

    StateComputerResult prepare(
        List<ExecutedTransaction> previous, VertexWithHash vertex, long timestamp);

    void commit(TransactionRun transactionRun, VertexStoreState vertexStoreState);
  }

  private final Comparator<LedgerProof> headerComparator;
  private final StateComputer stateComputer;
  private final SystemCounters counters;
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
      SystemCounters counters) {
    this.timeSupplier = Objects.requireNonNull(timeSupplier);
    this.headerComparator = Objects.requireNonNull(headerComparator);
    this.stateComputer = Objects.requireNonNull(stateComputer);
    this.counters = Objects.requireNonNull(counters);
    this.accumulator = Objects.requireNonNull(accumulator);
    this.verifier = Objects.requireNonNull(verifier);
    this.currentLedgerHeader = initialLedgerState;
  }

  public RemoteEventProcessor<MempoolAdd> mempoolAddRemoteEventProcessor() {
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
  public List<Transaction> getTransactionsForProposal(Round round, List<ExecutedVertex> prepared) {
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
      LinkedList<ExecutedVertex> previous, VertexWithHash vertex) {
    final LedgerHeader parentHeader = vertex.getParentHeader().getLedgerHeader();
    final AccumulatorState parentAccumulatorState = parentHeader.getAccumulatorState();
    final ImmutableList<ExecutedTransaction> prevTransactions =
        previous.stream()
            .flatMap(ExecutedVertex::successfulTransactions)
            .collect(ImmutableList.toImmutableList());
    final long quorumTimestamp;
    // if vertex has genesis parent then QC is mocked so just use previous timestamp
    // this does have the edge case of never increasing timestamps if configuration is
    // one round per epoch but good enough for now
    if (vertex.getParentHeader().getRound().isGenesis()) {
      quorumTimestamp = vertex.getParentHeader().getLedgerHeader().timestamp();
    } else {
      quorumTimestamp = vertex.getQC().getTimestampedSignatures().weightedTimestamp();
    }

    synchronized (lock) {
      if (this.currentLedgerHeader.getStateVersion() > parentAccumulatorState.getStateVersion()) {
        return Optional.empty();
      }

      // Don't execute atom if in process of epoch change
      if (parentHeader.isEndOfEpoch()) {
        return Optional.of(
            new ExecutedVertex(
                vertex,
                parentHeader.updateRoundAndTimestamp(vertex.getRound(), quorumTimestamp),
                ImmutableList.of(),
                ImmutableMap.of(),
                timeSupplier.currentTime()));
      }

      final var executedTransactionsOptional =
          this.verifier.verifyAndGetExtension(
              this.currentLedgerHeader.getAccumulatorState(),
              prevTransactions,
              p -> p.transaction().getId().asHashCode(),
              parentAccumulatorState);

      // TODO: Write a test to get here
      // Can possibly get here without maliciousness if parent vertex isn't locked by everyone else
      if (executedTransactionsOptional.isEmpty()) {
        return Optional.empty();
      }

      final var executedTransactions = executedTransactionsOptional.get();

      final StateComputerResult result =
          stateComputer.prepare(executedTransactions, vertex, quorumTimestamp);

      AccumulatorState accumulatorState = parentHeader.getAccumulatorState();
      for (ExecutedTransaction transaction : result.getSuccessfullyExecutedTransactions()) {
        accumulatorState =
            this.accumulator.accumulate(
                accumulatorState, transaction.transaction().getId().asHashCode());
      }

      final LedgerHeader ledgerHeader =
          LedgerHeader.create(
              parentHeader.getEpoch(),
              vertex.getRound(),
              accumulatorState,
              quorumTimestamp,
              result.getNextValidatorSet().orElse(null));

      return Optional.of(
          new ExecutedVertex(
              vertex,
              ledgerHeader,
              result.getSuccessfullyExecutedTransactions(),
              result.getFailedTransactions(),
              timeSupplier.currentTime()));
    }
  }

  public EventProcessor<BFTCommittedUpdate> bftCommittedUpdateEventProcessor() {
    return committedUpdate -> {
      final ImmutableList<Transaction> transactions =
          committedUpdate.committed().stream()
              .flatMap(ExecutedVertex::successfulTransactions)
              .map(ExecutedTransaction::transaction)
              .collect(ImmutableList.toImmutableList());
      var proof = committedUpdate.vertexStoreState().getRootHeader();
      var transactionRun = TransactionRun.create(transactions, proof);

      // TODO: Make these two atomic (RPNV1-827)
      this.commit(transactionRun, committedUpdate.vertexStoreState());
    };
  }

  public EventProcessor<TransactionRun> syncEventProcessor() {
    return p -> this.commit(p, null);
  }

  private void commit(TransactionRun transactionRun, VertexStoreState vertexStoreState) {
    synchronized (lock) {
      final LedgerProof nextHeader = transactionRun.getProof();
      if (headerComparator.compare(nextHeader, this.currentLedgerHeader) <= 0) {
        return;
      }

      var verifiedExtension =
          verifier.verifyAndGetExtension(
              this.currentLedgerHeader.getAccumulatorState(),
              transactionRun.getTransactions(),
              transaction -> transaction.getId().asHashCode(),
              transactionRun.getProof().getAccumulatorState());

      if (verifiedExtension.isEmpty()) {
        throw new ByzantineQuorumException(
            "Accumulator failure " + currentLedgerHeader + " " + transactionRun);
      }

      var transactions = verifiedExtension.get();
      if (vertexStoreState == null) {
        this.counters.add(CounterType.LEDGER_SYNC_TRANSACTIONS_PROCESSED, transactions.size());
      } else {
        this.counters.add(CounterType.LEDGER_BFT_TRANSACTIONS_PROCESSED, transactions.size());
      }

      var extensionToCommit = TransactionRun.create(transactions, transactionRun.getProof());

      // persist
      this.stateComputer.commit(extensionToCommit, vertexStoreState);

      // TODO: move all of the following to post-persist event handling
      this.currentLedgerHeader = nextHeader;
      this.counters.set(
          CounterType.LEDGER_STATE_VERSION, this.currentLedgerHeader.getStateVersion());
    }
  }
}
