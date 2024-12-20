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

package com.radixdlt.transaction;

import static com.radixdlt.lang.Tuple.tuple;

import com.google.common.reflect.TypeToken;
import com.radixdlt.crypto.*;
import com.radixdlt.exceptions.TransactionPreparationException;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.message.TransactionMessage;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.TransactionHeader;
import com.radixdlt.sbor.Natives;
import com.radixdlt.transactions.*;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Optional;

public final class TransactionPreparer {
  static {
    // This is idempotent with the other calls
    System.loadLibrary("corerust");
  }

  public static PreparedIntent prepareIntent(
      NetworkDefinition network,
      TransactionHeader header,
      String manifest,
      List<byte[]> blobs,
      Optional<TransactionMessage> message) {
    return prepareIntentFunc
        .call(new PrepareIntentRequest(network, header, manifest, blobs, Option.from(message)))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<PrepareIntentRequest, Result<PreparedIntent, String>>
      prepareIntentFunc =
          Natives.builder(TransactionPreparer::prepareIntent).build(new TypeToken<>() {});

  public static PreparedTransactionIntentV2 prepareTransactionIntentV2(
      NetworkDefinition network, TransactionHeader header, List<Integer> subintentDiscriminators) {
    var rustSubintentDiscriminators =
        subintentDiscriminators.stream().map(UInt64::fromNonNegativeLong).toList();
    return prepareTransactionIntentV2Func
        .call(new PrepareTransactionIntentV2Request(network, header, rustSubintentDiscriminators))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          PrepareTransactionIntentV2Request, Result<PreparedTransactionIntentV2, String>>
      prepareTransactionIntentV2Func =
          Natives.builder(TransactionPreparer::prepareTransactionIntentV2)
              .build(new TypeToken<>() {});

  public static PreparedSignedIntent prepareSignedIntent(
      byte[] intent, List<SignatureWithPublicKey> signatures) {

    return prepareSignedIntentFunc
        .call(tuple(intent, signatures))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          Tuple.Tuple2<byte[], List<SignatureWithPublicKey>>, Result<PreparedSignedIntent, String>>
      prepareSignedIntentFunc =
          Natives.builder(TransactionPreparer::prepareSignedIntent).build(new TypeToken<>() {});

  public static PreparedSignedIntent prepareSignedIntentV2(
      byte[] transactionIntent,
      List<SignatureWithPublicKey> signatures,
      List<List<SignatureWithPublicKey>> subintentSignatures) {

    return prepareSignedTransactionIntentV2Func
        .call(tuple(transactionIntent, signatures, subintentSignatures))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          Tuple.Tuple3<byte[], List<SignatureWithPublicKey>, List<List<SignatureWithPublicKey>>>,
          Result<PreparedSignedIntent, String>>
      prepareSignedTransactionIntentV2Func =
          Natives.builder(TransactionPreparer::prepareSignedTransactionIntentV2)
              .build(new TypeToken<>() {});

  public static PreparedNotarizedTransaction prepareNotarizedTransaction(
      byte[] signedIntent, Signature signature) {
    return prepareNotarizedTransactionFunc
        .call(tuple(signedIntent, signature))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          Tuple.Tuple2<byte[], Signature>, Result<PreparedNotarizedTransaction, String>>
      prepareNotarizedTransactionFunc =
          Natives.builder(TransactionPreparer::prepareNotarizedTransaction)
              .build(new TypeToken<>() {});

  public static PreparedNotarizedTransaction prepareNotarizedTransactionV2(
      byte[] signedIntent, Signature signature) {
    return prepareNotarizedTransactionV2Func
        .call(tuple(signedIntent, signature))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          Tuple.Tuple2<byte[], Signature>, Result<PreparedNotarizedTransaction, String>>
      prepareNotarizedTransactionV2Func =
          Natives.builder(TransactionPreparer::prepareNotarizedTransactionV2)
              .build(new TypeToken<>() {});

  public static PreparedPreviewTransaction prepareUnsignedPreviewTransactionV2(
      PreparedTransactionIntentV2 transactionIntent) {
    return prepareUnsignedPreviewTransactionV2Func
        .call(tuple(transactionIntent.transactionIntentBytes()))
        .unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<
          Tuple.Tuple1<byte[]>, Result<PreparedPreviewTransaction, String>>
      prepareUnsignedPreviewTransactionV2Func =
          Natives.builder(TransactionPreparer::prepareUnsignedPreviewTransactionV2)
              .build(new TypeToken<>() {});

  public static RawLedgerTransaction rawNotarizedTransactionToRawLedgerTransaction(
      RawNotarizedTransaction notarized) {
    return userTransactionToLedger.call(notarized).unwrap(TransactionPreparationException::new);
  }

  private static final Natives.Call1<RawNotarizedTransaction, Result<RawLedgerTransaction, String>>
      userTransactionToLedger =
          Natives.builder(TransactionPreparer::userTransactionToLedger).build(new TypeToken<>() {});

  private static native byte[] prepareIntent(byte[] requestPayload);

  private static native byte[] prepareTransactionIntentV2(byte[] requestPayload);

  private static native byte[] prepareSignedIntent(byte[] requestPayload);

  private static native byte[] prepareSignedTransactionIntentV2(byte[] requestPayload);

  private static native byte[] prepareNotarizedTransaction(byte[] requestPayload);

  private static native byte[] prepareNotarizedTransactionV2(byte[] requestPayload);

  private static native byte[] prepareUnsignedPreviewTransactionV2(byte[] requestPayload);

  private static native byte[] userTransactionToLedger(byte[] requestPayload);
}
