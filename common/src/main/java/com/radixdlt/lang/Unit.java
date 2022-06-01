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

package com.radixdlt.lang;

import java.util.function.Supplier;

/**
 * From https://en.wikipedia.org/wiki/Unit_type :
 *
 * <blockquote>
 *
 * In the area of mathematical logic and computer science known as type theory, a unit type is a
 * type that allows only one value (and thus can hold no information). The carrier (underlying set)
 * associated with a unit type can be any singleton set. There is an isomorphism between any two
 * such sets, so it is customary to talk about the unit type and ignore the details of its value.
 * One may also regard the unit type as the type of 0-tuples, i.e. the product of no types.
 *
 * <p>The unit type is the terminal object in the category of types and typed functions. It should
 * not be confused with the zero or bottom type, which allows no values and is the initial object in
 * this category. Similarly, the Boolean is the type with two values.
 *
 * <p>The unit type is implemented in most functional programming languages. The void type that is
 * used in some imperative programming languages serves some of its functions, but because its
 * carrier set is empty, it has some limitations.
 *
 * </blockquote>
 */
public final class Unit implements Tuple.Tuple0 {
  private Unit() {}

  private static final Unit UNIT = new Unit();
  private static final Result<Unit> UNIT_RESULT = Result.success(UNIT);

  public static Unit unit() {
    return UNIT;
  }

  public static <T1, T2> Unit unit(final T1 ignored1, final T2 ignored2) {
    return UNIT;
  }

  public static <T1, T2, T3> Unit unit(final T1 ignored1, final T2 ignored2, final T3 ignored3) {
    return UNIT;
  }

  public static <T1, T2, T3, T4> Unit unit(
      final T1 ignored1, final T2 ignored2, final T3 ignored3, final T4 ignored4) {
    return UNIT;
  }

  public static <T1, T2, T3, T4, T5> Unit unit(
      final T1 ignored1,
      final T2 ignored2,
      final T3 ignored3,
      final T4 ignored4,
      final T5 ignored5) {
    return UNIT;
  }

  public static <T1, T2, T3, T4, T5, T6> Unit unit(
      final T1 ignored1,
      final T2 ignored2,
      final T3 ignored3,
      final T4 ignored4,
      final T5 ignored5,
      final T6 ignored6) {
    return UNIT;
  }

  public static <T1, T2, T3, T4, T5, T6, T7> Unit unit(
      final T1 ignored1,
      final T2 ignored2,
      final T3 ignored3,
      final T4 ignored4,
      final T5 ignored5,
      final T6 ignored6,
      final T7 ignored7) {
    return UNIT;
  }

  public static <T1, T2, T3, T4, T5, T6, T7, T8> Unit unit(
      final T1 ignored1,
      final T2 ignored2,
      final T3 ignored3,
      final T4 ignored4,
      final T5 ignored5,
      final T6 ignored6,
      final T7 ignored7,
      final T8 ignored8) {
    return UNIT;
  }

  public static <T1, T2, T3, T4, T5, T6, T7, T8, T9> Unit unit(
      final T1 ignored1,
      final T2 ignored2,
      final T3 ignored3,
      final T4 ignored4,
      final T5 ignored5,
      final T6 ignored6,
      final T7 ignored7,
      final T8 ignored8,
      final T9 ignored9) {
    return UNIT;
  }

  public static Result<Unit> unitResult() {
    return UNIT_RESULT;
  }

  @Override
  public String toString() {
    return "()";
  }

  @Override
  public boolean equals(Object obj) {
    return obj instanceof Tuple0;
  }

  @Override
  public int hashCode() {
    return super.hashCode();
  }

  @Override
  public <T> T map(Supplier<T> mapper) {
    return mapper.get();
  }
}
