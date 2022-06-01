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

import com.radixdlt.lang.Functions.*;
import java.util.function.Supplier;

/** Tuples of various size (0-9). */
public interface Tuple {
  int size();

  interface Tuple0 extends Tuple {
    <T> T map(Supplier<T> mapper);

    default int size() {
      return 0;
    }
  }

  interface Tuple1<T1> extends Tuple {
    <T> T map(FN1<T, T1> mapper);

    default int size() {
      return 1;
    }
  }

  interface Tuple2<T1, T2> extends Tuple {
    <T> T map(FN2<T, T1, T2> mapper);

    default int size() {
      return 2;
    }

    T1 first();

    T2 last();
  }

  interface Tuple3<T1, T2, T3> extends Tuple {
    <T> T map(FN3<T, T1, T2, T3> mapper);

    default int size() {
      return 3;
    }
  }

  interface Tuple4<T1, T2, T3, T4> extends Tuple {
    <T> T map(FN4<T, T1, T2, T3, T4> mapper);

    default int size() {
      return 4;
    }
  }

  interface Tuple5<T1, T2, T3, T4, T5> extends Tuple {
    <T> T map(FN5<T, T1, T2, T3, T4, T5> mapper);

    default int size() {
      return 5;
    }
  }

  interface Tuple6<T1, T2, T3, T4, T5, T6> extends Tuple {
    <T> T map(FN6<T, T1, T2, T3, T4, T5, T6> mapper);

    default int size() {
      return 6;
    }
  }

  interface Tuple7<T1, T2, T3, T4, T5, T6, T7> extends Tuple {
    <T> T map(FN7<T, T1, T2, T3, T4, T5, T6, T7> mapper);

    default int size() {
      return 7;
    }
  }

  interface Tuple8<T1, T2, T3, T4, T5, T6, T7, T8> extends Tuple {
    <T> T map(FN8<T, T1, T2, T3, T4, T5, T6, T7, T8> mapper);

    default int size() {
      return 8;
    }
  }

  interface Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9> extends Tuple {
    <T> T map(FN9<T, T1, T2, T3, T4, T5, T6, T7, T8, T9> mapper);

    default int size() {
      return 9;
    }
  }

  interface Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> extends Tuple {
    <T> T map(FN10<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> mapper);

    default int size() {
      return 10;
    }
  }

  interface Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> extends Tuple {
    <T> T map(FN11<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> mapper);

    default int size() {
      return 11;
    }
  }

  interface Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> extends Tuple {
    <T> T map(FN12<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> mapper);

    default int size() {
      return 11;
    }
  }

  static Tuple0 tuple() {
    return Unit.unit();
  }

  static <T1> Tuple1<T1> tuple(T1 param1) {
    record tuple1<T1>(T1 param1) implements Tuple1<T1> {
      @Override
      public <T> T map(FN1<T, T1> mapper) {
        return mapper.apply(param1());
      }
    }

    return new tuple1<>(param1);
  }

  static <T1, T2> Tuple2<T1, T2> tuple(T1 param1, T2 param2) {
    record tuple2<T1, T2>(T1 param1, T2 param2) implements Tuple2<T1, T2> {
      @Override
      public <T> T map(FN2<T, T1, T2> mapper) {
        return mapper.apply(param1(), param2());
      }

      @Override
      public T1 first() {
        return param1();
      }

      @Override
      public T2 last() {
        return param2();
      }
    }

    return new tuple2<>(param1, param2);
  }

  static <T1, T2, T3> Tuple3<T1, T2, T3> tuple(T1 param1, T2 param2, T3 param3) {
    record tuple3<T1, T2, T3>(T1 param1, T2 param2, T3 param3) implements Tuple3<T1, T2, T3> {
      @Override
      public <T> T map(FN3<T, T1, T2, T3> mapper) {
        return mapper.apply(param1(), param2(), param3());
      }
    }

    return new tuple3<>(param1, param2, param3);
  }

  static <T1, T2, T3, T4> Tuple4<T1, T2, T3, T4> tuple(T1 param1, T2 param2, T3 param3, T4 param4) {
    record tuple4<T1, T2, T3, T4>(T1 param1, T2 param2, T3 param3, T4 param4)
        implements Tuple4<T1, T2, T3, T4> {
      @Override
      public <T> T map(FN4<T, T1, T2, T3, T4> mapper) {
        return mapper.apply(param1(), param2(), param3(), param4());
      }
    }

    return new tuple4<>(param1, param2, param3, param4);
  }

