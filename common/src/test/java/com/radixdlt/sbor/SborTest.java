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

import static com.radixdlt.lang.Option.none;
import static com.radixdlt.lang.Option.some;
import static com.radixdlt.lang.Tuple.*;
import static org.junit.Assert.*;

import com.google.common.primitives.UnsignedInteger;
import com.google.common.primitives.UnsignedLong;
import com.google.common.reflect.TypeToken;
import com.radixdlt.lang.*;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.dto.SimpleEnum;
import com.radixdlt.sbor.dto.SimpleRecord;
import com.radixdlt.sbor.exceptions.SborDecodeException;
import com.radixdlt.utils.Int128Codec;
import com.radixdlt.utils.UInt128;
import java.util.*;
import org.junit.Test;

public class SborTest {

  @Test
  public void booleanCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode(true, boolean.class);

    assertEquals(3, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(0x01, r0[1]); // Type == 0x01 - Boolean
    assertEquals(0x01, r0[2]); // Value = 1 == true

    var r1 = BasicDefaultSbor.decode(r0, boolean.class);

    assertTrue(r1);
  }

  @Test
  public void byteCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode((byte) 0x5A, byte.class);

    assertEquals(3, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(0x07, r0[1]); // Type == 0x07 - u8
    assertEquals(0x5A, r0[2]); // Value == 0x5A

    var r1 = BasicDefaultSbor.decode(r0, byte.class);

    assertEquals((byte) 0x5A, (byte) r1);
  }

  @Test
  public void shortCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode((short) 0x1234, short.class);

    assertEquals(4, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(0x03, r0[1]); // Type == 0x03 - i16
    assertEquals(0x34, r0[2]); // Value
    assertEquals(0x12, r0[3]); // Value

    var r1 = BasicDefaultSbor.decode(r0, short.class);

    assertEquals((short) 0x1234, (short) r1);
  }

  @Test
  public void intCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode(0x12345678, int.class);

    assertEquals(6, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(4, r0[1]); // Type == 0x04 - i32
    assertEquals(0x78, r0[2]); // Value
    assertEquals(0x56, r0[3]); // Value
    assertEquals(0x34, r0[4]); // Value
    assertEquals(0x12, r0[5]); // Value

    var r1 = BasicDefaultSbor.decode(r0, int.class);

    assertEquals(0x12345678, (int) r1);
  }

  @Test
  public void intEdgeCasesCanBeEncodedAndDecoded() {
    var encodedMaxValue = BasicDefaultSbor.encode(Integer.MAX_VALUE, int.class);
    var encodedMinValue = BasicDefaultSbor.encode(Integer.MIN_VALUE, int.class);

    var decodedMaxValue = BasicDefaultSbor.decode(encodedMaxValue, int.class);
    var decodedMinValue = BasicDefaultSbor.decode(encodedMinValue, int.class);

    assertEquals(Integer.MAX_VALUE, (int) decodedMaxValue);
    assertEquals(Integer.MIN_VALUE, (int) decodedMinValue);
  }

  @Test
  public void unsignedIntEdgeCasesCanBeEncodedAndDecoded() {
    var encodedMaxValue = BasicDefaultSbor.encode(UnsignedInteger.MAX_VALUE);
    var encodedMinValue = BasicDefaultSbor.encode(UnsignedInteger.ZERO);

    var decodedMaxValue = BasicDefaultSbor.decode(encodedMaxValue, UnsignedInteger.class);
    var decodedMinValue = BasicDefaultSbor.decode(encodedMinValue, UnsignedInteger.class);

    assertEquals(UnsignedInteger.MAX_VALUE, decodedMaxValue);
    assertEquals(UnsignedInteger.ZERO, decodedMinValue);
  }

  @Test
  public void longCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode(0x0123_4567_89AB_CDEFL, long.class);

