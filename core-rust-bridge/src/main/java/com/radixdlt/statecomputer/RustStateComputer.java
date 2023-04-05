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
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.mempool.MempoolInserter;
import com.radixdlt.mempool.MempoolReader;
import com.radixdlt.mempool.RustMempool;
import com.radixdlt.monitoring.LabelledTimer;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.MethodId;
import com.radixdlt.recovery.VertexStoreRecovery;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.ValidatorInfo;
import com.radixdlt.sbor.Natives;
import com.radixdlt.statecomputer.commit.*;
import com.radixdlt.statemanager.StateManager;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Objects;

public class RustStateComputer {
  private final RustMempool mempool;
  private final REv2TransactionAndProofStore transactionStore;
  private final VertexStoreRecovery vertexStoreRecovery;

  public RustStateComputer(Metrics metrics, StateManager stateManager) {
    Objects.requireNonNull(stateManager);

    this.mempool = new RustMempool(metrics, stateManager);
    this.transactionStore = new REv2TransactionAndProofStore(metrics, stateManager);
    this.vertexStoreRecovery = new VertexStoreRecovery(metrics, stateManager);

    LabelledTimer<MethodId> timer = metrics.stateManager().nativeCall();
    this.verifyFunc =
        Natives.builder(stateManager, RustStateComputer::verify)
            .measure(timer.label(new MethodId(RustStateComputer.class, "verify")))
            .build(new TypeToken<>() {});
    this.saveVertexStoreFunc =
        Natives.builder(stateManager, RustStateComputer::saveVertexStore)
            .measure(timer.label(new MethodId(RustStateComputer.class, "saveVertexStore")))
            .build(new TypeToken<>() {});
    this.prepareGenesisFunc =
        Natives.builder(stateManager, RustStateComputer::prepareGenesis)
            .measure(timer.label(new MethodId(RustStateComputer.class, "prepareGenesis")))
            .build(new TypeToken<>() {});
    this.prepareFunc =
        Natives.builder(stateManager, RustStateComputer::prepare)
            .measure(timer.label(new MethodId(RustStateComputer.class, "prepare")))
            .build(new TypeToken<>() {});
    this.commitFunc =
        Natives.builder(stateManager, RustStateComputer::commit)
            .measure(timer.label(new MethodId(RustStateComputer.class, "commit")))
            .build(new TypeToken<>() {});
    this.componentXrdAmountFunc =
        Natives.builder(stateManager, RustStateComputer::componentXrdAmount)
            .measure(timer.label(new MethodId(RustStateComputer.class, "componentXrdAmount")))
            .build(new TypeToken<>() {});
    this.validatorInfoFunc =
        Natives.builder(stateManager, RustStateComputer::validatorInfo)
            .measure(timer.label(new MethodId(RustStateComputer.class, "validatorInfo")))
            .build(new TypeToken<>() {});
    this.epochFunc =
        Natives.builder(stateManager, RustStateComputer::epoch)
            .measure(timer.label(new MethodId(RustStateComputer.class, "epoch")))
            .build(new TypeToken<>() {});
  }

  public VertexStoreRecovery getVertexStoreRecovery() {
    return vertexStoreRecovery;
  }

  public REv2TransactionAndProofStore getTransactionAndProofStore() {
    return this.transactionStore;
  }

  public MempoolReader<RawNotarizedTransaction> getMempoolReader() {
    return this.mempool;
  }

  public MempoolInserter<RawNotarizedTransaction> getMempoolInserter() {
    return this.mempool::addTransaction;
  }

  public List<RawNotarizedTransaction> getTransactionsForProposal(
      int maxCount, int maxPayloadSizeBytes, List<RawNotarizedTransaction> transactionToExclude) {
    return this.mempool.getTransactionsForProposal(
        maxCount, maxPayloadSizeBytes, transactionToExclude);
  }

  public Result<Tuple.Tuple0, String> verify(RawNotarizedTransaction transaction) {
    return verifyFunc.call(transaction);
  }

  private final Natives.Call1<RawNotarizedTransaction, Result<Tuple.Tuple0, String>> verifyFunc;

  private static native byte[] verify(StateManager stateManager, byte[] payload);

  public void saveVertexStore(byte[] vertexStoreBytes) {
    saveVertexStoreFunc.call(vertexStoreBytes);
  }

  private final Natives.Call1<byte[], Tuple.Tuple0> saveVertexStoreFunc;

  private static native byte[] saveVertexStore(StateManager stateManager, byte[] payload);

  public PrepareGenesisResult prepareGenesis(PrepareGenesisRequest prepareGenesisRequest) {
    return prepareGenesisFunc.call(prepareGenesisRequest);
  }

  private final Natives.Call1<PrepareGenesisRequest, PrepareGenesisResult> prepareGenesisFunc;

  private static native byte[] prepareGenesis(StateManager stateManager, byte[] payload);

  public PrepareResult prepare(PrepareRequest prepareRequest) {
    return prepareFunc.call(prepareRequest);
  }

  private final Natives.Call1<PrepareRequest, PrepareResult> prepareFunc;

  private static native byte[] prepare(StateManager stateManager, byte[] payload);

  public Result<Tuple.Tuple0, CommitError> commit(CommitRequest commitRequest) {
    return commitFunc.call(commitRequest);
  }

  private final Natives.Call1<CommitRequest, Result<Tuple.Tuple0, CommitError>> commitFunc;

  private static native byte[] commit(StateManager stateManager, byte[] payload);

  private final Natives.Call1<ComponentAddress, Decimal> componentXrdAmountFunc;

  public Decimal getComponentXrdAmount(ComponentAddress componentAddress) {
    return componentXrdAmountFunc.call(componentAddress);
  }

  private static native byte[] componentXrdAmount(StateManager stateManager, byte[] payload);

  private final Natives.Call1<Tuple.Tuple0, UInt64> epochFunc;

  public UInt64 getEpoch() {
    return epochFunc.call(Tuple.tuple());
  }

  private static native byte[] epoch(StateManager stateManager, byte[] payload);

  private final Natives.Call1<ComponentAddress, ValidatorInfo> validatorInfoFunc;

  public ValidatorInfo getValidatorInfo(ComponentAddress validatorAddress) {
    return validatorInfoFunc.call(validatorAddress);
  }

  private static native byte[] validatorInfo(StateManager stateManager, byte[] payload);
}
