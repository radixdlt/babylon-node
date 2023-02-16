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

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.ledger.StateComputerLedger.ExecutedTransaction;
import com.radixdlt.ledger.StateComputerLedger.StateComputer;
import com.radixdlt.ledger.StateComputerLedger.StateComputerResult;
import com.radixdlt.mempool.Mempool;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.REv2TestTransactions;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.Pair;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.TypedMocks;
import com.radixdlt.utils.UInt256;
import java.util.*;
import org.junit.Before;
import org.junit.Test;

public class StateComputerLedgerTest {

  private Mempool<RawNotarizedTransaction, RawNotarizedTransaction> mempool;
  private StateComputer stateComputer;
  private StateComputerLedger sut;
  private LedgerProof currentLedgerHeader;
  private Metrics metrics;
  private Comparator<LedgerProof> headerComparator;
  private LedgerAccumulator accumulator;
  private LedgerAccumulatorVerifier accumulatorVerifier;

  private LedgerHeader ledgerHeader;
  private VertexWithHash genesisVertex;
  private QuorumCertificate initialEpochQC;

  // Doesn't matter what kind of transaction it is, but needs to be a valid tx payload
  // to be able to convert it from NotarizedTransaction to LedgerTransaction.
  private final RawNotarizedTransaction nextTransaction =
      REv2TestTransactions.constructDepositFromFaucetTransaction(
          NetworkDefinition.LOCAL_SIMULATOR, 0, 0);
  private final Hasher hasher = new Sha256Hasher(DefaultSerialization.getInstance());
  private final ExecutedTransaction successfulNextTransaction =
      nextTransaction::INCORRECTInterpretDirectlyAsRawLedgerTransaction;

  private final long genesisEpoch = 3L;
  private final long genesisStateVersion = 123L;

  @Before
  public void setup() {
    this.mempool = TypedMocks.rmock(Mempool.class);
    // No type check issues with mocking generic here
    this.stateComputer = mock(StateComputer.class);
    this.metrics = new MetricsInitializer().initialize();
    this.headerComparator = TypedMocks.rmock(Comparator.class);

    this.accumulator = new SimpleLedgerAccumulatorAndVerifier(hasher);
    this.accumulatorVerifier = new SimpleLedgerAccumulatorAndVerifier(hasher);

    var accumulatorState = new AccumulatorState(0, HashUtils.zero256());
    this.ledgerHeader = LedgerHeader.genesis(accumulatorState, HashUtils.zero256(), null, 0, 0);
    this.genesisVertex = Vertex.createInitialEpochVertex(ledgerHeader).withId(hasher);
    this.initialEpochQC = QuorumCertificate.createInitialEpochQC(genesisVertex, ledgerHeader);
    this.currentLedgerHeader =
        this.initialEpochQC
            .getCommittedAndLedgerStateProof(hasher)
            .map(Pair::getSecond)
            .orElseThrow();

    this.sut =
        new StateComputerLedger(
            mock(TimeSupplier.class),
            currentLedgerHeader,
            headerComparator,
            stateComputer,
            accumulator,
            accumulatorVerifier,
            metrics);
  }

  public void genesisIsEndOfEpoch(boolean endOfEpoch) {
    this.ledgerHeader =
        LedgerHeader.create(
            genesisEpoch,
            Round.of(5),
            new AccumulatorState(genesisStateVersion, HashUtils.zero256()),
            HashUtils.zero256(),
            12345,
            12345,
            endOfEpoch
                ? NextEpoch.create(
                    genesisEpoch + 1,
                    ImmutableSet.of(BFTValidator.from(BFTValidatorId.random(), UInt256.ONE)))
                : null);
    this.genesisVertex = Vertex.createInitialEpochVertex(ledgerHeader).withId(hasher);
    this.initialEpochQC = QuorumCertificate.createInitialEpochQC(genesisVertex, ledgerHeader);
    this.currentLedgerHeader =
        this.initialEpochQC
            .getCommittedAndLedgerStateProof(hasher)
            .map(Pair::getSecond)
            .orElseThrow();

    this.sut =
        new StateComputerLedger(
            mock(TimeSupplier.class),
            currentLedgerHeader,
            headerComparator,
            stateComputer,
            accumulator,
            accumulatorVerifier,
            metrics);
  }

