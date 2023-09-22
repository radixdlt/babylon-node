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

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.*;

import java.math.BigInteger;
import java.util.Arrays;
import nl.jqno.equalsverifier.EqualsVerifier;
import org.junit.Test;

/** Basic unit tests for {@link UInt192}. */
public class UInt192Test {

  // Range of numbers to be tested at extremes for Integer and Long.
  private static final int TEST_RANGE = 1_000_000;

  /**
   * Exhaustively test construction for all non-negative {@code short} values, from {@code 0} up to
   * and including {@link Short#MAX_VALUE}.
   */
  @Test
  public void when_constructing_int192_from_short_values__values_compare_equal() {
    for (int i = 0; i <= Short.MAX_VALUE; ++i) {
      short s = (short) i;
      UInt192 int192 = UInt192.from(s);
      assertEqualToLong(s, int192);
    }
  }

  /**
   * Test construction for two ranges of {@code int} values from {@code 0} through to {@code
   * TEST_RANGE}, and also from {@code Integer.MAX_VALUE} down to {@code Integer.MAX_VALUE -
   * TEST_RANGE}.
   */
  @Test
  public void when_constructing_int192_from_int_values__values_compare_equal() {
    // Here we will just assume that testing some values near the
    // extremes of the range will suffice.
    for (int i = 0; i <= TEST_RANGE; ++i) {
      UInt192 int192 = UInt192.from(i);
      assertEqualToLong(i, int192);
    }
    for (int i = 0; i <= TEST_RANGE; ++i) {
      int ii = Integer.MAX_VALUE - i;
      UInt192 int192 = UInt192.from(ii);
      assertEqualToLong(ii, int192);
    }
  }

  /**
   * Test construction for two ranges of {@code long} values from {@code 0} through to {@code
   * TEST_RANGE}, and also from {@code Long.MAX_VALUE} down to {@code Long.MAX_VALUE - TEST_RANGE}.
   */
  @Test
  public void when_constructing_int192_from_long_values__accessors_compare_equal() {
    // Here we will just assume that testing some values near the
    // extremes of the range will suffice.
    for (int i = 0; i < TEST_RANGE; ++i) {
      long l = i;
      UInt192 int192 = UInt192.from(l);
      assertEqualToLong(l, int192);
    }
    for (int i = 0; i < TEST_RANGE; ++i) {
      long l = Long.MAX_VALUE - i;
      UInt192 int192 = UInt192.from(l);
      assertEqualToLong(l, int192);
    }
  }

  @Test
  public void when_performing_basic_addition__the_correct_values_are_returned() {
    // Some basics
    // 0 + 1 = 1
    assertEqualToLong(1L, UInt192.ZERO.add(UInt192.ONE));
    // 1 + 1 = 2
    assertEqualToLong(2L, UInt192.ONE.add(UInt192.ONE));
    // max + 1 = 0 (overflow)
    assertEqualToLong(0L, UInt192.MAX_VALUE.add(UInt192.ONE));
    // 1 + max = 0 (overflow)
    assertEqualToLong(0L, UInt192.ONE.add(UInt192.MAX_VALUE));

    // The half-add methods too
    // 0 + 1 = 1
    assertEqualToLong(1L, UInt192.ZERO.add(UInt192.ONE));
    // 1 + 1 = 2
    assertEqualToLong(2L, UInt192.ONE.add(UInt192.ONE));
    // max + 1 = 0 (overflow)
    assertEqualToLong(0L, UInt192.MAX_VALUE.add(UInt192.ONE));
  }

  @Test
  public void
      when_performing_addition_overflowing_between_words__the_correct_values_are_returned() {
    // Test adding with carry.
    UInt192 carry1 = UInt192.from(UInt64.ZERO, UInt128.MAX_VALUE).add(UInt192.ONE);
    assertEquals(UInt64.ONE, carry1.getHigh());
    assertEquals(UInt128.ZERO, carry1.getLow());

    // And also for the half-add method (actually 2/3 add in this case)
    // Test adding with carry.
    UInt192 carry2 = UInt192.from(UInt64.ZERO, UInt128.MAX_VALUE).add(UInt192.ONE);
    assertEquals(UInt64.ONE, carry2.getHigh());
    assertEquals(UInt128.ZERO, carry2.getLow());
  }

