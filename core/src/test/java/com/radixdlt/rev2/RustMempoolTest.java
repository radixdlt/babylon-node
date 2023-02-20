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

import static com.radixdlt.rev2.REv2TestTransactions.constructValidRawTransaction;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.consensus.Sha256Hasher;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerAccumulator;
import com.radixdlt.ledger.SimpleLedgerAccumulatorAndVerifier;
import com.radixdlt.mempool.MempoolDuplicateException;
import com.radixdlt.mempool.MempoolFullException;
import com.radixdlt.mempool.RustMempool;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statecomputer.commit.CommitRequest;
import com.radixdlt.statemanager.*;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt64;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Stream;
import org.junit.Assert;
import org.junit.Test;

public final class RustMempoolTest {
  private static CommittedTransactionsWithProof buildGenesis(LedgerAccumulator accumulator) {
    var initialAccumulatorState = new AccumulatorState(0, HashUtils.zero256());
    var genesis =
        TransactionBuilder.createGenesis(
            Map.of(),
            Map.of(),
            UInt64.fromNonNegativeLong(1),
            UInt64.fromNonNegativeLong(10),
            UInt64.fromNonNegativeLong(1));
    var accumulatorState =
        accumulator.accumulate(initialAccumulatorState, genesis.getPayloadHash());
    // The accumulator state is computed correctly, but we cannot easily do the same for state hash
    var stateHash = HashUtils.random256();
    var proof =
        LedgerProof.genesis(accumulatorState, stateHash, BFTValidatorSet.from(Stream.of()), 0, 0);
    return CommittedTransactionsWithProof.create(List.of(genesis), proof);
  }

  private static void initStateComputer(StateManager stateManager) {
    final var metrics = new MetricsInitializer().initialize();
    var stateComputer = new RustStateComputer(metrics, stateManager);
    var accumulator =
        new SimpleLedgerAccumulatorAndVerifier(
            new Sha256Hasher(DefaultSerialization.getInstance()));
    var transactionsWithProof = buildGenesis(accumulator);
    var transactions = transactionsWithProof.getTransactions();
    var proof = transactionsWithProof.getProof();
    stateComputer
        .commit(
            new CommitRequest(
                transactions,
                UInt64.fromNonNegativeLong(proof.getStateVersion()),
                proof.getStateHash(),
                DefaultSerialization.getInstance().toDson(proof, DsonOutput.Output.ALL),
                Option.none()))
        .unwrap();
  }

