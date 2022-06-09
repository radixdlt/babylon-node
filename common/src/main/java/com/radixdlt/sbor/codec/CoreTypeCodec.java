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
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.sbor.coding.DecoderApi;
import com.radixdlt.sbor.coding.EncoderApi;
import java.nio.charset.StandardCharsets;
import java.util.function.Consumer;

public abstract sealed class CoreTypeCodec<T> implements Codec<T> {

  protected <V> void encodePlainType(
      EncoderApi encoder, TypeId typeId, V value, Consumer<V> typeEncoder) {
    encoder.encodeTypeId(typeId);
    typeEncoder.accept(value);
  }

  public static final class UnitCodec extends CoreTypeCodec<Unit> {
    @Override
    public void encode(EncoderApi encoder, Unit value) {
      encoder.encodeTypeId(TYPE_UNIT);
    }

    @Override
    public Unit decode(DecoderApi decoder) {
      decoder.expectType(TYPE_UNIT);
      return Unit.unit();
    }
  }

  public static final class BooleanCodec extends CoreTypeCodec<Boolean> {
    @Override
    public void encode(EncoderApi encoder, Boolean value) {
      encodePlainType(encoder, TYPE_BOOL, (byte) (value ? 1 : 0), encoder::writeByte);
    }

    @Override
    public Boolean decode(DecoderApi decoder) {
      return decoder.decodeBoolean();
    }
  }

  public static final class StringCodec extends CoreTypeCodec<String> {
    @Override
    public void encode(EncoderApi encoder, String string) {
      encodePlainType(
          encoder,
          TYPE_STRING,
          string,
          stringValue -> {
            var stringBytes = stringValue.getBytes(StandardCharsets.UTF_8);
            encoder.writeInt(stringBytes.length);
            encoder.writeBytes(stringBytes);
          });
    }

    @Override
    public String decode(DecoderApi decoder) {
      decoder.expectType(TYPE_STRING);
      var length = decoder.readInt();
      var bytes = decoder.readBytes(length);
      return new String(bytes, StandardCharsets.UTF_8);
    }
  }

  public static final class ByteCodec extends CoreTypeCodec<Byte> {
    @Override
    public void encode(EncoderApi encoder, Byte value) {
      encodePlainType(encoder, TYPE_U8, value, encoder::writeByte);
    }

    @Override
    public Byte decode(DecoderApi decoder) {
      return decoder.decodeByte();
    }
  }

  public static final class ShortCodec extends CoreTypeCodec<Short> {
    @Override
    public void encode(EncoderApi encoder, Short value) {
      encodePlainType(encoder, TYPE_I16, value, encoder::writeShort);
    }

    @Override
    public Short decode(DecoderApi decoder) {
      return decoder.decodeShort();
    }
  }

  public static final class IntegerCodec extends CoreTypeCodec<Integer> {
    @Override
    public void encode(EncoderApi encoder, Integer value) {
      encodePlainType(encoder, TYPE_I32, value, encoder::writeInt);
    }

    @Override
    public Integer decode(DecoderApi decoder) {
      return decoder.decodeInt();
    }
  }

  public static final class LongCodec extends CoreTypeCodec<Long> {
    @Override
    public void encode(EncoderApi encoder, Long value) {
      encodePlainType(encoder, TYPE_I64, value, encoder::writeLong);
    }

    @Override
    public Long decode(DecoderApi decoder) {
      return decoder.decodeLong();
    }
  }

  public static final class ByteArrayCodec extends CoreTypeCodec<byte[]> {
    @Override
    public void encode(EncoderApi encoder, byte[] value) {
      encoder.encodeArrayHeader(TYPE_U8, value.length);
      encoder.writeBytes(value);
    }

    @Override
    public byte[] decode(DecoderApi decoder) {
      var length = decoder.decodeArrayHeaderAndGetArrayLength(TYPE_U8);
      return decoder.readBytes(length);
    }
  }

  public static final class ShortArrayCodec extends CoreTypeCodec<short[]> {
    @Override
    public void encode(EncoderApi encoder, short[] value) {
      encoder.encodeArrayHeader(TYPE_I16, value.length);

      for (var singleValue : value) {
        encoder.writeShort(singleValue);
      }
    }

    @Override
    public short[] decode(DecoderApi decoder) {
      var length = decoder.decodeArrayHeaderAndGetArrayLength(TYPE_I16);
      return decoder.readShorts(length);
    }
  }

  public static final class IntegerArrayCodec extends CoreTypeCodec<int[]> {
    @Override
    public void encode(EncoderApi encoder, int[] value) {
      encoder.encodeArrayHeader(TYPE_I32, value.length);

      for (var singleValue : value) {
        encoder.writeInt(singleValue);
      }
    }

    @Override
    public int[] decode(DecoderApi decoder) {
      var length = decoder.decodeArrayHeaderAndGetArrayLength(TYPE_I32);
      return decoder.readIntegers(length);
    }
  }

  public static final class LongArrayCodec extends CoreTypeCodec<long[]> {
    @Override
    public void encode(EncoderApi encoder, long[] value) {
      encoder.encodeArrayHeader(TYPE_I64, value.length);

      for (var singleValue : value) {
        encoder.writeLong(singleValue);
      }
    }

    @Override
    public long[] decode(DecoderApi decoder) {
      var length = decoder.decodeArrayHeaderAndGetArrayLength(TYPE_I64);
      return decoder.readLongs(length);
    }
  }
}
