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

package com.radixdlt.rev1;

import static com.radixdlt.monitoring.SystemCounters.*;
import static com.radixdlt.substate.TxAction.*;

import com.google.common.collect.ImmutableClassToInstanceMap;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.inject.Inject;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.constraintmachine.PermissionLevel;
import com.radixdlt.constraintmachine.REEvent;
import com.radixdlt.constraintmachine.REProcessedTxn;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.engine.PostProcessorException;
import com.radixdlt.engine.RadixEngine;
import com.radixdlt.engine.RadixEngine.RadixEngineBranch;
import com.radixdlt.engine.RadixEngineException;
import com.radixdlt.engine.RadixEngineResult;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.ledger.ByzantineQuorumException;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.StateComputerLedger.ExecutedTransaction;
import com.radixdlt.ledger.StateComputerLedger.StateComputer;
import com.radixdlt.ledger.StateComputerLedger.StateComputerResult;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.mempool.MempoolDuplicateException;
import com.radixdlt.mempool.MempoolRejectedException;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.rev1.forks.Forks;
import com.radixdlt.substate.*;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.function.LongFunction;
import javax.annotation.Nullable;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Wraps the Radix Engine and emits messages based on success or failure */
public final class RadixEngineStateComputer implements StateComputer {
  private static final Logger log = LogManager.getLogger();

  private final RadixEngineMempool mempool;
  private final RadixEngine<LedgerAndBFTProof> radixEngine;
  private final EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher;
  private final EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher;
  private final EventDispatcher<InvalidProposedTransaction>
      invalidProposedTransactionEventDispatcher;
  private final SystemCounters systemCounters;
  private final Hasher hasher;
  private final Forks forks;
  private final Object lock = new Object();

  private ProposerElection proposerElection;
  private long epochMaxRoundNumber;
  private OptionalInt maxSigsPerRound;

  @Inject
  public RadixEngineStateComputer(
      ProposerElection proposerElection, // TODO: Should be able to load this directly from state
      RadixEngine<LedgerAndBFTProof> radixEngine,
      Forks forks,
      RadixEngineMempool mempool, // TODO: Move this into radixEngine
      @EpochMaxRound Round epochMaxRound, // TODO: Move this into radixEngine
      @MaxSigsPerRound OptionalInt maxSigsPerRound, // TODO: Move this into radixEngine
      EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher,
      EventDispatcher<InvalidProposedTransaction> invalidProposedTransactionEventDispatcher,
      EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
      Hasher hasher,
      SystemCounters systemCounters) {
    if (epochMaxRound.isGenesis()) {
      throw new IllegalArgumentException("Epoch change round must not be genesis.");
    }

    this.radixEngine = Objects.requireNonNull(radixEngine);
    this.forks = Objects.requireNonNull(forks);
    this.epochMaxRoundNumber = epochMaxRound.number();
    this.maxSigsPerRound = maxSigsPerRound;
    this.mempool = Objects.requireNonNull(mempool);
    this.mempoolAddSuccessEventDispatcher =
        Objects.requireNonNull(mempoolAddSuccessEventDispatcher);
    this.invalidProposedTransactionEventDispatcher =
        Objects.requireNonNull(invalidProposedTransactionEventDispatcher);
    this.ledgerUpdateDispatcher = Objects.requireNonNull(ledgerUpdateDispatcher);
    this.hasher = Objects.requireNonNull(hasher);
    this.systemCounters = Objects.requireNonNull(systemCounters);
    this.proposerElection = proposerElection;
  }

  public record RadixEngineTransaction(
      RawLedgerTransaction transaction, REProcessedTxn processed, PermissionLevel permissionLevel)
      implements ExecutedTransaction {}

  public REProcessedTxn test(byte[] payload, boolean isSigned) throws RadixEngineException {
    synchronized (lock) {
      var txn =
          isSigned
              ? RawLedgerTransaction.create(payload)
              : TxLowLevelBuilder.newBuilder(payload)
                  .sig(ECDSASecp256k1Signature.zeroSignature())
                  .build();

      var checker = radixEngine.transientBranch();

      try {
        return checker.execute(List.of(txn), !isSigned).getProcessedTxn();
      } finally {
        radixEngine.deleteBranches();
      }
    }
  }

