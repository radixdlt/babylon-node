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

package com.radixdlt.api.core;

import static com.radixdlt.harness.predicates.NodesPredicate.nodeAt;
import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.hash.HashCode;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.lang.Option;
import com.radixdlt.statecomputer.commit.LedgerHeader;
import com.radixdlt.transaction.ExecutedTransaction;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import java.util.List;
import java.util.stream.LongStream;
import org.junit.Test;

public class TransactionAccuTreeTest extends DeterministicCoreApiTestBase {

  private static final int EPOCH_TRANSACTION_LENGTH = 4;

  @Test
  public void stateManagerMaintainsCorrectTransactionMerkleTree() {
    // Run and capture an example epoch
    CapturedEpoch epoch = captureEpoch(2);

    // Compute the transaction hashes
    var transactionHashes =
        epoch.transactions().stream()
            .map(transaction -> HashUtils.blake2b256(transaction.transactionBytes()))
            .toArray(HashCode[]::new);

    // Assert that header's root hash is equal to manually computed one
    assertThat(epoch.header().hashes().transactionRoot())
        .isEqualTo(
            merkle(
                merkle(
                    // previous epoch's root hash as first leaf
                    merkle(epoch.previousHeader().hashes().transactionRoot(), transactionHashes[0]),
                    merkle(transactionHashes[1], transactionHashes[2])),
                merkle(
                    merkle(transactionHashes[3], HashUtils.zero256()), // placeholder hash at leaf
                    HashUtils.zero256()))); // placeholder hash at internal node
  }

  @Test
  public void stateManagerMaintainsCorrectReceiptMerkleTree() {
    // Run and capture an example epoch
    CapturedEpoch epoch = captureEpoch(2);

    // Compute the ledger receipt hashes
    var receiptHashes =
        epoch.transactions().stream()
            .map(transaction -> HashUtils.blake2b256(transaction.consensusReceiptBytes()))
            .toArray(HashCode[]::new);

    // Assert that header's root hash is equal to manually computed one
    assertThat(epoch.header().hashes().receiptRoot())
        .isEqualTo(
            merkle(
                merkle(
                    // previous epoch's root hash as first leaf
                    merkle(epoch.previousHeader().hashes().receiptRoot(), receiptHashes[0]),
                    merkle(receiptHashes[1], receiptHashes[2])),
                merkle(
                    merkle(receiptHashes[3], HashUtils.zero256()), // placeholder hash at leaf
                    HashUtils.zero256()))); // placeholder hash at internal node
  }

  private CapturedEpoch captureEpoch(int epochNumber) {
    try (var test = buildRunningServerTest(EPOCH_TRANSACTION_LENGTH)) {
      test.suppressUnusedWarning();

      // Run the setup until 2 epoch proofs are captured
      var reader = test.getInstance(0, REv2TransactionAndProofStore.class);
      test.runUntilState(nodeAt(0, NodePredicate.atOrOverEpoch(epochNumber)), 1000);
      var previousHeader = reader.getEpochProof(epochNumber - 1).get().ledgerHeader();
      var epochHeader = reader.getEpochProof(epochNumber).get().ledgerHeader();

      // Capture the transactions between these 2 proofs
      var epochTransactions =
          LongStream.range(
                  previousHeader.stateVersion().toNonNegativeLong().unwrap() + 1,
                  epochHeader.stateVersion().toNonNegativeLong().unwrap() + 1)
              .mapToObj(reader::getTransactionAtStateVersion)
              .map(Option::unwrap)
              .toList();

      // Assert a certain count (on which we rely during latter manual merkle computation)
      assertThat(epochTransactions.size()).isEqualTo(EPOCH_TRANSACTION_LENGTH);

      return new CapturedEpoch(previousHeader, epochHeader, epochTransactions);
    }
  }

  private static HashCode merkle(HashCode left, HashCode right) {
    var leftBytes = left.asBytes();
    var rightBytes = right.asBytes();
    byte[] concat = new byte[leftBytes.length + rightBytes.length];
    System.arraycopy(leftBytes, 0, concat, 0, leftBytes.length);
    System.arraycopy(rightBytes, 0, concat, leftBytes.length, rightBytes.length);
    return HashUtils.blake2b256(concat);
  }

  record CapturedEpoch(
      LedgerHeader previousHeader, LedgerHeader header, List<ExecutedTransaction> transactions) {}
}
