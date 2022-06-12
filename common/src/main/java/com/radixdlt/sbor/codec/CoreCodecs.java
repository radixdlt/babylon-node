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

package com.radixdlt.sbor.codec;

import static com.radixdlt.sbor.codec.constants.TypeId.*;

import com.radixdlt.lang.Unit;
import com.radixdlt.sbor.codec.Codec;
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.sbor.coding.DecoderApi;
import com.radixdlt.sbor.coding.EncoderApi;
import com.radixdlt.sbor.exceptions.SborDecodeException;

@SuppressWarnings("unused")
public final class CoreCodecs {

  public static final class UnitCodec implements Codec<Unit> {
    @Override
    public TypeId getTypeId() {
      return TYPE_UNIT;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Unit value) {
      // NO-OP
    }

    @Override
    public Unit decodeWithoutTypeId(DecoderApi decoder) {
      return Unit.unit();
    }
  }

  public static final class BooleanCodec implements Codec<Boolean> {
    @Override
    public TypeId getTypeId() {
      return TYPE_BOOL;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Boolean value) {
      encoder.writeBoolean(value);
    }

    @Override
    public Boolean decodeWithoutTypeId(DecoderApi decoder) {
      return decoder.readBoolean();
    }
  }

  public static final class StringCodec implements Codec<String> {
    @Override
    public TypeId getTypeId() {
      return TYPE_STRING;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, String value) {
      encoder.writeString(value);
    }

    @Override
    public String decodeWithoutTypeId(DecoderApi decoder) {
      return decoder.readString();
    }
  }

  public static final class ByteCodec implements Codec<Byte> {
    private final TypeId typeId;

    public ByteCodec() {
      this(false);
    }

    /**
     * Note that shorts are always signed in Java.
     * If signed is false, you may have a positive SBOR value mapping to a
     * negative Java value - but this is pretty normal/expected with Java bytes,
     * so we don't add the possibility for asserts (unlike the other types).
     * This also improves performance marginally on what might be a critical code path.
     * @param signed - whether to map to the signed SBOR type or not
     */
    public ByteCodec(boolean signed) {
      typeId = signed ? TYPE_I8 : TYPE_U8;
    }

    @Override
    public TypeId getTypeId() {
      return typeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Byte value) {
      encoder.writeByte(value);
    }

    @Override
    public Byte decodeWithoutTypeId(DecoderApi decoder) {
      return decoder.readByte();
    }
  }

  public static final class ShortCodec implements Codec<Short> {
    private final TypeId typeId;
    private final boolean assertNonNegativeOnDecode;

    public ShortCodec() {
      this(true);
    }

    public ShortCodec(boolean signed) {
      this(signed, !signed);
    }

    /**
     * Note that shorts are always signed in Java.
     * If signed is false, you may have a positive SBOR value mapping to a
     * negative Java value. assertNonNegativeOnDecode protects against this.
     * @param signed - whether to map to the signed SBOR type or not
     * @param assertNonNegativeOnDecode - if true, raises an exception on decoding a negative value
     */
    public ShortCodec(boolean signed, boolean assertNonNegativeOnDecode) {
      typeId = signed ? TYPE_I16 : TYPE_U16;
      this.assertNonNegativeOnDecode = assertNonNegativeOnDecode;
    }

    @Override
    public TypeId getTypeId() {
      return typeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Short value) {
      encoder.writeShort(value);
    }

    @Override
    public Short decodeWithoutTypeId(DecoderApi decoder) {
      if (!assertNonNegativeOnDecode) {
        return decoder.readShort();
      }
      var value = decoder.readShort();
      if (value < 0) {
        throw new SborDecodeException(String.format("Decoded short was negative %s.", value));
      }
      return value;
    }
  }

  public static final class IntegerCodec implements Codec<Integer> {
    private final TypeId typeId;
    private final boolean assertNonNegativeOnDecode;

    public IntegerCodec() {
      this(true);
    }

    public IntegerCodec(boolean signed) {
      this(signed, !signed);
    }

    /**
     * Note that ints are always signed in Java.
     * If signed is false, you may have a positive SBOR value mapping to a
     * negative Java value. assertNonNegativeOnDecode protects against this.
     * @param signed - whether to map to the signed SBOR type or not
     * @param assertNonNegativeOnDecode - if true, raises an exception on decoding a negative value
     */
    public IntegerCodec(boolean signed, boolean assertNonNegativeOnDecode) {
      typeId = signed ? TYPE_I32 : TYPE_U32;
      this.assertNonNegativeOnDecode = assertNonNegativeOnDecode;
    }

    @Override
    public TypeId getTypeId() {
      return typeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Integer value) {
      encoder.writeInt(value);
    }

