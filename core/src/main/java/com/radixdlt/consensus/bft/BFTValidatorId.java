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

package com.radixdlt.consensus.bft;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.utils.Bytes;
import java.util.Objects;

/**
 * A node in a BFT network which can run BFT validation
 *
 * <p>TODO: turn this into an interface so that an ECPublicKey is not required TODO: Serialization
 * of BFT messages are currently what prevent this from happening
 */
public final class BFTValidatorId {
  private final ECDSASecp256k1PublicKey key;
  private final ComponentAddress validatorAddress;
  private final String shortenedName;

  private BFTValidatorId(
      ComponentAddress validatorAddress, ECDSASecp256k1PublicKey key, String shortenedName) {
    this.validatorAddress = Objects.requireNonNull(validatorAddress);
    this.key = Objects.requireNonNull(key);
    this.shortenedName = Objects.requireNonNull(shortenedName);
  }

  public static BFTValidatorId create(
      ComponentAddress validatorAddress, ECDSASecp256k1PublicKey key) {
    final String shortenedName =
        validatorAddress.toHexString().substring(0, 6) + ":" + key.toHex().substring(0, 6);
    return new BFTValidatorId(validatorAddress, key, shortenedName);
  }

  // This method is only used in deserialization methods so should be okay
  // to throw exceptions.
  // TODO: Need a better serialization mechanism for BFTValidatorId
  public static BFTValidatorId fromSerializedString(String str) {
    var strings = str.split(":");
    if (strings.length != 2) {
      throw new IllegalStateException("Error decoding node");
    }
    try {
      var validatorAddress = ComponentAddress.create(Bytes.fromHexString(strings[0]));
      var key = ECDSASecp256k1PublicKey.fromBytes(Bytes.fromHexString(strings[1]));
      return create(validatorAddress, key);
    } catch (PublicKeyException e) {
      throw new IllegalStateException("Error decoding public key", e);
    }
  }

  public String toSerializedString() {
    final var addressString = Bytes.toHexString(this.validatorAddress.value());
    final var keyString = Bytes.toHexString(this.key.getCompressedBytes());
    return addressString + ":" + keyString;
  }

  public ECDSASecp256k1PublicKey getKey() {
    return key;
  }

  public ComponentAddress getValidatorAddress() {
    return validatorAddress;
  }

  /** Only for use in tests */
  public static BFTValidatorId random() {
    return withKeyAndFakeDeterministicAddress(ECKeyPair.generateNew().getPublicKey());
  }

  /** Only for use in tests */
  public static BFTValidatorId withKeyAndFakeDeterministicAddress(ECDSASecp256k1PublicKey key) {
    final var addressBytes = new byte[ComponentAddress.BYTE_LENGTH];
    final var hashedKeyBytes = HashUtils.blake2b256(key.getCompressedBytes()).asBytes();
    System.arraycopy(hashedKeyBytes, 0, addressBytes, 0, addressBytes.length);
    addressBytes[0] = ComponentAddress.VALIDATOR_COMPONENT_ADDRESS_ENTITY_ID;
    return create(ComponentAddress.create(addressBytes), key);
  }

  @Override
  public int hashCode() {
    return Objects.hash(key, validatorAddress);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (o == null || getClass() != o.getClass()) return false;
    BFTValidatorId that = (BFTValidatorId) o;
    return Objects.equals(key, that.key) && Objects.equals(validatorAddress, that.validatorAddress);
  }

  @Override
  public String toString() {
    return shortenedName;
  }
}
