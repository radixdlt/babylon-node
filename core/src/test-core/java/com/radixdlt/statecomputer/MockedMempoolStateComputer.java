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

import com.google.inject.Inject;
import com.radixdlt.consensus.LedgerHashes;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.ledger.*;
import com.radixdlt.ledger.StateComputerLedger.StateComputer;
import com.radixdlt.ledger.StateComputerLedger.StateComputerPrepareResult;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolRejectedException;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.targeted.mempool.SimpleMempool;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.List;
import java.util.Set;
import javax.annotation.Nullable;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class MockedMempoolStateComputer implements StateComputer {

  private static final Logger log = LogManager.getLogger();

  private final SimpleMempool mempool;
  private final MockedStateComputer stateComputer;

  @Inject
  public MockedMempoolStateComputer(
      SimpleMempool mempool,
      EventDispatcher<LedgerUpdate> ledgerUpdateDispatcher,
      Hasher hasher,
      LedgerProofBundle latestProofs) {
    this.mempool = mempool;
    this.stateComputer = new MockedStateComputer(ledgerUpdateDispatcher, hasher, latestProofs);
  }

  public void addToMempool(MempoolAdd mempoolAdd, @Nullable NodeId origin) {
    mempoolAdd
        .transactions()
        .forEach(
            txn -> {
              try {
                this.mempool.addTransaction(txn);
              } catch (MempoolRejectedException e) {
                log.error(e);
              }
            });
  }

  @Override
  public List<RawNotarizedTransaction> getTransactionsForProposal(
      List<StateComputerLedger.ExecutedTransaction> executedTransactions) {
    return this.mempool.getTransactionsForProposal(1, Integer.MAX_VALUE, Set.of());
  }

  @Override
  public StateComputerPrepareResult prepare(
      LedgerHashes committedLedgerHashes,
      List<ExecutedVertex> preparedUncommittedVertices,
      LedgerHashes preparedUncommittedLedgerHashes,
      List<RawNotarizedTransaction> proposedTransactions,
      RoundDetails roundDetails) {
    return this.stateComputer.prepare(
        committedLedgerHashes,
        preparedUncommittedVertices,
        preparedUncommittedLedgerHashes,
        proposedTransactions,
        roundDetails);
  }

  @Override
  public LedgerProofBundle commit(LedgerExtension ledgerExtension, VertexStoreState vertexStore) {
    final var proof = this.stateComputer.commit(ledgerExtension, vertexStore);
    this.mempool.handleTransactionsCommitted(
        ledgerExtension.getTransactions().stream()
            // This undoes the (hacky) re-mapping done by a fake `prepare()` using `MockExecuted`
            // (see e.g. `MockedStateComputer` implementation).
            .map(t -> RawNotarizedTransaction.create(t.getPayload()))
            .toList());
    return proof;
  }
}
