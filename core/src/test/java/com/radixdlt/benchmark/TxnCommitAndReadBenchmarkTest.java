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

package com.radixdlt.benchmark;

import com.google.common.base.Stopwatch;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.StreamTransactionsRequest;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.exception.PrivateKeyException;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.DtoLedgerProof;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.REv2TestTransactions;
import com.radixdlt.rev2.REv2TestTransactions.NotarizedTransactionBuilder;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.rev2.REv2TransactionsAndProofReader;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statecomputer.commit.CommitRequest;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.utils.Longs;
import java.util.ArrayList;
import java.util.List;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.junit.Ignore;
import org.junit.Test;

public final class TxnCommitAndReadBenchmarkTest extends DeterministicCoreApiTestBase {
  private static final Logger log = LogManager.getLogger();

  private static final int NUM_TXNS_IN_A_COMMIT = 50;
  private static final int NUM_COMMITS = 1000;

  @Test
  @Ignore("this test is meant to be run manually")
  public void test_txn_commit_and_read_time() throws Exception {
    try (var test = buildRunningServerTest()) {
      final var stateComputer = test.getInstance(0, RustStateComputer.class);

      final var commitStopwatch = Stopwatch.createStarted();

      long stateVersion = 1L;
      for (int i = 0; i < NUM_COMMITS; i++) {
        /* Using a mocked proof here is quite fragile, and I imagine at some point this may break.
        When that happens, make sure this benchmark is still relevant before spending the time
        fixing the lines below :) */
        final var proof = LedgerProof.mockAtStateVersion(stateVersion + NUM_TXNS_IN_A_COMMIT);
        final var commitRequest =
            new CommitRequest(
                createUniqueTransactions(NUM_TXNS_IN_A_COMMIT, i),
                REv2ToConsensus.ledgerProof(proof),
                Option.none());
        stateComputer.commit(commitRequest);
        stateVersion = proof.getStateVersion();
      }
      log.info(
          "It took {} ms to commit {} txns in {} batches.",
          commitStopwatch.elapsed().toMillis(),
          NUM_COMMITS * NUM_TXNS_IN_A_COMMIT,
          NUM_COMMITS);

      readAllTxnsThroughCoreApi(10000);
      readAllTxnsThroughLedgerSync(test);
    }
  }

  private void readAllTxnsThroughCoreApi(int batchSize) throws Exception {
    final var stopwatch = Stopwatch.createStarted();
    final var api = getStreamApi();
    long totalTxnsRead = 0;
    long stateVersion = 1;
    long totalResponsesReceived = 0;
    while (true) {
      final var resp =
          api.streamTransactionsPost(
              new StreamTransactionsRequest()
                  .network(networkLogicalName)
                  .limit(batchSize)
                  .fromStateVersion(stateVersion));
      totalResponsesReceived += 1;

      if (resp.getTransactions().isEmpty()) {
        break;
      }

      totalTxnsRead += resp.getTransactions().size();

      final var maxReceivedStateVersion =
          resp.getTransactions().get(resp.getTransactions().size() - 1).getStateVersion();
      stateVersion = maxReceivedStateVersion + 1;
    }

    log.info(
        "It took {} ms to read all ({}) txns through the core API. Received {} responses.",
        stopwatch.elapsed().toMillis(),
        totalTxnsRead,
        totalResponsesReceived);
  }

  private void readAllTxnsThroughLedgerSync(DeterministicTest test) {
    final var stopwatch = Stopwatch.createStarted();
    final var txnReader = test.getInstance(0, REv2TransactionsAndProofReader.class);

    long totalTxnsRead = 0;
    long totalTxnReaderCalls = 0;
    DtoLedgerProof proof = LedgerProof.mockAtStateVersion(1L).toDto();
    while (true) {
      var res = txnReader.getTransactions(proof);
      totalTxnReaderCalls += 1;
      if (res == null) {
        break;
      }
      totalTxnsRead += res.getTransactions().size();
      proof = res.getProof().toDto();
    }

    log.info(
        "It took {} ms to read all ({}) txns through the txn reader (ledger sync). Made {} calls to"
            + " txn reader.",
        stopwatch.elapsed().toMillis(),
        totalTxnsRead,
        totalTxnReaderCalls);
  }

  private List<RawLedgerTransaction> createUniqueTransactions(
      int numTransactions, int notaryPrivKeySeed) throws PrivateKeyException, PublicKeyException {
    final List<RawLedgerTransaction> res = new ArrayList<>();
    final byte[] prvBytes = new byte[ECKeyPair.BYTES];
    final byte[] seedBytes = Longs.toByteArray(notaryPrivKeySeed + 1);
    System.arraycopy(seedBytes, 0, prvBytes, prvBytes.length - seedBytes.length, seedBytes.length);
    final var notary = ECKeyPair.fromPrivateKey(prvBytes);
    for (int i = 0; i < numTransactions; i++) {
      final var intentBytes =
          REv2TestTransactions.constructValidIntentBytes(
              NetworkDefinition.INT_TEST_NET, 0, i, notary.getPublicKey().toPublicKey());
      final var rawTransaction =
          new NotarizedTransactionBuilder(intentBytes, notary, List.of()).constructRawTransaction();
      res.add(
          RawLedgerTransaction.create(
              TransactionBuilder.userTransactionToLedgerBytes(rawTransaction.getPayload())));
    }
    return res;
  }
}
