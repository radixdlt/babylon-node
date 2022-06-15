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
import java.util.*;

@SuppressWarnings("unused")
public interface MapCodec {
  record MapCodecViaHashMap<TMap, TKey, TItem>(
      TypeId mapTypeId,
      Codec<TKey> keyCodec,
      Codec<TItem> itemCodec,
      Functions.Func1<TMap, Integer> getSize,
      Functions.Func1<TMap, Iterable<Map.Entry<TKey, TItem>>> getIterable,
      Functions.Func1<HashMap<TKey, TItem>, TMap> mapFromHashMap)
      implements Codec<TMap> {

    @Override
    public TypeId getTypeId() {
      return mapTypeId;
    }

    public void encodeFromIterable(
        EncoderApi encoder, int size, Iterable<Map.Entry<TKey, TItem>> iterable) {
      encoder.encodeTypeId(keyCodec.getTypeId());
      encoder.encodeTypeId(itemCodec.getTypeId());
      encoder.writeInt(size);

      for (var item : iterable) {
        keyCodec.encodeWithoutTypeId(encoder, item.getKey());
        itemCodec.encodeWithoutTypeId(encoder, item.getValue());
      }
    }

    public HashMap<TKey, TItem> decodeToHashMap(DecoderApi decoder) {
      decoder.expectType(keyCodec.getTypeId());
      decoder.expectType(itemCodec.getTypeId());

      var length = decoder.readInt();
      var map = new HashMap<TKey, TItem>(length);

      for (var i = 0; i < length; i++) {
        map.put(keyCodec.decodeWithoutTypeId(decoder), itemCodec.decodeWithoutTypeId(decoder));
      }

      if (map.size() != length) {
        throw new SborDecodeException(
            String.format(
                "Duplicate keys in map. Expected size %s, de-duplicated key count %s",
                length, map.size()));
      }

      return map;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, TMap map) {
      encodeFromIterable(encoder, getSize.apply(map), getIterable.apply(map));
    }

    @Override
    public TMap decodeWithoutTypeId(DecoderApi decoder) {
      return mapFromHashMap.apply(decodeToHashMap(decoder));
    }
  }

  private static <T> void assertNoDuplicates(Map<T, ?> map, List<T> list) {
    if (map.size() != list.size()) {
      throw new SborDecodeException(
          String.format(
              "Duplicate keys in map. Expected size %s, de-duplicated key count %s",
              list.size(), map.size()));
    }
  }

  /**
   * Uses the map's iterable to provide the encoding ordering. This is assumed to match the
   * determinism requirements of the encoding.
   *
   * <p>NB - When using SBOR to encode state (eg in the Radix Engine), encoding must be
   * deterministic. This isn't really relevant in Java SBOR, so we don't enforce any ordering here.
   * In the future, we may wish to add a forMapWithDeterministicOrder method.
   */
  static <TKey, TItem> Codec<Map<TKey, TItem>> forMap(
      Codec<TKey> keyCodec, Codec<TItem> itemCodec, TypeId mapTypeId) {
    mapTypeId.assertMapType();
    return new MapCodec.MapCodecViaHashMap<>(
        mapTypeId, keyCodec, itemCodec, Map::size, Map::entrySet, map -> map);
  }

  /**
   * Uses the hashmap's iterable to provide the encoding ordering. This is assumed to match the
   * determinism requirements of the encoding.
   *
   * <p>NB - When using SBOR to encode state (eg in the Radix Engine), encoding must be
   * deterministic. This isn't really relevant in Java SBOR, so we don't enforce any ordering here.
   * In the future, we may wish to add a forHashMapWithDeterministicOrder method.
   */
  static <TKey, TItem> Codec<HashMap<TKey, TItem>> forHashMap(
      Codec<TKey> keyCodec, Codec<TItem> itemCodec, TypeId mapTypeId) {
    mapTypeId.assertMapType();
    return new MapCodec.MapCodecViaHashMap<>(
        mapTypeId, keyCodec, itemCodec, HashMap::size, HashMap::entrySet, map -> map);
  }

  static <TKey, TItem> Codec<TreeMap<TKey, TItem>> forTreeMap(
      Codec<TKey> keyCodec, Codec<TItem> itemCodec, TypeId mapTypeId) {
    mapTypeId.assertMapType();
    return new MapCodec.MapCodecViaHashMap<>(
        mapTypeId, keyCodec, itemCodec, TreeMap::size, TreeMap::entrySet, TreeMap::new);
  }

  static void registerMapToMapTo(CodecMap codecMap, TypeId mapTypeId) {
    codecMap.registerForGeneric(
        Map.class,
        (codecs, mapType) -> {
          try {
            var keyType = TypeTokenUtils.getGenericTypeParameter(mapType, 0);
            var itemType = TypeTokenUtils.getGenericTypeParameter(mapType, 1);
            return forMap(codecs.of(keyType), codecs.of(itemType), mapTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Map type codec for %s", mapType), ex);
          }
        });
  }

  static void registerHashMapToMapTo(CodecMap codecMap, TypeId mapTypeId) {
    codecMap.registerForGeneric(
        HashMap.class,
        (codecs, mapType) -> {
          try {
            var keyType = TypeTokenUtils.getGenericTypeParameter(mapType, 0);
            var itemType = TypeTokenUtils.getGenericTypeParameter(mapType, 1);
            return forHashMap(codecs.of(keyType), codecs.of(itemType), mapTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating HashMap type codec for %s", mapType), ex);
          }
        });
  }

  static void registerTreeMapToMapTo(CodecMap codecMap, TypeId mapTypeId) {
    codecMap.registerForGeneric(
        TreeMap.class,
        (codecs, mapType) -> {
          try {
            var keyType = TypeTokenUtils.getGenericTypeParameter(mapType, 0);
            var itemType = TypeTokenUtils.getGenericTypeParameter(mapType, 1);
            return forTreeMap(codecs.of(keyType), codecs.of(itemType), mapTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating TreeMap type codec for %s", mapType), ex);
          }
        });
  }
}