  static <T1, T2, T3, T4, T5> Tuple5<T1, T2, T3, T4, T5> tuple(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5) {
    record tuple5<T1, T2, T3, T4, T5>(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5)
        implements Tuple5<T1, T2, T3, T4, T5> {
      @Override
      public <T> T map(FN5<T, T1, T2, T3, T4, T5> mapper) {
        return mapper.apply(param1(), param2(), param3(), param4(), param5());
      }
    }

    return new tuple5<>(param1, param2, param3, param4, param5);
  }

  static <T1, T2, T3, T4, T5, T6> Tuple6<T1, T2, T3, T4, T5, T6> tuple(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6) {
    record tuple6<T1, T2, T3, T4, T5, T6>(
        T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6)
        implements Tuple6<T1, T2, T3, T4, T5, T6> {
      @Override
      public <T> T map(FN6<T, T1, T2, T3, T4, T5, T6> mapper) {
        return mapper.apply(param1(), param2(), param3(), param4(), param5(), param6());
      }
    }

    return new tuple6<>(param1, param2, param3, param4, param5, param6);
  }

  static <T1, T2, T3, T4, T5, T6, T7> Tuple7<T1, T2, T3, T4, T5, T6, T7> tuple(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7) {
    record tuple7<T1, T2, T3, T4, T5, T6, T7>(
        T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7)
        implements Tuple7<T1, T2, T3, T4, T5, T6, T7> {
      @Override
      public <T> T map(FN7<T, T1, T2, T3, T4, T5, T6, T7> mapper) {
        return mapper.apply(param1(), param2(), param3(), param4(), param5(), param6(), param7());
      }
    }

    return new tuple7<>(param1, param2, param3, param4, param5, param6, param7);
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8> Tuple8<T1, T2, T3, T4, T5, T6, T7, T8> tuple(
      T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8) {
    record tuple8<T1, T2, T3, T4, T5, T6, T7, T8>(
        T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8)
        implements Tuple8<T1, T2, T3, T4, T5, T6, T7, T8> {
      @Override
      public <T> T map(FN8<T, T1, T2, T3, T4, T5, T6, T7, T8> mapper) {
        return mapper.apply(
            param1(), param2(), param3(), param4(), param5(), param6(), param7(), param8());
      }
    }

    return new tuple8<>(param1, param2, param3, param4, param5, param6, param7, param8);
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9> Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9> tuple(
      T1 param1,
      T2 param2,
      T3 param3,
      T4 param4,
      T5 param5,
      T6 param6,
      T7 param7,
      T8 param8,
      T9 param9) {
    record tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9>(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9)
        implements Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9> {
      @Override
      public <T> T map(FN9<T, T1, T2, T3, T4, T5, T6, T7, T8, T9> mapper) {
        return mapper.apply(
            param1(), param2(), param3(), param4(), param5(), param6(), param7(), param8(),
            param9());
      }
    }

    return new tuple9<>(param1, param2, param3, param4, param5, param6, param7, param8, param9);
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
      Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> tuple(
          T1 param1,
          T2 param2,
          T3 param3,
          T4 param4,
          T5 param5,
          T6 param6,
          T7 param7,
          T8 param8,
          T9 param9,
          T10 param10) {
    record tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>(
        T1 param1,
        T2 param2,
        T3 param3,
        T4 param4,
        T5 param5,
        T6 param6,
        T7 param7,
        T8 param8,
        T9 param9,
        T10 param10)
        implements Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> {
      @Override
      public <T> T map(FN10<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> mapper) {
        return mapper.apply(
            param1(), param2(), param3(), param4(), param5(), param6(), param7(), param8(),
            param9(), param10());
      }
    }

    return new tuple10<>(
        param1, param2, param3, param4, param5, param6, param7, param8, param9, param10);
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
      Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> tuple(
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
          T11 param11) {
    record tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>(
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
        T11 param11)
        implements Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> {
      @Override
      public <T> T map(FN11<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> mapper) {
        return mapper.apply(
            param1(), param2(), param3(), param4(), param5(), param6(), param7(), param8(),
            param9(), param10(), param11());
      }
    }

    return new tuple11<>(
        param1, param2, param3, param4, param5, param6, param7, param8, param9, param10, param11);
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
      Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> tuple(
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
          T12 param12) {
    record tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>(
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
        T12 param12)
        implements Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
      @Override
      public <T> T map(FN12<T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> mapper) {
        return mapper.apply(
            param1(), param2(), param3(), param4(), param5(), param6(), param7(), param8(),
            param9(), param10(), param11(), param12());
      }
    }

    return new tuple12<>(
        param1, param2, param3, param4, param5, param6, param7, param8, param9, param10, param11,
        param12);
  }
}