  @Test
  public void when_performing_basic_subtraction__the_correct_values_are_returned() {
    // Some basics
    // 1 - 1 = 0
    assertEqualToLong(0L, UInt192.ONE.subtract(UInt192.ONE));
    // 2 - 1 = 1
    assertEqualToLong(1L, UInt192.TWO.subtract(UInt192.ONE));
    // 0 - 1 = max (underflow)
    assertEquals(UInt192.MAX_VALUE, UInt192.ZERO.subtract(UInt192.ONE));
    // 0 - max = 1 (underflow)
    assertEqualToLong(1L, UInt192.ZERO.subtract(UInt192.MAX_VALUE));

    // Half subtract methods also
    // 1 - 1 = 0
    assertEqualToLong(0L, UInt192.ONE.subtract(UInt192.ONE));
    // 2 - 1 = 1
    assertEqualToLong(1L, UInt192.TWO.subtract(UInt192.ONE));
    // 0 - 1 = max (underflow)
    assertEquals(UInt192.MAX_VALUE, UInt192.ZERO.subtract(UInt192.ONE));
  }

  @Test
  public void
      when_performing_subtraction_underflowing_between_words__the_correct_value_is_returned() {
    // Test subtraction with carry.
    UInt192 carry1 = UInt192.from(UInt64.ONE, UInt128.ZERO).subtract(UInt192.ONE);
    assertEquals(UInt64.ZERO, carry1.high);
    assertEquals(UInt128.MAX_VALUE, carry1.low);
    UInt192 carry2 = UInt192.ZERO.subtract(UInt192.ONE); // underflow
    assertEquals(UInt64.MAX_VALUE, carry2.high);
    assertEquals(UInt128.MAX_VALUE, carry2.low);

    // also with half-subtract methods
    UInt192 carry3 = UInt192.from(UInt64.ONE, UInt128.ZERO).subtract(UInt192.ONE);
    assertEquals(UInt64.ZERO, carry3.high);
    assertEquals(UInt128.MAX_VALUE, carry3.low);
    UInt192 carry4 = UInt192.ZERO.subtract(UInt192.ONE); // underflow
    assertEquals(UInt64.MAX_VALUE, carry4.high);
    assertEquals(UInt128.MAX_VALUE, carry4.low);
  }

  @Test
  public void when_decrementing_int192__the_correct_values_are_returned() {
    assertEquals(UInt192.ZERO, UInt192.ONE.decrement());
    assertEquals(UInt192.MAX_VALUE, UInt192.ZERO.decrement()); // Internal and full overflow
  }

  @Test
  public void when_multiplying_two_values__the_correct_value_is_returned() {
    // Some basics
    assertEquals(UInt192.ZERO, UInt192.ZERO.multiply(UInt192.ZERO));
    assertEquals(UInt192.ZERO, UInt192.ZERO.multiply(UInt192.ONE));
    assertEquals(UInt192.ZERO, UInt192.ONE.multiply(UInt192.ZERO));
    assertEquals(UInt192.ONE, UInt192.ONE.multiply(UInt192.ONE));

    // Some values in the long range
    assertEquals(
        UInt192.from(12345678L * 13L), UInt192.from(12345678L).multiply(UInt192.from(13L)));
  }

  @Test
  public void when_dividing_one_value_by_another__the_correct_value_is_returned() {
    // Some basics
    assertEquals(UInt192.ZERO, UInt192.ZERO.divide(UInt192.ONE));
    assertEquals(UInt192.ONE, UInt192.ONE.divide(UInt192.ONE));

    // Some values in the long range
    assertEquals(UInt192.from(12345678L / 13L), UInt192.from(12345678L).divide(UInt192.from(13L)));
  }

  @Test(expected = IllegalArgumentException.class)
  public void when_dividing_by_zero__an_exception_is_thrown() {
    UInt192.ONE.divide(UInt192.ZERO);
    fail();
  }

  @Test
  public void
      when_computing_the_remainder_of_dividing_one_value_by_another__the_correct_value_is_returned() {
    // Some basics
    assertEquals(UInt192.ZERO, UInt192.ZERO.remainder(UInt192.ONE));
    assertEquals(UInt192.ZERO, UInt192.ONE.remainder(UInt192.ONE));
    assertEquals(UInt192.ONE, UInt192.ONE.remainder(UInt192.TWO));

    // Some values in the long range
    assertEquals(
        UInt192.from(12345678L % 13L), UInt192.from(12345678L).remainder(UInt192.from(13L)));
  }

