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

package com.radixdlt.networks;

import com.google.common.hash.HashCode;
import java.util.Optional;
import java.util.function.Predicate;
import java.util.stream.Stream;
import org.bouncycastle.util.encoders.Hex;

@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public enum Network {

  /// Public Facing Permanent Networks (0x00 - 0x09)
  // - mainnet
  // - stokenet
  // TODO(post-babylon): add a fixed genesis from resources for mainnet
  MAINNET(1 /* 0x01 */, "mainnet", "rdx"),
  STOKENET(2 /* 0x02 */, "stokenet", "tdx_2_"),

  // Temporary networks that match Olympia - for genesis testing mostly
  OLYMPIA_RELEASENET(3, "releasenet", "tdx_3_"),
  OLYMPIA_RCNET(4, "rcnet", "tdx_4_"),
  OLYMPIA_MILESTONENET(5, "milestonenet", "tdx_5_"),
  OLYMPIA_DEVOPSNET(6, "devopsnet", "tdx_6_"),
  OLYMPIA_SANDPITNET(7, "sandpitnet", "tdx_7_"),

  /// Babylon Temporary Testnets (0x0a - 0x0f)
  // - adapanet = Babylon Alphanet, after Adapa
  // - nebunet = Babylon Betanet, after Nebuchadnezzar
  // - kisharnet = Babylon RCnet v1, after Kishar (from the Babylonian Creation Story)
  // - ansharnet = Babylon RCnet v2, after Anshar (from the Babylonian Creation Story)
  // - zabanet = Babylon RCnet v3, after Zababa, a war god from the Mesopotamian city of Kish
  ADAPANET(10 /* 0x0a */, "adapanet", "tdx_a_"),
  NEBUNET(11 /* 0x0b */, "nebunet", "tdx_b_"),
  KISHARNET(12 /* 0x0c */, "kisharnet", "tdx_c_"),
  ANSHARNET(13 /* 0x0d */, "ansharnet", "tdx_d_"),
  ZABANET(14 /* 0x0e */, "zabanet", "tdx_e_"),

  /// RDX Development - Semi-permanent Testnets (start with 0x2)
  // - gilganet = Node integration network, after Gilgamesh
  // - enkinet = Misc Network 1, after Enki / Enkidu
  // - hammunet = Misc Network 2, after Hammurabi (typically a twin / staging Network for testing
  // new releases to the primary public environment)
  // - nergalnet = A Network for DevOps testing, after the Mesopotamian god Nergal
  // - mardunet = Network Gateway integration network, after the Babylonian god Marduk
  // - dumunet = A network for load testing, after the Mesopotamian god Dumuzid, the god of
  // shepherds
  GILGANET(32 /* 0x20 */, "gilganet", "tdx_20_"),
  ENKINET(33 /* 0x21 */, "enkinet", "tdx_21_"),
  HAMMUNET(34 /* 0x22 */, "hammunet", "tdx_22_"),
  NERGALNET(35 /* 0x23 */, "nergalnet", "tdx_23_"),
  MARDUNET(36 /* 0x24 */, "mardunet", "tdx_24_"),
  DUMUNET(37 /* 0x25 */, "dumunet", "tdx_25_"),

  /// Ephemeral Networks (start with 0xF)
  // - localnet = The network used when running locally in development
  // - inttestnet = The network used when running integration tests
  LOCALNET(240 /* 0xF0 */, "localnet", "loc"),
  INTEGRATIONTESTNET(241 /* 0xF1 */, "inttestnet", "test"),
  LOCALSIMULATOR(242 /* 0xF1 */, "simulator", "sim"),
  // A dedicated network for testing genesis
  GENESIS_TEST(
      243 /* 0xF2 */,
      "genesis_test",
      "genesis_test",
      FixedNetworkGenesis.resource(
          HashCode.fromBytes(
              Hex.decode("ac79e815b39beb756d9afe261e8c4deff4ed95f8fcc59af001eef060d820b266")),
          "genesis/test_genesis.bin"));

  private final int intId;
  private final byte byteId;
  private final String logicalName;
  private final String hrpSuffix;
  private final String packageHrp;
  private final String normalComponentHrp;
  private final String accountComponentHrp;
  private final String internalVaultHrp;
  private final String validatorHrp;
  private final String resourceHrp;
  private final String nodeHrp;
  private final String intentHashHrp;
  private final String notarizedTransactionHashHrp;
  private final Optional<FixedNetworkGenesis> maybeFixedGenesis;

  Network(
      int id,
      String logicalName,
      String hrpSuffix,
      Optional<FixedNetworkGenesis> maybeFixedGenesis) {
    if (id <= 0 || id > 255) {
      throw new IllegalArgumentException(
          "Id should be between 1 and 255 so it isn't default(int) = 0 and will fit into a byte if"
              + " we change in future");
    }
    this.intId = id;
    this.byteId = (byte) id;
    this.logicalName = logicalName;
    this.hrpSuffix = hrpSuffix;
    this.packageHrp = "package_" + hrpSuffix;
    this.normalComponentHrp = "component_" + hrpSuffix;
    this.accountComponentHrp = "account_" + hrpSuffix;
    this.internalVaultHrp = "internal_vault_" + hrpSuffix;
    this.validatorHrp = "validator_" + hrpSuffix;
    this.resourceHrp = "resource_" + hrpSuffix;
    this.nodeHrp = "node_" + hrpSuffix;
    this.intentHashHrp = "txid_" + hrpSuffix;
    this.notarizedTransactionHashHrp = "notarizedtransaction_" + hrpSuffix;
    this.maybeFixedGenesis = maybeFixedGenesis;
  }

  Network(int id, String logicalName, String hrpSuffix, FixedNetworkGenesis fixedGenesis) {
    this(id, logicalName, hrpSuffix, Optional.of(fixedGenesis));
  }

  Network(int id, String logicalName, String hrpSuffix) {
    this(id, logicalName, hrpSuffix, Optional.empty());
  }

  public String getPackageHrp() {
    return packageHrp;
  }

  public String getNormalComponentHrp() {
    return normalComponentHrp;
  }

  public String getAccountComponentHrp() {
    return accountComponentHrp;
  }

  public String getInternalVaultHrp() {
    return internalVaultHrp;
  }

  public String getValidatorHrp() {
    return validatorHrp;
  }

  public String getResourceHrp() {
    return resourceHrp;
  }

  public String getNodeHrp() {
    return nodeHrp;
  }

  public String getIntentHashHrp() {
    return intentHashHrp;
  }

  public String getNotarizedTransactionHashHrp() {
    return notarizedTransactionHashHrp;
  }

  public int getId() {
    return intId;
  }

  public byte getByteId() {
    return byteId;
  }

  public String getLogicalName() {
    return logicalName;
  }

  public String getHrpSuffix() {
    return hrpSuffix;
  }

  public Optional<FixedNetworkGenesis> fixedGenesis() {
    return this.maybeFixedGenesis;
  }

  public static Optional<Network> ofId(int id) {
    return find(network -> network.intId == id);
  }

  public static Network ofIdOrThrow(int networkId) {
    return ofId(networkId)
        .orElseThrow(
            () ->
                new RuntimeException(
                    "Provided Network ID does not match any known networks: " + networkId));
  }

  public static Optional<Network> ofName(String logicalName) {
    return find(network -> network.logicalName.equalsIgnoreCase(logicalName));
  }

  private static Optional<Network> find(Predicate<Network> predicate) {
    return Stream.of(values()).filter(predicate).findAny();
  }
}
