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
import com.radixdlt.lang.Either;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Unit;
import java.util.*;
import java.util.function.Function;

/** Container for mapping between codec and class. */
@SuppressWarnings({
  "rawtypes",
  "unchecked"
}) // This class is required to play fast and loose with generics
public final class CodecMap {

  private final Map<Class, Codec> classEncodingMap = new HashMap<>();

  private final Map<TypeToken, Codec> explicitTypeEncodingMap = new HashMap<>();

  private final Map<Class, Function<TypeToken, Codec>> codecCreatorMap = new HashMap<>();

  // QQ: Make this all static
  public CodecMap() {
    addCoreCodec(Unit.class, new CoreTypeCodec.UnitCodec());
    addCoreCodec(String.class, new CoreTypeCodec.StringCodec());

    addCoreCodec(Boolean.class, new CoreTypeCodec.BooleanCodec());
    addCoreCodec(boolean.class, new CoreTypeCodec.BooleanCodec());

    addCoreCodec(Byte.class, new CoreTypeCodec.ByteCodec());
    addCoreCodec(byte.class, new CoreTypeCodec.ByteCodec());

    addCoreCodec(Short.class, new CoreTypeCodec.ShortCodec());
    addCoreCodec(short.class, new CoreTypeCodec.ShortCodec());

    addCoreCodec(Integer.class, new CoreTypeCodec.IntegerCodec());
    addCoreCodec(int.class, new CoreTypeCodec.IntegerCodec());

    addCoreCodec(Long.class, new CoreTypeCodec.LongCodec());
    addCoreCodec(long.class, new CoreTypeCodec.LongCodec());

    addCoreCodec(byte[].class, new CoreTypeCodec.ByteArrayCodec());
    addCoreCodec(short[].class, new CoreTypeCodec.ShortArrayCodec());
    addCoreCodec(int[].class, new CoreTypeCodec.IntegerArrayCodec());
    addCoreCodec(long[].class, new CoreTypeCodec.LongArrayCodec());

    registerCodecCreatorForSealedClassAndSubclasses(
        Either.class,
        eitherType -> {
          try {
            var leftType = eitherType.method(Either.class.getMethod("unwrapLeft")).getReturnType();
            var rightType =
                eitherType.method(Either.class.getMethod("unwrapRight")).getReturnType();
            return new EitherTypeCodec(leftType, rightType);
          } catch (Exception ex) {
            throw new RuntimeException(ex);
          }
        });

    registerCodecCreatorForSealedClassAndSubclasses(
        Option.class,
        optionType -> {
          try {
            var innerType = optionType.method(Option.class.getMethod("unwrap")).getReturnType();
            return new OptionTypeCodec(innerType);
          } catch (Exception ex) {
            throw new RuntimeException(ex);
          }
        });
  }

  private <T> void addCoreCodec(Class<T> clazz, Codec<T> codec) {
    classEncodingMap.put(clazz, codec);
    explicitTypeEncodingMap.put(TypeToken.of(clazz), codec);
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

      // We cache the codec for future use
      registerExplicitGeneric(type, newCodec);
      return newCodec;
    }

    throw new RuntimeException(
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
      // QQ: Add a better exception
      throw new RuntimeException(
          String.format(
              "The class object %s itself has no registered SBOR codec, BUT a codec creator is"
                  + " registered. You should use an explicit TypeToken<X<Y,Z>>.",
              clazz));
    }

    throw new RuntimeException(
        String.format(
            "The class object %s itself has no registered SBOR codec, nor has codec creator"
                + " registered.",
            clazz));
  }

  public <T> CodecMap register(Class<T> clazz, Codec<T> codec) {
    synchronized (classEncodingMap) {
      classEncodingMap.put(clazz, codec);
    }
    return this;
  }

  public <T> CodecMap registerForSealedClassAndSubclasses(
      Class<T> clazz, Codec<? extends T> codec) {
    if (!clazz.isSealed()) {
      throw new RuntimeException(
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

  public <T> CodecMap registerCodecCreator(
      Class<T> clazz, Function<TypeToken<T>, Codec> createCodec) {
    synchronized (codecCreatorMap) {
      codecCreatorMap.put(clazz, createCodec::apply);
    }
    return this;
  }

  public <T> CodecMap registerCodecCreatorForSealedClassAndSubclasses(
      Class<T> clazz, Function<TypeToken<T>, Codec> createCodec) {
    if (!clazz.isSealed()) {
      throw new RuntimeException(
          String.format(
              "The class object %s is not sealed, so cannot be passed into "
                  + "registerCodecCreatorForSubclassesOfSealed.",
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

  /**
   * This is mostly intended for internal use - for registering a codec for a concrete generic.
   * Externally, it's recommended to register via a register (for non-generic types) or
   * registerGenericCodecCreator (for generic types).
   *
   * @param type An explicit type to register a codec for
   * @param codec The codec to register
   * @return the CodecMap
   * @param <T> The (generic) type the codec is being registered for
   */
  public <T> CodecMap registerExplicitGeneric(TypeToken<T> type, Codec<T> codec) {
    synchronized (explicitTypeEncodingMap) {
      explicitTypeEncodingMap.put(type, codec);
    }
    return this;
  }
}
