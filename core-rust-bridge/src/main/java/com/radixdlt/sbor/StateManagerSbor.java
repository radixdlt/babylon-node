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

package com.radixdlt.sbor;

import com.google.common.hash.HashCode;
import com.google.common.reflect.TypeToken;
import com.radixdlt.crypto.*;
import com.radixdlt.exceptions.StateManagerRuntimeError;
import com.radixdlt.identifiers.TID;
import com.radixdlt.mempool.MempoolError;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.rev2.*;
import com.radixdlt.sbor.codec.Codec;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import com.radixdlt.statecomputer.commit.CommitRequest;
import com.radixdlt.statecomputer.commit.PrepareRequest;
import com.radixdlt.statecomputer.commit.PrepareResult;
import com.radixdlt.statemanager.*;
import com.radixdlt.transaction.CommittedTransactionStatus;
import com.radixdlt.transaction.ExecutedTransaction;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;

public final class StateManagerSbor {
  private static final ScryptoSbor sbor = createSborForStateManager();

  private static ScryptoSbor createSborForStateManager() {
    return new ScryptoSbor(
        new CodecMap()
            .register(StateManagerSbor::registerCodecsWithCodecMap)
            .register(StateManagerSbor::registerCodecsForExistingTypes));
  }

  public static <T> byte[] encode(T value, Codec<T> codec) {
    return sbor.encode_payload(value, codec);
  }

  public static <T> T decode(byte[] sborBytes, Codec<T> codec) {
    return sbor.decode_payload(sborBytes, codec);
  }

  public static <T> Codec<T> resolveCodec(TypeToken<T> typeToken) {
    return sbor.resolveCodec(typeToken);
  }

  public static void registerCodecsWithCodecMap(CodecMap codecMap) {
    UInt32.registerCodec(codecMap);
    UInt64.registerCodec(codecMap);
    RustMempoolConfig.registerCodec(codecMap);
    NetworkDefinition.registerCodec(codecMap);
    LoggingConfig.registerCodec(codecMap);
    StateManagerLoggingConfig.registerCodec(codecMap);
    StateManagerConfig.registerCodec(codecMap);
    RawLedgerTransaction.registerCodec(codecMap);
    RawNotarizedTransaction.registerCodec(codecMap);
    TransactionStatus.registerCodec(codecMap);
    Decimal.registerCodec(codecMap);
    LogLevel.registerCodec(codecMap);
    ComponentAddress.registerCodec(codecMap);
    PackageAddress.registerCodec(codecMap);
    ResourceAddress.registerCodec(codecMap);
    TID.registerCodec(codecMap);
    StateManagerRuntimeError.registerCodec(codecMap);
    MempoolError.registerCodec(codecMap);
    CommittedTransactionStatus.registerCodec(codecMap);
    ExecutedTransaction.registerCodec(codecMap);
    PublicKey.registerCodec(codecMap);
    ECDSASecp256k1PublicKey.registerCodec(codecMap);
    EdDSAEd25519PublicKey.registerCodec(codecMap);
    Signature.registerCodec(codecMap);
    ECDSASecp256k1Signature.registerCodec(codecMap);
    EdDSAEd25519Signature.registerCodec(codecMap);
    SignatureWithPublicKey.registerCodec(codecMap);
    PrepareRequest.registerCodec(codecMap);
    PrepareResult.registerCodec(codecMap);
    CommitRequest.registerCodec(codecMap);
    REv2DatabaseConfig.registerCodec(codecMap);
    TransactionHeader.registerCodec(codecMap);
    CoreApiServerConfig.registerCodec(codecMap);
    REv2StateConfig.registerCodec(codecMap);
  }

  public static void registerCodecsForExistingTypes(CodecMap codecMap) {
    registerCodecForHashCode(codecMap);
  }

  public static void registerCodecForHashCode(CodecMap codecMap) {
    codecMap.register(
        HashCode.class,
        codecs ->
            StructCodec.with(
                HashCode::fromBytes,
                codecs.of(byte[].class),
                (t, encoder) -> encoder.encode(t.asBytes())));
  }
}
