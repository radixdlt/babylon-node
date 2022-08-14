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

import com.google.common.reflect.TypeToken;
import com.radixdlt.exceptions.StateManagerRuntimeError;
import com.radixdlt.lang.Result;
import com.radixdlt.mempool.MempoolInserter;
import com.radixdlt.mempool.MempoolRelayReader;
import com.radixdlt.mempool.RustMempool;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.sbor.StateManagerSbor;
import com.radixdlt.statecomputer.preview.PreviewError;
import com.radixdlt.statecomputer.preview.PreviewRequest;
import com.radixdlt.statecomputer.preview.PreviewResult;
import com.radixdlt.statemanager.StateManager;
import com.radixdlt.statemanager.StateManagerResponse;
import com.radixdlt.transaction.RustTransactionStore;
import com.radixdlt.transaction.TransactionStoreReader;
import com.radixdlt.transactions.RawTransaction;
import java.math.BigInteger;
import java.util.List;
import java.util.Objects;
import java.util.function.BiFunction;
import org.bouncycastle.util.Arrays;

public class RustStateComputer {
  private final StateManager.RustState rustState;
  private final RustMempool mempool;
  private final RustTransactionStore transactionStore;

  public RustStateComputer(StateManager.RustState rustState) {
    this.rustState = Objects.requireNonNull(rustState);
    this.mempool = new RustMempool(rustState);
    this.transactionStore = new RustTransactionStore(rustState);
  }

  private static final TypeToken<Result<byte[], StateManagerRuntimeError>> byteArrayType =
      new TypeToken<>() {};

  public TransactionStoreReader getTransactionStoreReader() {
    return this.transactionStore;
  }

  public MempoolRelayReader getMempoolRelayReader() {
    return this.mempool::getTransactionsToRelay;
  }

  public MempoolInserter<RawTransaction> getMempoolInserter() {
    return this.mempool::addTransaction;
  }

  public List<RawTransaction> getTransactionsForProposal(
      int count, List<RawTransaction> transactionToExclude) {
    return this.mempool.getTransactionsForProposal(count, transactionToExclude);
  }

  public BigInteger getComponentXrdAmount(ComponentAddress componentAddress) {
    final var encodedRequest =
        StateManagerSbor.sbor.encode(componentAddress, ComponentAddress.class);
    var encodedResponse = componentXrdAmount(this.rustState, encodedRequest);
    var amount = StateManagerResponse.decode(encodedResponse, byteArrayType);
    return new BigInteger(1, Arrays.reverse(amount));
  }

  public void commit(List<RawTransaction> transactions, long committedStateVersion) {
    var encodedTransactions =
        StateManagerSbor.sbor.encode(transactions, new TypeToken<List<RawTransaction>>() {});
    commit(this.rustState, encodedTransactions, committedStateVersion);
  }

  public boolean verify(RawTransaction transaction) {
    return callNativeFn(
        transaction, RawTransaction.class, new TypeToken<>() {}, RustStateComputer::verify);
  }

  public Result<PreviewResult, PreviewError> preview(PreviewRequest previewRequest) {
    return callNativeFn(
        previewRequest, PreviewRequest.class, new TypeToken<>() {}, RustStateComputer::preview);
  }

  private <Req, Res> Res callNativeFn(
      Req request,
      Class<Req> requestClass,
      TypeToken<Result<Res, StateManagerRuntimeError>> resultTypeToken,
      BiFunction<StateManager.RustState, byte[], byte[]> nativeFn) {
    final var encodedRequest = StateManagerSbor.sbor.encode(request, requestClass);
    final var encodedResponse = nativeFn.apply(this.rustState, encodedRequest);
    return StateManagerResponse.decode(encodedResponse, resultTypeToken);
  }

  private static native byte[] verify(StateManager.RustState rustState, byte[] encodedArgs);

  private static native byte[] preview(StateManager.RustState rustState, byte[] encodedArgs);

  private static native byte[] commit(
      StateManager.RustState rustState, byte[] encodedTransaction, long committedStateVersion);

  private static native byte[] componentXrdAmount(
      StateManager.RustState rustState, byte[] encodedArgs);
}
