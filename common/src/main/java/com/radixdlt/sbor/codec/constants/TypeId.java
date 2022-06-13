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

package com.radixdlt.sbor.codec.constants;

import static com.radixdlt.lang.Option.none;
import static com.radixdlt.lang.Option.some;

import com.google.common.collect.ImmutableSet;
import com.radixdlt.lang.Option;
import com.radixdlt.sbor.exceptions.SborCodecException;
import java.util.Set;

public enum TypeId {
  // Primitive Types
  TYPE_UNIT(0x00),
  TYPE_BOOL(0x01),
  TYPE_I8(0x02),
  TYPE_I16(0x03),
  TYPE_I32(0x04),
  TYPE_I64(0x05),
  TYPE_I128(0x06),
  TYPE_U8(0x07),
  TYPE_U16(0x08),
  TYPE_U32(0x09),
  TYPE_U64(0x0a),
  TYPE_U128(0x0b),
  TYPE_STRING(0x0c),

  // Enum and struct
  TYPE_STRUCT(0x10),
  TYPE_ENUM(0x11),

  // Composite types
  TYPE_OPTION(0x20),
  TYPE_ARRAY(0x22),
  TYPE_TUPLE(0x23),
  TYPE_RESULT(0x24),

  // Collections + Maps
  TYPE_VEC(0x30),
  TYPE_TREE_SET(0x31),
  TYPE_TREE_MAP(0x32),
  TYPE_HASH_SET(0x33),
  TYPE_HASH_MAP(0x34),

  // Custom Start
  TYPE_CUSTOM_START(0x80); // custom types start from 0x80 and values are encoded as `len + data`

  private final byte id;

  TypeId(int id) {
    this.id = (byte) id;
  }

  public byte id() {
    return id;
  }

  public static final Set<TypeId> collectionTypes =
      ImmutableSet.of(TYPE_VEC, TYPE_ARRAY, TYPE_TREE_SET, TYPE_HASH_SET);
  public static final Set<TypeId> mapTypes = ImmutableSet.of(TYPE_TREE_MAP, TYPE_HASH_MAP);

  public boolean isCollectionType() {
    return collectionTypes.contains(this);
  }

  public void assertCollectionType() {
    if (!isCollectionType()) {
      throw new SborCodecException(String.format("Type id %s is not a collection type id", this));
    }
  }

  public boolean isMapType() {
    return mapTypes.contains(this);
  }

  public void assertMapType() {
    if (!isMapType()) {
      throw new SborCodecException(String.format("Type id %s is not a map type id", this));
    }
  }

  /** Intended for debugging - not particularly performant. */
  public static Option<TypeId> fromId(byte id) {
    for (var enumValue : values()) {
      if (enumValue.id() == id) {
        return some(enumValue);
      }
    }
    return none();
  }
}
