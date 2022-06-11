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
import com.radixdlt.sbor.coding.DecoderApi;
import com.radixdlt.sbor.coding.EncoderApi;
import com.radixdlt.sbor.exceptions.SborDecodeException;
import java.util.List;

@SuppressWarnings("unused")
public interface TypeFields<T> extends UntypedCodec<T> {
  List<Field<T, ?>> fields();

  Class<T> getBaseClass();

  @Override
  default void encodeWithoutTypeId(EncoderApi encoder, T value) {
    var fields = fields();

    encoder.writeInt(fields.size());

    for (var field : fields) {
      field.encode(encoder, value);
    }
  }

  @Override
  default T decodeWithoutTypeId(DecoderApi decoder) {
    var decodedFieldsLength = decoder.readInt();

    var fields = fields();

    if (decodedFieldsLength != fields.size()) {
      throw new SborDecodeException(
          String.format(
              "Incorrect number of fields detected. Expected field count was %s, but encoded was"
                  + " %s",
              fields.size(), decodedFieldsLength));
    }

    return this.decodeFields(decoder);
  }

  T decodeFields(DecoderApi decoder);

  class FieldsCodecImpl<T> implements TypeFields<T> {
    private final Class<T> clazz;
    private final List<Field<T, ?>> fields;
    private final Functions.Func1<DecoderApi, T> decodeFields;

    public FieldsCodecImpl(
        Class<T> baseClass, List<Field<T, ?>> fields, Functions.Func1<DecoderApi, T> decodeFields) {
      this.clazz = baseClass;
      this.fields = fields; // Avoid allocation on each call of fields()
      this.decodeFields = decodeFields;
    }

    @Override
    public List<Field<T, ?>> fields() {
      return fields;
    }

    @Override
    public Class<T> getBaseClass() {
      return clazz;
    }

    @Override
    public T decodeFields(DecoderApi decoder) {
      return decodeFields.apply(decoder);
    }
  }

  static <T> TypeFields<T> noFields(Class<T> baseClass, Functions.Func0<T> creator) {
    return new FieldsCodecImpl<>(baseClass, List.of(), decoder -> creator.apply());
  }

  static <T, T1> TypeFields<T> of(
      Class<T> baseClass, Field<T, T1> field1, Functions.Func1<T1, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass, List.of(field1), decoder -> creator.apply(field1.decode(decoder)));
  }

  static <T, T1, T2> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Functions.Func2<T1, T2, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2),
        decoder -> creator.apply(field1.decode(decoder), field2.decode(decoder)));
  }

  static <T, T1, T2, T3> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Functions.Func3<T1, T2, T3, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3),
        decoder ->
            creator.apply(field1.decode(decoder), field2.decode(decoder), field3.decode(decoder)));
  }

  static <T, T1, T2, T3, T4> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Functions.Func4<T1, T2, T3, T4, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Functions.Func5<T1, T2, T3, T4, T5, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Functions.Func6<T1, T2, T3, T4, T5, T6, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5, field6),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Functions.Func7<T1, T2, T3, T4, T5, T6, T7, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5, field6, field7),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8> TypeFields<T> of(
      Class<T> baseClass,
      Field<T, T1> field1,
      Field<T, T2> field2,
      Field<T, T3> field3,
      Field<T, T4> field4,
      Field<T, T5> field5,
      Field<T, T6> field6,
      Field<T, T7> field7,
      Field<T, T8> field8,
      Functions.Func8<T1, T2, T3, T4, T5, T6, T7, T8, T> creator) {
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5, field6, field7, field8),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder),
                field8.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9> TypeFields<T> of(
      Class<T> baseClass,
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
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5, field6, field7, field8, field9),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder),
                field8.decode(decoder),
                field9.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> TypeFields<T> of(
      Class<T> baseClass,
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
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(field1, field2, field3, field4, field5, field6, field7, field8, field9, field10),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder),
                field8.decode(decoder),
                field9.decode(decoder),
                field10.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> TypeFields<T> of(
      Class<T> baseClass,
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
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(
            field1, field2, field3, field4, field5, field6, field7, field8, field9, field10,
            field11),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder),
                field8.decode(decoder),
                field9.decode(decoder),
                field10.decode(decoder),
                field11.decode(decoder)));
  }

  static <T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> TypeFields<T> of(
      Class<T> baseClass,
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
    return new FieldsCodecImpl<>(
        baseClass,
        List.of(
            field1, field2, field3, field4, field5, field6, field7, field8, field9, field10,
            field11, field12),
        decoder ->
            creator.apply(
                field1.decode(decoder),
                field2.decode(decoder),
                field3.decode(decoder),
                field4.decode(decoder),
                field5.decode(decoder),
                field6.decode(decoder),
                field7.decode(decoder),
                field8.decode(decoder),
                field9.decode(decoder),
                field10.decode(decoder),
                field11.decode(decoder),
                field12.decode(decoder)));
  }
}
