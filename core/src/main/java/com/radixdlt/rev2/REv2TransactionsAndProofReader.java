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

import com.google.inject.Inject;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.DtoLedgerProof;
import com.radixdlt.serialization.DeserializeException;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.RawLedgerTransaction;
import java.util.ArrayList;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class REv2TransactionsAndProofReader implements TransactionsAndProofReader {

  private static final Logger logger = LogManager.getLogger();

  /* Maximum transaction size (in terms of their total byte size) to return in a single getTransactions response.
   * See also MAX_PACKET_LENGTH in PeerChannelInitializer and OVERRIDE_MAX_PAYLOAD_SIZE for transaction size */
  private static final int MAX_TXN_BYTES_FOR_A_SINGLE_RESPONSE = 24_800_000; // 24.8MB

  private final REv2TransactionAndProofStore transactionStore;
  private final Serialization serialization;

  @Inject
  public REv2TransactionsAndProofReader(
      REv2TransactionAndProofStore transactionStore, Serialization serialization) {
    this.transactionStore = transactionStore;
    this.serialization = serialization;
  }

  @Override
  public CommittedTransactionsWithProof getTransactions(DtoLedgerProof start) {
    // TODO - It would likely be more efficient to move this down into the State Manager.
    final var startStateVersion = start.getLedgerHeader().getAccumulatorState().getStateVersion();

    var latestStateVersion = startStateVersion;
    var maybeNextProof = getNextProof(startStateVersion);
    var maybeLatestUsableProof = maybeNextProof;
    var transactionsUpToLatestUsableProof = new ArrayList<RawLedgerTransaction>();
    var totalTxnBytesSoFar = 0;

    if (!maybeNextProof.isPresent()) {
      // No transactions to ledger sync
      return null;
    }

    top_level_while:
    while (maybeNextProof.isPresent()) {
      final var nextProofStateVersion = maybeNextProof.unwrap().getStateVersion();
      // Using a separate list for next proof txns, in case we run out
      // of space mid-proof and need to ignore the txns read so far
      final var transactionsUpToNextProof = new ArrayList<RawLedgerTransaction>();
      for (var i = latestStateVersion + 1; i <= nextProofStateVersion; i++) {
        final var executedTxn = transactionStore.getTransactionAtStateVersion(i).unwrap();
        totalTxnBytesSoFar += executedTxn.transactionBytes().length;
        if (totalTxnBytesSoFar > MAX_TXN_BYTES_FOR_A_SINGLE_RESPONSE) {
          // At this point we know that the additional transactions up to nextProof
          // can't fit. Stop the processing and use the transactions up to latestUsableProof.
          break top_level_while;
        }
        if (maybeNextProof.unwrap().getNextValidatorSet().isPresent()) {
          // New epoch proofs should always be returned
          break top_level_while;
        }
        transactionsUpToNextProof.add(RawLedgerTransaction.create(executedTxn.transactionBytes()));
      }

      // All transactions up to the next proof could fit.
      // Added them to the list and use a new latestUsableProof.
      transactionsUpToLatestUsableProof.addAll(transactionsUpToNextProof);
      maybeLatestUsableProof = maybeNextProof;

      // Continue the loop: try fitting some more txns up to the next proof (if there is any)
      latestStateVersion = nextProofStateVersion;
      maybeNextProof = getNextProof(nextProofStateVersion);
    }

    if (maybeLatestUsableProof.isPresent() && !transactionsUpToLatestUsableProof.isEmpty()) {
      return CommittedTransactionsWithProof.create(
          transactionsUpToLatestUsableProof, maybeLatestUsableProof.unwrap());
    } else {
      // Once we fix block limits, this shouldn't be possible
      logger.warn(
          String.format(
              "Unable to form a ledger sync response from version %s - there are more than %s bytes"
                  + " of transactions between that version and the next proof at %s",
              startStateVersion,
              MAX_TXN_BYTES_FOR_A_SINGLE_RESPONSE,
              maybeNextProof
                  .map(p -> String.format("%s", p.getStateVersion()))
                  .or(() -> "UNKNOWN")));
      return null;
    }
  }

  private Option<LedgerProof> getNextProof(long stateVersion) {
    return this.transactionStore
        .getNextProof(stateVersion)
        .map(
            rawProof -> {
              try {
                return serialization.fromDson(rawProof.last(), LedgerProof.class);
              } catch (DeserializeException e) {
                throw new RuntimeException(e);
              }
            });
  }

  @Override
  public Optional<LedgerProof> getEpochProof(long epoch) {
    return this.transactionStore
        .getEpochProof(epoch)
        .map(
            b -> {
              try {
                return serialization.fromDson(b, LedgerProof.class);
              } catch (DeserializeException e) {
                throw new RuntimeException(e);
              }
            });
  }

  @Override
  public Optional<LedgerProof> getLastProof() {
    return this.transactionStore
        .getLastProof()
        .map(
            b -> {
              try {
                return serialization.fromDson(b, LedgerProof.class);
              } catch (DeserializeException e) {
                throw new RuntimeException(e);
              }
            });
  }
}
