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

package com.radixdlt.harness.predicates;

import com.google.inject.Injector;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.PacemakerState;
import com.radixdlt.consensus.safety.SafetyRules;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.statecomputer.commit.LedgerProof;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.transaction.CommittedTransactionStatus;
import com.radixdlt.transaction.TransactionPreparer;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.function.Predicate;

public class NodePredicate {
  private NodePredicate() {
    throw new IllegalStateException("Cannot instanitate.");
  }

  public static Predicate<Injector> committedFailedUserTransaction(
      RawNotarizedTransaction userTransaction) {
    var committedTransaction =
        TransactionPreparer.rawNotarizedTransactionToRawLedgerTransaction(userTransaction);
    return i -> {
      var reader = i.getInstance(TestStateReader.class);
      for (long version = 1; true; version++) {
        var maybeTxn = reader.getTransactionAtStateVersion(version);
        if (maybeTxn.isEmpty()) {
          break;
        } else {
          var txn = maybeTxn.unwrap();
          if (txn.rawTransaction().equals(committedTransaction)) {
            return txn.status() instanceof CommittedTransactionStatus.Failure;
          }
        }
      }

      return false;
    };
  }

  /**
   * @param userTransaction - The transaction to lookup
   * @param requireSuccess - Whether the transaction needed to be committed successfully to pass the
   *     predicate
   * @param assertSuccess - Whether a commit of this transaction must be successful (an exception is
   *     thrown if not)
   * @return the predicate
   */
  // TODO - could consider speeding this up by looking at transaction payload hashes instead, and/or
  // moving this to rust
  // Although this test is better as an end-to-end style perhaps
  public static Predicate<Injector> committedUserTransaction(
      RawNotarizedTransaction userTransaction, boolean requireSuccess, boolean assertSuccess) {
    var committedTransaction =
        TransactionPreparer.rawNotarizedTransactionToRawLedgerTransaction(userTransaction);
    return i -> {
      var reader = i.getInstance(TestStateReader.class);
      for (long version = 1; true; version++) {
        var maybeTxn = reader.getTransactionAtStateVersion(version);
        if (maybeTxn.isEmpty()) {
          break;
        } else {
          var txn = maybeTxn.unwrap();
          if (txn.rawTransaction().equals(committedTransaction)) {
            var isSuccess = txn.status() instanceof CommittedTransactionStatus.Success;
            if (assertSuccess && !isSuccess) {
              var errorMessage = txn.errorMessage().or("No error message");
              throw new RuntimeException("Transaction was committed as failure: " + errorMessage);
            }
            if (!requireSuccess || isSuccess) {
              return true;
            }
          }
        }
      }

      return false;
    };
  }

  public static Predicate<Injector> atExactlyStateVersion(long stateVersion) {
    return proofCommitted(p -> p.stateVersion() == stateVersion);
  }

  public static Predicate<Injector> atOrOverStateVersion(long stateVersion) {
    return proofCommitted(p -> p.stateVersion() >= stateVersion);
  }

  public static Predicate<Injector> proofCommitted(Predicate<LedgerProof> predicate) {
    return i ->
        i.getInstance(TransactionsAndProofReader.class)
            .getLatestProofBundle()
            .map(proofs -> predicate.test(proofs.primaryProof()))
            .orElse(false);
  }

  public static Predicate<Injector> atOrOverEpoch(long epoch) {
    return i -> i.getInstance(TestStateReader.class).getEpoch() >= epoch;
  }

  public static Predicate<Injector> bftAtOrOverRound(Round round) {
    return i -> i.getInstance(PacemakerState.class).highQC().getHighestRound().gte(round);
  }

  public static Predicate<Injector> atOrOverRound(Round round) {
    return metricsPredicate(metrics -> metrics.bft().pacemaker().round().get() >= round.number());
  }

  public static Predicate<Injector> metricsPredicate(Predicate<Metrics> predicate) {
    return i -> predicate.test(i.getInstance(Metrics.class));
  }

  public static Predicate<Injector> votedAtRound(Round round) {
    return i -> i.getInstance(SafetyRules.class).getLastVote(round).isPresent();
  }
}