  @Test(expected = IllegalArgumentException.class)
  public void when_computing_the_remainder_of_dividing_by_zero__an_exception_is_thrown() {
    UInt192.ONE.remainder(UInt192.ZERO);
    fail();
  }

  @Test
  public void when_comparing_int192_values_using_compareTo__the_correct_value_is_returned() {
    assertThat(UInt192.ZERO)
        .isEqualByComparingTo(UInt192.ZERO)
        .isLessThan(UInt192.ONE)
        .isLessThan(UInt192.MAX_VALUE);
    assertThat(UInt192.ONE).isGreaterThan(UInt192.ZERO);
    assertThat(UInt192.MAX_VALUE).isGreaterThan(UInt192.ZERO);

    UInt192 i63 = UInt192.from(UInt64.ZERO, UInt128.HIGH_BIT);
    UInt192 i64 = i63.add(i63);
    UInt192 i129 = i64.add(i64);
    assertThat(i64).isGreaterThan(i63); // In case something has gone horribly wrong.
    assertThat(i64.add(i63)).isGreaterThan(i64);
    assertThat(i129).isGreaterThan(i64.add(i63));
  }

  @Test
  public void when_comparing_int192_values_using_equals__the_correct_value_is_returned() {
    assertNotEquals(UInt192.ZERO, null); // Nothing should be equal to null
    assertEquals(UInt192.ZERO, UInt192.ZERO); // Same object check
    UInt192 i63a = UInt192.from(UInt64.ZERO, UInt128.HIGH_BIT);
    UInt192 i63b = UInt192.from(UInt64.ZERO, UInt128.HIGH_BIT);
    assertEquals(i63a, i63b);
    assertNotEquals(i63a, i63b.add(i63b));
    assertNotEquals(i63a, UInt192.ZERO);
  }

  @Test
  public void when_binary_oring_two_values__the_correct_value_is_returned() {
    // Use bit positions in both high and low word for tests
    UInt192 b0 = UInt192.ONE;
    UInt192 b64 = UInt192.from(UInt64.ONE, UInt128.ZERO);
    // Basic sanity checks to make sure nothing is horribly wrong
    assertThat(b64).isGreaterThan(b0);
    assertThat(b0).isGreaterThan(UInt192.ZERO);
    assertThat(b64).isGreaterThan(UInt192.ZERO);

    // Now for the real tests
    assertEquals(b0, UInt192.ZERO.or(b0));
    assertEquals(b0, b0.or(b0));
    assertEquals(b64, UInt192.ZERO.or(b64));
    assertEquals(b64, b64.or(b64));
    assertEquals(UInt192.MAX_VALUE, UInt192.ZERO.or(UInt192.MAX_VALUE));
    assertEquals(UInt192.MAX_VALUE, UInt192.MAX_VALUE.or(UInt192.MAX_VALUE));
  }

  @Test
  public void when_creating_int192_from_byte_array__the_correct_value_is_created() {
    byte[] m1 = {-1};
    byte[] p1 = {1};
    byte[] bytesArray = new byte[UInt192.BYTES];
    Arrays.fill(bytesArray, (byte) 0);
    bytesArray[UInt192.BYTES - 1] = 1;
    UInt192 m1Bits64 = UInt192.fromBigEndianBytes(m1);
    UInt192 p1Bits64 = UInt192.fromBigEndianBytes(p1);
    UInt192 bytesArrayBits64 = UInt192.fromBigEndianBytes(bytesArray);

    assertEquals(UInt192.from(255), m1Bits64); // Sign extension did not happen
    assertEquals(UInt192.ONE, p1Bits64); // Zero fill happened correctly
    assertEquals(UInt192.ONE, bytesArrayBits64); // Correct size array OK
  }