    assertEquals(10, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(5, r0[1]); // Type == 0x05 - i64
    assertEquals(0xEF, r0[2] & 0xFF); // Value
    assertEquals(0xCD, r0[3] & 0xFF); // Value
    assertEquals(0xAB, r0[4] & 0xFF); // Value
    assertEquals(0x89, r0[5] & 0xFF); // Value
    assertEquals(0x67, r0[6]); // Value
    assertEquals(0x45, r0[7]); // Value
    assertEquals(0x23, r0[8]); // Value
    assertEquals(0x01, r0[9]); // Value

    var r1 = BasicDefaultSbor.decode(r0, long.class);

    assertEquals(0x0123_4567_89AB_CDEFL, (long) r1);
  }

  @Test
  public void unsignedLongEdgeCasesCanBeEncodedAndDecoded() {
    var encodedMaxValue = BasicDefaultSbor.encode(UnsignedLong.MAX_VALUE);
    var encodedMinValue = BasicDefaultSbor.encode(UnsignedLong.ZERO);

    var decodedMaxValue = BasicDefaultSbor.decode(encodedMaxValue, UnsignedLong.class);
    var decodedMinValue = BasicDefaultSbor.decode(encodedMinValue, UnsignedLong.class);

    assertEquals(UnsignedLong.MAX_VALUE, decodedMaxValue);
    assertEquals(UnsignedLong.ZERO, decodedMinValue);
  }

