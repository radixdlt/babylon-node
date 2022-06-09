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

package com.radixdlt.sbor.coding;

import static com.radixdlt.lang.Result.success;

import com.google.inject.TypeLiteral;
import com.radixdlt.lang.Functions;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Unit;
import com.radixdlt.sbor.codec.Codec;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.constants.TypeId;
import java.io.ByteArrayInputStream;

/**
 * Performs the role of an AnyDecoder in the Rust SBOR implementation
 *
 * @param input
 * @param codecMap
 */
public record Decoder(ByteArrayInputStream input, CodecMap codecMap) implements DecoderApi {
  private static final int EOF_RC = -1;

  @Override
  public <T> Result<T> decode(Class<T> clazz) {
    return codecMap
        .get(clazz)
        .fold(DecodingError.UNSUPPORTED_TYPE::result, codec -> codec.decode(this));
  }

  @Override
  public <T> Result<T> decode(TypeLiteral<T> type) {
    return codecMap
        .get(type)
        .fold(DecodingError.UNSUPPORTED_TYPE::result, codec -> codec.decode(this));
  }

  @Override
  public <T> Result<T> decode(Codec<T> codec) {
    return codec.decode(this);
  }

  @Override
  public Result<Integer> decodeArrayHeader(TypeId expectedId) {
    return expectType(TypeId.TYPE_VEC).flatMap(() -> expectType(expectedId)).flatMap(this::readInt);
  }

  @Override
  public Result<Unit> expectType(TypeId typeId) {
    return readByte()
        .filter(DecodingError.TYPE_MISMATCH, typeByte -> typeByte == typeId.typeId())
        .map(Unit::unit);
  }

  @Override
  public Result<Byte> readByte() {
    var value = input.read();

    return value == EOF_RC ? DecodingError.EOF.result() : success((byte) value);
  }

  @Override
  public Result<Short> readShort() {
    var v0 = input.read();
    var v1 = input.read();

    if (v0 == EOF_RC || v1 == EOF_RC) {
      return DecodingError.EOF.result();
    }

    short value = (short) (v0 & 0xFF);
    value |= (short) ((v1 & 0xFF) << 8);

    return success(value);
  }

  @Override
  public Result<Integer> readInt() {
    var v0 = input.read();
    var v1 = input.read();
    var v2 = input.read();
    var v3 = input.read();

    if (v0 == EOF_RC || v1 == EOF_RC || v2 == EOF_RC || v3 == EOF_RC) {
      return DecodingError.EOF.result();
    }

    int value = v0 & 0xFF;
    value |= ((v1 & 0xFF) << 8);
    value |= ((v2 & 0xFF) << 16);
    value |= ((v3 & 0xFF) << 24);

    return success(value);
  }

  @Override
  public Result<Long> readLong() {
    var v0 = input.read();
    var v1 = input.read();
    var v2 = input.read();
    var v3 = input.read();
    var v4 = input.read();
    var v5 = input.read();
    var v6 = input.read();
    var v7 = input.read();

    if (v0 == EOF_RC
        || v1 == EOF_RC
        || v2 == EOF_RC
        || v3 == EOF_RC
        || v4 == EOF_RC
        || v5 == EOF_RC
        || v6 == EOF_RC
        || v7 == EOF_RC) {
      return DecodingError.EOF.result();
    }

    long value = v0 & 0xFF;
    value |= (((long) v1 & 0xFF) << 8);
    value |= (((long) v2 & 0xFF) << 16);
    value |= (((long) v3 & 0xFF) << 24);
    value |= (((long) v4 & 0xFF) << 32);
    value |= (((long) v5 & 0xFF) << 40);
    value |= (((long) v6 & 0xFF) << 48);
    value |= (((long) v7 & 0xFF) << 56);

    return success(value);
  }

  @Override
  public Result<byte[]> readBytes(int length) {
    var bytes = new byte[length];

    return Result.lift(unused -> DecodingError.EOF, () -> input.read(bytes))
        .filter(DecodingError.EOF, readLen -> readLen == length)
        .map(() -> bytes);
  }

  @Override
  public Result<short[]> readShorts(int length) {
    var output = new short[length];
    var index = new int[] {0};

    for (index[0] = 0; index[0] < length; index[0] += 1) {
      var result = readShort();

      if (result.isFailure()) {
        return DecodingError.EOF.result();
      }

      result.apply(Functions::unitFn, value -> output[index[0]] = value);
    }

    return success(output);
  }

  @Override
  public Result<int[]> readIntegers(int length) {
    var output = new int[length];
    var index = new int[] {0};

    for (index[0] = 0; index[0] < length; index[0] += 1) {
      var result = readInt();

      if (result.isFailure()) {
        return DecodingError.EOF.result();
      }

      result.apply(Functions::unitFn, value -> output[index[0]] = value);
    }

    return success(output);
  }

  @Override
  public Result<long[]> readLongs(int length) {
    var output = new long[length];
    var index = new int[] {0};

    for (index[0] = 0; index[0] < length; index[0] += 1) {
      var result = readLong();

      if (result.isFailure()) {
        return DecodingError.EOF.result();
      }

      result.apply(Functions::unitFn, value -> output[index[0]] = value);
    }

    return success(output);
  }
}
