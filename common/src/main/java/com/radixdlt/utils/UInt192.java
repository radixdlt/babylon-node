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

package com.radixdlt.utils;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.SecurityCritical;
import com.radixdlt.SecurityCritical.SecurityKind;
import java.math.BigInteger;
import java.util.Arrays;
import java.util.Objects;

/** A 192-bit unsigned integer, with comparison and some basic arithmetic operations. */
@SecurityCritical(SecurityKind.NUMERIC)
public final class UInt192 implements Comparable<UInt192> {
  // Some sizing constants in line with Integer, Long etc
  /** Size of this numeric type in bits. */
  public static final int SIZE = UInt64.SIZE + UInt128.SIZE;

  /** Size of this numeric type in bytes. */
  public static final int BYTES = UInt64.BYTES + UInt128.BYTES;

  /** A constant holding the minimum value a {@code UInt192} can have, 0. */
  public static final UInt192 MIN_VALUE = new UInt192(UInt64.ZERO, UInt128.ZERO);

  /** A constant holding the maximum value an {@code UInt192} can have, 2<sup>198</sup>-1. */
  public static final UInt192 MAX_VALUE = new UInt192(UInt64.MAX_VALUE, UInt128.MAX_VALUE);

  // Some commonly used values
  public static final UInt192 ZERO = new UInt192(UInt64.ZERO, UInt128.ZERO);
  public static final UInt192 ONE = new UInt192(UInt64.ZERO, UInt128.ONE);
  public static final UInt192 TWO = new UInt192(UInt64.ZERO, UInt128.TWO);
  public static final UInt192 THREE = new UInt192(UInt64.ZERO, UInt128.THREE);
  public static final UInt192 FOUR = new UInt192(UInt64.ZERO, UInt128.FOUR);
  public static final UInt192 FIVE = new UInt192(UInt64.ZERO, UInt128.FIVE);
  public static final UInt192 SIX = new UInt192(UInt64.ZERO, UInt128.SIX);
  public static final UInt192 SEVEN = new UInt192(UInt64.ZERO, UInt128.SEVEN);
  public static final UInt192 EIGHT = new UInt192(UInt64.ZERO, UInt128.EIGHT);
  public static final UInt192 NINE = new UInt192(UInt64.ZERO, UInt128.NINE);
  public static final UInt192 TEN = new UInt192(UInt64.ZERO, UInt128.TEN);

  // Numbers in order.  This is used by factory methods.
  private static final UInt192[] numbers = {
    ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, TEN
  };

  final UInt64 high;
  final UInt128 low;

  public static UInt192 from(long value) {
    return from(UInt128.from(value));
  }

  public static UInt192 from(String s) {
    Objects.requireNonNull(s);

    int len = s.length();
    if (len > 0) {
      int i = 0;
      char ch = s.charAt(0);
      if (ch == '+') {
        i += 1; // skip first char
      }
      if (i >= len) {
        throw new NumberFormatException(s);
      }
      // No real effort to catch overflow here
      UInt192 result = UInt192.ZERO;
      while (i < len) {
        int digit = Character.digit(s.charAt(i++), 10);
        if (digit < 0) {
          throw new NumberFormatException(s);
        }
        result = result.multiply(UInt192.TEN).add(numbers[digit]);
      }
      return result;
    } else {
      throw new NumberFormatException(s);
    }
  }

  public static UInt192 from(UInt128 value) {
    return from(UInt64.ZERO, value);
  }

  public static UInt192 from(UInt64 high, UInt128 low) {
    return new UInt192(high, low);
  }

  /**
   * Factory method for materialising an {@link UInt192} from an array of bytes. The array is
   * most-significant byte first, and must not be zero length.
   *
   * <p>If the array is smaller than {@link #BYTES}, then it is effectively padded with leading zero
   * bytes.
   *
   * <p>If the array is longer than {@link #BYTES}, then values at index {@link #BYTES} and beyond
   * are ignored.
   *
   * @param bytes The array of bytes to be used.
   * @return {@code bytes} as an {@link UInt192} type.
   * @throws IllegalArgumentException if {@code bytes} is 0 length.
   * @see #toByteArray()
   */
  @JsonCreator
  public static UInt192 from(byte[] bytes) {
    Objects.requireNonNull(bytes);
    if (bytes.length == 0) {
      throw new IllegalArgumentException("bytes is 0 bytes long");
    }
    byte[] newBytes = extend(bytes);
    return from(newBytes, 0);
  }

