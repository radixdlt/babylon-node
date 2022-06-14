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

package com.radixdlt.sbor.codec;

import com.radixdlt.lang.Functions;
import com.radixdlt.sbor.codec.FieldsEncoders.*;
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.sbor.coding.DecoderApi;
import com.radixdlt.sbor.coding.EncoderApi;
import com.radixdlt.sbor.exceptions.SborDecodeException;

@SuppressWarnings("unused")
public interface UntypedCodec<T> {
  void encodeWithoutTypeId(EncoderApi encoder, T value);

  T decodeWithoutTypeId(DecoderApi decoder);

  default Codec<T> addType(TypeId typeId) {
    return Codec.from(typeId, this);
  }

  static <T> UntypedCodec<T> of(
      EncoderWithoutTypeId<T> valueEncoder, DecoderWithoutTypeId<T> valueDecoder) {
    return new CompositeUntypedCodec<>(valueEncoder, valueDecoder);
  }

  @FunctionalInterface
  interface EncoderWithoutTypeId<T> {
    void encodeWithoutTypeId(EncoderApi encoder, T value);
  }

  @FunctionalInterface
  interface DecoderWithoutTypeId<T> {
    T decodeWithoutTypeId(DecoderApi decoder);
  }

  record CompositeUntypedCodec<T>(
      EncoderWithoutTypeId<T> valueEncoder, DecoderWithoutTypeId<T> valueDecoder)
      implements UntypedCodec<T> {
    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, T value) {
      valueEncoder.encodeWithoutTypeId(encoder, value);
    }

