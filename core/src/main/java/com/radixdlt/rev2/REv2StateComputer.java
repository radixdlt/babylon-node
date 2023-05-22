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

package com.radixdlt.rev2;

import com.google.common.collect.ImmutableClassToInstanceMap;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.BFTConfiguration;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.NextEpoch;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.RoundDetails;
import com.radixdlt.ledger.StateComputerLedger;
import com.radixdlt.mempool.*;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statecomputer.commit.CommitRequest;
import com.radixdlt.statecomputer.commit.PrepareRequest;
import com.radixdlt.statecomputer.commit.PreviousVertex;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.Collectors;
import java.util.stream.LongStream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** REv2 State Computer implementation */
public final class REv2StateComputer implements StateComputerLedger.StateComputer {
  private static final Logger log = LogManager.getLogger();

  private final RustStateComputer stateComputer;

  private final RustMempool mempool;

  // Maximum number of transactions to include in a proposal
  private final int maxNumTransactionsPerProposal;
  // Maximum number of transaction payload bytes to include in a proposal
  private final int maxProposalTotalTxnsPayloadSize;
  // Maximum number of transaction payload bytes to include in a proposal and its previous vertices
  // chain.
  // Effectively limits the size of a commit batch (i.e. the size of transactions under a single
  // proof).
  private final int maxUncommittedUserTransactionsTotalPayloadSize;
  private final EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher;

  private final EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher;
  private final EventDispatcher<ConsensusByzantineEvent> consensusByzantineEventEventDispatcher;
  private final Serialization serialization;
  private final Hasher hasher;
  private final Metrics metrics;
  private final AtomicReference<ProposerElection> proposerElection;

  public REv2StateComputer(
      RustStateComputer stateComputer,
      RustMempool mempool,
      int maxNumTransactionsPerProposal,
      int maxProposalTotalTxnsPayloadSize,
      int maxUncommittedUserTransactionsTotalPayloadSize,
      Hasher hasher,
      EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher,
      EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher,
      EventDispatcher<ConsensusByzantineEvent> consensusByzantineEventEventDispatcher,
      Serialization serialization,
      ProposerElection initialProposerElection,
      Metrics metrics) {
    this.stateComputer = stateComputer;
    this.mempool = mempool;
    this.maxNumTransactionsPerProposal = maxNumTransactionsPerProposal;
    this.maxProposalTotalTxnsPayloadSize = maxProposalTotalTxnsPayloadSize;
    this.maxUncommittedUserTransactionsTotalPayloadSize =
        maxUncommittedUserTransactionsTotalPayloadSize;
    this.hasher = hasher;
    this.ledgerUpdateEventDispatcher = ledgerUpdateEventDispatcher;
    this.mempoolAddSuccessEventDispatcher = mempoolAddSuccessEventDispatcher;
    this.consensusByzantineEventEventDispatcher = consensusByzantineEventEventDispatcher;
    this.serialization = serialization;
    this.proposerElection = new AtomicReference<>(initialProposerElection);
    this.metrics = metrics;
  }

  @Override
  public void addToMempool(MempoolAdd mempoolAdd, NodeId origin) {
    mempoolAdd
        .transactions()
        .forEach(
            transaction -> {
              try {
                mempool.addTransaction(transaction);
                // Please note that a `MempoolAddSuccess` event is only dispatched when the above
                // call does not throw. This is deliberate: we do not want to propagate the
                // transaction to other nodes if it is invalid or a duplicate (to prevent an
                // infinite flood-fill effect across the network).
                var success =
                    MempoolAddSuccess.create(
                        RawNotarizedTransaction.create(transaction.getPayload()), origin);
                mempoolAddSuccessEventDispatcher.dispatch(success);
              } catch (MempoolFullException | MempoolDuplicateException ignored) {
                // Ignore these 2 specific subclasses of the `MempoolRejectedException` logged below
              } catch (MempoolRejectedException e) {
                log.debug(e);
              }
            });
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      List<StateComputerLedger.ExecutedTransaction> previousExecutedTransactions) {
    final var rawPreviousExecutedTransactions =
        previousExecutedTransactions.stream()
            .flatMap(
                executedTx ->
                    TransactionBuilder.convertTransactionBytesToNotarizedTransactionBytes(
                            executedTx.transaction().getPayload())
                        .stream()
                        .map(RawNotarizedTransaction::create))
            .toList();

    final var rawPreviousExecutedTransactionsTotalSize =
        rawPreviousExecutedTransactions.stream()
            .map(tx -> tx.getPayload().length)
            .reduce(0, Integer::sum);
    final var remainingSizeInUncommittedVertices =
        maxUncommittedUserTransactionsTotalPayloadSize - rawPreviousExecutedTransactionsTotalSize;

    final var maxPayloadSize =
        Math.min(remainingSizeInUncommittedVertices, maxProposalTotalTxnsPayloadSize);

    metrics.bft().leaderMaxProposalPayloadSize().observe(maxPayloadSize);

    // TODO: Don't include transactions if NextEpoch is to occur
    // TODO: This will require Proposer to simulate a NextRound update before proposing
    final var result =
        maxPayloadSize > 0 && maxNumTransactionsPerProposal > 0
            ? mempool.getTransactionsForProposal(
                maxNumTransactionsPerProposal, maxPayloadSize, rawPreviousExecutedTransactions)
            : List.<RawNotarizedTransaction>of();

    final var resultTotalTxnPayloadSize =
        result.stream().map(tx -> tx.getPayload().length).reduce(0, Integer::sum);

    final var totalUncommittedTxnBytesIncludingThisProposal =
        resultTotalTxnPayloadSize + rawPreviousExecutedTransactionsTotalSize;

    metrics.bft().leaderNumTransactionsIncludedInProposal().observe(result.size());
    metrics.bft().leaderTransactionBytesIncludedInProposal().observe(resultTotalTxnPayloadSize);
    metrics
        .bft()
        .leaderTransactionBytesIncludedInProposalAndPreviousVertices()
        .observe(totalUncommittedTxnBytesIncludingThisProposal);

    return result;
  }

