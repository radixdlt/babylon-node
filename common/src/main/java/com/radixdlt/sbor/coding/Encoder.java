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

import com.radixdlt.sbor.codec.Codec;
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.sbor.exceptions.SborEncodeException;
import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;

/**
 * Performs the role of an AnyEncoder in the Rust SBOR implementation
 *
 * @param output
 */
public record Encoder(ByteArrayOutputStream output) implements EncoderApi {

  public <T> void encodePayload(byte prefixByte, T value, Codec<T> codec) {
    writeByte(prefixByte);
    codec.encodeWithTypeId(this, value);
  }

  @Override
  public <T> void encodeWithTypeId(T value, Codec<T> codec) {
    codec.encodeWithTypeId(this, value);
  }

  @Override
  public void encodeTypeId(TypeId typeId) {
    writeByte(typeId.id());
  }

  @Override
  public void writeSize(int size) {
    // LEB128 and 4 bytes max
    // This means the max size is 0x0FFFFFFF = 268,435,455
    final int MAX_SIZE = 0x0FFFFFFF;
    if (size > MAX_SIZE) {
      throw new SborEncodeException(
          String.format("Size was %s, larger than max allowed %s", size, MAX_SIZE));
    }
    while (true) {
      var sevenBits = (byte) (size & 0x7F);
      size = size >> 7;
      if (size == 0) {
        writeByte(sevenBits);
        return;
      } else {
        var bytesWithContinuePrefix = (byte) (sevenBits | 0x80);
        writeByte(bytesWithContinuePrefix);
      }
    }
  }

  @Override
  public void writeBoolean(boolean value) {
    writeByte((byte) (value ? 1 : 0));
  }

  @Override
  public void writeByte(byte value) {
    output.write(value);
  }

  @Override
  public void writeBytes(byte[] value) {
    if (value.length > 0) {
      output.writeBytes(value);
    }
  }

  @Override
  public void writeShort(short value) {
    writeByte((byte) (value & 0xFF));
    writeByte((byte) ((value >> 8) & 0xFF));
  }

  @Override
  public void writeInt(int value) {
    writeByte((byte) (value & 0xFF));
    writeByte((byte) ((value >> 8) & 0xFF));
    writeByte((byte) ((value >> 16) & 0xFF));
    writeByte((byte) ((value >> 24) & 0xFF));
  }

  @Override
  public void writeLong(long value) {
    writeByte((byte) (value & 0xFF));
    writeByte((byte) ((value >> 8) & 0xFF));
    writeByte((byte) ((value >> 16) & 0xFF));
    writeByte((byte) ((value >> 24) & 0xFF));
    writeByte((byte) ((value >> 32) & 0xFF));
    writeByte((byte) ((value >> 40) & 0xFF));
    writeByte((byte) ((value >> 48) & 0xFF));
    writeByte((byte) ((value >> 56) & 0xFF));
  }

  @Override
  public void writeString(String value) {
    var stringBytes = value.getBytes(StandardCharsets.UTF_8);
    writeSize(stringBytes.length);
    writeBytes(stringBytes);
  }
}
