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
import com.radixdlt.exceptions.ManifestCompilationException;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.TransactionHeader;
import com.radixdlt.sbor.Natives;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

public final class TransactionBuilder {
  static {
    // This is idempotent with the other calls
    System.loadLibrary("corerust");
  }

  public static RawLedgerTransaction createGenesis(
      Map<ECDSASecp256k1PublicKey, Tuple.Tuple2<Decimal, ComponentAddress>>
          validatorSetAndStakeOwners,
      Map<ECDSASecp256k1PublicKey, Decimal> accountXrdAllocations,
      UInt64 initialEpoch,
      UInt64 roundsPerEpoch,
      UInt64 numUnstakeEpochs) {
    return RawLedgerTransaction.create(
        createGenesisFunc.call(
            tuple(
                validatorSetAndStakeOwners,
                accountXrdAllocations,
                initialEpoch,
                roundsPerEpoch,
                numUnstakeEpochs)));
  }

  public static RawLedgerTransaction createGenesis(
      ECDSASecp256k1PublicKey validator,
      Map<ECDSASecp256k1PublicKey, Decimal> accountXrdAllocations,
      Decimal initialStake,
      UInt64 roundsPerEpoch,
      UInt64 numUnstakeEpochs) {
    final var stakingAccount = Address.virtualAccountAddress(validator);
    return RawLedgerTransaction.create(
        createGenesisFunc.call(
            tuple(
                Map.of(validator, Tuple.tuple(initialStake, stakingAccount)),
                accountXrdAllocations,
                UInt64.fromNonNegativeLong(1),
                roundsPerEpoch,
                numUnstakeEpochs)));
  }

  public static RawLedgerTransaction createGenesisWithNumValidators(
      long numValidators, Decimal initialStake, UInt64 roundsPerEpoch) {
    return createGenesisWithNumValidatorsAndXrdAlloc(
        numValidators, Map.of(), initialStake, roundsPerEpoch);
  }

  public static RawLedgerTransaction createGenesisWithNumValidatorsAndXrdAlloc(
      long numValidators,
      Map<ECDSASecp256k1PublicKey, Decimal> xrdAlloc,
      Decimal initialStake,
      UInt64 roundsPerEpoch) {

    final var stakingAccount =
        Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    var validators =
        PrivateKeys.numeric(1)
            .limit(numValidators)
            .map(ECKeyPair::getPublicKey)
            .collect(Collectors.toMap(k -> k, k -> Tuple.tuple(initialStake, stakingAccount)));
    return RawLedgerTransaction.create(
        createGenesisFunc.call(
            tuple(
                validators,
                xrdAlloc,
                UInt64.fromNonNegativeLong(1),
                roundsPerEpoch,
                UInt64.fromNonNegativeLong(1))));
  }

  public static byte[] compileManifest(
      NetworkDefinition network, String manifest, List<byte[]> blobs) {
    return compileManifestFunc
        .call(tuple(network, manifest, blobs))
        .unwrap(ManifestCompilationException::new);
  }

  public static byte[] createIntent(
      NetworkDefinition network, TransactionHeader header, String manifest, List<byte[]> blobs) {
    return createIntentFunc
        .call(tuple(network, header, manifest, blobs))
        .unwrap(ManifestCompilationException::new);
  }

  public static byte[] createSignedIntentBytes(
      byte[] intent, List<SignatureWithPublicKey> signatures) {
    return createSignedIntentBytesFunc.call(tuple(intent, signatures));
  }

  public static byte[] createNotarizedBytes(byte[] signedIntent, Signature signature) {
    return createNotarizedBytesFunc.call(tuple(signedIntent, signature));
  }

  private static final Natives.Call1<
          Tuple.Tuple3<NetworkDefinition, String, List<byte[]>>, Result<byte[], String>>
      compileManifestFunc =
          Natives.builder(TransactionBuilder::compileManifest).build(new TypeToken<>() {});

  private static native byte[] compileManifest(byte[] payload);

  private static final Natives.Call1<
          Tuple.Tuple5<
              Map<ECDSASecp256k1PublicKey, Tuple.Tuple2<Decimal, ComponentAddress>>,
              Map<ECDSASecp256k1PublicKey, Decimal>,
              UInt64,
              UInt64,
              UInt64>,
          byte[]>
      createGenesisFunc =
          Natives.builder(TransactionBuilder::createGenesisLedgerTransaction)
              .build(new TypeToken<>() {});

  private static native byte[] createGenesisLedgerTransaction(byte[] requestPayload);

  private static final Natives.Call1<
          Tuple.Tuple4<NetworkDefinition, TransactionHeader, String, List<byte[]>>,
          Result<byte[], String>>
      createIntentFunc =
          Natives.builder(TransactionBuilder::createIntent).build(new TypeToken<>() {});

  private static native byte[] createIntent(byte[] requestPayload);

  private static final Natives.Call1<Tuple.Tuple2<byte[], List<SignatureWithPublicKey>>, byte[]>
      createSignedIntentBytesFunc =
          Natives.builder(TransactionBuilder::createSignedIntentBytes).build(new TypeToken<>() {});

  private static native byte[] createSignedIntentBytes(byte[] requestPayload);

  private static final Natives.Call1<Tuple.Tuple2<byte[], Signature>, byte[]>
      createNotarizedBytesFunc =
          Natives.builder(TransactionBuilder::createNotarizedBytes).build(new TypeToken<>() {});

  private static native byte[] createNotarizedBytes(byte[] requestPayload);

  public static byte[] userTransactionToLedgerBytes(byte[] userTransactionBytes) {
    return userTransactionToLedger.call(userTransactionBytes);
  }

  public static Option<byte[]> convertTransactionBytesToNotarizedTransactionBytes(
      byte[] transactionBytes) {
    return transactionBytesToNotarizedTransactionBytesFn.call(transactionBytes);
  }

  private static final Natives.Call1<byte[], byte[]> userTransactionToLedger =
      Natives.builder(TransactionBuilder::userTransactionToLedger).build(new TypeToken<>() {});

  private static final Natives.Call1<byte[], Option<byte[]>>
      transactionBytesToNotarizedTransactionBytesFn =
          Natives.builder(TransactionBuilder::transactionBytesToNotarizedTransactionBytes)
              .build(new TypeToken<>() {});

  private static native byte[] userTransactionToLedger(byte[] requestPayload);

  private static native byte[] transactionBytesToNotarizedTransactionBytes(byte[] transactionBytes);
}