  /**
   * Factory method for materialising an {@link UInt192} from an array of bytes. The array is
   * most-significant byte first.
   *
   * @param bytes The array of bytes to be used.
   * @param offset The offset within the array to be used.
   * @return {@code bytes} from {@code offset} as an {@link UInt192} type.
   * @see #toByteArray()
   */
  public static UInt192 from(byte[] bytes, int offset) {
    UInt64 high = UInt64.from(bytes, offset);
    UInt128 low = UInt128.from(bytes, offset + UInt64.BYTES);
    return from(high, low);
  }

  // Pad short (< BYTES length) array with appropriate lead bytes.
  private static byte[] extend(byte[] bytes) {
    if (bytes.length >= BYTES) {
      return bytes;
    }
    byte[] newBytes = new byte[BYTES];
    int newPos = BYTES - bytes.length;
    Arrays.fill(newBytes, 0, newPos, (byte) 0);
    System.arraycopy(bytes, 0, newBytes, newPos, bytes.length);
    return newBytes;
  }

  private UInt192(UInt64 high, UInt128 low) {
    this.high = Objects.requireNonNull(high);
    this.low = Objects.requireNonNull(low);
  }

  public UInt192 add(UInt192 other) {
    UInt128 newLow = this.low.add(other.low);
    // Hacker's Delight section 2-13:
    // "The following branch-free code can be used to compute the
    // overflow predicate for unsigned add/subtract, with the result
    // being in the sign position."
    // Note that the use of method calls and the ternary operator
    // very likely precludes this from being branch-free in java.
    UInt64 carry =
        this.low
                .shiftRight()
                .add(other.low.shiftRight())
                .add(this.low.and(other.low).and(UInt128.ONE))
                .isHighBitSet()
            ? UInt64.ONE
            : UInt64.ZERO;
    UInt64 newHigh = this.high.add(other.high).add(carry);
    return UInt192.from(newHigh, newLow);
  }

  public UInt192 subtract(UInt192 other) {
    UInt128 newLow = this.low.subtract(other.low);
    // Hacker's Delight section 2-13:
    // "The following branch-free code can be used to compute the
    // overflow predicate for unsigned add/subtract, with the result
    // being in the sign position."
    // Note that the use of method calls and the ternary operator
    // very likely precludes this from being branch-free in java.
    UInt64 carry =
        this.low
                .shiftRight()
                .subtract(other.low.shiftRight())
                .subtract(this.low.invert().and(other.low).and(UInt128.ONE))
                .isHighBitSet()
            ? UInt64.ONE
            : UInt64.ZERO;
    UInt64 newHigh = this.high.subtract(other.high).subtract(carry);
    return UInt192.from(newHigh, newLow);
  }

  public UInt192 decrement() {
    UInt128 l = this.low.decrement();
    UInt64 h = this.low.isZero() ? this.high.decrement() : this.high;
    return UInt192.from(h, l);
  }

  public UInt192 multiply(UInt192 multiplicand) {
    // Russian peasant
    UInt192 result = UInt192.ZERO;
    UInt192 multiplier = this;

    while (!multiplicand.isZero()) {
      if (multiplicand.isOdd()) {
        result = result.add(multiplier);
      }

      multiplier = multiplier.shiftLeft();
      multiplicand = multiplicand.shiftRight();
    }
    return result;
  }

  public UInt192 pow(int exp) {
    if (exp < 0) {
      throw new IllegalArgumentException("exp must be >= 0");
    }

    // Mirrors algorithm in multiply(...)
    UInt192 result = UInt192.ONE;
    UInt192 base = this;

    while (exp != 0) {
      if ((exp & 1) != 0) {
        result = result.multiply(base);
      }

      base = base.multiply(base);
      exp >>>= 1;
    }
    return result;
  }

  public UInt192 divide(UInt192 divisor) {
    if (divisor.isZero()) {
      throw new IllegalArgumentException("Can't divide by zero");
    }
    UInt192 q = UInt192.ZERO;
    UInt192 r = UInt192.ZERO;
    UInt192 n = this;
    for (int i = 0; i < SIZE; ++i) {
      r = r.shiftLeft();
      q = q.shiftLeft();
      if (n.high.isHighBitSet()) {
        r = r.or(UInt192.ONE);
      }
      n = n.shiftLeft();
      if (r.compareTo(divisor) >= 0) {
        r = r.subtract(divisor);
        q = q.or(UInt192.ONE);
      }
    }
    return q;
  }

