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

import com.google.common.collect.ImmutableList;
import com.radixdlt.sbor.SborEnumDiscriminator;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.EnumCodec;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;

public sealed interface MetadataValue {
  static void registerCodec(CodecMap codecMap) {
    codecMap.registerForSealedClassAndSubclasses(
        MetadataValue.class,
        (codecs) -> EnumCodec.fromPermittedRecordSubclasses(MetadataValue.class, codecs));
  }

  @SborEnumDiscriminator(0)
  record String(java.lang.String value) implements MetadataValue {}

  @SborEnumDiscriminator(1)
  record Bool(boolean value) implements MetadataValue {}

  @SborEnumDiscriminator(2)
  record U8(byte value) implements MetadataValue {}

  @SborEnumDiscriminator(3)
  record U32(UInt32 value) implements MetadataValue {}

  @SborEnumDiscriminator(4)
  record U64(UInt64 value) implements MetadataValue {}

  @SborEnumDiscriminator(5)
  record I32(int value) implements MetadataValue {}

  @SborEnumDiscriminator(6)
  record I64(long value) implements MetadataValue {}

  @SborEnumDiscriminator(7)
  record Decimal(com.radixdlt.rev2.Decimal value) implements MetadataValue {}

  @SborEnumDiscriminator(8)
  record GlobalAddress(com.radixdlt.rev2.GlobalAddress value) implements MetadataValue {}

  @SborEnumDiscriminator(9)
  record PublicKey(com.radixdlt.crypto.PublicKey value) implements MetadataValue {}

  @SborEnumDiscriminator(10)
  record NonFungibleGlobalId(com.radixdlt.rev2.NonFungibleGlobalId value)
      implements MetadataValue {}

  @SborEnumDiscriminator(11)
  record NonFungibleLocalId(com.radixdlt.rev2.NonFungibleLocalId value) implements MetadataValue {}

  @SborEnumDiscriminator(12)
  record Instant(long value) implements MetadataValue {}

  @SborEnumDiscriminator(13)
  record Url(java.lang.String value) implements MetadataValue {}

  @SborEnumDiscriminator(14)
  record Origin(java.lang.String value) implements MetadataValue {}

  @SborEnumDiscriminator(15)
  record PublicKeyHash(com.radixdlt.rev2.PublicKeyHash value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) 0x80)
  record StringArray(ImmutableList<java.lang.String> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 1))
  record BoolArray(ImmutableList<Boolean> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 2))
  record U8Array(ImmutableList<Byte> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 3))
  record U32Array(ImmutableList<UInt32> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 4))
  record U64Array(ImmutableList<UInt64> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 5))
  record I32Array(ImmutableList<Integer> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 6))
  record I64Array(ImmutableList<Long> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 7))
  record DecimalArray(ImmutableList<com.radixdlt.rev2.Decimal> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 8))
  record GlobalAddressArray(ImmutableList<com.radixdlt.rev2.GlobalAddress> value)
      implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 9))
  record PublicKeyArray(ImmutableList<com.radixdlt.crypto.PublicKey> value)
      implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 10))
  record NonFungibleGlobalIdArray(ImmutableList<com.radixdlt.rev2.NonFungibleGlobalId> value)
      implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 11))
  record NonFungibleLocalIdArray(ImmutableList<com.radixdlt.rev2.NonFungibleLocalId> value)
      implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 12))
  record InstantArray(ImmutableList<Long> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 13))
  record UrlArray(ImmutableList<java.lang.String> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 14))
  record OriginArray(ImmutableList<java.lang.String> value) implements MetadataValue {}

  @SborEnumDiscriminator((byte) (0x80 + 15))
  record PublicKeyHashArray(ImmutableList<com.radixdlt.rev2.PublicKeyHash> value)
      implements MetadataValue {}
}
