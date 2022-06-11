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
import com.radixdlt.sbor.exceptions.SborCodecException;
import com.radixdlt.sbor.exceptions.SborDecodeException;
import java.util.HashMap;
import java.util.Map;

public class EnumCodec<T> implements Codec<T> {
  private final Map<String, EnumEntry<?>> entries;
  private final Functions.Func1<T, String> mapEnumValueToKey;

  public EnumCodec(
      Map<String, EnumEntry<?>> entries, Functions.Func1<T, String> mapEnumValueToKey) {
    this.entries = entries;
    this.mapEnumValueToKey = mapEnumValueToKey;
  }

  @Override
  public TypeId getTypeId() {
    return TypeId.TYPE_ENUM;
  }

  @SuppressWarnings("unchecked")
  public void encodeWithoutTypeId(EncoderApi encoder, T value) {
    var enumKey = mapEnumValueToKey.apply(value);

    var enumEntry = entries.get(enumKey);

    if (enumEntry == null) {
      throw new SborCodecException(
          String.format("Enum entry enumKey %s not found in entries map", enumKey));
    }

    encoder.writeString(enumKey);
    ((EnumEntry<T>) enumEntry).encodeWithoutTypeId(encoder, value);
  }

  @SuppressWarnings("unchecked")
  public T decodeWithoutTypeId(DecoderApi decoder) {
    var enumKey = decoder.readString();

    var enumEntry = entries.get(enumKey);

    if (enumEntry == null) {
      throw new SborDecodeException(
          String.format("Enum entry enumKey %s not found in entries map", enumKey));
    }

    return ((EnumEntry<T>) enumEntry).decodeWithoutTypeId(decoder);
  }

  /**
   * An Enum Codec for Abstract Data Types (ie sealed interfaces with distinct subclasses). It
   * assumes each FieldsCodec entry relates to a distinct subclass, and that the names of the
   * subclasses are the enum key names to use in serialization.
   *
   * @param <T> The interface type, of which each FieldsCodec is a subclass
   */
  public static <T> EnumCodec<T> fromEntries(EnumEntry<?>... enumEntries) {
    var enumEntryCodecMap = new HashMap<String, EnumEntry<?>>();
    var inverseClassMap = new HashMap<Class<?>, String>();
    for (var enumEntry : enumEntries) {
      var baseClass = enumEntry.getBaseClass();
      var enumKey = baseClass.getSimpleName();
      enumEntryCodecMap.put(enumKey, enumEntry);
      inverseClassMap.put(baseClass, enumKey);
    }

    if (inverseClassMap.size() < enumEntries.length) {
      throw new SborCodecException(
          "Some field codecsMap are for the same class - this isn't supported with the fromEntries"
              + " builder");
    }

    if (enumEntryCodecMap.size() < enumEntries.length) {
      throw new SborCodecException(
          "Some field codecsMap are for classes which have the same simple name - this isn't"
              + " supported with the fromEntries builder");
    }

    return new EnumCodec<>(
        enumEntryCodecMap,
        enumEntry -> {
          var key = inverseClassMap.get(enumEntry.getClass());
          if (key == null) {
            throw new SborCodecException(
                String.format("Class %s not found in inverseClassMap", enumEntry.getClass()));
          }
          return key;
        });
  }
}