    @Override
    public T decodeWithoutTypeId(DecoderApi decoder) {
      return valueDecoder.decodeWithoutTypeId(decoder);
    }
  }

  static <T> UntypedCodec<T> emptyWithLength(Functions.Func0<T> creator) {
    return UntypedCodec.of(
        (encoder, value) -> encoder.writeInt(0),
        decoder -> {
          expectFieldCount(decoder, 0);
          return creator.apply();
        });
  }

  static <T, T1> UntypedCodec<T> wrapWithLength(
      Functions.Func1<T, T1> wrap, Codec<T1> codec1, Functions.Func1<T1, T> unwrap) {
    return UntypedCodec.of(
        (encoder, value) -> {
          encoder.writeInt(1);
          codec1.encodeWithTypeId(encoder, wrap.apply(value));
        },
        decoder -> {
          expectFieldCount(decoder, 1);
          return unwrap.apply(codec1.decodeWithTypeId(decoder));
        });
  }

  /** Allows separation of a type into separate fields. */
  @FunctionalInterface
  interface Separator<T, TFieldsEncoder> {
    void accept(T t, TFieldsEncoder encoder);
  }

  static <T> UntypedCodec<T> fromWithLength(
      Functions.Func0<T> creator, Separator<T, FieldsEncoder0> separator) {
    return UntypedCodec.of(
        (encoder, value) -> separator.accept(value, () -> encoder.writeInt(0)),
        decoder -> {
          expectFieldCount(decoder, 0);
          return creator.apply();
        });
  }

  static <T, T1> UntypedCodec<T> fromWithLength(
      Functions.Func1<T1, T> creator,
      Codec<T1> codec1,
      Separator<T, FieldsEncoder1<T1>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1) -> {
                  encoder.writeInt(1);
                  codec1.encodeWithTypeId(encoder, param1);
                }),
        decoder -> {
          expectFieldCount(decoder, 1);
          return creator.apply(codec1.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2> UntypedCodec<T> fromWithLength(
      Functions.Func2<T1, T2, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Separator<T, FieldsEncoder2<T1, T2>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2) -> {
                  encoder.writeInt(2);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                }),
        decoder -> {
          expectFieldCount(decoder, 2);
          return creator.apply(codec1.decodeWithTypeId(decoder), codec2.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3> UntypedCodec<T> fromWithLength(
      Functions.Func3<T1, T2, T3, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Separator<T, FieldsEncoder3<T1, T2, T3>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3) -> {
                  encoder.writeInt(3);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                }),
        decoder -> {
          expectFieldCount(decoder, 3);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4> UntypedCodec<T> fromWithLength(
      Functions.Func4<T1, T2, T3, T4, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Separator<T, FieldsEncoder4<T1, T2, T3, T4>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4) -> {
                  encoder.writeInt(4);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                }),
        decoder -> {
          expectFieldCount(decoder, 4);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5> UntypedCodec<T> fromWithLength(
      Functions.Func5<T1, T2, T3, T4, T5, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Separator<T, FieldsEncoder5<T1, T2, T3, T4, T5>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5) -> {
                  encoder.writeInt(5);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                }),
        decoder -> {
          expectFieldCount(decoder, 5);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6> UntypedCodec<T> fromWithLength(
      Functions.Func6<T1, T2, T3, T4, T5, T6, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Separator<T, FieldsEncoder6<T1, T2, T3, T4, T5, T6>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6) -> {
                  encoder.writeInt(6);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                }),
        decoder -> {
          expectFieldCount(decoder, 6);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7> UntypedCodec<T> fromWithLength(
      Functions.Func7<T1, T2, T3, T4, T5, T6, T7, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Separator<T, FieldsEncoder7<T1, T2, T3, T4, T5, T6, T7>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7) -> {
                  encoder.writeInt(7);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                }),
        decoder -> {
          expectFieldCount(decoder, 7);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8> UntypedCodec<T> fromWithLength(
      Functions.Func8<T1, T2, T3, T4, T5, T6, T7, T8, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Separator<T, FieldsEncoder8<T1, T2, T3, T4, T5, T6, T7, T8>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7, param8) -> {
                  encoder.writeInt(8);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                }),
        decoder -> {
          expectFieldCount(decoder, 8);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder),
              codec8.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9> UntypedCodec<T> fromWithLength(
      Functions.Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Codec<T9> codec9,
      Separator<T, FieldsEncoder9<T1, T2, T3, T4, T5, T6, T7, T8, T9>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7, param8, param9) -> {
                  encoder.writeInt(9);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                }),
        decoder -> {
          expectFieldCount(decoder, 9);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder),
              codec8.decodeWithTypeId(decoder),
              codec9.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> UntypedCodec<T> fromWithLength(
      Functions.Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T> creator,
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
      Separator<T, FieldsEncoder10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10) -> {
                  encoder.writeInt(10);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                }),
        decoder -> {
          expectFieldCount(decoder, 10);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder),
              codec8.decodeWithTypeId(decoder),
              codec9.decodeWithTypeId(decoder),
              codec10.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> UntypedCodec<T> fromWithLength(
      Functions.Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T> creator,
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
      Separator<T, FieldsEncoder11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10,
                    param11) -> {
                  encoder.writeInt(11);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                  codec11.encodeWithTypeId(encoder, param11);
                }),
        decoder -> {
          expectFieldCount(decoder, 11);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder),
              codec8.decodeWithTypeId(decoder),
              codec9.decodeWithTypeId(decoder),
              codec10.decodeWithTypeId(decoder),
              codec11.decodeWithTypeId(decoder));
        });
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> UntypedCodec<T> fromWithLength(
      Functions.Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T> creator,
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
      Codec<T12> codec12,
      Separator<T, FieldsEncoder12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10,
                    param11,
                    param12) -> {
                  encoder.writeInt(12);
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                  codec11.encodeWithTypeId(encoder, param11);
                  codec12.encodeWithTypeId(encoder, param12);
                }),
        decoder -> {
          expectFieldCount(decoder, 12);
          return creator.apply(
              codec1.decodeWithTypeId(decoder),
              codec2.decodeWithTypeId(decoder),
              codec3.decodeWithTypeId(decoder),
              codec4.decodeWithTypeId(decoder),
              codec5.decodeWithTypeId(decoder),
              codec6.decodeWithTypeId(decoder),
              codec7.decodeWithTypeId(decoder),
              codec8.decodeWithTypeId(decoder),
              codec9.decodeWithTypeId(decoder),
              codec10.decodeWithTypeId(decoder),
              codec11.decodeWithTypeId(decoder),
              codec12.decodeWithTypeId(decoder));
        });
  }

  private static void expectFieldCount(DecoderApi decoder, int expected) {
    var actual = decoder.readInt();
    if (expected != actual) {
      throw new SborDecodeException(
          String.format("Expected to have %s fields, but there were %s", expected, actual));
    }
  }

  static <T> UntypedCodec<T> emptyWithoutLength(Functions.Func0<T> creator) {
    return UntypedCodec.of((encoder, value) -> {}, decoder -> creator.apply());
  }

  static <T, T1> UntypedCodec<T> wrapWithoutLength(
      Functions.Func1<T, T1> wrap, Codec<T1> codec1, Functions.Func1<T1, T> unwrap) {
    return UntypedCodec.of(
        (encoder, value) -> codec1.encodeWithTypeId(encoder, wrap.apply(value)),
        decoder -> unwrap.apply(codec1.decodeWithTypeId(decoder)));
  }

  static <T> UntypedCodec<T> fromWithoutLength(
      Functions.Func0<T> creator, Separator<T, FieldsEncoder0> separator) {
    return UntypedCodec.of(
        (encoder, value) -> separator.accept(value, () -> {}), decoder -> creator.apply());
  }

  static <T, T1> UntypedCodec<T> fromWithoutLength(
      Functions.Func1<T1, T> creator,
      Codec<T1> codec1,
      Separator<T, FieldsEncoder1<T1>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(value, (param1) -> codec1.encodeWithTypeId(encoder, param1)),
        decoder -> creator.apply(codec1.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2> UntypedCodec<T> fromWithoutLength(
      Functions.Func2<T1, T2, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Separator<T, FieldsEncoder2<T1, T2>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                }),
        decoder ->
            creator.apply(codec1.decodeWithTypeId(decoder), codec2.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3> UntypedCodec<T> fromWithoutLength(
      Functions.Func3<T1, T2, T3, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Separator<T, FieldsEncoder3<T1, T2, T3>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4> UntypedCodec<T> fromWithoutLength(
      Functions.Func4<T1, T2, T3, T4, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Separator<T, FieldsEncoder4<T1, T2, T3, T4>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5> UntypedCodec<T> fromWithoutLength(
      Functions.Func5<T1, T2, T3, T4, T5, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Separator<T, FieldsEncoder5<T1, T2, T3, T4, T5>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6> UntypedCodec<T> fromWithoutLength(
      Functions.Func6<T1, T2, T3, T4, T5, T6, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Separator<T, FieldsEncoder6<T1, T2, T3, T4, T5, T6>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7> UntypedCodec<T> fromWithoutLength(
      Functions.Func7<T1, T2, T3, T4, T5, T6, T7, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Separator<T, FieldsEncoder7<T1, T2, T3, T4, T5, T6, T7>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8> UntypedCodec<T> fromWithoutLength(
      Functions.Func8<T1, T2, T3, T4, T5, T6, T7, T8, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Separator<T, FieldsEncoder8<T1, T2, T3, T4, T5, T6, T7, T8>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7, param8) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder),
                codec8.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9> UntypedCodec<T> fromWithoutLength(
      Functions.Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, T> creator,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Codec<T9> codec9,
      Separator<T, FieldsEncoder9<T1, T2, T3, T4, T5, T6, T7, T8, T9>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1, param2, param3, param4, param5, param6, param7, param8, param9) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder),
                codec8.decodeWithTypeId(decoder),
                codec9.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> UntypedCodec<T> fromWithoutLength(
      Functions.Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T> creator,
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
      Separator<T, FieldsEncoder10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder),
                codec8.decodeWithTypeId(decoder),
                codec9.decodeWithTypeId(decoder),
                codec10.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> UntypedCodec<T> fromWithoutLength(
      Functions.Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T> creator,
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
      Separator<T, FieldsEncoder11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10,
                    param11) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                  codec11.encodeWithTypeId(encoder, param11);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder),
                codec8.decodeWithTypeId(decoder),
                codec9.decodeWithTypeId(decoder),
                codec10.decodeWithTypeId(decoder),
                codec11.decodeWithTypeId(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> UntypedCodec<T> fromWithoutLength(
      Functions.Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T> creator,
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
      Codec<T12> codec12,
      Separator<T, FieldsEncoder12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>> separator) {
    return UntypedCodec.of(
        (encoder, value) ->
            separator.accept(
                value,
                (param1,
                    param2,
                    param3,
                    param4,
                    param5,
                    param6,
                    param7,
                    param8,
                    param9,
                    param10,
                    param11,
                    param12) -> {
                  codec1.encodeWithTypeId(encoder, param1);
                  codec2.encodeWithTypeId(encoder, param2);
                  codec3.encodeWithTypeId(encoder, param3);
                  codec4.encodeWithTypeId(encoder, param4);
                  codec5.encodeWithTypeId(encoder, param5);
                  codec6.encodeWithTypeId(encoder, param6);
                  codec7.encodeWithTypeId(encoder, param7);
                  codec8.encodeWithTypeId(encoder, param8);
                  codec9.encodeWithTypeId(encoder, param9);
                  codec10.encodeWithTypeId(encoder, param10);
                  codec11.encodeWithTypeId(encoder, param11);
                  codec12.encodeWithTypeId(encoder, param12);
                }),
        decoder ->
            creator.apply(
                codec1.decodeWithTypeId(decoder),
                codec2.decodeWithTypeId(decoder),
                codec3.decodeWithTypeId(decoder),
                codec4.decodeWithTypeId(decoder),
                codec5.decodeWithTypeId(decoder),
                codec6.decodeWithTypeId(decoder),
                codec7.decodeWithTypeId(decoder),
                codec8.decodeWithTypeId(decoder),
                codec9.decodeWithTypeId(decoder),
                codec10.decodeWithTypeId(decoder),
                codec11.decodeWithTypeId(decoder),
                codec12.decodeWithTypeId(decoder)));
  }
}
