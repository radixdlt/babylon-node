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

/**
 * Collection of basic function interfaces and functions for various use cases. The naming of the
 * generic functional interfaces follows the C# naming convention of Func and Action.
 */
@SuppressWarnings("unused")
public interface Functions {
  /** Function with no parameters (supplier). Provided for consistency. */
  @FunctionalInterface
  interface Func0<R> {
    R apply();
  }

  /** Function with one parameter. */
  @FunctionalInterface
  interface Func1<T1, R> {
    R apply(T1 param1);

    default <N> Func1<T1, N> then(Func1<R, N> function) {
      return v1 -> function.apply(apply(v1));
    }

    default <N> Func1<N, R> before(Func1<N, T1> function) {
      return v1 -> apply(function.apply(v1));
    }

    static <T> Func1<T, T> id() {
      return Functions::id;
    }
  }

  /** Function with two parameters. */
  @FunctionalInterface
  interface Func2<T1, T2, R> {
    R apply(T1 param1, T2 param2);
  }

  /** Function with three parameters. */
  @FunctionalInterface
  interface Func3<T1, T2, T3, R> {
    R apply(T1 param1, T2 param2, T3 param3);
  }

  /** Function with four parameters. */
  @FunctionalInterface
  interface Func4<T1, T2, T3, T4, R> {
    R apply(T1 param1, T2 param2, T3 param3, T4 param4);
  }

  /** Function with five parameters. */
  @FunctionalInterface
  interface Func5<T1, T2, T3, T4, T5, R> {
    R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5);
  }

  /** Function with six parameters. */
  @FunctionalInterface
  interface Func6<T1, T2, T3, T4, T5, T6, R> {
    R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6);
  }

  /** Function with seven parameters. */
  @FunctionalInterface
  interface Func7<T1, T2, T3, T4, T5, T6, T7, R> {
    R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7);
  }

  /** Function with eight parameters. */
  @FunctionalInterface
  interface Func8<T1, T2, T3, T4, T5, T6, T7, T8, R> {
    R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8);
  }

  /** Function with nine parameters. */
  @FunctionalInterface
  interface Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, R> {
    R apply(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9);
  }

  @FunctionalInterface
  interface Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R> {
    R apply(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10);
  }

  @FunctionalInterface
  interface Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R> {
    R apply(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10,
        T11 param11);
  }

  @FunctionalInterface
  interface Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, R> {
    R apply(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10,
        T11 param11,
        T12 param12);
  }

  /** Universal identity function. */
  static <T> T id(T value) {
    return value;
  }

  /** Supplier which can throw an exception. */
  @FunctionalInterface
  interface ThrowingSupplier<T> {
    T get() throws Throwable;
  }

  /** Action with no parameters. */
  @FunctionalInterface
  interface Action0 {
    void accept();
  }

  /** Action with one parameter. */
  @FunctionalInterface
  interface Action1<T1> {
    void accept(T1 param1);
  }

  /** Action with two parameters. */
  @FunctionalInterface
  interface Action2<T1, T2> {
    void accept(T1 param1, T2 param2);
  }

  /** Action with three parameters. */
  @FunctionalInterface
  interface Action3<T1, T2, T3> {
    void accept(T1 param1, T2 param2, T3 param3);
  }

  /** Action with four parameters. */
  @FunctionalInterface
  interface Action4<T1, T2, T3, T4> {
    void accept(T1 param1, T2 param2, T3 param3, T4 param4);
  }

  /** Action with five parameters. */
  @FunctionalInterface
  interface Action5<T1, T2, T3, T4, T5> {
    void accept(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5);
  }

  /** Action with six parameters. */
  @FunctionalInterface
  interface Action6<T1, T2, T3, T4, T5, T6> {
    void accept(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6);
  }

  /** Action with seven parameters. */
  @FunctionalInterface
  interface Action7<T1, T2, T3, T4, T5, T6, T7> {
    void accept(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7);
  }

  /** Action with eight parameters. */
  @FunctionalInterface
  interface Action8<T1, T2, T3, T4, T5, T6, T7, T8> {
    void accept(
        T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8);
  }

  /** Action with nine parameters. */
  @FunctionalInterface
  interface Action9<T1, T2, T3, T4, T5, T6, T7, T8, T9> {
    void accept(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9);
  }

  /** Action with ten parameters. */
  @FunctionalInterface
  interface Action10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> {
    void accept(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10);
  }

  /** Action with eleven parameters. */
  @FunctionalInterface
  interface Action11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> {
    void accept(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10,
        T11 param11);
  }

  /** Action with twelve parameters. */
  @FunctionalInterface
  interface Action12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
    void accept(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10,
        T11 param11,
        T12 param12);
  }

  /** Function with variable argument list. */
  @FunctionalInterface
  interface FuncVar<R> {
    R apply(Object... values);
  }

  /**
   * Universal consumers of values which do nothing with input values. Useful for cases when API
   * requires function, but there is no need to do anything with the received values.
   */
  static void unitFn() {}

  static <T1> void unitFn(T1 value) {}

  static <T1, T2> void unitFn(T1 param1, T2 param2) {}

  static <T1, T2, T3> void unitFn(T1 param1, T2 param2, T3 param3) {}

  static <T1, T2, T3, T4> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4) {}

  static <T1, T2, T3, T4, T5> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5) {}

  static <T1, T2, T3, T4, T5, T6> void unitFn(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6) {}

  static <T1, T2, T3, T4, T5, T6, T7> void unitFn(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7) {}

  static <T1, T2, T3, T4, T5, T6, T7, T8> void unitFn(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8) {}

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9> void unitFn(
      T1 param1,
      T2 param2,
      T3 param3,
      T4 param4,
      T5 param5,
      T6 param6,
      T7 param7,
      T8 param8,
      T9 param9) {}

  static <R, T1> R toNull(T1 value) {
    return null;
  }

  static <T1> boolean toTrue(T1 value) {
    return true;
  }

  static <T1> boolean toFalse(T1 value) {
    return false;
  }
}
