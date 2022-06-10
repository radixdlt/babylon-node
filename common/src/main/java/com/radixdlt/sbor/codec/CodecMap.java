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

import com.google.common.reflect.TypeToken;
import com.radixdlt.lang.EitherTypeCodec;
import com.radixdlt.lang.OptionTypeCodec;
import com.radixdlt.lang.Unit;
import com.radixdlt.sbor.exceptions.SborCodecException;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.function.Function;

/**
 * The CodecMap registers default strategies to encode/decode a type.
 *
 * <p>You can register codecs for:
 *
 * <ul>
 *   <li>A class object - this captures types without their generic parameters
 *   <li>A concrete TypeToken - this is specific to all the given generic parameters
 * </ul>
 *
 * <p>If multiple codecs are registered against the same object/TypeToken, the latest to be
 * registered is used.
 *
 * <p>You can also register a codec creator, which allows automatic creation of codecs for explicit
 * type parameters of a given class. This works well with types such as Option&lt;T&rt;, where you
 * may wish to decode into an Option&lt;String&rt; without registering a codec for
 * Option&lt;String&rt; explicitly. The generated codecs are cached against their explicit
 * TypeToken.
 *
 * <p>Finally, you can also register a class object codec and codec creators for a sealed class and
 * all its subclasses in one go - this is to provide easy support for ADTs (abstract data types).
 */
@SuppressWarnings({
  "rawtypes",
  "unchecked",
  "UnusedReturnValue",
  "unused"
}) // This class is required to play fast and loose with generics
public final class CodecMap {
  /**
   * Codecs can be registered on the static CodecMap.DEFAULT which is used by SborCoder.DEFAULT. It
   * is recommended to do this in the static constructor of a class being encoded/decoded. It is
   * safe to register twice - the latest registration will apply.
   */
  public static final CodecMap DEFAULT = new CodecMap();

  private final Map<Class, Codec> classEncodingMap = new HashMap<>();

  private final Map<TypeToken, Codec> explicitTypeEncodingMap = new HashMap<>();

  private final Map<Class, Function<TypeToken, Codec>> codecCreatorMap = new HashMap<>();

  public CodecMap() {
    register(Unit.class, new CoreTypeCodec.UnitCodec());
    register(String.class, new CoreTypeCodec.StringCodec());

    register(Boolean.class, new CoreTypeCodec.BooleanCodec());
    register(boolean.class, new CoreTypeCodec.BooleanCodec());

    register(Byte.class, new CoreTypeCodec.ByteCodec());
    register(byte.class, new CoreTypeCodec.ByteCodec());

    register(Short.class, new CoreTypeCodec.ShortCodec());
    register(short.class, new CoreTypeCodec.ShortCodec());

    register(Integer.class, new CoreTypeCodec.IntegerCodec());
    register(int.class, new CoreTypeCodec.IntegerCodec());

    register(Long.class, new CoreTypeCodec.LongCodec());
    register(long.class, new CoreTypeCodec.LongCodec());

    register(byte[].class, new CoreTypeCodec.ByteArrayCodec());
    register(short[].class, new CoreTypeCodec.ShortArrayCodec());
    register(int[].class, new CoreTypeCodec.IntegerArrayCodec());
    register(long[].class, new CoreTypeCodec.LongArrayCodec());

    OptionTypeCodec.registerWith(this);
    EitherTypeCodec.registerWith(this);
  }

  public <T> Codec<T> get(TypeToken<T> type) {
    // First - let's try to find a pre-registered codec for this explicit type literal
    var codec = explicitTypeEncodingMap.get(type);
    if (codec != null) {
      return codec;
    }

    // Failing that - let's see if we can create one with a codec creator
    var rawType = type.getRawType();
    var codecCreator = codecCreatorMap.get(rawType);
    if (codecCreator != null) {
      var newCodec = codecCreator.apply(type);

      // Let's cache the codec for future use
      register(type, newCodec);
      return newCodec;
    }

    throw new SborCodecException(
        String.format(
            "The type token %s itself has no SBOR codec, and its raw type class %s has no codec"
                + " creator registered.",
            type, rawType));
  }

  public <T> Codec<T> get(Class<T> clazz) {
    var codec = classEncodingMap.get(clazz);

    if (codec != null) {
      return codec;
    }

    // We are in a failure case here - so let's try to be helpful
    var codecCreator = codecCreatorMap.get(clazz);
    if (codecCreator != null) {
      throw new SborCodecException(
          String.format(
              "The class object %s itself has no registered SBOR codec, BUT a codec creator is"
                  + " registered. You should use an explicit TypeToken<X<Y,Z>>.",
              clazz));
    }

    throw new SborCodecException(
        String.format(
            "The class object %s itself has no registered SBOR codec, nor has codec creator"
                + " registered.",
            clazz));
  }

  public <T> CodecMap register(StructCodec<T> codec) {
    register(codec.fieldsCodec().getBaseClass(), codec);
    return this;
  }

  public <T> CodecMap register(Class<T> clazz, Codec<T> codec) {
    synchronized (classEncodingMap) {
      classEncodingMap.put(clazz, codec);
      explicitTypeEncodingMap.put(TypeToken.of(clazz), codec);
    }
    return this;
  }

  public <T> CodecMap registerForSealedClassAndSubclasses(
      Class<T> clazz, Codec<? extends T> codec) {
    if (!clazz.isSealed()) {
      throw new SborCodecException(
          String.format(
              "The class object %s is not sealed, so cannot be passed into "
                  + "registerForSubclassesOfSealed.",
              clazz));
    }

    classEncodingMap.put(clazz, codec);
    var implementers = clazz.getPermittedSubclasses();
    Arrays.stream(implementers)
        .forEach(
            subClass -> {
              synchronized (classEncodingMap) {
                classEncodingMap.put(subClass, codec);
              }
            });
    return this;
  }

  public <T> CodecMap registerCreator(Class<T> clazz, Function<TypeToken<T>, Codec> createCodec) {
    synchronized (codecCreatorMap) {
      codecCreatorMap.put(clazz, createCodec::apply);
    }
    return this;
  }

  public <T> CodecMap registerCreatorForSealedClassAndSubclasses(
      Class<T> clazz, Function<TypeToken<T>, Codec> createCodec) {
    if (!clazz.isSealed()) {
      throw new SborCodecException(
          String.format(
              "The class object %s is not sealed, so cannot be passed into "
                  + "registerCreatorForSealedClassAndSubclasses.",
              clazz));
    }

    codecCreatorMap.put(clazz, createCodec::apply);
    var implementers = clazz.getPermittedSubclasses();
    Arrays.stream(implementers)
        .forEach(
            subClass -> {
              synchronized (explicitTypeEncodingMap) {
                codecCreatorMap.put(subClass, createCodec::apply);
              }
            });
    return this;
  }

  public <T> CodecMap register(TypeToken<T> type, Codec<T> codec) {
    synchronized (explicitTypeEncodingMap) {
      explicitTypeEncodingMap.put(type, codec);
    }
    return this;
  }
}
