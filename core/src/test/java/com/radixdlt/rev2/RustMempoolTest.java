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

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import com.radixdlt.consensus.Blake2b256Hasher;
import com.radixdlt.environment.*;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.RawGenesisDataWithHash;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.*;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.Assert;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class RustMempoolTest {

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  /** A no-op dispatcher of transactions to be relayed. */
  private static final MempoolRelayDispatcher<RawNotarizedTransaction> NOOP_DISPATCHER = tx -> {};

  /**
   * A no-op fatal panic handler. Please note that a JNI-invoking test (like this one) will observe
   * panics as runtime exceptions propagated up the stack (through JNI), which will fail the test
   * gracefully anyway.
   */
  private static final FatalPanicHandler NOOP_HANDLER = () -> {};

  private static void initStateComputer(NodeRustEnvironment nodeRustEnvironment) {
    final var metrics = new MetricsInitializer().initialize();
    final var genesisProvider =
        RawGenesisDataWithHash.fromGenesisData(GenesisData.testingDefaultEmpty());
    new REv2LedgerInitializer(
            new Blake2b256Hasher(DefaultSerialization.getInstance()),
            new RustStateComputer(metrics, nodeRustEnvironment),
            new REv2TransactionsAndProofReader(
                new REv2TransactionAndProofStore(metrics, nodeRustEnvironment),
                LedgerSyncLimitsConfig.defaults()))
        .initialize(genesisProvider);
  }

  @Test
  public void test_rust_mempool_add() throws Exception {
    final var mempoolMaxTotalTransactionsSize = 10 * 1024 * 1024;
    final var mempoolMaxTransactionCount = 20;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(
                new RustMempoolConfig(mempoolMaxTotalTransactionsSize, mempoolMaxTransactionCount)),
            Option.none(),
            new DatabaseBackendConfig(folder.newFolder().getPath()),
            new DatabaseFlags(false, false),
            LoggingConfig.getDefault(),
            StateHashTreeGcConfig.forTesting(),
            LedgerProofsGcConfig.forTesting(),
            LedgerSyncLimitsConfig.defaults(),
            ProtocolConfig.testingDefault(),
            false);
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = new NodeRustEnvironment(NOOP_DISPATCHER, NOOP_HANDLER, config)) {
      initStateComputer(stateManager);
      final var rustMempool = new RustMempool(metrics, stateManager);
      final var transaction1 = constructValidTransaction(0, 0);
      final var transaction2 = constructValidTransaction(0, 1);

      assertEquals(0, rustMempool.getCount());

      // Add a transaction.
      rustMempool.addTransaction(transaction1.raw());

      assertEquals(1, rustMempool.getCount());

      Assert.assertThrows(
          MempoolDuplicateException.class,
          () -> {
            // Duplicate transaction - this should fail
            rustMempool.addTransaction(transaction1.raw());
          });
      assertEquals(1, rustMempool.getCount());

      // This transaction is new, and the mempool has size 2 - this should be fine, and
      // the mempool will now be full
      rustMempool.addTransaction(transaction2.raw());
      assertEquals(2, rustMempool.getCount());

      // With a full mempool, a duplicate transaction returns Duplicate, not MempoolFull
      // This is an implementation detail, not mandated behaviour, feel free to change it in future
      Assert.assertThrows(
          MempoolDuplicateException.class,
          () -> {
            // Duplicate transaction - this should fail
            rustMempool.addTransaction(transaction1.raw());
          });
      assertEquals(2, rustMempool.getCount());
    }
  }

  @Test
  public void test_rust_mempool_getTxns() throws Exception {
    final var mempoolMaxTotalTransactionsSize = 10 * 1024 * 1024;
    final var mempoolMaxTransactionCount = 20;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(
                new RustMempoolConfig(mempoolMaxTotalTransactionsSize, mempoolMaxTransactionCount)),
            Option.none(),
            new DatabaseBackendConfig(folder.newFolder().getPath()),
            new DatabaseFlags(false, false),
            LoggingConfig.getDefault(),
            StateHashTreeGcConfig.forTesting(),
            LedgerProofsGcConfig.forTesting(),
            LedgerSyncLimitsConfig.defaults(),
            ProtocolConfig.testingDefault(),
            false);
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = new NodeRustEnvironment(NOOP_DISPATCHER, NOOP_HANDLER, config)) {
      initStateComputer(stateManager);
      final var rustMempool = new RustMempool(metrics, stateManager);
      final var transaction1 = constructValidTransaction(0, 0);
      final var transaction2 = constructValidTransaction(0, 1);
      final var transaction3 = constructValidTransaction(0, 2);

      // Add Transactions
      rustMempool.addTransaction(transaction1.raw());
      rustMempool.addTransaction(transaction2.raw());
      rustMempool.addTransaction(transaction3.raw());
      assertEquals(3, rustMempool.getCount());

      // Simple Test. Get transactions, and check that are returned.

      // Get zero transactions.
      List<PreparedNotarizedTransaction> returnedList;
      Set<PreparedNotarizedTransaction> returnedSet;

      final var unlimitedBytesSize = Integer.MAX_VALUE;
      Assert.assertThrows(
          IllegalArgumentException.class,
          () -> rustMempool.getTransactionsForProposal(-1, unlimitedBytesSize, Set.of()));

      Assert.assertThrows(
          IllegalArgumentException.class,
          () -> rustMempool.getTransactionsForProposal(0, unlimitedBytesSize, Set.of()));

      // Get one to three transaction.
      returnedList = rustMempool.getTransactionsForProposal(1, unlimitedBytesSize, Set.of());
      // Check if it contains 1 element only, either transaction1, transaction2, transaction3
      assertEquals(1, returnedList.size());
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      returnedList = rustMempool.getTransactionsForProposal(2, unlimitedBytesSize, Set.of());
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      returnedList = rustMempool.getTransactionsForProposal(3, unlimitedBytesSize, Set.of());
      assertEquals(3, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(3, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      // Get transactions, using seen to avoid existing transactions.
      returnedList =
          rustMempool.getTransactionsForProposal(
              3, unlimitedBytesSize, Set.of(transaction1.notarizedTransactionHash()));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction2, transaction3).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(
              3, unlimitedBytesSize, Set.of(transaction2.notarizedTransactionHash()));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction3).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(
              3, unlimitedBytesSize, Set.of(transaction3.notarizedTransactionHash()));
      assertEquals(2, returnedList.size());
      // Transform it into a set to avoid duplicates.
      returnedSet = new HashSet<>(returnedList);
      // Check no duplicates
      assertEquals(2, returnedSet.size());
      // Check that elements are our expected transactions
      assertTrue(List.of(transaction1, transaction2).containsAll(returnedList));

      returnedList =
          rustMempool.getTransactionsForProposal(
              3,
              unlimitedBytesSize,
              Set.of(
                  transaction1.notarizedTransactionHash(),
                  transaction2.notarizedTransactionHash(),
                  transaction3.notarizedTransactionHash()));
      assertEquals(List.of(), returnedList);

      final var txnPayloadSize = transaction1.raw().payload().length;
      // The assertions below assume txns are of equal size; making sure that it holds
      assertEquals(txnPayloadSize, transaction2.raw().payload().length);
      assertEquals(txnPayloadSize, transaction3.raw().payload().length);

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize, Set.of());
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize - 1, Set.of());
      assertEquals(0, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 2, Set.of());
      assertEquals(2, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 2 - 1, Set.of());
      assertEquals(1, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 3, Set.of());
      assertEquals(3, returnedList.size());

      returnedList = rustMempool.getTransactionsForProposal(3, txnPayloadSize * 3 - 1, Set.of());
      assertEquals(2, returnedList.size());
    }
  }

  @Test
  public void test_rust_mempool_getRelayTxns() throws Exception {
    final var mempoolMaxTotalTransactionsSize = 10 * 1024 * 1024;
    final var mempoolMaxTransactionCount = 20;
    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(
                new RustMempoolConfig(mempoolMaxTotalTransactionsSize, mempoolMaxTransactionCount)),
            Option.none(),
            new DatabaseBackendConfig(folder.newFolder().getPath()),
            new DatabaseFlags(false, false),
            LoggingConfig.getDefault(),
            StateHashTreeGcConfig.forTesting(),
            LedgerProofsGcConfig.forTesting(),
            LedgerSyncLimitsConfig.defaults(),
            ProtocolConfig.testingDefault(),
            false);
    final var metrics = new MetricsInitializer().initialize();

    try (var stateManager = new NodeRustEnvironment(NOOP_DISPATCHER, NOOP_HANDLER, config)) {
      initStateComputer(stateManager);
      final var rustMempool = new RustMempool(metrics, stateManager);
      final var transaction1 = constructValidTransaction(0, 0);
      final var transaction2 = constructValidTransaction(0, 1);
      final var transaction3 = constructValidTransaction(0, 2);

      rustMempool.addTransaction(transaction1.raw());
      rustMempool.addTransaction(transaction2.raw());
      rustMempool.addTransaction(transaction3.raw());
      assertEquals(3, rustMempool.getCount());

      var returnedList = rustMempool.getTransactionsToRelay(3, Integer.MAX_VALUE);
      assertEquals(3, returnedList.size());
      assertTrue(List.of(transaction1, transaction2, transaction3).containsAll(returnedList));

      final var txnPayloadSize = transaction1.raw().payload().length;
      // The assertions below assume txns are of equal size; making sure that it holds
      assertEquals(txnPayloadSize, transaction2.raw().payload().length);
      assertEquals(txnPayloadSize, transaction3.raw().payload().length);

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

  private static PreparedNotarizedTransaction constructValidTransaction(long fromEpoch, int nonce) {
    return TransactionBuilder.forTests().fromEpoch(fromEpoch).nonce(nonce).prepare();
  }
}