  @Override
  public StateComputerLedger.StateComputerResult prepare(
      HashCode parentAccumulatorHash,
      List<ExecutedVertex> previousVertices,
      List<RawNotarizedTransaction> proposedTransactions,
      RoundDetails roundDetails) {
    var mappedPreviousVertices =
        previousVertices.stream()
            .map(
                v ->
                    new PreviousVertex(
                        v.successfulTransactions()
                            .map(StateComputerLedger.ExecutedTransaction::transaction)
                            .toList(),
                        v.getLedgerHeader().getAccumulatorState().getAccumulatorHash()))
            .toList();
    var gapRoundLeaderKeys =
        LongStream.range(roundDetails.previousQcRoundNumber() + 1, roundDetails.roundNumber())
            .mapToObj(Round::of)
            .map(this.proposerElection.get()::getProposer)
            .map(BFTValidatorId::getKey)
            .collect(Collectors.toList());
    var prepareRequest =
        new PrepareRequest(
            parentAccumulatorHash,
            mappedPreviousVertices,
            proposedTransactions,
            roundDetails.isFallback(),
            UInt64.fromNonNegativeLong(roundDetails.epoch()),
            UInt64.fromNonNegativeLong(roundDetails.roundNumber()),
            gapRoundLeaderKeys,
            roundDetails.roundProposer().getKey(),
            roundDetails.proposerTimestampMs());

    var result = stateComputer.prepare(prepareRequest);
    var committableTransactions =
        result.committed().stream()
            .map(RawLedgerTransaction::create)
            .map(REv2ExecutedTransaction::new)
            .map(StateComputerLedger.ExecutedTransaction.class::cast)
            .collect(Collectors.toList());
    var rejectedTransactions =
        result.rejected().stream()
            .collect(
                Collectors.toMap(
                    r -> RawNotarizedTransaction.create(r.first()),
                    r -> (Exception) new InvalidREv2Transaction(r.last())));

    rejectedTransactions.forEach(
        (rejected, exception) ->
            consensusByzantineEventEventDispatcher.dispatch(
                new ConsensusByzantineEvent.InvalidProposedTransaction(
                    roundDetails, rejected, exception)));

    var nextEpoch = result.nextEpoch().map(REv2ToConsensus::nextEpoch).or((NextEpoch) null);
    var ledgerHashes = REv2ToConsensus.ledgerHashes(result.ledgerHashes());
    return new StateComputerLedger.StateComputerResult(
        committableTransactions, rejectedTransactions, nextEpoch, ledgerHashes);
  }

  @Override
  public void commit(CommittedTransactionsWithProof txnsAndProof, VertexStoreState vertexStore) {
    var proof = txnsAndProof.getProof();
    final Option<byte[]> vertexStoreBytes;
    if (vertexStore != null) {
      vertexStoreBytes =
          Option.some(serialization.toDson(vertexStore.toSerialized(), DsonOutput.Output.ALL));
    } else {
      vertexStoreBytes = Option.none();
    }

    var commitRequest =
        new CommitRequest(
            txnsAndProof.getTransactions(), REv2ToConsensus.ledgerProof(proof), vertexStoreBytes);

    var result = stateComputer.commit(commitRequest);
    if (result.isError()) {
      log.warn("Could not commit: {}", result.unwrapError());
      return;
    }

    var epochChangeOptional =
        txnsAndProof
            .getProof()
            .getNextEpoch()
            .map(
                nextEpoch -> {
                  var header = txnsAndProof.getProof();
                  final var initialState = VertexStoreState.createNewForNextEpoch(header, hasher);
                  var validatorSet = BFTValidatorSet.from(nextEpoch.getValidators());
                  var proposerElection = new WeightedRotatingLeaders(validatorSet);
                  var bftConfiguration =
                      new BFTConfiguration(proposerElection, validatorSet, initialState);
                  return new EpochChange(header, bftConfiguration);
                });

    var outputBuilder = ImmutableClassToInstanceMap.builder();
    epochChangeOptional.ifPresent(
        epochChange -> {
          this.proposerElection.set(epochChange.getBFTConfiguration().getProposerElection());
          outputBuilder.put(EpochChange.class, epochChange);
        });
    var ledgerUpdate = new LedgerUpdate(txnsAndProof, outputBuilder.build());
    ledgerUpdateEventDispatcher.dispatch(ledgerUpdate);
  }
}
