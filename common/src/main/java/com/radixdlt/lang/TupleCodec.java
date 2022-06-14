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

import static com.radixdlt.sbor.codec.constants.TypeId.TYPE_TUPLE;

import com.radixdlt.lang.Tuple.*;
import com.radixdlt.sbor.codec.Codec;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.TypeTokenUtils;
import com.radixdlt.sbor.codec.UntypedCodec;
import com.radixdlt.sbor.exceptions.SborCodecException;

@SuppressWarnings("unused")
public interface TupleCodec {

  static <T> Codec<T> from(UntypedCodec<T> untypedCodec) {
    return untypedCodec.addType(TYPE_TUPLE);
  }

  static Codec<Tuple0> ofEmpty() {
    return from(UntypedCodec.emptyWithLength(Tuple0::of));
  }

  static <T1> Codec<Tuple1<T1>> of(Codec<T1> codec1) {
    return from(UntypedCodec.fromWithLength(Tuple1::of, codec1, (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2> Codec<Tuple2<T1, T2>> of(Codec<T1> codec1, Codec<T2> codec2) {
    return from(
        UntypedCodec.fromWithLength(Tuple2::of, codec1, codec2, (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3> Codec<Tuple3<T1, T2, T3>> of(
      Codec<T1> codec1, Codec<T2> codec2, Codec<T3> codec3) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple3::of, codec1, codec2, codec3, (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4> Codec<Tuple4<T1, T2, T3, T4>> of(
      Codec<T1> codec1, Codec<T2> codec2, Codec<T3> codec3, Codec<T4> codec4) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple4::of, codec1, codec2, codec3, codec4, (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5> Codec<Tuple5<T1, T2, T3, T4, T5>> of(
      Codec<T1> codec1, Codec<T2> codec2, Codec<T3> codec3, Codec<T4> codec4, Codec<T5> codec5) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple5::of, codec1, codec2, codec3, codec4, codec5, (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6> Codec<Tuple6<T1, T2, T3, T4, T5, T6>> of(
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple6::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7> Codec<Tuple7<T1, T2, T3, T4, T5, T6, T7>> of(
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple7::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8> Codec<Tuple8<T1, T2, T3, T4, T5, T6, T7, T8>> of(
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple8::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            codec8,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9> Codec<Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9>> of(
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Codec<T9> codec9) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple9::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            codec8,
            codec9,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
      Codec<Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>> of(
          Codec<T1> codec1,
          Codec<T2> codec2,
          Codec<T3> codec3,
          Codec<T4> codec4,
          Codec<T5> codec5,
          Codec<T6> codec6,
          Codec<T7> codec7,
          Codec<T8> codec8,
          Codec<T9> codec9,
          Codec<T10> codec10) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple10::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            codec8,
            codec9,
            codec10,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
      Codec<Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>> of(
          Codec<T1> codec1,
          Codec<T2> codec2,
          Codec<T3> codec3,
          Codec<T4> codec4,
          Codec<T5> codec5,
          Codec<T6> codec6,
          Codec<T7> codec7,
          Codec<T8> codec8,
          Codec<T9> codec9,
          Codec<T10> codec10,
          Codec<T11> codec11) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple11::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            codec8,
            codec9,
            codec10,
            codec11,
            (t, s) -> t.accept(s::encode)));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
      Codec<Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>> of(
          Codec<T1> codec1,
          Codec<T2> codec2,
          Codec<T3> codec3,
          Codec<T4> codec4,
          Codec<T5> codec5,
          Codec<T6> codec6,
          Codec<T7> codec7,
          Codec<T8> codec8,
          Codec<T9> codec9,
          Codec<T10> codec10,
          Codec<T11> codec11,
          Codec<T12> codec12) {
    return from(
        UntypedCodec.fromWithLength(
            Tuple12::of,
            codec1,
            codec2,
            codec3,
            codec4,
            codec5,
            codec6,
            codec7,
            codec8,
            codec9,
            codec10,
            codec11,
            codec12,
            (t, s) -> t.accept(s::encode)));
  }

  static void registerAllWith(CodecMap codecMap) {
    codecMap.registerForGeneric(
        Tuple0.class,
        (codecs, tupleType) -> {
          try {
            return ofEmpty();
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple1.class,
        (codecs, tupleType) -> {
          try {
            return of(codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple2.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple3.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple4.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple5.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple6.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple7.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple8.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 7)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple9.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 7)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 8)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple10.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 7)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 8)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 9)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple11.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 7)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 8)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 9)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 10)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });

    codecMap.registerForGeneric(
        Tuple12.class,
        (codecs, tupleType) -> {
          try {
            return of(
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 0)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 1)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 2)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 3)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 4)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 5)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 6)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 7)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 8)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 9)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 10)),
                codecs.of(TypeTokenUtils.getGenericTypeParameter(tupleType, 11)));
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Tuple type codec for %s", tupleType), ex);
          }
        });
  }
}
