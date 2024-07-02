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

package com.radixdlt.statecomputer;

import com.google.common.collect.ImmutableList;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.consensus.liveness.ProposerElections;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.*;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.statecomputer.commit.CommitSummary;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.WrappedByteArray;
import java.util.ArrayList;
import java.util.List;

/**
 * Ledger Computer for testing which stores no state but only statelessly verifies whether a
 * transaction is correct or not.
 */
public final class StatelessComputer implements StateComputerLedger.StateComputer {
  private final StatelessTransactionVerifier verifier;
  private final EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher;
  private final Hasher hasher;
  private final LedgerHashes fixedLedgerHashes;
  private int successCount = 0;
  private int invalidCount = 0;

  public StatelessComputer(
      StatelessTransactionVerifier verifier,
      EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
      Hasher hasher) {
    this(verifier, ledgerUpdateDispatcher, hasher, LedgerHashes.zero());
  }

  public StatelessComputer(
      StatelessTransactionVerifier verifier,
      EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
      Hasher hasher,
      LedgerHashes fixedLedgerHashes) {
    this.verifier = verifier;
    this.ledgerUpdateDispatcher = ledgerUpdateDispatcher;
    this.hasher = hasher;
    this.fixedLedgerHashes = fixedLedgerHashes;
  }

  public int getSuccessCount() {
    return successCount;
  }

  public int getInvalidCount() {
    return invalidCount;
  }

  @Override
  public void addToMempool(MempoolAdd mempoolAdd, NodeId origin) {}

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      List<StateComputerLedger.ExecutedTransaction> executedTransactions) {
    return List.of();
  }

  @Override
  public StateComputerLedger.StateComputerPrepareResult prepare(
      LedgerHashes committedLedgerHashes,
      List<ExecutedVertex> preparedUncommittedVertices,
      LedgerHashes preparedUncommittedLedgerHashes,
      List<RawNotarizedTransaction> proposedTransactions,
      RoundDetails roundDetails) {
    var successfulTransactions = new ArrayList<StateComputerLedger.ExecutedTransaction>();

    var invalidTransactionCount = 0;
    for (var transaction : proposedTransactions) {
      var success = verifier.verify(transaction);
      if (success) {
        successfulTransactions.add(new MockExecuted(transaction));
      } else {
        invalidTransactionCount++;
      }
    }

    successCount += successfulTransactions.size();
    invalidCount += invalidTransactionCount;

    return new StateComputerLedger.StateComputerPrepareResult(
        successfulTransactions, invalidTransactionCount, fixedLedgerHashes);
  }

  private LedgerUpdate generateLedgerUpdate(LedgerExtension ledgerExtension) {
    // `closestEpochProofOnOrBefore` isn't really correct here, but that's fine
    final var latestProof =
        new LedgerProofBundle(
            ledgerExtension.proof(), ledgerExtension.proof(), Option.none(), Option.none());
    final var maybeEpochChange =
        ledgerExtension
            .proof()
            .ledgerHeader()
            .nextEpoch()
            .map(
                nextEpoch -> {
                  final var initialHeader =
                      REv2ToConsensus.ledgerHeader(latestProof.epochInitialHeader());
                  VertexWithHash genesisVertex =
                      Vertex.createInitialEpochVertex(initialHeader).withId(hasher);
                  LedgerHeader nextLedgerHeader =
                      LedgerHeader.create(
                          nextEpoch.epoch().toLong(),
                          Round.epochInitial(),
                          initialHeader.getStateVersion(),
                          initialHeader.getHashes(),
                          initialHeader.consensusParentRoundTimestamp(),
                          initialHeader.proposerTimestamp());
                  QuorumCertificate initialEpochQC =
                      QuorumCertificate.createInitialEpochQC(genesisVertex, nextLedgerHeader);
                  final var initialState =
                      VertexStoreState.create(
                          HighQC.ofInitialEpochQc(initialEpochQC), genesisVertex, hasher);
                  var validatorSet = REv2ToConsensus.validatorSet(nextEpoch.validators());
                  var proposerElection =
                      ProposerElections.defaultRotation(nextEpoch.epoch().toLong(), validatorSet);
                  var bftConfiguration =
                      new BFTConfiguration(proposerElection, validatorSet, initialState);
                  return new EpochChange(latestProof, bftConfiguration);
                });

    return new LedgerUpdate(
        new CommitSummary(ImmutableList.of(), UInt32.fromNonNegativeInt(0)),
        latestProof,
        maybeEpochChange,
        ProtocolState.testingEmpty(),
        ledgerExtension.transactions());
  }

  @Override
  public LedgerProofBundle commit(
      LedgerExtension ledgerExtension, Option<WrappedByteArray> serializedVertexStoreState) {
    var ledgerUpdate = this.generateLedgerUpdate(ledgerExtension);
    ledgerUpdateDispatcher.dispatch(ledgerUpdate);
    return ledgerUpdate.committedProof();
  }
}