    @Override
    public Integer decodeWithoutTypeId(DecoderApi decoder) {
      if (!assertNonNegativeOnDecode) {
        return decoder.readInt();
      }
      var value = decoder.readInt();
      if (value < 0) {
        throw new SborDecodeException(String.format("Decoded int was negative %s.", value));
      }
      return value;
    }
  }

  public static final class LongCodec implements Codec<Long> {
    private final TypeId typeId;
    private final boolean assertNonNegativeOnDecode;

    public LongCodec() {
      this(true, false);
    }

    public LongCodec(boolean signed) {
      this(signed, !signed);
    }

    /**
     * Note that Longs are always signed in Java.
     * If signed is false, you may have a positive SBOR value mapping to a
     * negative Java value. assertNonNegativeOnDecode protects against this.
     * @param signed - whether to map to the signed SBOR type or not
     * @param assertNonNegativeOnDecode - if true, raises an exception on decoding a negative value
     */
    public LongCodec(boolean signed, boolean assertNonNegativeOnDecode) {
      typeId = signed ? TYPE_I64 : TYPE_U64;
      this.assertNonNegativeOnDecode = assertNonNegativeOnDecode;
    }

    @Override
    public TypeId getTypeId() {
      return typeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, Long value) {
      encoder.writeLong(value);
    }

    @Override
    public Long decodeWithoutTypeId(DecoderApi decoder) {
      if (!assertNonNegativeOnDecode) {
        return decoder.readLong();
      }
      var value = decoder.readLong();
      if (value < 0) {
        throw new SborDecodeException(String.format("Decoded long was negative %s.", value));
      }
      return value;
    }
  }

  public static final class ByteArrayCodec implements Codec<byte[]> {
    private final TypeId collectionTypeId;

    public ByteArrayCodec(TypeId collectionTypeId) {
      collectionTypeId.assertCollectionType();
      this.collectionTypeId = collectionTypeId;
    }

    public ByteArrayCodec() {
      this(TYPE_VEC);
    }

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, byte[] value) {
      encoder.encodeTypeId(TYPE_U8);
      encoder.writeInt(value.length);
      encoder.writeBytes(value);
    }

    @Override
    public byte[] decodeWithoutTypeId(DecoderApi decoder) {
      decoder.expectType(TYPE_U8);
      var length = decoder.readInt();
      return decoder.readBytes(length);
    }
  }

  public static final class ShortArrayCodec implements Codec<short[]> {
    private final TypeId collectionTypeId;

    public ShortArrayCodec(TypeId collectionTypeId) {
      collectionTypeId.assertCollectionType();
      this.collectionTypeId = collectionTypeId;
    }

    public ShortArrayCodec() {
      this(TYPE_VEC);
    }

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, short[] value) {
      encoder.encodeTypeId(TYPE_I16);
      encoder.writeInt(value.length);

      for (var singleValue : value) {
        encoder.writeShort(singleValue);
      }
    }

    @Override
    public short[] decodeWithoutTypeId(DecoderApi decoder) {
      decoder.expectType(TYPE_I16);
      var length = decoder.readInt();
      return decoder.readShorts(length);
    }
  }

  public static final class IntegerArrayCodec implements Codec<int[]> {
    private final TypeId collectionTypeId;

    public IntegerArrayCodec(TypeId collectionTypeId) {
      collectionTypeId.assertCollectionType();
      this.collectionTypeId = collectionTypeId;
    }

    public IntegerArrayCodec() {
      this(TYPE_VEC);
    }

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, int[] value) {
      encoder.encodeTypeId(TYPE_I32);
      encoder.writeInt(value.length);

      for (var singleValue : value) {
        encoder.writeInt(singleValue);
      }
    }

    @Override
    public int[] decodeWithoutTypeId(DecoderApi decoder) {
      decoder.expectType(TYPE_I32);
      var length = decoder.readInt();
      return decoder.readIntegers(length);
    }
  }

  public static final class LongArrayCodec implements Codec<long[]> {
    private final TypeId collectionTypeId;

    public LongArrayCodec(TypeId collectionTypeId) {
      collectionTypeId.assertCollectionType();
      this.collectionTypeId = collectionTypeId;
    }

    public LongArrayCodec() {
      this(TYPE_VEC);
    }

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, long[] value) {
      encoder.encodeTypeId(TYPE_I64);
      encoder.writeInt(value.length);

      for (var singleValue : value) {
        encoder.writeLong(singleValue);
      }
    }

    @Override
    public long[] decodeWithoutTypeId(DecoderApi decoder) {
      decoder.expectType(TYPE_I64);
      var length = decoder.readInt();
      return decoder.readLongs(length);
    }
  }
}