  public UInt192 remainder(UInt192 divisor) {
    if (divisor.isZero()) {
      throw new IllegalArgumentException("Can't divide by zero");
    }
    UInt192 r = UInt192.ZERO;
    UInt192 n = this;
    for (int i = 0; i < SIZE; ++i) {
      r = r.shiftLeft();
      if (n.high.isHighBitSet()) {
        r = r.or(UInt192.ONE);
      }
      n = n.shiftLeft();
      if (r.compareTo(divisor) >= 0) {
        r = r.subtract(divisor);
      }
    }
    return r;
  }

  public UInt192 or(UInt192 other) {
    return UInt192.from(this.high.or(other.high), this.low.or(other.low));
  }

  public UInt192 shiftLeft() {
    UInt64 h = this.high.shiftLeft();
    if (this.low.isHighBitSet()) {
      h = h.or(UInt64.ONE);
    }
    UInt128 l = this.low.shiftLeft();
    return UInt192.from(h, l);
  }

  public UInt192 shiftRight() {
    UInt64 h = this.high.shiftRight();
    UInt128 l = this.low.shiftRight();
    if (this.high.isOdd()) {
      l = l.or(UInt128.HIGH_BIT);
    }
    return UInt192.from(h, l);
  }

  public boolean isZero() {
    return this.high.isZero() && this.low.isZero();
  }

  public boolean isOdd() {
    return this.low.isOdd();
  }

  public static UInt192 cappedLCM(UInt192 cap, UInt192... numbers) {
    Objects.requireNonNull(cap);
    UInt192 r = numbers[0];
    for (int i = 1; i < numbers.length; i++) {
      r = lcm(r, numbers[i]);
      if (r == null || r.compareTo(cap) > 0) {
        return null;
      }
    }
    return r;
  }

  private static UInt192 lcm(UInt192 x, UInt192 y) {
    UInt192 d = y.divide(gcd(x, y));
    UInt192 r = x.multiply(d);
    boolean overflow = !x.isZero() && !r.divide(x).equals(d);
    return overflow ? null : r;
  }

  private static UInt192 gcd(UInt192 x, UInt192 y) {
    return (y.isZero()) ? x : gcd(y, x.remainder(y));
  }

  public UInt64 getHigh() {
    return high;
  }

  public UInt128 getLow() {
    return low;
  }

  public BigInteger toBigInt() {
    return new BigInteger(1, this.toByteArray());
  }

  /**
   * Converts {@code this} to an array of bytes. The most significant byte will be returned in index
   * zero. The array will always be {@link #BYTES} bytes long, and will be zero filled to suit the
   * actual value.
   *
   * @return An array of {@link #BYTES} bytes representing the value of this {@link UInt192}.
   */
  public byte[] toByteArray() {
    return toByteArray(new byte[BYTES], 0);
  }

  /**
   * Converts {@code this} to an array of bytes. The most significant byte will be returned in index
   * {@code offset}. The array must be at least {@code offset + BYTES} long.
   *
   * @param bytes The array to place the bytes in.
   * @param offset The offset within the array to place the bytes.
   * @return The passed-in value of {@code bytes}.
   */
  public byte[] toByteArray(byte[] bytes, int offset) {
    this.high.toByteArray(bytes, offset);
    this.low.toByteArray(bytes, offset + UInt64.BYTES);
    return bytes;
  }

  public String toString(int radix) {
    if (radix < Character.MIN_RADIX || radix > Character.MAX_RADIX) {
      throw new IllegalArgumentException("Illegal radix: " + radix);
    }
    if (isZero()) {
      return "0";
    }
    StringBuilder sb = new StringBuilder();
    UInt192 n = this;
    UInt192 r = UInt192.from(radix);
    while (!n.isZero()) {
      UInt192 digit = n.remainder(r);
      sb.append(Character.forDigit(digit.low.low, radix));
      n = n.divide(r);
    }
    return sb.reverse().toString();
  }

  @JsonValue
  public byte[] toJson() {
    return toByteArray();
  }

  @Override
  public int compareTo(UInt192 n) {
    int cmp = this.high.compareTo(n.high);
    if (cmp == 0) {
      cmp = this.low.compareTo(n.low);
    }
    return cmp;
  }

  @Override
  public int hashCode() {
    return Objects.hashCode(this.high) * 31 + Objects.hashCode(this.low);
  }

  @Override
  public boolean equals(Object obj) {
    // Note that this needs to be consistent with compareTo
    if (this == obj) {
      return true;
    }

    if (obj instanceof UInt192 other) {
      return Objects.equals(this.high, other.high) && Objects.equals(this.low, other.low);
    }
    return false;
  }

  @Override
  public String toString() {
    return toString(10);
  }
}
