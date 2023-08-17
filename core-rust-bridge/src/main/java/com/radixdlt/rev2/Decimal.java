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
import com.radixdlt.utils.UInt192;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.Objects;
import org.bouncycastle.util.Arrays;

/** Decimal represents a 192 bit representation of a fixed-scale decimal number. */
@SecurityCritical(SecurityKind.NUMERIC)
public class Decimal implements Comparable<Decimal> {

  public static final Decimal ZERO = Decimal.of(0L);
  private static final int SCALE = 18;

  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        Decimal.class,
        codecs ->
            new CustomTypeKnownLengthCodec<>(
                TypeId.TYPE_CUSTOM_DECIMAL,
                SBOR_BYTE_LENGTH,
                decimal -> Arrays.reverse(decimal.underlyingValue.toByteArray()),
                bytes -> new Decimal(UInt192.from(Arrays.reverse(bytes)))));
  }

  public static final int SBOR_BYTE_LENGTH = 24;

  private final UInt192 underlyingValue;

  private Decimal(UInt192 underlyingValue) {
    this.underlyingValue = Objects.requireNonNull(underlyingValue);
  }

  public static Decimal from(UInt192 fixedPointRepresentation) {
    return new Decimal(fixedPointRepresentation);
  }

  /**
   * Creates a Decimal from raw, non-negative BigInteger representation. Note that a BigInteger
   * value of 1 translates to 1e-18 Decimal unit, not 1. Throws if the BigInteger exceeds UInt192.
   */
  public static Decimal fromBigIntegerSubunits(BigInteger bigInt) {
    if (bigInt.compareTo(BigInteger.ZERO) < 0) {
      throw new IllegalArgumentException(
          "Can't create a Decimal from a negative BigInteger representation");
    }
    final var bigIntSignedBytes = bigInt.toByteArray();
    final byte[] uint192Bytes;
    if (bigIntSignedBytes.length == UInt192.BYTES + 1 && bigIntSignedBytes[0] == 0x00) {
      // A signed representation of a positive 32-bytes integer can actually contain
      // 33 bytes (additional 0x00 byte), so we just skip it.
      uint192Bytes = Arrays.copyOfRange(bigIntSignedBytes, 1, bigIntSignedBytes.length);
    } else if (bigIntSignedBytes.length > UInt192.BYTES) {
      // Can't fit
      throw new IllegalArgumentException("Decimal overflow");
    } else {
      uint192Bytes = bigIntSignedBytes;
    }
    return Decimal.from(UInt192.from(uint192Bytes));
  }

  public Decimal add(Decimal other) {
    var newUnderlying = this.underlyingValue.add(other.underlyingValue);
    return new Decimal(newUnderlying);
  }

  public BigInteger toBigIntegerSubunits() {
    return underlyingValue.toBigInt();
  }

  public static Decimal of(long amount) {
    var underlying = UInt192.from(amount).multiply(UInt192.from(10).pow(SCALE));
    return new Decimal(underlying);
  }

  public static Decimal fraction(long numerator, long denominator) {
    return Decimal.of(numerator).divide(denominator);
  }

  public Decimal subtract(Decimal other) {
    var newUnderlying = this.underlyingValue.subtract(other.underlyingValue);
    return new Decimal(newUnderlying);
  }

  public Decimal divide(long divisor) {
    var newUnderlying = this.underlyingValue.divide(UInt192.from(divisor));
    return new Decimal(newUnderlying);
  }

  public UInt192 toUInt192() {
    return underlyingValue;
  }

  @Override
  public String toString() {
    var str = new BigDecimal(underlyingValue.toBigInt(), SCALE).toString();
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
    return Objects.hashCode(underlyingValue);
  }

  @Override
  public boolean equals(Object o) {
    return o instanceof Decimal other
        && Objects.equals(this.underlyingValue, other.underlyingValue);
  }

  @Override
  public int compareTo(Decimal o) {
    return this.underlyingValue.compareTo(o.underlyingValue);
  }
}