  @Test
  public void when_converting_int192_to_byte_array__the_correct_values_are_returned() {
    UInt64 bp0 = UInt64.fromNonNegativeLong(0x0001_0203_0405_0607L);
    UInt128 bp1 =
        UInt128.from(
            UInt64.fromNonNegativeLong(0x0809_0a0b_0c0d_0e0fL),
            UInt64.fromNonNegativeLong(0x1011_1213_1415_1617L));
    UInt192 bitPattern = UInt192.from(bp0, bp1);
    byte[] bytes2 = new byte[UInt192.BYTES * 3];
    Arrays.fill(bytes2, (byte) -1);

    // Make sure we got the value in big-endian order
    byte[] bytes = bitPattern.toBigEndianBytes();
    for (int i = 0; i < UInt192.BYTES; ++i) {
      assertEquals(i, bytes[i]);
    }

    bitPattern.toBigEndianBytes(bytes2, UInt192.BYTES);
    // Make sure we didn't overwrite bytes outside our range
    for (int i = 0; i < UInt192.BYTES; ++i) {
      assertEquals(-1, bytes2[i]);
      assertEquals(-1, bytes2[i + UInt192.BYTES * 2]);
    }
    // Make sure we got the value in big-endian order
    for (int i = 0; i < UInt192.BYTES; ++i) {
      assertEquals(i, bytes2[UInt192.BYTES + i]);
    }
  }

  @Test
  public void when_performing_binary_shifts__the_correct_value_is_returned() {
    final UInt128 minusTwo = UInt128.ZERO.decrement().decrement();
    final UInt64 maxSigned = UInt64.HIGH_BIT.decrement();

    // Basic cases, left shift
    assertEquals(UInt192.ZERO, UInt192.ZERO.shiftLeft());
    // Zero extend on left
    assertEquals(UInt192.from(UInt64.MAX_VALUE, minusTwo), UInt192.MAX_VALUE.shiftLeft());
    assertEquals(UInt192.from(2), UInt192.ONE.shiftLeft());
    // Make sure bit crosses word boundary correctly
    assertEquals(
        UInt192.from(UInt64.ONE, UInt128.ZERO),
        UInt192.from(UInt64.ZERO, UInt128.HIGH_BIT).shiftLeft());

    // Basic cases, right shift
    assertEquals(UInt192.ZERO, UInt192.ZERO.shiftRight());
    // Zeros inserted at right
    assertEquals(UInt192.from(maxSigned, UInt128.MAX_VALUE), UInt192.MAX_VALUE.shiftRight());
    assertEquals(UInt192.ZERO, UInt192.ONE.shiftRight());
    assertEquals(UInt192.ONE, UInt192.from(2).shiftRight());
    // Make sure bit crosses word boundary correctly
    assertEquals(
        UInt192.from(UInt64.ZERO, UInt128.HIGH_BIT),
        UInt192.from(UInt64.ONE, UInt128.ZERO).shiftRight());
  }

  @Test
  public void when_using_predicates__the_correct_value_is_returned() {
    // Basic tests for odd/even
    assertTrue(UInt192.ONE.isOdd());
    assertTrue(UInt192.MAX_VALUE.isOdd());
    UInt192 two = UInt192.ONE.add(UInt192.ONE);
    assertFalse(two.isOdd());
    UInt192 minusTwo = UInt192.MAX_VALUE.add(UInt192.MAX_VALUE);
    assertFalse(minusTwo.isOdd());

    assertFalse(UInt192.ONE.isZero());
    assertFalse(UInt192.MAX_VALUE.isZero());
    assertTrue(UInt192.ZERO.isZero());
  }

  @Test
  public void when_converting_int192_to_string__the_correct_value_is_returned() {
    // Some basics
    assertEquals("0", UInt192.ZERO.toString());
    assertEquals("1", UInt192.ONE.toString());
    assertEquals("10", UInt192.TEN.toString());

    assertEquals("12345678", UInt192.from(12345678L).toString());
  }

  @Test
  public void when_converting_string_to_int192__the_correct_value_is_returned() {
    testRoundTrip("0");
    testRoundTrip("123456789");
    testRoundTrip("123456789123456789");
    testRoundTrip("123456789123456789123456789123456789");
    assertEquals(
        UInt192.from(UInt64.MAX_VALUE, UInt128.MAX_VALUE),
        UInt192.from(BigInteger.ONE.shiftLeft(192).subtract(BigInteger.ONE).toString()));
  }

  @Test
  public void when_calculating_powers__the_correct_value_is_returned() {
    assertEquals(UInt192.from(1L << 9), UInt192.TWO.pow(9));
    assertEquals(UInt192.from(10_000_000_000L), UInt192.TEN.pow(10));
    assertEquals(UInt192.ONE, UInt192.ZERO.pow(0)); // At least in the limit
    assertEquals(UInt192.ONE, UInt192.ONE.pow(0));
  }

  @Test
  public void equalsContract() {
    EqualsVerifier.forClass(UInt192.class).verify();
  }