  public REProcessedTxn addToMempool(RawLedgerTransaction transaction)
      throws MempoolRejectedException {
    return addToMempool(transaction, null);
  }

  public REProcessedTxn addToMempool(RawLedgerTransaction transaction, BFTNode origin)
      throws MempoolRejectedException {
    synchronized (lock) {
      try {
        var processed = mempool.addTransaction(transaction);

        systemCounters.increment(CounterType.MEMPOOL_ADD_SUCCESS);
        systemCounters.set(CounterType.MEMPOOL_CURRENT_SIZE, mempool.getCount());

        var success =
            MempoolAddSuccess.create(
                RawNotarizedTransaction.create(transaction.getPayload()), processed, origin);
        mempoolAddSuccessEventDispatcher.dispatch(success);

        return processed;
      } catch (MempoolDuplicateException e) {
        throw e;
      } catch (MempoolRejectedException e) {
        systemCounters.increment(CounterType.MEMPOOL_ADD_FAILURE);
        throw e;
      }
    }
  }

  @Override
  public void addToMempool(MempoolAdd mempoolAdd, @Nullable BFTNode origin) {
    mempoolAdd
        .transactions()
        .forEach(
            txn -> {
              try {
                addToMempool(txn.INCORRECTInterpretDirectlyAsRawLedgerTransaction(), origin);
              } catch (MempoolDuplicateException ex) {
                log.trace(
                    "Transaction {} was not added as it was already in the mempool",
                    txn.getPayloadHash());
              } catch (MempoolRejectedException ex) {
                log.debug("Transaction {} was not added to the mempool", txn.getPayloadHash(), ex);
              }
            });
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      List<ExecutedTransaction> executedTransactions) {
    synchronized (lock) {
      var cmds =
          executedTransactions.stream()
              .map(RadixEngineTransaction.class::cast)
              .map(RadixEngineTransaction::processed)
              .toList();

      // TODO: only return transactions which will not cause a missing dependency error
      return mempool.getTransactionsForProposal(maxSigsPerRound.orElse(50), cmds).stream()
          .map(tx -> RawNotarizedTransaction.create(tx.getPayload()))
          .toList();
    }
  }

  @Override
  public StateComputerResult prepare(
      List<ExecutedTransaction> previousTransactions,
      List<RawNotarizedTransaction> proposedTransactions,
      RoundDetails roundDetails) {
    synchronized (lock) {
      var transientBranch = this.radixEngine.transientBranch();

      reexecutePreviousTransactions(transientBranch, previousTransactions);

      var systemUpdateTransaction = this.executeSystemUpdate(transientBranch, roundDetails);
      var successBuilder = ImmutableList.<ExecutedTransaction>builder();

      successBuilder.add(systemUpdateTransaction);

      var exceptionBuilder = ImmutableMap.<RawLedgerTransaction, Exception>builder();
      var nextValidatorSet =
          systemUpdateTransaction.processed().getEvents().stream()
              .filter(REEvent.NextValidatorSetEvent.class::isInstance)
              .map(REEvent.NextValidatorSetEvent.class::cast)
              .findFirst()
              .map(
                  e ->
                      BFTValidatorSet.from(
                          e.nextValidators().stream()
                              .map(
                                  v ->
                                      BFTValidator.from(
                                          BFTNode.create(v.validatorKey()), v.amount()))));
      // Don't execute and user transactions if changing epochs
      if (nextValidatorSet.isEmpty()) {
        this.executeUserTransactions(
            roundDetails.roundProposer(),
            transientBranch,
            proposedTransactions.stream()
                .map(tx -> RawLedgerTransaction.create(tx.getPayload()))
                .toList(),
            successBuilder,
            exceptionBuilder);
      }
      this.radixEngine.deleteBranches();

      return new StateComputerResult(
          successBuilder.build(), exceptionBuilder.build(), nextValidatorSet.orElse(null));
    }
  }

  private void reexecutePreviousTransactions(
      RadixEngineBranch<LedgerAndBFTProof> transientBranch,
      List<ExecutedTransaction> previousTransactions) {
    for (var transaction : previousTransactions) {
      // TODO: fix this cast with generics. Currently the fix would become a bit too messy
      final var radixEngineTransaction = (RadixEngineTransaction) transaction;
      try {
        transientBranch.execute(
            List.of(radixEngineTransaction.transaction()),
            radixEngineTransaction.permissionLevel());
      } catch (RadixEngineException e) {
        throw new IllegalStateException(
            "Re-execution of already prepared transaction failed: "
                + radixEngineTransaction.processed.getTxn().getPayloadHash(),
            e);
      }
    }
  }

