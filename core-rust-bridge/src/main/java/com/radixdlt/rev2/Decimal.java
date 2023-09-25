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

import com.radixdlt.SecurityCritical;
import com.radixdlt.SecurityCritical.SecurityKind;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.CustomTypeKnownLengthCodec;
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.utils.Bytes;
import com.radixdlt.utils.UInt192;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.Objects;
import org.bouncycastle.util.Arrays;

/**
 * Decimal represents a 192-bit representation of a fixed-scale decimal number.
 *
 * <p>Note that the Java implementation is fairly basic and mostly acts as a container for
 * node<->engine interop. Some basic operations (e.g. add, subtract) have been implemented using
 * UInt192.
 */
// TODO: Create an explicit I192 type and have Decimal wrap that.
@SecurityCritical(SecurityKind.NUMERIC)
public final class Decimal {
  public static final BigInteger MIN_SUBUNITS =
      new BigInteger("-3138550867693340381917894711603833208051177722232017256448");
  public static final BigInteger MAX_SUBUNITS =
      new BigInteger("3138550867693340381917894711603833208051177722232017256447");

  // NOTE: These only work because MAX/MIN subunits are also stored as 24 big-endian bytes in a
  // BigInteger
  // In general this trick doesn't work - see comment in fromNonNegativeBigIntegerSubunits
  public static final Decimal MIN_VALUE = fromBigEndianBytes(MIN_SUBUNITS.toByteArray());

  public static final Decimal MAX_VALUE = fromBigEndianBytes(MAX_SUBUNITS.toByteArray());
  public static final Decimal ONE_SUBUNIT = fromNonNegativeBigIntegerSubunits(BigInteger.ONE);
  public static final Decimal ZERO = ofNonNegative(0L);
  public static final Decimal ONE = ofNonNegative(1L);
  private static final int SCALE = 18;

  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        Decimal.class,
        codecs ->
            new CustomTypeKnownLengthCodec<>(
                TypeId.TYPE_CUSTOM_DECIMAL,
                BYTE_LENGTH,
                // Decimal codec uses little endian
                decimal -> Arrays.reverse(decimal.underlyingBigEndianBytes),
                bytes -> new Decimal(Arrays.reverse(bytes))));
  }

  public static final int BYTE_LENGTH = 24;

  private final byte[] underlyingBigEndianBytes;

  private Decimal(byte[] underlyingBigEndianBytes) {
    this.underlyingBigEndianBytes = Objects.requireNonNull(underlyingBigEndianBytes);
  }

  /** Creates a Decimal from a raw byte array representation. */
  public static Decimal fromBigEndianBytes(byte[] bigEndianBytes) {
    if (bigEndianBytes.length != BYTE_LENGTH) {
      throw new IllegalArgumentException("Wrong number of bytes");
    }
    return new Decimal(bigEndianBytes);
  }

  /**
   * Creates a Decimal from raw, non-negative BigInteger representation. Note that a BigInteger
   * value of 1 translates to 1e-18 Decimal unit, not 1. Throws if the BigInteger representation
   * exceeds 24 bytes.
   */
  public static Decimal fromNonNegativeBigIntegerSubunits(BigInteger subunits) {
    // Note - BigInteger also stores its bytes as big endian, but only to the length it needs.
    // * If it's too small and positive, we can just pad it
    // * If it's too small and negative, it's more complex to map.
    //   For now, we have no need for negatives, so we throw an error.
    Objects.requireNonNull(subunits);
    if (subunits.compareTo(BigInteger.ZERO) < 0) {
      throw new IllegalArgumentException("Expected a non-negative BigInteger");
    }
    if (subunits.compareTo(MAX_SUBUNITS) > 0) {
      throw new IllegalArgumentException("Value is too large to fit into a Decimal");
    }
    return fromBigEndianBytes(Bytes.leftPadWithZeros(subunits.toByteArray(), BYTE_LENGTH));
  }

  /** Creates a Decimal from a raw byte array representation. */
  public static Decimal fromU192Subunits(UInt192 subunits) {
    var newDecimal = Decimal.fromBigEndianBytes(subunits.toBigEndianBytes());
    if (newDecimal.isNegative()) {
      throw new IllegalArgumentException(
          "Overflow: UInt192 subunits are too big to fit in the Decimal");
    }
    return newDecimal;
  }

  public static Decimal ofNonNegative(long amount) {
    return fromNonNegativeBigIntegerSubunits(
        BigInteger.valueOf(amount).multiply(BigInteger.TEN.pow(SCALE)));
  }

  public static Decimal ofNonNegativeFraction(long numerator, long denominator) {
    if (numerator < 0 || denominator < 0) {
      throw new IllegalArgumentException("Numerator and denominator must be non-negative");
    }
    return fromNonNegativeBigIntegerSubunits(
        BigInteger.valueOf(numerator)
            .multiply(BigInteger.TEN.pow(SCALE))
            .divide(BigInteger.valueOf(denominator)));
  }

  public BigInteger toBigIntegerSubunits() {
    return new BigInteger(underlyingBigEndianBytes);
  }

  public byte[] toBigEndianBytes() {
    return underlyingBigEndianBytes;
  }

  public BigDecimal toBigDecimal() {
    return new BigDecimal(toBigIntegerSubunits(), SCALE);
  }

  public UInt192 toU192Subunits() {
    if (isNegative()) {
      throw new IllegalArgumentException("Can't convert negative decimal to UInt192");
    }
    return UInt192.fromBigEndianBytes(toBigEndianBytes());
  }

  public boolean isNegative() {
    return toBigIntegerSubunits().compareTo(BigInteger.ZERO) < 0;
  }

  public Decimal wrappingAdd(Decimal other) {
    // This is currently only used in tests.
    // Using UInt192 for calculations - addition arithmetic works, the same way for signed/unsigned
    // representation.
    return fromBigEndianBytes(
        UInt192.fromBigEndianBytes(underlyingBigEndianBytes)
            .add(UInt192.fromBigEndianBytes(other.underlyingBigEndianBytes))
            .toBigEndianBytes());
  }

  public Decimal wrappingSubtract(Decimal other) {
    // This is currently only used in tests.
    // Using UInt192 for calculations - addition arithmetic works, the same way for signed/unsigned
    // representation.
    return new Decimal(
        UInt192.fromBigEndianBytes(underlyingBigEndianBytes)
            .subtract(UInt192.fromBigEndianBytes(other.underlyingBigEndianBytes))
            .toBigEndianBytes());
  }

  @Override
  public String toString() {
    var str = toBigDecimal().toPlainString();
    if (str.contains(".")) {
      // The outputted string contains the full precision (18 decimals) - but the rust Decimal
      // doesn't include these characters...
      // EG 10000.000000000000000000 rather than 10000
      // So, to enable toString to be comparable, we fix up the format to enable comparisons.
      var trimTo = str.length();
      for (var i = str.length() - 1; i >= 0; i--) {
        var theChar = str.charAt(i);
        if (theChar == '.') {
          trimTo = i;
          break;
        } else if (theChar == '0') {
          trimTo = i;
        } else {
          break;
        }
      }
      return str.substring(0, trimTo);
    }
    return str;
  }

  @Override
  public int hashCode() {
    return Arrays.hashCode(underlyingBigEndianBytes);
  }

  @Override
  public boolean equals(Object o) {
    return o instanceof Decimal other
        && java.util.Arrays.equals(this.underlyingBigEndianBytes, other.underlyingBigEndianBytes);
  }
}
