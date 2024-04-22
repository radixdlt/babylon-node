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

package com.radixdlt.mempool;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.monitoring.LabelledTimer;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.MethodId;
import com.radixdlt.sbor.Natives;
import com.radixdlt.transactions.NotarizedTransactionHash;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt32;
import java.util.List;
import java.util.Set;

public class RustMempool
    implements MempoolReader<PreparedNotarizedTransaction, NotarizedTransactionHash>,
        MempoolInserter<RawNotarizedTransaction>,
        MempoolReevaluator {

  public RustMempool(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    LabelledTimer<MethodId> timer = metrics.stateManager().nativeCall();
    addFunc =
        Natives.builder(nodeRustEnvironment, RustMempool::add)
            .measure(timer.label(new MethodId(RustMempool.class, "add")))
            .build(new TypeToken<>() {});
    getTransactionsForProposalFunc =
        Natives.builder(nodeRustEnvironment, RustMempool::getTransactionsForProposal)
            .measure(timer.label(new MethodId(RustMempool.class, "getTransactionsForProposal")))
            .build(new TypeToken<>() {});
    getTransactionsToRelayFunc =
        Natives.builder(nodeRustEnvironment, RustMempool::getTransactionsToRelay)
            .measure(timer.label(new MethodId(RustMempool.class, "getTransactionsToRelay")))
            .build(new TypeToken<>() {});
    reevaluateTransactionCommittabilityFunc =
        Natives.builder(nodeRustEnvironment, RustMempool::reevaluateTransactionCommittability)
            .measure(
                timer.label(new MethodId(RustMempool.class, "reevaluateTransactionCommittability")))
            .build(new TypeToken<>() {});
    getCountFunc =
        Natives.builder(nodeRustEnvironment, RustMempool::getCount)
            .measure(timer.label(new MethodId(RustMempool.class, "getCount")))
            .build(new TypeToken<>() {});
  }

  @Override
  public void addTransaction(RawNotarizedTransaction transaction) throws MempoolRejectedException {
    final var result = addFunc.call(transaction);

    // Handle Errors.
    if (result.isError()) {
      switch (result.unwrapError()) {
        case MempoolError.PriorityThresholdNotMet e ->
            throw new MempoolPriorityThresholdNotMetException(
                String.format(
                    "Mempool is full and transaction's priority threshold not met, min tip"
                        + " percentage required: %s",
                    e.minTipPercentageRequired()));
        case MempoolError.Duplicate e ->
            throw new MempoolDuplicateException(
                String.format(
                    "Mempool already has transaction with notarized hash %s",
                    e.notarizedTransactionHash()));
        case MempoolError.TransactionValidationError e ->
            throw new MempoolRejectedException(e.errorDescription());
        case MempoolError.Rejected rejected ->
            throw new MempoolRejectedException(rejected.reason());
      }
    }
  }

  @Override
  public List<PreparedNotarizedTransaction> getTransactionsForProposal(
      int maxCount, int maxPayloadSizeBytes, Set<NotarizedTransactionHash> hashesToExclude) {
    if (maxCount <= 0) {
      throw new IllegalArgumentException(
          "State Manager Mempool: maxCount must be > 0: " + maxCount);
    }

    if (maxPayloadSizeBytes <= 0) {
      throw new IllegalArgumentException(
          "State Manager Mempool: maxPayloadSizeBytes must be > 0: " + maxPayloadSizeBytes);
    }

    return getTransactionsForProposalFunc.call(
        new ProposalTransactionsRequest(
            UInt32.fromNonNegativeInt(maxCount),
            UInt32.fromNonNegativeInt(maxPayloadSizeBytes),
            hashesToExclude));
  }

  @Override
  public List<PreparedNotarizedTransaction> getTransactionsToRelay(
      int maxNumTxns, int maxTotalTxnsPayloadSize) {
    return getTransactionsToRelayFunc.call(
        Tuple.tuple(
            UInt32.fromNonNegativeInt(maxNumTxns),
            UInt32.fromNonNegativeInt(maxTotalTxnsPayloadSize)));
  }

  @Override
  public void reevaluateTransactionCommittability(int maxReevaluatedCount) {
    reevaluateTransactionCommittabilityFunc.call(UInt32.fromNonNegativeInt(maxReevaluatedCount));
  }

  @Override
  public int getCount() {
    return getCountFunc.call(Tuple.Tuple0.of());
  }

  private static native byte[] add(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Natives.Call1<RawNotarizedTransaction, Result<Tuple.Tuple0, MempoolError>> addFunc;

  private static native byte[] getTransactionsForProposal(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Natives.Call1<ProposalTransactionsRequest, List<PreparedNotarizedTransaction>>
      getTransactionsForProposalFunc;

  private static native byte[] getTransactionsToRelay(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Natives.Call1<Tuple.Tuple2<UInt32, UInt32>, List<PreparedNotarizedTransaction>>
      getTransactionsToRelayFunc;

  private final Natives.Call1<UInt32, Tuple.Tuple0> reevaluateTransactionCommittabilityFunc;

  private static native byte[] reevaluateTransactionCommittability(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] getCount(Object stateManager, byte[] payload);

  private final Natives.Call1<Tuple.Tuple0, Integer> getCountFunc;
}
