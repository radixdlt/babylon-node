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

package com.radixdlt.cli;

import com.google.common.collect.ImmutableList;
import com.google.common.reflect.TypeToken;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.identifiers.Address;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.StateManagerSbor;
import com.radixdlt.utils.Bytes;
import com.radixdlt.utils.Compress;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UniqueListBuilder;
import java.io.File;
import java.util.Base64;
import java.util.Map;
import java.util.Set;
import java.util.stream.IntStream;
import org.apache.commons.cli.*;

/** Generates the genesis data for the Babylon Radix network */
public final class GenerateGenesis {

  // Genesis parameters for XRD allocation for testnets
  private static final Set<Network> NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT =
      Set.of(
          Network.GILGANET,
          Network.ENKINET,
          Network.HAMMUNET,
          Network.MARDUNET,
          Network.DUMUNET,
          Network.NERGALNET,
          Network.NEBUNET,
          Network.KISHARNET,
          Network.ANSHARNET);

  private static final Set<Network> PRODUCTION_NETWORKS = Set.of(Network.MAINNET);
  private static final Set<Network> NETWORKS_TO_DISABLE_FAUCET = PRODUCTION_NETWORKS;
  private static final Set<Network> NETWORKS_TO_DISABLE_SCENARIOS = NETWORKS_TO_DISABLE_FAUCET;

