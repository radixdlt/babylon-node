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
import com.radixdlt.sbor.codec.constants.TypeId;
import com.radixdlt.sbor.coding.DecoderApi;
import com.radixdlt.sbor.coding.EncoderApi;

@SuppressWarnings("unused")
public interface StructCodec<T> extends Codec<T> {
  @Override
  default TypeId getTypeId() {
    return TypeId.TYPE_STRUCT;
  }

  record StructFieldsCodec<T>(UntypedCodec<T> fields) implements StructCodec<T> {
    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, T value) {
      fields.encodeWithoutTypeId(encoder, value);
    }

    @Override
    public T decodeWithoutTypeId(DecoderApi decoder) {
      return fields.decodeWithoutTypeId(decoder);
    }
  }

  static <T> StructCodec<T> with(Fields<T> fields) {
    return new StructFieldsCodec<>(fields);
  }

  // Version 1 - Uses Fields
  // EXAMPLE
  /*
     codecMap.register(
       SimpleRecord.class,
       (codecs) ->
           StructCodec.fromFields(
               Field.of(SimpleRecord::first, codecs.of(int.class)),
               Field.of(SimpleRecord::second, codecs.of(String.class)),
               Field.of(SimpleRecord::third, codecs.of(new TypeToken<Either<Long, String>>() {})),
               Field.of(SimpleRecord::fourth, codecs.of(new TypeToken<Option<Boolean>>() {})),
               SimpleRecord::new));
  */

  static <T, T1> StructCodec<T> fromFields(Field<T, T1> field1, Functions.Func1<T1, T> creator) {
    return with(Fields.of(field1, creator));
  }

  static <T, T1, T2> StructCodec<T> fromFields(
      Field<T, T1> field1, Field<T, T2> field2, Functions.Func2<T1, T2, T> creator) {
    return with(Fields.of(field1, field2, creator));
  }

  static <T, T1, T2, T3> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Functions.Func3<T1, T2, T3, T> creator) {
    return with(Fields.of(field1, field2, field3, creator));
  }

  static <T, T1, T2, T3, T4> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Functions.Func4<T1, T2, T3, T4, T> creator) {
    return with(Fields.of(field1, field2, field3, field4, creator));
  }

  static <T, T1, T2, T3, T4, T5> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Functions.Func5<T1, T2, T3, T4, T5, T> creator) {
    return with(Fields.of(field1, field2, field3, field4, field5, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Functions.Func6<T1, T2, T3, T4, T5, T6, T> creator) {
    return with(Fields.of(field1, field2, field3, field4, field5, field6, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Functions.Func7<T1, T2, T3, T4, T5, T6, T7, T> creator) {
    return with(Fields.of(field1, field2, field3, field4, field5, field6, field7, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Functions.Func8<T1, T2, T3, T4, T5, T6, T7, T8, T> creator) {
    return with(Fields.of(field1, field2, field3, field4, field5, field6, field7, field8, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Field<T, T9> field9,
      Functions.Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, T> creator) {
    return with(
        Fields.of(field1, field2, field3, field4, field5, field6, field7, field8, field9, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Field<T, T9> field9,
      Field<T, T10> field10,
      Functions.Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T> creator) {
    return with(
        Fields.of(
            field1, field2, field3, field4, field5, field6, field7, field8, field9, field10,
            creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Field<T, T9> field9,
      Field<T, T10> field10,
      Field<T, T11> field11,
      Functions.Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T> creator) {
    return with(
        Fields.of(
            field1, field2, field3, field4, field5, field6, field7, field8, field9, field10,
            field11, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> StructCodec<T> fromFields(
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Field<T, T9> field9,
      Field<T, T10> field10,
      Field<T, T11> field11,
      Field<T, T12> field12,
      Functions.Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T> creator) {
    return with(
        Fields.of(
            field1, field2, field3, field4, field5, field6, field7, field8, field9, field10,
            field11, field12, creator));
  }

  // Option 2 "with" - Uses separation / reunification tactic
  // EXAMPLE:
  /*
     codecMap.register(
       SimpleRecord.class,
       (codecs) ->
           StructCodec.with(
               (r, s) -> s.accept(r.first, r.second, r.third, r.fourth),
               codecs.of(int.class),
               codecs.of(String.class),
               codecs.of(new TypeToken<Either<Long, String>>() {}),
               codecs.of(new TypeToken<Option<Boolean>>() {}),
               SimpleRecord::new));
  */

  static <T> StructCodec<T> with(UntypedCodec<T> untypedCodec) {
    return new StructFieldsCodec<>(untypedCodec);
  }

  static <T> StructCodec<T> empty(Functions.Func0<T> creator) {
    return new StructFieldsCodec<>(UntypedCodec.empty(creator));
  }

  static <T, T1> StructCodec<T> single(
      Functions.Func1<T, T1> getter, Codec<T1> codec1, Functions.Func1<T1, T> creator) {
    return with(UntypedCodec.wrapWithoutLength(getter, codec1, creator));
  }

  static <T, T1> StructCodec<T> with(
      Functions.Action2<T, Functions.Action1<T1>> splitter,
      Codec<T1> codec1,
      Functions.Func1<T1, T> creator) {
    return with(UntypedCodec.fromWithLength(splitter, codec1, creator));
  }

  static <T, T1, T2> StructCodec<T> with(
      Functions.Action2<T, Functions.Action2<T1, T2>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Functions.Func2<T1, T2, T> creator) {
    return with(UntypedCodec.fromWithLength(splitter, codec1, codec2, creator));
  }

  static <T, T1, T2, T3> StructCodec<T> with(
      Functions.Action2<T, Functions.Action3<T1, T2, T3>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Functions.Func3<T1, T2, T3, T> creator) {
    return with(UntypedCodec.fromWithLength(splitter, codec1, codec2, codec3, creator));
  }

  static <T, T1, T2, T3, T4> StructCodec<T> with(
      Functions.Action2<T, Functions.Action4<T1, T2, T3, T4>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Functions.Func4<T1, T2, T3, T4, T> creator) {
    return with(UntypedCodec.fromWithLength(splitter, codec1, codec2, codec3, codec4, creator));
  }

  static <T, T1, T2, T3, T4, T5> StructCodec<T> with(
      Functions.Action2<T, Functions.Action5<T1, T2, T3, T4, T5>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Functions.Func5<T1, T2, T3, T4, T5, T> creator) {
    return with(UntypedCodec.fromWithLength(splitter, codec1, codec2, codec3, codec4, codec5, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6> StructCodec<T> with(
      Functions.Action2<T, Functions.Action6<T1, T2, T3, T4, T5, T6>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Functions.Func6<T1, T2, T3, T4, T5, T6, T> creator) {
    return with(
        UntypedCodec.fromWithLength(splitter, codec1, codec2, codec3, codec4, codec5, codec6, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7> StructCodec<T> with(
      Functions.Action2<T, Functions.Action7<T1, T2, T3, T4, T5, T6, T7>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Functions.Func7<T1, T2, T3, T4, T5, T6, T7, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8> StructCodec<T> with(
      Functions.Action2<T, Functions.Action8<T1, T2, T3, T4, T5, T6, T7, T8>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Functions.Func8<T1, T2, T3, T4, T5, T6, T7, T8, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, codec8, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9> StructCodec<T> with(
      Functions.Action2<T, Functions.Action9<T1, T2, T3, T4, T5, T6, T7, T8, T9>> splitter,
      Codec<T1> codec1,
      Codec<T2> codec2,
      Codec<T3> codec3,
      Codec<T4> codec4,
      Codec<T5> codec5,
      Codec<T6> codec6,
      Codec<T7> codec7,
      Codec<T8> codec8,
      Codec<T9> codec9,
      Functions.Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, codec8, codec9,
            creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> StructCodec<T> with(
      Functions.Action2<T, Functions.Action10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>> splitter,
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
      Functions.Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, codec8, codec9,
            codec10, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> StructCodec<T> with(
      Functions.Action2<T, Functions.Action11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>>
          splitter,
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
      Functions.Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, codec8, codec9,
            codec10, codec11, creator));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> StructCodec<T> with(
      Functions.Action2<T, Functions.Action12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>>
          splitter,
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
      Functions.Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T> creator) {
    return with(
        UntypedCodec.fromWithLength(
            splitter, codec1, codec2, codec3, codec4, codec5, codec6, codec7, codec8, codec9,
            codec10, codec11, codec12, creator));
  }
}