  @Test
  public void should_not_change_accumulator_when_there_is_no_transaction() {
    // Arrange
    genesisIsEndOfEpoch(false);
    when(stateComputer.prepare(any(), any(), any(), any()))
        .thenReturn(new StateComputerResult(ImmutableList.of(), Map.of(), HashUtils.zero256()));
    var proposedVertex =
        Vertex.create(initialEpochQC, Round.of(1), List.of(), BFTValidatorId.random(), 0L)
            .withId(hasher);

    // Act
    Optional<ExecutedVertex> nextPrepared = sut.prepare(new LinkedList<>(), proposedVertex);

    // Assert
    assertThat(nextPrepared)
        .hasValueSatisfying(x -> assertThat(x.getLedgerHeader().isEndOfEpoch()).isFalse());
    assertThat(nextPrepared)
        .hasValueSatisfying(
            x ->
                assertThat(x.getLedgerHeader().getAccumulatorState())
                    .isEqualTo(ledgerHeader.getAccumulatorState()));
  }

  @Test
  public void should_not_change_header_when_past_end_of_epoch_even_with_transaction() {
    // Arrange
    genesisIsEndOfEpoch(true);
    when(stateComputer.prepare(any(), any(), any(), any()))
        .thenReturn(
            new StateComputerResult(
                ImmutableList.of(successfulNextTransaction), Map.of(), HashUtils.zero256()));
    var proposedVertex =
        Vertex.create(
                initialEpochQC, Round.of(1), List.of(nextTransaction), BFTValidatorId.random(), 0)
            .withId(hasher);

    // Act
    Optional<ExecutedVertex> nextPrepared = sut.prepare(new LinkedList<>(), proposedVertex);

    // Assert
    assertThat(nextPrepared)
        .hasValueSatisfying(x -> assertThat(x.getLedgerHeader().isEndOfEpoch()).isTrue());
    assertThat(nextPrepared)
        .hasValueSatisfying(
            x ->
                assertThat(x.getLedgerHeader().getAccumulatorState())
                    .isEqualTo(ledgerHeader.getAccumulatorState()));
  }

  @Test
  public void should_accumulate_when_next_transaction_valid() {
    // Arrange
    genesisIsEndOfEpoch(false);
    when(stateComputer.prepare(any(), any(), any(), any()))
        .thenReturn(
            new StateComputerResult(
                ImmutableList.of(successfulNextTransaction), Map.of(), HashUtils.zero256()));

    // Act
    var proposedVertex =
        Vertex.create(
                initialEpochQC, Round.of(1), List.of(nextTransaction), BFTValidatorId.random(), 0)
            .withId(hasher);
    Optional<ExecutedVertex> nextPrepared = sut.prepare(new LinkedList<>(), proposedVertex);

    // Assert
    assertThat(nextPrepared)
        .hasValueSatisfying(x -> assertThat(x.getLedgerHeader().isEndOfEpoch()).isFalse());
    assertThat(
            nextPrepared.flatMap(
                x ->
                    accumulatorVerifier.verifyAndGetExtension(
                        ledgerHeader.getAccumulatorState(),
                        List.of(nextTransaction),
                        RawNotarizedTransaction::getPayloadHash,
                        x.getLedgerHeader().getAccumulatorState())))
        .contains(List.of(nextTransaction));
  }

  @Test
  public void should_propagate_state_hash_from_result() {
    // Arrange
    HashCode stateHash = HashUtils.random256();
    when(stateComputer.prepare(any(), any(), any(), any()))
        .thenReturn(
            new StateComputerResult(
                ImmutableList.of(successfulNextTransaction), Map.of(), stateHash));

    // Act
    var proposedVertex =
        Vertex.create(
                initialEpochQC, Round.of(1), List.of(nextTransaction), BFTValidatorId.random(), 0)
            .withId(hasher);
    ExecutedVertex nextPrepared = sut.prepare(new LinkedList<>(), proposedVertex).get();

    // Assert
    assertThat(nextPrepared.getLedgerHeader().getStateHash()).isEqualTo(stateHash);
  }

  @Test
  public void should_do_nothing_if_committing_lower_state_version() {
    // Arrange
    genesisIsEndOfEpoch(false);
    when(stateComputer.prepare(any(), any(), any(), any()))
        .thenReturn(
            new StateComputerResult(
                ImmutableList.of(successfulNextTransaction), Map.of(), HashUtils.zero256()));
    final AccumulatorState accumulatorState =
        new AccumulatorState(genesisStateVersion - 1, HashUtils.zero256());
    final LedgerHeader ledgerHeader =
        LedgerHeader.create(
            genesisEpoch, Round.of(2), accumulatorState, HashUtils.zero256(), 1234, 1234);
    final LedgerProof header =
        new LedgerProof(HashUtils.random256(), ledgerHeader, new TimestampedECDSASignatures());
    var verified =
        CommittedTransactionsWithProof.create(
            List.of(
                RawLedgerTransaction.create(
                    TransactionBuilder.userTransactionToLedgerBytes(nextTransaction.getPayload()))),
            header);

    // Act
    sut.syncEventProcessor().process(verified);

    // Assert
    verify(stateComputer, never()).commit(any(), any());
    verify(mempool, never()).handleTransactionsCommitted(any());
  }
}
