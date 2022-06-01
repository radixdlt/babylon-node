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

package com.radixdlt.interop.sbor.codec.core;

import static com.radixdlt.interop.sbor.api.DecodingError.INVALID_BOOLEAN;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_BOOL;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_I16;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_I32;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_I64;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_I8;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_STRING;
import static com.radixdlt.interop.sbor.api.TypeId.TYPE_UNIT;
import static com.radixdlt.lang.Result.success;

import com.radixdlt.interop.sbor.api.DecoderApi;
import com.radixdlt.interop.sbor.api.DecodingError;
import com.radixdlt.interop.sbor.api.EncoderApi;
import com.radixdlt.interop.sbor.api.TypeId;
import com.radixdlt.interop.sbor.codec.ClassCodec;
import com.radixdlt.interop.sbor.codec.ClassField;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Unit;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.function.Consumer;

public abstract sealed class CoreTypeCodec<T> implements ClassCodec<T> {
  @Override
  public List<ClassField<T>> fields() {
    return List.of();
  }

  @Override
  public Result<T> decodeFields(DecoderApi anyDecoder) {
    return Result.failure(DecodingError.UNSUPPORTED_TYPE);
  }

  @Override
  public abstract Result<T> decode(DecoderApi decoder);

  protected <V> Result<Unit> encodePlainType(
      EncoderApi encoder, TypeId typeId, V value, Consumer<V> typeEncoder) {
    encoder.encodeTypeId(typeId);
    typeEncoder.accept(value);

    return Unit.unitResult();
  }

  public static final class UnitCodec extends CoreTypeCodec<Unit> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Unit value) {
      return encoder.encodeTypeId(TYPE_UNIT);
    }

    @Override
    public Result<Unit> decode(DecoderApi decoder) {
      return decoder.expectType(TYPE_UNIT);
    }
  }

  public static final class BooleanCodec extends CoreTypeCodec<Boolean> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Boolean value) {
      return encodePlainType(encoder, TYPE_BOOL, (byte) (value ? 1 : 0), encoder::writeByte);
    }

    @Override
    public Result<Boolean> decode(DecoderApi decoder) {
      return decoder
          .expectType(TYPE_BOOL)
          .flatMap(decoder::readByte)
          .flatMap(
              value ->
                  value == 0
                      ? Result.ok(false)
                      : value == 1 ? success(true) : INVALID_BOOLEAN.result());
    }
  }

  public static final class StringCodec extends CoreTypeCodec<String> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, String string) {
      return encodePlainType(
          encoder,
          TYPE_STRING,
          string,
          (stringValue) -> {
            var stringBytes = stringValue.getBytes(StandardCharsets.UTF_8);
            encoder.writeInt(stringBytes.length);
            encoder.writeBytes(stringBytes);
          });
    }

    @Override
    public Result<String> decode(DecoderApi decoder) {
      return decoder
          .expectType(TYPE_STRING)
          .flatMap(decoder::readInt)
          .flatMap(decoder::readBytes)
          .map(bytes -> new String(bytes, StandardCharsets.UTF_8));
    }
  }

  public static final class ByteCodec extends CoreTypeCodec<Byte> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Byte value) {
      return encodePlainType(encoder, TYPE_I8, value, encoder::writeByte);
    }

    @Override
    public Result<Byte> decode(DecoderApi decoder) {
      return decoder.expectType(TYPE_I8).flatMap(decoder::readByte);
    }
  }

  public static final class ShortCodec extends CoreTypeCodec<Short> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Short value) {
      return encodePlainType(encoder, TYPE_I16, value, encoder::writeShort);
    }

    @Override
    public Result<Short> decode(DecoderApi decoder) {
      return decoder.expectType(TYPE_I16).flatMap(decoder::readShort);
    }
  }

  public static final class IntegerCodec extends CoreTypeCodec<Integer> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Integer value) {
      return encodePlainType(encoder, TYPE_I32, value, encoder::writeInt);
    }

    @Override
    public Result<Integer> decode(DecoderApi decoder) {
      return decoder.expectType(TYPE_I32).flatMap(decoder::readInt);
    }
  }

  public static final class LongCodec extends CoreTypeCodec<Long> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, Long value) {
      return encodePlainType(encoder, TYPE_I64, value, encoder::writeLong);
    }

    @Override
    public Result<Long> decode(DecoderApi decoder) {
      return decoder.expectType(TYPE_I64).flatMap(decoder::readLong);
    }
  }

  public static final class ByteArrayCodec extends CoreTypeCodec<byte[]> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, byte[] value) {
      encoder.encodeArrayHeader(TYPE_I8, value.length);
      encoder.writeBytes(value);

      return Unit.unitResult();
    }

    @Override
    public Result<byte[]> decode(DecoderApi decoder) {
      return decoder.decodeArrayHeader(TYPE_I8).flatMap(decoder::readBytes);
    }
  }

  public static final class ShortArrayCodec extends CoreTypeCodec<short[]> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, short[] value) {
      encoder.encodeArrayHeader(TYPE_I16, value.length);

      for (var singleValue : value) {
        encoder.writeShort(singleValue);
      }

      return Unit.unitResult();
    }

    @Override
    public Result<short[]> decode(DecoderApi decoder) {
      return decoder.decodeArrayHeader(TYPE_I16).flatMap(decoder::readShorts);
    }
  }

  public static final class IntegerArrayCodec extends CoreTypeCodec<int[]> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, int[] value) {
      encoder.encodeArrayHeader(TYPE_I32, value.length);

      for (var singleValue : value) {
        encoder.writeInt(singleValue);
      }

      return Unit.unitResult();
    }

    @Override
    public Result<int[]> decode(DecoderApi decoder) {
      return decoder.decodeArrayHeader(TYPE_I32).flatMap(decoder::readIntegers);
    }
  }

  public static final class LongArrayCodec extends CoreTypeCodec<long[]> {
    @Override
    public Result<Unit> encode(EncoderApi encoder, long[] value) {
      encoder.encodeArrayHeader(TYPE_I64, value.length);

      for (var singleValue : value) {
        encoder.writeLong(singleValue);
      }

      return Unit.unitResult();
    }

    @Override
    public Result<long[]> decode(DecoderApi decoder) {
      return decoder.decodeArrayHeader(TYPE_I64).flatMap(decoder::readLongs);
    }
  }
}