  private RadixEngineTransaction executeSystemUpdate(
      RadixEngineBranch<LedgerAndBFTProof> branch, RoundDetails roundDetails) {
    var systemActions = TxnConstructionRequest.create();

    /*
     * Note that for committing an end-of-epoch, we currently do some tricks.
     *
     * - The consensus rounds actually extend beyond epochMaxRoundNumber
     * - All rounds with roundNumber > epochMaxRoundNumber are filled with empty transactions.
     * - Hopefully we change epoch at roundNumber = epochMaxRoundNumber + 1, but if rounds timeout,
     *   the roundNumber may keep going beyond this.
     * - We ignore round results (commit/timeouts) after the end of the epoch - so these won't factor into (eg)
     *   emissions calculations
     *
     * Essentially, we just wait till we get a chain of consecutive QCs and can have something commit!
     */
    var roundIsDuringEpoch = roundDetails.roundNumber() <= epochMaxRoundNumber;
    if (roundIsDuringEpoch) {
      systemActions.action(
          new NextRound(
              roundDetails.roundNumber(),
              roundDetails.roundWasTimeout(),
              roundDetails.consensusParentRoundTimestamp(),
              getValidatorMapping()));
    } else {
      // We shouldn't record the outcome of rounds beyond the end of the epoch, BUT we do need to
      // ensure we record any timeouts of the "standard" rounds at the end of the epoch.
      var shouldRecordRoundTimeoutsUpToEndOfEpoch =
          roundDetails.previousQcRoundNumber() < epochMaxRoundNumber;
      if (shouldRecordRoundTimeoutsUpToEndOfEpoch) {
        systemActions.action(
            new NextRound(
                epochMaxRoundNumber,
                true,
                roundDetails.consensusParentRoundTimestamp(),
                getValidatorMapping()));
      }
      systemActions.action(new NextEpoch(roundDetails.consensusParentRoundTimestamp()));
    }

    try {
      final var systemUpdate =
          RawLedgerTransaction.create(
              branch.construct(systemActions).buildWithoutSignature().getPayload());
      final var result = branch.execute(List.of(systemUpdate), PermissionLevel.SUPER_USER);
      return new RadixEngineTransaction(
          systemUpdate, result.getProcessedTxn(), PermissionLevel.SUPER_USER);
    } catch (RadixEngineException | TxBuilderException e) {
      throw new IllegalStateException(
          String.format("Failed to execute system updates: %s", systemActions), e);
    }
  }

  private LongFunction<ECDSASecp256k1PublicKey> getValidatorMapping() {
    return l -> proposerElection.getProposer(Round.of(l)).getKey();
  }

  private void executeUserTransactions(
      BFTNode proposer,
      RadixEngineBranch<LedgerAndBFTProof> branch,
      List<RawLedgerTransaction> nextTransactions,
      ImmutableList.Builder<ExecutedTransaction> successBuilder,
      ImmutableMap.Builder<RawLedgerTransaction, Exception> errorBuilder) {
    // TODO: This check should probably be done before getting into state computer
    this.maxSigsPerRound.ifPresent(
        max -> {
          if (nextTransactions.size() > max) {
            log.warn(
                "{} proposing {} txns when limit is {}", proposer, nextTransactions.size(), max);
          }
        });
    var numToProcess =
        Integer.min(nextTransactions.size(), this.maxSigsPerRound.orElse(Integer.MAX_VALUE));
    for (int i = 0; i < numToProcess; i++) {
      var txn = nextTransactions.get(i);
      final RadixEngineResult<LedgerAndBFTProof> result;
      try {
        result = branch.execute(List.of(txn));
      } catch (RadixEngineException e) {
        errorBuilder.put(txn, e);
        invalidProposedTransactionEventDispatcher.dispatch(
            InvalidProposedTransaction.create(proposer.getKey(), txn, e));
        return;
      }

      var radixEngineTransaction =
          new RadixEngineTransaction(txn, result.getProcessedTxn(), PermissionLevel.USER);
      successBuilder.add(radixEngineTransaction);
    }
  }