  private static final Set<Network> NETWORKS_TO_ENSURE_PRODUCTION_EMISSIONS =
      Set.of(Network.KISHARNET, Network.ANSHARNET, Network.STOKENET, Network.MAINNET);

  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE =
      Decimal.of(700_000_000_000L); // 70% XRD_MAX_SUPPLY
  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1_000_000_000L); // 0.1% XRD_MAX_SUPPLY
  private static final ECDSASecp256k1PublicKey GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY =
      ECDSASecp256k1PublicKey.tryFromHex(
              "026f08db98ef1d0231eb15580da9123db8e25aa1747c8c32e5fd2ec47b8db73d5c")
          .unwrap();
  private static final Decimal GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1); // Allow it to be easily changed in eg tests

  private GenerateGenesis() {}

  public static void main(String[] args) throws Exception {
    Options options = new Options();
    options.addOption("h", "help", false, "Show usage information (this message)");
    options.addOption("p", "public-keys", true, "Specify validator keys");
    options.addOption("v", "validator-count", true, "Specify number of validators to generate");
    options.addOption("n", "network", true, "Specify the network name or ID");

    CommandLineParser parser = new DefaultParser();
    CommandLine cmd = parser.parse(options, args);
    if (!cmd.getArgList().isEmpty()) {
      System.err.println("Extra arguments: " + String.join(" ", cmd.getArgList()));
      usage(options);
      return;
    }

    if (cmd.hasOption('h')) {
      usage(options);
      return;
    }

    final var validatorsBuilder = new UniqueListBuilder<ECDSASecp256k1PublicKey>();
    if (cmd.getOptionValue("p") != null) {
      var hexKeys = cmd.getOptionValue("p").split(",");
      for (var hexKey : hexKeys) {
        final var publicKey = ECDSASecp256k1PublicKey.fromHex(hexKey);
        if (validatorsBuilder.contains(publicKey)) {
          throw new RuntimeException("Duplicate validator key: " + hexKey);
        }
        validatorsBuilder.insertIfMissingAndGetIndex(publicKey);
      }
    }
    final int validatorsCount =
        cmd.getOptionValue("v") != null ? Integer.parseInt(cmd.getOptionValue("v")) : 0;
    var generatedValidatorKeys = PrivateKeys.numeric(6).limit(validatorsCount).toList();
    generatedValidatorKeys.stream()
        .map(ECKeyPair::getPublicKey)
        .forEach(validatorsBuilder::insertIfMissingAndGetIndex);
    IntStream.range(0, generatedValidatorKeys.size())
        .forEach(
            i -> {
              System.out.format(
                  "export RADIXDLT_VALIDATOR_%s_PRIVKEY=%s%n",
                  i, Bytes.toBase64String(generatedValidatorKeys.get(i).getPrivateKey()));
              System.out.format(
                  "export RADIXDLT_VALIDATOR_%s_PUBKEY=%s%n",
                  i,
                  Addressing.ofNetwork(Network.LOCALNET)
                      .encodeNodeAddress(generatedValidatorKeys.get(i).getPublicKey()));
            });

    final var network = parseNetwork(cmd.getOptionValue("n"));
    final var validators = validatorsBuilder.build();
    final var genesisData = createGenesisData(network, validators);
    final var encodedGenesisData =
        StateManagerSbor.encode(genesisData, StateManagerSbor.resolveCodec(new TypeToken<>() {}));
    final var compressedGenesisData = Compress.compress(encodedGenesisData);
    final var compressedGenesisDataBase64 =
        Base64.getEncoder().encodeToString(compressedGenesisData);

    if (validatorsCount > 0) {
      // For some reason we use the presence of the validatorsCount command to decide whether we are
      // running this for docker, or for an actual environment...
      System.out.format("export RADIXDLT_GENESIS_DATA=%s%n", compressedGenesisDataBase64);
    } else {
      System.out.format(
          "The base64-encoded genesis for use with RADIXDLT_GENESIS_DATA / network.genesis_data"
              + " is:%n%s%n",
          compressedGenesisDataBase64);
      System.out.println();
      var filePath = new File("genesis_data.bin").getAbsolutePath();
      System.out.format(
          "Also saving the genesis data file in binary format for use with"
              + " RADIXDLT_GENESIS_DATA_FILE / network.genesis_data_file to: %s%n",
          filePath);
      try (var outputStream = new java.io.FileOutputStream(filePath)) {
        outputStream.write(compressedGenesisData);
      }
    }
  }

  private static Network parseNetwork(String commandLineValue) {
    final var asName = Network.ofName(commandLineValue);
    if (asName.isPresent()) {
      return asName.get();
    }
    try {
      final var asId = Network.ofId(Integer.parseInt(commandLineValue));
      if (asId.isPresent()) {
        return asId.get();
      }
      throw new RuntimeException("Was not a valid network id");
    } catch (Exception ex) {
      throw new RuntimeException(
          "Could not resolve (lower case) logical network name, or network id. Try specifying eg"
              + " -Pnetwork=gilganet",
          ex);
    }
  }

  private static GenesisData createGenesisData(
      Network network, ImmutableList<ECDSASecp256k1PublicKey> validators) {
    final var usePowerfulStakingAccount =
        NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT.contains(network);

    final var stakeAmount =
        usePowerfulStakingAccount
            ? GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR
            : GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR;

    final var stakingAccount =
        usePowerfulStakingAccount
            ? Address.virtualAccountAddress(GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY)
            : Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());

    final Map<ECDSASecp256k1PublicKey, Decimal> xrdBalances =
        usePowerfulStakingAccount
            ? Map.of(
                GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY,
                GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE)
            : Map.of();

    var consensusConfig =
        PRODUCTION_NETWORKS.contains(network)
            ? GenesisConsensusManagerConfig.Builder.productionDefaults()
            : GenesisConsensusManagerConfig.Builder.testEnvironmentDefaults();

    final var mustUseProductionEmissions =
        NETWORKS_TO_ENSURE_PRODUCTION_EMISSIONS.contains(network);
    if (!mustUseProductionEmissions && !usePowerfulStakingAccount) {
      consensusConfig =
          consensusConfig.totalEmissionXrdPerEpoch(
              GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR.divide(10000));
    }

    final var useFaucet = !NETWORKS_TO_DISABLE_FAUCET.contains(network);
    final var scenariosToRun =
        NETWORKS_TO_DISABLE_SCENARIOS.contains(network)
            ? GenesisData.NO_SCENARIOS
            : GenesisData.ALL_SCENARIOS;

    return GenesisBuilder.createGenesisWithValidatorsAndXrdBalances(
        validators,
        stakeAmount,
        stakingAccount,
        xrdBalances,
        consensusConfig,
        useFaucet,
        scenariosToRun);
  }

  private static void usage(Options options) {
    HelpFormatter formatter = new HelpFormatter();
    formatter.printHelp(GenerateGenesis.class.getSimpleName(), options, true);
  }
}
