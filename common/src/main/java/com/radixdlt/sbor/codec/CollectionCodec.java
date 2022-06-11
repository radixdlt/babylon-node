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
import java.lang.reflect.Array;
import java.util.*;

@SuppressWarnings("unused")
interface CollectionCodec {
  record CollectionCodecViaArrayList<TCollection, TItem>(
      TypeId collectionTypeId,
      Codec<TItem> itemCodec,
      Functions.Func1<TCollection, Integer> getSize,
      Functions.Func1<TCollection, Iterable<TItem>> getIterable,
      Functions.Func1<ArrayList<TItem>, TCollection> mapFromList)
      implements Codec<TCollection> {

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    public void encodeFromIterable(EncoderApi encoder, int size, Iterable<TItem> iterable) {
      encoder.encodeTypeId(itemCodec.getTypeId());
      encoder.writeInt(size);

      for (var item : iterable) {
        itemCodec.encodeWithoutTypeId(encoder, item);
      }
    }

    public ArrayList<TItem> decodeToList(DecoderApi decoder) {
      decoder.expectType(itemCodec.getTypeId());
      var length = decoder.readInt();
      var list = new ArrayList<TItem>(length);

      for (var i = 0; i < length; i++) {
        list.add(itemCodec.decodeWithoutTypeId(decoder));
      }

      return list;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, TCollection collection) {
      encodeFromIterable(encoder, getSize.apply(collection), getIterable.apply(collection));
    }

    @Override
    public TCollection decodeWithoutTypeId(DecoderApi decoder) {
      return mapFromList.apply(decodeToList(decoder));
    }
  }

  record CollectionCodecViaArray<TItem>(
      TypeId collectionTypeId, Codec<TItem> itemCodec, Class<TItem> itemClazz)
      implements Codec<TItem[]> {

    @Override
    public TypeId getTypeId() {
      return collectionTypeId;
    }

    @Override
    public void encodeWithoutTypeId(EncoderApi encoder, TItem[] array) {
      encoder.encodeTypeId(itemCodec.getTypeId());
      encoder.writeInt(array.length);

      for (var item : array) {
        itemCodec.encodeWithoutTypeId(encoder, item);
      }
    }

    @Override
    @SuppressWarnings("unchecked")
    public TItem[] decodeWithoutTypeId(DecoderApi decoder) {
      decoder.expectType(itemCodec.getTypeId());
      var length = decoder.readInt();
      var array = (TItem[]) Array.newInstance(itemClazz, length);

      for (var i = 0; i < length; i++) {
        array[i] = itemCodec.decodeWithoutTypeId(decoder);
      }

      return array;
    }
  }

  static <T> Codec<Set<T>> forSet(Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArrayList<>(
        collectionTypeId,
        itemCodec,
        Set::size,
        set -> set,
        list -> {
          var set = new HashSet<>(list);
          if (set.size() != list.size()) {
            throw new SborDecodeException(
                String.format(
                    "Duplicate elements in set. Expected size %s, de-duplicated size %s",
                    list.size(), set.size()));
          }
          return set;
        });
  }

  static <T> Codec<HashSet<T>> forHashSet(Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArrayList<>(
        collectionTypeId,
        itemCodec,
        HashSet::size,
        set -> set,
        list -> {
          var set = new HashSet<>(list);
          if (set.size() != list.size()) {
            throw new SborDecodeException(
                String.format(
                    "Duplicate elements in set. Expected size %s, de-duplicated size %s",
                    list.size(), set.size()));
          }
          return set;
        });
  }

  static <T> Codec<TreeSet<T>> forTreeSet(Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArrayList<>(
        collectionTypeId,
        itemCodec,
        TreeSet::size,
        set -> set,
        list -> {
          var set = new TreeSet<>(list);
          if (set.size() != list.size()) {
            throw new SborDecodeException(
                String.format(
                    "Duplicate elements in set. Expected size %s, de-duplicated size %s",
                    list.size(), set.size()));
          }
          return set;
        });
  }

  static <T> Codec<List<T>> forList(Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArrayList<>(
        collectionTypeId, itemCodec, List::size, list -> list, list -> list);
  }

  static <T> Codec<ArrayList<T>> forArrayList(Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArrayList<>(
        collectionTypeId, itemCodec, List::size, list -> list, list -> list);
  }

  static <T> Codec<T[]> forArray(Class<T> itemClazz, Codec<T> itemCodec, TypeId collectionTypeId) {
    collectionTypeId.assertCollectionType();
    return new CollectionCodecViaArray<>(collectionTypeId, itemCodec, itemClazz);
  }

  static void registerSetToMapTo(CodecMap codecMap, TypeId collectionTypeId) {
    codecMap.registerForGeneric(
        Set.class,
        (codecs, collectionType) -> {
          try {
            var itemType = TypeTokenUtils.getGenericTypeParameter(collectionType, 0);
            return forSet(codecs.of(itemType), collectionTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating Set type codec for %s", collectionType), ex);
          }
        });
  }

  static void registerHashSetToMapTo(CodecMap codecMap, TypeId collectionTypeId) {
    codecMap.registerForGeneric(
        HashSet.class,
        (codecs, collectionType) -> {
          try {
            var itemType = TypeTokenUtils.getGenericTypeParameter(collectionType, 0);
            return forHashSet(codecs.of(itemType), collectionTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating HashSet type codec for %s", collectionType), ex);
          }
        });
  }

  static void registerTreeSetToMapTo(CodecMap codecMap, TypeId collectionTypeId) {
    codecMap.registerForGeneric(
        TreeSet.class,
        (codecs, collectionType) -> {
          try {
            var itemType = TypeTokenUtils.getGenericTypeParameter(collectionType, 0);
            return forTreeSet(codecs.of(itemType), collectionTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating TreeSet type codec for %s", collectionType), ex);
          }
        });
  }

  static void registerListToMapTo(CodecMap codecMap, TypeId collectionTypeId) {
    codecMap.registerForGeneric(
        List.class,
        (codecs, collectionType) -> {
          try {
            var itemType = TypeTokenUtils.getGenericTypeParameter(collectionType, 0);
            return forList(codecs.of(itemType), collectionTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating List type codec for %s", collectionType), ex);
          }
        });
  }

  static void registerArrayListToMapTo(CodecMap codecMap, TypeId collectionTypeId) {
    codecMap.registerForGeneric(
        ArrayList.class,
        (codecs, collectionType) -> {
          try {
            var itemType = TypeTokenUtils.getGenericTypeParameter(collectionType, 0);
            return forArrayList(codecs.of(itemType), collectionTypeId);
          } catch (Exception ex) {
            throw new SborCodecException(
                String.format("Exception creating ArrayList type codec for %s", collectionType),
                ex);
          }
        });
  }
}