  @Test
  public void test_rust_mempool_add() throws Exception {
    final var mempoolSize = 2;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(new RustMempoolConfig(mempoolSize)),
            REv2DatabaseConfig.inMemory(),
            LoggingConfig.getDefault());
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = StateManager.createAndInitialize(config)) {
      initStateComputer(stateManager);
      var rustMempool = new RustMempool(metrics, stateManager);
      var transaction1 = constructValidRawTransaction(0, 0);
      var transaction2 = constructValidRawTransaction(0, 1);
      var transaction3 = constructValidRawTransaction(0, 2);

      assertEquals(0, rustMempool.getCount());

      // Add a transaction.
      rustMempool.addTransaction(transaction1);

      assertEquals(1, rustMempool.getCount());

      Assert.assertThrows(
          MempoolDuplicateException.class,
          () -> {
            // Duplicate transaction - this should fail
            rustMempool.addTransaction(transaction1);
          });
      assertEquals(1, rustMempool.getCount());

      // This transaction is new, and the mempool has size 2 - this should be fine, and
      // the mempool will now be full
      rustMempool.addTransaction(transaction2);
      assertEquals(2, rustMempool.getCount());

      try {
        // Mempool is full - adding a new transaction should fail
        rustMempool.addTransaction(transaction3);
        // Because we want to assert properties of the exception, we have to use this weird
        // try/catch approach, instead of assertThrows
        Assert.fail();
      } catch (MempoolFullException ex) {
        assertEquals(2, ex.getMaxSize());
        assertEquals(2, ex.getCurrentSize());
      }
      assertEquals(2, rustMempool.getCount());

      // With a full mempool, a duplicate transaction returns Duplicate, not MempoolFull
      // This is an implementation detail, not mandated behaviour, feel free to change it in future
      Assert.assertThrows(
          MempoolDuplicateException.class,
          () -> {
            // Duplicate transaction - this should fail
            rustMempool.addTransaction(transaction1);
          });
      assertEquals(2, rustMempool.getCount());
    }
  }

  @Test
  public void test_rust_mempool_getTxns() throws Exception {
    final var mempoolSize = 3;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(new RustMempoolConfig(mempoolSize)),
            REv2DatabaseConfig.inMemory(),
            LoggingConfig.getDefault());
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = StateManager.createAndInitialize(config)) {
      initStateComputer(stateManager);
      var rustMempool = new RustMempool(metrics, stateManager);
      var transaction1 = constructValidRawTransaction(0, 0);
      var transaction2 = constructValidRawTransaction(0, 1);
      var transaction3 = constructValidRawTransaction(0, 2);

      // Add Transactions
      rustMempool.addTransaction(transaction1);
      rustMempool.addTransaction(transaction2);
      rustMempool.addTransaction(transaction3);
      assertEquals(3, rustMempool.getCount());

      // Simple Test. Get transactions, and check that are returned.

      // Get zero transactions.
      List<RawNotarizedTransaction> returnedList;
      Set<RawNotarizedTransaction> returnedSet;

      final var unlimitedBytesSize = Integer.MAX_VALUE;
      Assert.assertThrows(
          IllegalArgumentException.class,
          () -> rustMempool.getTransactionsForProposal(-1, unlimitedBytesSize, List.of()));

      Assert.assertThrows(
          IllegalArgumentException.class,
          () -> rustMempool.getTransactionsForProposal(0, unlimitedBytesSize, List.of()));

      // Get one to three transaction.
      returnedList = rustMempool.getTransactionsForProposal(1, unlimitedBytesSize, List.of());
      // Check if it contains 1 element only, either transaction1, transaction2, transaction3
      assertEquals(1, returnedList.size());
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      returnedList = rustMempool.getTransactionsForProposal(2, unlimitedBytesSize, List.of());
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      returnedList = rustMempool.getTransactionsForProposal(3, unlimitedBytesSize, List.of());
      assertEquals(3, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(3, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      // Get transactions, using seen to avoid existing transactions.
      returnedList =
          rustMempool.getTransactionsForProposal(3, unlimitedBytesSize, List.of(transaction1));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction2, transaction3).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(3, unlimitedBytesSize, List.of(transaction2));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction3).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(3, unlimitedBytesSize, List.of(transaction3));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(
              3, unlimitedBytesSize, List.of(transaction1, transaction2, transaction3));
      assertEquals(List.of(), returnedList);

      final var txnPayloadSize = transaction1.getPayload().length;
      // The assertions below assume txns are of equal size; making sure that it holds
      assertEquals(txnPayloadSize, transaction2.getPayload().length);
      assertEquals(txnPayloadSize, transaction3.getPayload().length);

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize, List.of());
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize - 1, List.of());
      assertEquals(0, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 2, List.of());
      assertEquals(2, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 2 - 1, List.of());
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 3, List.of());
      assertEquals(3, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 3 - 1, List.of());
      assertEquals(2, returnedList.size());
    }
  }

  @Test
  public void test_rust_mempool_getRelayTxns() throws Exception {
    final var mempoolSize = 3;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(new RustMempoolConfig(mempoolSize)),
            REv2DatabaseConfig.inMemory(),
            LoggingConfig.getDefault());
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = StateManager.createAndInitialize(config)) {
      initStateComputer(stateManager);

      var rustMempool = new RustMempool(metrics, stateManager);
      var transaction1 = constructValidRawTransaction(0, 0);
      var transaction2 = constructValidRawTransaction(0, 1);
      var transaction3 = constructValidRawTransaction(0, 2);

      rustMempool.addTransaction(transaction1);
      rustMempool.addTransaction(transaction2);
      rustMempool.addTransaction(transaction3);
      assertEquals(3, rustMempool.getCount());

      var returnedList = rustMempool.getTransactionsToRelay(3, Integer.MAX_VALUE);
      assertEquals(3, returnedList.size());
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      final var txnPayloadSize = transaction1.getPayload().length;
      // The assertions below assume txns are of equal size; making sure that it holds
      assertEquals(txnPayloadSize, transaction2.getPayload().length);
      assertEquals(txnPayloadSize, transaction3.getPayload().length);

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize);
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize - 1);
      assertEquals(0, returnedList.size());

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize * 2);
      assertEquals(2, returnedList.size());

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize * 2 - 1);
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize * 3);
      assertEquals(3, returnedList.size());

      returnedList = rustMempool.getTransactionsToRelay(3, txnPayloadSize * 3 - 1);
      assertEquals(2, returnedList.size());
    }
  }
}
