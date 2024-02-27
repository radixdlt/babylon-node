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

import com.radixdlt.consensus.BFTConfiguration;
import com.radixdlt.consensus.LedgerHashes;
import com.radixdlt.consensus.NextEpoch;
import com.radixdlt.consensus.ProposalLimitsConfig;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.SelfValidatorInfo;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.ProposerElections;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.*;
import com.radixdlt.mempool.*;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.protocol.RustProtocolUpdate;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statecomputer.commit.*;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Optional;
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
  private final RustProtocolUpdate rustProtocolUpdate;

  private final ProposalLimitsConfig proposalLimitsConfig;
  private final EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher;

  private final EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher;
  private final Serialization serialization;
  private final Hasher hasher;
  private final Metrics metrics;
  private final Optional<ValidatorId> selfValidatorId;
  private final AtomicReference<ProposerElection> currentProposerElection;
  private LedgerProofBundle latestProof;

  public REv2StateComputer(
      RustStateComputer stateComputer,
      RustMempool mempool,
      RustProtocolUpdate rustProtocolUpdate,
      ProposalLimitsConfig proposalLimitsConfig,
      Hasher hasher,
      EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher,
      EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher,
      Serialization serialization,
      ProposerElection initialProposerElection,
      Metrics metrics,
      SelfValidatorInfo selfValidatorInfo,
      LedgerProofBundle initialLatestProof) {
    this.stateComputer = stateComputer;
    this.mempool = mempool;
    this.rustProtocolUpdate = rustProtocolUpdate;
    this.proposalLimitsConfig = proposalLimitsConfig;
    this.hasher = hasher;
    this.ledgerUpdateEventDispatcher = ledgerUpdateEventDispatcher;
    this.mempoolAddSuccessEventDispatcher = mempoolAddSuccessEventDispatcher;
    this.serialization = serialization;
    this.currentProposerElection = new AtomicReference<>(initialProposerElection);
    this.metrics = metrics;
    this.selfValidatorId = selfValidatorInfo.bftValidatorId().map(REv2ToConsensus::validatorId);
    this.latestProof = initialLatestProof;
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
              } catch (MempoolDuplicateException ignored) {
                // Ignore these specific subclass of the `MempoolRejectedException` logged below
              } catch (MempoolRejectedException e) {
                log.debug(e);
              }
            });
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      List<StateComputerLedger.ExecutedTransaction> previousExecutedTransactions) {

    final var previousTransactionHashes =
        previousExecutedTransactions.stream()
            .flatMap(t -> t.notarizedTransactionHash().stream())
            .collect(Collectors.toSet());

    final var rawPreviousExecutedTransactionsSize =
        previousExecutedTransactions.stream()
            .map(tx -> tx.transaction().payloadLength())
            .reduce(0, Integer::sum);

    final var remainingSizeInUncommittedVertices =
        proposalLimitsConfig.maxUncommittedTransactionsPayloadSize()
            - rawPreviousExecutedTransactionsSize;

    final var maxPayloadSize =
        Math.min(
            remainingSizeInUncommittedVertices, proposalLimitsConfig.maxTransactionsPayloadSize());

    metrics.bft().leaderMaxProposalPayloadSize().observe(maxPayloadSize);

    // TODO: Don't include transactions if NextEpoch is to occur
    // TODO: This will require Proposer to simulate a NextRound update before proposing
    final var result =
        maxPayloadSize > 0 && proposalLimitsConfig.maxTransactionCount() > 0
            ? mempool.getTransactionsForProposal(
                proposalLimitsConfig.maxTransactionCount(),
                maxPayloadSize,
                previousTransactionHashes)
            : List.<PreparedNotarizedTransaction>of();

    final var proposedRawTransactions =
        result.stream().map(PreparedNotarizedTransaction::raw).toList();

    final var proposedTotalNotarizedTxnPayloadSize =
        proposedRawTransactions.stream().map(tx -> tx.payload().length).reduce(0, Integer::sum);

    final var totalUncommittedTxnBytesIncludingThisProposal =
        proposedTotalNotarizedTxnPayloadSize + rawPreviousExecutedTransactionsSize;

    metrics.bft().leaderNumTransactionsIncludedInProposal().observe(result.size());
    metrics
        .bft()
        .leaderTransactionBytesIncludedInProposal()
        .observe(proposedTotalNotarizedTxnPayloadSize);
    metrics
        .bft()
        .leaderTransactionBytesIncludedInProposalAndPreviousVertices()
        .observe(totalUncommittedTxnBytesIncludingThisProposal);

    return proposedRawTransactions;
  }

  @Override
  public StateComputerLedger.StateComputerPrepareResult prepare(
      LedgerHashes committedLedgerHashes,
      List<ExecutedVertex> preparedUncommittedVertices,
      LedgerHashes preparedUncommittedLedgerHashes,
      List<RawNotarizedTransaction> proposedTransactions,
      RoundDetails roundDetails) {
    var ancestorTransactions =
        preparedUncommittedVertices.stream()
            .flatMap(
                vertex ->
                    vertex
                        .successfulTransactions()
                        .map(StateComputerLedger.ExecutedTransaction::transaction))
            .toList();
    var gapRoundLeaderAddresses =
        LongStream.range(roundDetails.previousQcRoundNumber() + 1, roundDetails.roundNumber())
            .mapToObj(Round::of)
            .map(this.currentProposerElection.get()::getProposer)
            .map(BFTValidatorId::getValidatorAddress)
            .toList();
    var prepareRequest =
        new PrepareRequest(
            REv2ToConsensus.ledgerHashes(committedLedgerHashes),
            ancestorTransactions,
            REv2ToConsensus.ledgerHashes(preparedUncommittedLedgerHashes),
            proposedTransactions,
            new RoundHistory(
                roundDetails.isFallback(),
                UInt64.fromNonNegativeLong(roundDetails.epoch()),
                UInt64.fromNonNegativeLong(roundDetails.roundNumber()),
                gapRoundLeaderAddresses,
                roundDetails.roundProposer().getValidatorAddress(),
                roundDetails.proposerTimestampMs()));

    var result = stateComputer.prepare(prepareRequest);
    var committableTransactions =
        result.committed().stream()
            .map(REv2ExecutedTransaction::new)
            .map(StateComputerLedger.ExecutedTransaction.class::cast)
            .collect(Collectors.toList());
    var rejectedTransactionsCount = result.rejected().size();

    // TODO - Some time post Babylon mainnet launch we can consider adding a metric to monitor
    // number of rejected transactions by proposer

    final var nextEpoch = result.nextEpoch().map(REv2ToConsensus::nextEpoch).or((NextEpoch) null);
    final var nextProtocolVersion = result.nextProtocolVersion().or((String) null);
    final var ledgerHashes = REv2ToConsensus.ledgerHashes(result.ledgerHashes());
    return new StateComputerLedger.StateComputerPrepareResult(
        committableTransactions,
        rejectedTransactionsCount,
        nextEpoch,
        nextProtocolVersion,
        ledgerHashes);
  }

  @Override
  public LedgerProofBundle commit(
      LedgerExtension ledgerExtension, Option<byte[]> serializedVertexStoreState) {
    final var proof = ledgerExtension.proof();
    final var header = proof.ledgerHeader();

    var commitRequest =
        new CommitRequest(
            ledgerExtension.transactions(),
            proof,
            serializedVertexStoreState,
            Option.from(selfValidatorId));

    final var result = stateComputer.commit(commitRequest);
    final var commitSummary =
        result
            .onError(
                error -> {
                  throw new InvalidCommitRequestException(error);
                })
            .unwrap();

    final var maybeNextEpoch = header.nextEpoch();
    final var maybeNextProtocolVersion = header.nextProtocolVersion();

    if (maybeNextProtocolVersion.isPresent() && maybeNextEpoch.isEmpty()) {
      throw new IllegalStateException("Protocol updates must happen at epoch boundary");
    }

    // Synchronously apply a protocol update while we still hold a StateComputerResult lock
    final var maybePostProtocolUpdateProof =
        maybeNextProtocolVersion.map(
            nextProtocolVersion ->
                this.rustProtocolUpdate.applyProtocolUpdate(nextProtocolVersion).postUpdateProof());

    final var newLatestProof = maybePostProtocolUpdateProof.orElse(proof);

    // This presence of the protocol update in the proof is validated in rust - to ensure that if
    // any protocol update
    // is present, our node agrees it should be committed.
    // We then can trust that we should trigger the application of the protocol update here.
    // NOTE: In the future, we may be able to move this down into Rust.
    this.latestProof =
        new LedgerProofBundle(
            newLatestProof,
            maybeNextEpoch.isPresent() ? proof : this.latestProof.closestEpochProofOnOrBefore(),
            maybeNextProtocolVersion.isPresent()
                ? Option.some(proof)
                : this.latestProof.closestProtocolUpdateInitProofOnOrBefore(),
            maybePostProtocolUpdateProof.isPresent()
                ? Option.some(maybePostProtocolUpdateProof.unwrap())
                : this.latestProof.closestProtocolUpdateExecutionProofOnOrBefore());

    final var maybeEpochChange =
        maybeNextEpoch.map(
            nextEpoch -> {
              final var initialState =
                  VertexStoreState.createNewForNextEpoch(
                      REv2ToConsensus.ledgerHeader(latestProof.epochInitialHeader()),
                      nextEpoch.epoch().toLong(),
                      hasher);
              final var validatorSet = REv2ToConsensus.validatorSet(nextEpoch.validators());
              final var proposerElection =
                  ProposerElections.defaultRotation(nextEpoch.epoch().toLong(), validatorSet);
              final var bftConfiguration =
                  new BFTConfiguration(proposerElection, validatorSet, initialState);
              return new EpochChange(latestProof, bftConfiguration);
            });

    maybeEpochChange.ifPresent(
        epochChange ->
            this.currentProposerElection.set(epochChange.bftConfiguration().getProposerElection()));

    final var protocolState = stateComputer.protocolState();
    final var ledgerUpdate =
        new LedgerUpdate(
            commitSummary,
            latestProof,
            maybeEpochChange,
            protocolState,
            ledgerExtension.transactions());
    ledgerUpdateEventDispatcher.dispatch(ledgerUpdate);

    return latestProof;
  }
}
