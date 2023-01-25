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

package com.radixdlt.addressing;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.exceptions.Bech32DecodeException;
import com.radixdlt.identifiers.Bech32mCoder;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.*;
import com.radixdlt.serialization.DeserializeException;
import com.radixdlt.utils.Pair;

/** Performs Bech32m encoding/decoding. */
public final class Addressing {
  private final Network network;

  private Addressing(Network network) {
    this.network = network;
  }

  public static Addressing ofNetwork(Network network) {
    return new Addressing(network);
  }

  public static Addressing ofNetwork(NetworkDefinition networkDefinition) {
    return new Addressing(networkDefinition.toNetwork());
  }

  public static Addressing ofNetworkId(int networkId) {
    return ofNetwork(Network.ofIdOrThrow(networkId));
  }

  public String encodePackageAddress(PackageAddress packageAddress) {
    return Bech32mCoder.encode(network.getPackageHrp(), packageAddress.value());
  }

  public PackageAddress decodePackageAddress(String address) {
    return PackageAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getPackageHrp(), address));
  }

  public String encodeResourceAddress(ResourceAddress resourceAddress) {
    return Bech32mCoder.encode(network.getResourceHrp(), resourceAddress.value());
  }

  public ResourceAddress decodeResourceAddress(String address) {
    return ResourceAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getResourceHrp(), address));
  }

  public String encodeNormalComponentAddress(ComponentAddress componentAddress) {
    // TODO - checks on first byte of address
    return Bech32mCoder.encode(network.getNormalComponentHrp(), componentAddress.value());
  }

  public ComponentAddress decodeNormalComponentAddress(String address) {
    // TODO - checks on first byte of address
    return ComponentAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getNormalComponentHrp(), address));
  }

  public String encodeAccountAddress(ComponentAddress componentAddress) {
    // TODO - checks on first byte of address
    return Bech32mCoder.encode(network.getAccountComponentHrp(), componentAddress.value());
  }

  public ComponentAddress decodeAccountAddress(String address) {
    // TODO - checks on first byte of address
    return ComponentAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getAccountComponentHrp(), address));
  }

  public String encodeSystemAddress(SystemAddress systemAddress) {
    // TODO - checks on first byte of address
    return Bech32mCoder.encode(network.getSystemComponentHrp(), systemAddress.value());
  }

  public SystemAddress decodeSystemAddress(String address) {
    // TODO - checks on first byte of address
    return SystemAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getSystemComponentHrp(), address));
  }

  public ComponentAddress decodeSystemComponentAddress(String address) {
    // TODO - checks on first byte of address
    return ComponentAddress.create(
        Bech32mCoder.decodeWithExpectedHrp(network.getSystemComponentHrp(), address));
  }

  public String encodeNodeAddress(ECDSASecp256k1PublicKey publicKey) {
    return Bech32mCoder.encode(network.getNodeHrp(), publicKey.getCompressedBytes());
  }

  public ECDSASecp256k1PublicKey decodeNodeAddress(String address) throws DeserializeException {
    try {
      var pubKeyBytes = Bech32mCoder.decodeWithExpectedHrp(network.getNodeHrp(), address);
      return ECDSASecp256k1PublicKey.fromBytes(pubKeyBytes);
    } catch (Bech32DecodeException | PublicKeyException e) {
      throw new DeserializeException("Invalid address", e);
    }
  }

  public static String encodeNodeAddressWithHrp(String hrp, ECDSASecp256k1PublicKey publicKey) {
    return Bech32mCoder.encode(hrp, publicKey.getCompressedBytes());
  }

  public static Pair<String, ECDSASecp256k1PublicKey> decodeNodeAddressUnknownHrp(String address)
      throws DeserializeException {
    try {
      var hrpAndPubKeyBytes = Bech32mCoder.decode(address);
      return Pair.of(
          hrpAndPubKeyBytes.first(), ECDSASecp256k1PublicKey.fromBytes(hrpAndPubKeyBytes.last()));
    } catch (Bech32DecodeException | PublicKeyException e) {
      throw new DeserializeException("Invalid address", e);
    }
  }
}
