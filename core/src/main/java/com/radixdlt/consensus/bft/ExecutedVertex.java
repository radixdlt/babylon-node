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

package com.radixdlt.consensus.bft;

import com.google.common.hash.HashCode;
import com.radixdlt.consensus.LedgerHeader;
import com.radixdlt.consensus.Vertex;
import com.radixdlt.consensus.VertexWithHash;
import com.radixdlt.ledger.StateComputerLedger.ExecutedTransaction;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.utils.Pair;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.stream.Stream;

/**
 * A Vertex which has been executed by the engine.
 *
 * <p>In particular, a system transaction has been added, and user-transactions have been executed.
 * Some of these may fail, this is captured in transactionsWhichRaisedAnException. The system
 * transaction and user-transactions are captured in executedTransactions.
 *
 * <p>The ledger header captures an overview of the resultant state after ingesting these
 * transactions.
 */
public final class ExecutedVertex {
  private final long timeOfExecution;
  private final VertexWithHash vertexWithHash;

  private final LedgerHeader ledgerHeader;

  private final List<ExecutedTransaction> executedTransactions;
  private final Map<RawLedgerTransaction, Exception> transactionsWhichRaisedAnException;

  public ExecutedVertex(
      VertexWithHash vertexWithHash,
      LedgerHeader ledgerHeader,
      List<ExecutedTransaction> executedTransactions,
      Map<RawLedgerTransaction, Exception> transactionsWhichRaisedAnException,
      long timeOfExecution) {
    this.vertexWithHash = Objects.requireNonNull(vertexWithHash);
    this.ledgerHeader = Objects.requireNonNull(ledgerHeader);
    this.executedTransactions = Objects.requireNonNull(executedTransactions);
    this.transactionsWhichRaisedAnException =
        Objects.requireNonNull(transactionsWhichRaisedAnException);
    this.timeOfExecution = timeOfExecution;
  }

  public Vertex vertex() {
    return this.vertexWithHash.vertex();
  }

  public long getTimeOfExecution() {
    return timeOfExecution;
  }

  public HashCode getVertexHash() {
    return vertexWithHash.hash();
  }

  public HashCode getParentId() {
    return vertex().getParentVertexId();
  }

  public Round getRound() {
    return vertex().getRound();
  }

  public Stream<ExecutedTransaction> successfulTransactions() {
    return executedTransactions.stream();
  }

  public Stream<Pair<RawLedgerTransaction, Exception>> errorTransactions() {
    return transactionsWhichRaisedAnException.entrySet().stream()
        .map(e -> Pair.of(e.getKey(), e.getValue()));
  }

  public Stream<RawLedgerTransaction> getTransactions() {
    return Stream.concat(
        successfulTransactions().map(ExecutedTransaction::transaction),
        errorTransactions().map(Pair::getFirst));
  }

  /**
   * Retrieve the resulting header which is to be persisted on ledger
   *
   * @return the header
   */
  public LedgerHeader getLedgerHeader() {
    return ledgerHeader;
  }

  /**
   * Retrieve the vertex which was executed
   *
   * @return the executed vertex
   */
  public VertexWithHash getVertexWithHash() {
    return vertexWithHash;
  }

  @Override
  public String toString() {
    return String.format(
        "%s{vertex=%s ledgerHeader=%s}",
        this.getClass().getSimpleName(), this.vertex(), this.ledgerHeader);
  }
}