  private RadixEngineResult<LedgerAndBFTProof> commitInternal(
      CommittedTransactionsWithProof committedTransactionsWithProof,
      VertexStoreState vertexStoreState) {
    var proof = committedTransactionsWithProof.getProof();

    final RadixEngineResult<LedgerAndBFTProof> result;
    try {
      result =
          this.radixEngine.execute(
              committedTransactionsWithProof.getTransactions(),
              LedgerAndBFTProof.create(proof, vertexStoreState),
              PermissionLevel.SUPER_USER);
    } catch (RadixEngineException e) {
      throw new CommittedBadTxnException(committedTransactionsWithProof, e);
    } catch (PostProcessorException e) {
      throw new ByzantineQuorumException(e.getMessage(), e);
    }

    result.getMetadata().getNextForkName().ifPresent(this::forkRadixEngine);

    result
        .getProcessedTxns()
        .forEach(
            t ->
                systemCounters.increment(
                    t.isSystemOnly()
                        ? CounterType.RADIX_ENGINE_SYSTEM_TRANSACTIONS
                        : CounterType.RADIX_ENGINE_USER_TRANSACTIONS));

    return result;
  }

  private void forkRadixEngine(String nextForkName) {
    final var nextForkConfig =
        forks.getByName(nextForkName).orElseThrow(); // guaranteed to be present
    if (log.isInfoEnabled()) {
      log.info("Forking RadixEngine to {}", nextForkConfig.name());
    }
    final var rules = nextForkConfig.engineRules();
    this.radixEngine.replaceConstraintMachine(
        rules.constraintMachineConfig(),
        rules.serialization(),
        rules.actionConstructors(),
        rules.postProcessor(),
        rules.parser());
    this.epochMaxRoundNumber = rules.maxRounds().number();
    this.maxSigsPerRound = rules.maxSigsPerRound();
  }

  @Override
  public void commit(
      CommittedTransactionsWithProof txnsAndProof, VertexStoreState vertexStoreState) {
    synchronized (lock) {
      final var radixEngineResult = commitInternal(txnsAndProof, vertexStoreState);
      final var txCommitted = radixEngineResult.getProcessedTxns();

      // TODO: refactor mempool to be less generic and make this more efficient
      // TODO: Move this into engine
      this.mempool.handleTransactionsCommitted(txCommitted);
      systemCounters.set(CounterType.MEMPOOL_CURRENT_SIZE, mempool.getCount());

      var epochChangeOptional =
          txnsAndProof
              .getProof()
              .getNextValidatorSet()
              .map(
                  validatorSet -> {
                    var header = txnsAndProof.getProof();
                    // TODO: Move vertex stuff somewhere else
                    var genesisVertex = Vertex.createGenesis(header.getHeader()).withId(hasher);
                    var nextLedgerHeader =
                        LedgerHeader.create(
                            header.getNextEpoch(),
                            Round.genesis(),
                            header.getAccumulatorState(),
                            header.consensusParentRoundTimestamp(),
                            header.proposerTimestamp());
                    var genesisQC = QuorumCertificate.ofGenesis(genesisVertex, nextLedgerHeader);
                    final var initialState =
                        VertexStoreState.create(
                            HighQC.from(genesisQC), genesisVertex, Optional.empty(), hasher);
                    var proposerElection = new WeightedRotatingLeaders(validatorSet);
                    var bftConfiguration =
                        new BFTConfiguration(proposerElection, validatorSet, initialState);
                    return new EpochChange(header, bftConfiguration);
                  });
      var outputBuilder = ImmutableClassToInstanceMap.builder();
      epochChangeOptional.ifPresent(
          e -> {
            this.proposerElection = e.getBFTConfiguration().getProposerElection();
            outputBuilder.put(EpochChange.class, e);
          });
      outputBuilder.put(REOutput.class, REOutput.create(txCommitted));
      outputBuilder.put(LedgerAndBFTProof.class, radixEngineResult.getMetadata());
      var ledgerUpdate = new LedgerUpdate(txnsAndProof, outputBuilder.build());
      ledgerUpdateDispatcher.dispatch(ledgerUpdate);
    }
  }
}