  @Test
  public void unsignedInteger128EncodedCorrectly() {
    var value = UInt128.THREE;

    var encoded = BasicDefaultSbor.encode(value);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          11, // UINT128 Type
          3,
          0,
          0,
          0,
          0,
          0,
          0,
          0, // The lower long of the value in little endian
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          0 // The upper long of the value in little endian
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, UInt128.class);
    assertEquals(value, decoded);
  }

  @Test
  public void unsignedInteger128EdgeCasesCanBeEncodedAndDecoded() {
    var encodedMaxValue = BasicDefaultSbor.encode(UInt128.MAX_VALUE);
    var encodedMinValue = BasicDefaultSbor.encode(UInt128.ZERO);

    var decodedMaxValue = BasicDefaultSbor.decode(encodedMaxValue, UInt128.class);
    var decodedMinValue = BasicDefaultSbor.decode(encodedMinValue, UInt128.class);

    assertEquals(UInt128.MAX_VALUE, decodedMaxValue);
    assertEquals(UInt128.ZERO, decodedMinValue);
  }

  @Test
  public void signedInteger128EncodedCorrectly() {
    // Interpreted as signed, this would be negative as its top bit is 1.
    // But because we subtract -1L (all 1s), it has a 0 lower long, which makes the test more
    // specific.
    var negativeValue = UInt128.MAX_VALUE.subtract(UInt128.from(-1L));

    var signedCodecWithNoAsserts = new Int128Codec(true, false);
    var encodedNegativeValue = BasicDefaultSbor.encode(negativeValue, signedCodecWithNoAsserts);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          6, // Signed Int128 Type
          0, 0, 0, 0, 0, 0, 0, 0, // The lower long in little endian
          -1, -1, -1, -1, -1, -1, -1, -1, // The upper long in little endian
          // NB - the upper-most bit is negative, indicating this number is negative if interpreted
          // as signed
        },
        encodedNegativeValue);

    var decodedNegativeValue =
        BasicDefaultSbor.decode(encodedNegativeValue, signedCodecWithNoAsserts);
    assertEquals(negativeValue, decodedNegativeValue);
  }

  @Test
  public void negativeSignedInteger128TriggersAssertIfDecodedWithAssertsOn() {
    // Interpreted as signed, this would be negative as its top bit is 1.
    // But because we subtract -1L (all 1s), it has a 0 lower long, which makes the test more
    // specific.
    var negativeValue = UInt128.MAX_VALUE.subtract(UInt128.from(Long.MAX_VALUE));

    // Put assertions on for possibly-negative values
    var signedCodecWithAsserts = new Int128Codec(true, true);
    var encodedNegativeValue = BasicDefaultSbor.encode(negativeValue, signedCodecWithAsserts);

    assertThrows(
        SborDecodeException.class,
        () -> BasicDefaultSbor.decode(encodedNegativeValue, signedCodecWithAsserts));
  }

  @Test
  public void stringCanBeEncodedAndDecoded() {
    var encoded = BasicDefaultSbor.encode("Test string", String.class);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x0C, // Type == 0x0C - String
          11, // String length 11
          'T',
          'e',
          's',
          't',
          ' ', // Start of string
          's',
          't',
          'r',
          'i',
          'n',
          'g' // End of string
        },
        encoded);

    var r1 = BasicDefaultSbor.decode(encoded, String.class);

    assertEquals("Test string", r1);
  }

  @Test
  public void byteArrayCanBeEncodedAndDecoded() {
    var testArray = new byte[] {0x01, 0x02, 0x03, 0x04, 0x05};

    // Define Java arrays to map to SBOR fixed-length Arrays, instead of the default, List
    var sbor = new BasicSbor(new CodecMap(true));
    var encodedArray = sbor.encode_payload(testArray, byte[].class);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Type == 0x20 - Array
          0x07, // Type == 0x07 - u8
          5, // Array length 5
          0x01,
          0x02,
          0x03,
          0x04,
          0x05 // Five bytes
        },
        encodedArray);

    var r1 = sbor.decode_payload(encodedArray, byte[].class);

    assertArrayEquals(testArray, r1);
  }

  @Test
  public void shortArrayCanBeEncodedAndDecoded() {
    var testArray = new short[] {0x0102, 0x0304};

    var encodedArray = BasicDefaultSbor.encode(testArray, short[].class);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Type == 0x20 - Array
          0x03, // Type == 0x04 - i16
          2, // Array length 2
          0x02, 0x01, // First short
          0x04, 0x03, // Second short
        },
        encodedArray);

    var r1 = BasicDefaultSbor.decode(encodedArray, short[].class);

    assertArrayEquals(testArray, r1);
  }

  @Test
  public void intArrayCanBeEncodedAndDecoded() {
    var testArray = new int[] {0x01020304, 0x05060708, 0x090A0B0C};

    var encodedArray = BasicDefaultSbor.encode(testArray, int[].class);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Type == 0x20 - Array
          0x04, // Type == 0x04 - i32
          3, // Array length 2
          0x04, 0x03, 0x02, 0x01, // First int
          0x08, 0x07, 0x06, 0x05, // Second int
          0x0C, 0x0B, 0x0A, 0x09, // Third int
        },
        encodedArray);

    var r1 = BasicDefaultSbor.decode(encodedArray, int[].class);

    assertArrayEquals(testArray, r1);
  }

  @Test
  public void longArrayCanBeEncodedAndDecoded() {
    var testArray = new long[] {0x0102030405060708L, 0x090A0B0C11121314L};

    var encodedArray = BasicDefaultSbor.encode(testArray);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Type == 0x20 - Array
          0x05, // Type == 0x05 - i64
          2, // Array length 2
          0x08, 0x07, 0x06, 0x05, // First half of first long
          0x04, 0x03, 0x02, 0x01, // Second half of first long
          0x14, 0x13, 0x12, 0x11, // First half of second long
          0x0C, 0x0B, 0x0A, 0x09, // Second half of second long
        },
        encodedArray);

    var r1 = BasicDefaultSbor.decode(encodedArray, long[].class);

    assertArrayEquals(testArray, r1);
  }

  @Test
  public void someOptionCanBeEncodedAndDecoded() {
    var optionTypeLiteral = new TypeToken<Option<String>>() {};
    var encoded = BasicDefaultSbor.encode(some("Test value"), optionTypeLiteral);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x22, // Type == 0x22 - Enum
          1, // discriminator
          1, // Enum variant length
          0x0C, // Stored type - 0x0C - String
          10, // Length of string
          'T', 'e', 's', 't', ' ', // First half of string
          'v', 'a', 'l', 'u', 'e', // Second half of string
        },
        encoded);

    var r1 = BasicDefaultSbor.decode(encoded, optionTypeLiteral);

    assertEquals(some("Test value"), r1);
  }

  @Test
  public void noneOptionCanBeEncodedAndDecoded() {
    var optionTypeLiteral = new TypeToken<Option<String>>() {};
    var encoded = BasicDefaultSbor.encode(none(), optionTypeLiteral);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x22, // Type == 0x22 - Enum
          0, // discriminator
          0, // Enum variant length
        },
        encoded);

    var r1 = BasicDefaultSbor.decode(encoded, optionTypeLiteral);

    assertEquals(none(), r1);
  }

  @Test
  public void resultCanBeEncodedAndDecoded() {
    var resultTypeLiteral = new TypeToken<Result<String, Long>>() {};

    var successResult = Result.<String, Long>success("Some value");
    var successEncoded = BasicDefaultSbor.encode(successResult, resultTypeLiteral);

    assertEquals(16, successEncoded.length);
    assertEquals(0x5b, successEncoded[0]); // Prefix Byte
    assertEquals(0x22, successEncoded[1]); // Type == 0x22 - Enum
    assertEquals(0, successEncoded[2]); // discriminator
    assertEquals(1, successEncoded[3]); // Enum variant length of 1
    assertEquals(0x0C, successEncoded[4]); // Value type - String

    var successDecoded = BasicDefaultSbor.decode(successEncoded, resultTypeLiteral);
    assertEquals(successResult, successDecoded);

    var errorResult = Result.<String, Long>error(123L);
    var errorEncoded = BasicDefaultSbor.encode(errorResult, resultTypeLiteral);
    assertEquals(13, errorEncoded.length);
    assertEquals(0x5b, errorEncoded[0]); // Prefix Byte
    assertEquals(0x22, errorEncoded[1]); // Type == 0x22 - Enum
    assertEquals(1, errorEncoded[2]); // discriminator
    assertEquals(1, errorEncoded[3]); // Enum variant length of 1
    assertEquals(0x05, errorEncoded[4]); // Value type - i64

    var failureDecoded = BasicDefaultSbor.decode(errorEncoded, resultTypeLiteral);
    assertEquals(errorResult, failureDecoded);
  }

  @Test
  public void eitherCanBeEncodedAndDecoded() {
    var eitherTypeLiteral = new TypeToken<Either<String, Long>>() {};

    var leftValue = Either.<String, Long>left("Some value");
    var leftEncoded = BasicDefaultSbor.encode(leftValue, eitherTypeLiteral);

    // NB - Left is "not-right", AKA failure
    assertEquals(16, leftEncoded.length);
    assertEquals(0x5b, leftEncoded[0]); // Prefix Byte
    assertEquals(0x22, leftEncoded[1]); // Type == 0x22 - Enum
    assertEquals(1, leftEncoded[2]); // discriminator
    assertEquals(1, leftEncoded[3]); // Enum variant length of 1
    assertEquals(0x0C, leftEncoded[4]); // Value type - String

    var leftOut = BasicDefaultSbor.decode(leftEncoded, eitherTypeLiteral);
    assertEquals(leftValue, leftOut);

    var rightValue = Either.<String, Long>right(123L);
    var rightEncoded = BasicDefaultSbor.encode(rightValue, eitherTypeLiteral);

    // NB - Right is "right", AKA success
    assertEquals(13, rightEncoded.length);
    assertEquals(0x5b, rightEncoded[0]); // Prefix Byte
    assertEquals(0x22, rightEncoded[1]); // Type == 0x22 - Enum
    assertEquals(0, rightEncoded[2]); // discriminator
    assertEquals(1, rightEncoded[3]); // Enum variant length of 1
    assertEquals(0x05, rightEncoded[4]); // Value type - i64

    var rightOut = BasicDefaultSbor.decode(rightEncoded, eitherTypeLiteral);
    assertEquals(rightValue, rightOut);
  }

  @Test
  public void objectTreeCanBeEncodedAndDecodedWithAnyStructCodec() {
    var testValue = new SimpleRecord(1234567, "Some string", Either.left(4567L), some(false));

    // PART 1 - We use codec variant 1 (StructCodec.with)
    var sborUsingWith =
        new BasicSbor(new CodecMap().register(SimpleRecord::registerCodecUsingStructCodecWith));

    var encoded1 = sborUsingWith.encode_payload(testValue, SimpleRecord.class);

    assertEquals(38, encoded1.length);
    assertEquals(0x5b, encoded1[0]); // Prefix byte
    assertEquals(0x21, encoded1[1]); // Type == 0x21 - Struct
    assertEquals(0x04, encoded1[2]); // Field count - 4

    var decoded1 = sborUsingWith.decode_payload(encoded1, SimpleRecord.class);

    assertEquals(testValue, decoded1);

    // PART 2 - We use codec variant 2 (StructCodec.fromEntries), and check they match
    var sborUsingFromFields =
        new BasicSbor(
            new CodecMap().register(SimpleRecord::registerCodecUsingStructCodecFromEntries));

    var encoded2 = sborUsingFromFields.encode_payload(testValue, SimpleRecord.class);

    assertArrayEquals(encoded1, encoded2);

    var decoded2 = sborUsingFromFields.decode_payload(encoded2, SimpleRecord.class);

    assertEquals(testValue, decoded2);

    // PART 3 - We use codec variant 3 (StructCodec.fromRecordComponents), and check they match
    var sborUsingFromRecordComponents =
        new BasicSbor(
            new CodecMap()
                .register(SimpleRecord::registerCodecUsingStructCodecFromRecordComponents));

    var encoded3 = sborUsingFromRecordComponents.encode_payload(testValue, SimpleRecord.class);

    assertArrayEquals(encoded2, encoded3);

    var decoded3 = sborUsingFromRecordComponents.decode_payload(encoded3, SimpleRecord.class);

    assertEquals(testValue, decoded3);
  }

  @Test
  public void enumCanBeEncodedAndDecoded() {
    var enumOne = new SimpleEnum.A(4, "C");
    var enumTwo = new SimpleEnum.B(Either.left(5L));

    // PART 1 - We use codec variant 1 (EnumEntry.with)
    var sborUsingWith = new BasicSbor(new CodecMap().register(SimpleEnum::registerCodecUsingWith));

    // Enum one
    var encodedEnumOne = sborUsingWith.encode_payload(enumOne, SimpleEnum.class);
    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          34, // Enum Type
          0, // discriminator
          2, // Number of fields
          4, // Field 1 - Int Type
          4, 0, 0, 0, // Field 1 value
          0x0c, // Field 2 - String Type
          1, // String length 1
          67, // "C"
        },
        encodedEnumOne);
    var decodedEnumOne = sborUsingWith.decode_payload(encodedEnumOne, SimpleEnum.class);
    assertEquals(enumOne, decodedEnumOne);

    // Enum two
    var encodedEnumTwo = sborUsingWith.encode_payload(enumTwo, SimpleEnum.class);
    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          34, // Enum Type
          1, // discriminator
          1, // number of fields
          0x22, // Field 1 - Enum Type
          1, // discriminator
          1, // Variant size
          0x05, // Field 1 - Either left is of long type
          5,
          0,
          0,
          0,
          0,
          0,
          0,
          0 // Field 1 - long value
        },
        encodedEnumTwo);
    var decodedEnumTwo = sborUsingWith.decode_payload(encodedEnumTwo, SimpleEnum.class);
    assertEquals(enumTwo, decodedEnumTwo);

    // PART 2 - We use codec variant 2 (EnumEntry.fromEntries)
    var sborUsingFromEntries =
        new BasicSbor(new CodecMap().register(SimpleEnum::registerCodecUsingFromEntries));

    // Check Enum 1
    var encodedEnumOneV2 = sborUsingFromEntries.encode_payload(enumOne, SimpleEnum.class);
    assertArrayEquals(encodedEnumOne, encodedEnumOneV2);
    var decodedEnumOneV2 = sborUsingFromEntries.decode_payload(encodedEnumOne, SimpleEnum.class);
    assertEquals(decodedEnumOneV2, decodedEnumOne);

    // Check Enum 2
    var encodedEnumTwoV2 = sborUsingFromEntries.encode_payload(enumTwo, SimpleEnum.class);
    assertArrayEquals(encodedEnumTwo, encodedEnumTwoV2);
    var decodedEnumTwoV2 = sborUsingFromEntries.decode_payload(encodedEnumTwo, SimpleEnum.class);
    assertEquals(decodedEnumTwoV2, decodedEnumTwo);

    // PART 2 - We use codec variant 2 (EnumEntry.fromEntries)
    var sborUsingPermRecSubcls =
        new BasicSbor(
            new CodecMap().register(SimpleEnum::registerCodecUsingPermittedRecordSubclasses));

    // Check Enum 1
    var encodedEnumOneV3 = sborUsingPermRecSubcls.encode_payload(enumOne, SimpleEnum.class);
    assertArrayEquals(encodedEnumOne, encodedEnumOneV3);
    var decodedEnumOneV3 = sborUsingPermRecSubcls.decode_payload(encodedEnumOne, SimpleEnum.class);
    assertEquals(decodedEnumOneV3, decodedEnumOne);

    // Check Enum 2
    var encodedEnumTwoV3 = sborUsingPermRecSubcls.encode_payload(enumTwo, SimpleEnum.class);
    assertArrayEquals(encodedEnumTwo, encodedEnumTwoV3);
    var decodedEnumTwoV3 = sborUsingPermRecSubcls.decode_payload(encodedEnumTwo, SimpleEnum.class);
    assertEquals(decodedEnumTwoV3, decodedEnumTwo);
  }

  private record SborTestCase<T>(T value, TypeToken<T> type) {}

  @Test
  @SuppressWarnings("Convert2Diamond") // Otherwise we get a compiler error :'(
  public void allTupleSizesCanBeEncodedAndDecoded() {
    var allTupleSizes =
        List.of(
            new SborTestCase<>(tuple(), new TypeToken<Tuple0>() {}),
            new SborTestCase<>(tuple("hi"), new TypeToken<Tuple1<String>>() {}),
            new SborTestCase<>(tuple("hi", 1), new TypeToken<Tuple2<String, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2), new TypeToken<Tuple3<String, Integer, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3),
                new TypeToken<Tuple4<String, Integer, Integer, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4),
                new TypeToken<Tuple5<String, Integer, Integer, Integer, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5),
                new TypeToken<Tuple6<String, Integer, Integer, Integer, Integer, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6),
                new TypeToken<
                    Tuple7<String, Integer, Integer, Integer, Integer, Integer, Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6, 7),
                new TypeToken<
                    Tuple8<
                        String,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6, 7, 8),
                new TypeToken<
                    Tuple9<
                        String,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6, 7, 8, 9),
                new TypeToken<
                    Tuple10<
                        String,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
                new TypeToken<
                    Tuple11<
                        String,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer>>() {}),
            new SborTestCase<>(
                tuple("hi", 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),
                new TypeToken<
                    Tuple12<
                        String,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer,
                        Integer>>() {}));

    for (var testCase : allTupleSizes) {
      //noinspection unchecked
      TypeToken<Tuple> type = (TypeToken<Tuple>) testCase.type;
      var encoded = BasicDefaultSbor.encode(testCase.value, type);
      var decoded = BasicDefaultSbor.decode(encoded, type);
      assertEquals(testCase.value, decoded);
    }
  }

  @Test
  public void tupleEncodedCorrectly() {
    var tupleValue = tuple("hi", 1);
    var type = new TypeToken<Tuple2<String, Integer>>() {};

    var encoded = BasicDefaultSbor.encode(tupleValue, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          33, // Tuple Type
          2, // 2 elements in tuple
          12, // String type
          2, // String length 2
          'h', 'i', 4, // Int type
          1, 0, 0, 0, // 1 encoded as int
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(tupleValue, decoded);
  }

  @Test
  public void listEncodedCorrectly() {
    var value = List.of("hi", "and", "bye");
    var type = new TypeToken<List<String>>() {};

    var encoded = BasicDefaultSbor.encode(value, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Array Type
          12, // String type
          3, // 3 elements in array
          2, // String length 2
          104, // "h"
          105, // "i"
          3, // String length 3
          97, // "a"
          110, // "n"
          100, // "d"
          3, // String length 3
          98, // "b"
          121, // "y"
          101, // "e"
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(value, decoded);
  }

  @Test
  public void hashSetEncodedCorrectly() {
    var value = new HashSet<>(List.of("hi", "and", "bye"));
    var type = new TypeToken<HashSet<String>>() {};

    var encoded = BasicDefaultSbor.encode(value, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Array Type
          12, // String type
          3, // 3 elements in set; implicitly ingestion ordering (at least in Java 17)
          2, // String length 2
          104, // "h"
          105, // "i"
          3, // String length 3
          97, // "a"
          110, // "n"
          100, // "d"
          3, // String length 3
          98, // "b"
          121, // "y"
          101, // "e"
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(value, decoded);
  }

  @Test
  public void treeSetEncodedCorrectly() {
    var value = new TreeSet<>(List.of("hi", "and", "bye"));
    var type = new TypeToken<TreeSet<String>>() {};

    var encoded = BasicDefaultSbor.encode(value, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x20, // Array Type
          12, // String type
          3, // 3 elements in set; ordered by lexicographic key ordering
          3, // String length 3 - "and", first value in lexicographic ordering
          97, // "a"
          110, // "n"
          100, // "d"
          3, // String length 3 - "bye", first value in lexicographic ordering
          98, // "b"
          121, // "y"
          101, // "e"
          2, // String length 2 - "hi", first value in lexicographic ordering
          104, // "h"
          105, // "i"
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(value, decoded);
  }

  @Test
  public void hashMapEncodedCorrectly() {
    var map = new HashMap<String, Integer>();
    map.put("hi", 1);
    map.put("and", 2);
    map.put("bye", 3);

    var type = new TypeToken<HashMap<String, Integer>>() {};

    var encoded = BasicDefaultSbor.encode(map, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x23, // Map Type
          0x0C, // Key type: String
          0x04, // Value type: Signed Integer
          3, // 3 elements in map; implicitly ingestion ordering (at least in Java 17)
          2, // String length 2
          104, // "h"
          105, // "i"
          1, 0, 0, 0, // 1
          3, // String length 3
          97, // "a"
          110, // "n"
          100, // "d"
          2, 0, 0, 0, // 2
          3, // String length 3
          98, // "b"
          121, // "y"
          101, // "e"
          3, 0, 0, 0, // 2
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(map, decoded);
  }

  @Test
  public void treeMapEncodedCorrectly() {
    var map = new TreeMap<String, Integer>();
    map.put("hi", 1);
    map.put("and", 2);
    map.put("bye", 3);

    var type = new TypeToken<TreeMap<String, Integer>>() {};

    var encoded = BasicDefaultSbor.encode(map, type);

    assertArrayEquals(
        new byte[] {
          0x5b, // Prefix Byte
          0x23, // Map Type
          0x0C, // Key type: String
          0x04, // Value type: Signed Integer
          3, // 3 elements in map; ordered by lexicographic key ordering
          3, // String length 3 - "and", first key in lexicographic ordering
          97, // "a"
          110, // "n"
          100, // "d"
          2, 0, 0, 0, // 2
          3, // String length 3 - "bye", second key in lexicographic ordering
          98, // "b"
          121, // "y"
          101, // "e"
          3, 0, 0, 0, // 3
          2, // String length 2 - "hi", third key in lexicographic ordering
          104, // "h"
          105, // "i"
          1, 0, 0, 0, // 1
        },
        encoded);

    var decoded = BasicDefaultSbor.decode(encoded, type);

    assertEquals(map, decoded);
  }

  @Test
  public void emptyStringCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode("", String.class);

    assertEquals(3, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(0x0C, r0[1]); // Type == 0x0C - String
    assertEquals(0x00, r0[2]); // String length 0

    var r1 = BasicDefaultSbor.decode(r0, String.class);

    assertEquals("", r1);
  }

  @Test
  public void emptyByteArrayCanBeEncodedAndDecoded() {
    var r0 = BasicDefaultSbor.encode(new byte[] {}, byte[].class);

    assertEquals(4, r0.length);
    assertEquals(0x5b, r0[0]); // Prefix byte
    assertEquals(0x20, r0[1]); // Type == 0x20 - Array
    assertEquals(0x07, r0[2]); // Type == 0x07 - u8
    assertEquals(0x00, r0[3]); // Array length 0

    var r1 = BasicDefaultSbor.decode(r0, byte[].class);

    assertArrayEquals(new byte[] {}, r1);
  }
}