  /** Test div 0. */
  @Test(expected = IllegalArgumentException.class)
  public void testDiv0() {
    UInt192.ONE.divide(UInt192.ZERO);
  }

  /** Test rem 0. */
  @Test(expected = IllegalArgumentException.class)
  public void testRem0() {
    UInt192.ONE.remainder(UInt192.ZERO);
  }

  /** NumberFormatException on empty string. */
  @Test(expected = NumberFormatException.class)
  public void numberFormatExceptionOnEmpty() {
    UInt192.from("");
  }

  /** NumberFormatException if no actual number. */
  @Test(expected = NumberFormatException.class)
  public void numberFormatExceptionIfNoNumber() {
    UInt192.from("+");
  }

  /** NumberFormatException if invalid digit. */
  @Test(expected = NumberFormatException.class)
  public void numberFormatExceptionIfInvalidDigit() {
    UInt192.from("+a");
  }

  /** IllegalArgumentException if byte array is empty. */
  @Test(expected = IllegalArgumentException.class)
  public void illegalArgumentExceptionIfByteArrayEmpty() {
    UInt192.fromBigEndianBytes(new byte[0]);
  }

  /** IllegalArgumentException on radix too big. */
  @Test(expected = IllegalArgumentException.class)
  public void illegalArgumentExceptionOnRadixTooBig() {
    UInt192.ONE.toString(Character.MAX_RADIX + 1);
  }

  /** IllegalArgumentException on radix too small. */
  @Test(expected = IllegalArgumentException.class)
  public void illegalArgumentExceptionOnRadixTooSmall() {
    UInt192.ONE.toString(Character.MIN_RADIX - 1);
  }

  /** IllegalArgumentException on negative exponent for pow. */
  @Test(expected = IllegalArgumentException.class)
  public void illegalArgumentExceptionOnNegativeExponent() {
    UInt192.ONE.pow(-1);
  }

  /** Test some concrete large values div/mul */
  @Test
  public void test_large_number_div_mul() {
    assertEquals(UInt192.MAX_VALUE.divide(UInt192.MAX_VALUE), UInt192.ONE);
    assertEquals(UInt192.ONE.divide(UInt192.ONE), UInt192.ONE);
    assertEquals(UInt192.ONE.divide(UInt192.TWO), UInt192.ZERO);
    assertEquals(UInt192.ONE.divide(UInt192.MAX_VALUE), UInt192.ZERO);
    assertEquals(
        UInt192.MAX_VALUE.toBigInt(),
        new BigInteger("6277101735386680763835789423207666416102355444464034512895"));
    assertEquals(
        UInt192.MAX_VALUE.divide(UInt192.TWO).toBigInt(),
        new BigInteger("3138550867693340381917894711603833208051177722232017256447"));
    assertEquals(
        UInt192.MAX_VALUE.divide(UInt192.FIVE),
        UInt192.from("1255420347077336152767157884641533283220471088892806902579"));
    assertEquals(
        UInt192.MAX_VALUE.divide(UInt192.FIVE).multiply(UInt192.THREE),
        UInt192.from("3766261041232008458301473653924599849661413266678420707737"));
    assertEquals(
        UInt192.from("3138550867693340381917894711603833208051177722232017256448")
            .divide(UInt192.FOUR),
        UInt192.from("784637716923335095479473677900958302012794430558004314112"));
  }

  /** Test some concrete large values add/sub */
  @Test
  public void test_large_number_add_sub() {
    final var u1 = UInt192.from("3766261041232008458301473653924599849661413266678420707737");
    // An overflow
    assertEquals(
        u1.add(u1), UInt192.from("1255420347077336152767157884641533283220471088892806902578"));

    // A transient overflow
    assertEquals(UInt192.MAX_VALUE.add(UInt192.ONE).subtract(UInt192.ONE), UInt192.MAX_VALUE);

    assertEquals(
        UInt192.from("6277101735386680763835789423207666416102355444464034512895")
            .subtract(UInt192.from("6277101735386680763835789423207666416102355444464034512894")),
        UInt192.ONE);
  }

  private static void testRoundTrip(String s) {
    assertEquals(s, UInt192.from(s).toString());
  }

  private static void assertEqualToLong(long expectedValue, UInt192 testValue) {
    assertEquals(UInt64.ZERO, testValue.getHigh());
    assertEquals(0, testValue.getLow().getHigh());
    assertEquals(expectedValue, testValue.getLow().getLow());
  }
}
