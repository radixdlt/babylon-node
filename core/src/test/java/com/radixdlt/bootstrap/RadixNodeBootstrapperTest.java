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

package com.radixdlt.bootstrap;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertTrue;

import com.google.common.collect.ImmutableList;
import com.google.common.reflect.TypeToken;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.Blake2b256Hasher;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.genesis.*;
import com.radixdlt.identifiers.Address;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.discovery.SeedNodesConfigParser;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.NodeSborCodecs;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.utils.Compress;
import com.radixdlt.utils.UInt64;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.*;
import java.nio.file.Paths;
import java.util.Base64;
import java.util.Map;
import org.apache.commons.cli.ParseException;
import org.bouncycastle.util.encoders.Hex;
import org.junit.Ignore;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.mockito.Mockito;

public final class RadixNodeBootstrapperTest {

  @Rule public TemporaryFolder tmpFolder = new TemporaryFolder();

  @Test
  public void test_genesis_hash_verification_from_properties() throws ParseException, IOException {
    final var network = Network.LOCALNET;
    final var storageDirectory = tmpFolder.newFolder();
    final var genesisStore = new GenesisFileStore(new File(storageDirectory, "genesis"));

    final var genesisData1 =
        encodeToCompressedBase64(genesisWithSingleValidator(ECKeyPair.generateNew()));
    final var properties1 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData1));
    final var nodeHandle1 =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties1,
                new GenesisFromPropertiesLoader(properties1),
                genesisStore,
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();
    assertTrue(nodeHandle1 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);

    // Let's try again, same genesis store but a different genesis txn
    final var genesisData2 =
        encodeToCompressedBase64(genesisWithSingleValidator(ECKeyPair.generateNew()));
    final var properties2 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData2));
    final var nodeHandle2 =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties2,
                new GenesisFromPropertiesLoader(properties2),
                genesisStore,
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();
    assertTrue(nodeHandle2 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Failed);
  }

  @Test
  public void test_network_genesis_must_match_properties() throws ParseException, IOException {
    final var storageDirectory = tmpFolder.newFolder();

    // This network has a fixed genesis, so property (if set) must match
    final var network = Network.GENESIS_TEST;

    // This transaction matches the genesis of GENESIS_TEST network
    final var genesisData1 = encodeToCompressedBase64(staticGenesisTestNetworkGenesisData());

    final var properties1 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData1));
    final var nodeHandle1 =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties1,
                new GenesisFromPropertiesLoader(properties1),
                new GenesisFileStore(new File(storageDirectory, "genesis")),
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();
    assertThat(nodeHandle1)
        .isInstanceOf(RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved.class);

    final var storageDirectory2 = tmpFolder.newFolder();
    // This transaction doesn't match the genesis of GENESIS_TEST network
    final var genesisData2 =
        encodeToCompressedBase64(
            genesisWithSingleValidator(ECKeyPair.fromSeed(new byte[] {9, 9, 9})));
    final var properties2 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData2));
    final var nodeHandle2 =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties2,
                new GenesisFromPropertiesLoader(properties2),
                new GenesisFileStore(new File(storageDirectory2, "genesis")),
                storageDirectory2.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();
    assertTrue(nodeHandle2 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Failed);
  }

  @Test
  public void network_can_boot_from_test_genesis() throws ParseException, IOException {
    final var storageDirectory = tmpFolder.newFolder();

    // This network has a fixed genesis, so property (if set) must match
    final var network = Network.GENESIS_TEST;

    // Note - this test relies on `test_genesis.bin` being valid
    // Unignore and run can_generate_test_genesis to regenerate it if necessary
    final var properties = RuntimeProperties.empty();
    final var nodeHandle =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties,
                new GenesisFromPropertiesLoader(properties),
                new GenesisFileStore(new File(storageDirectory, "genesis")),
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();
    assertTrue(nodeHandle instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);
  }

  @Test
  @Ignore
  public void can_generate_test_genesis() throws IOException {
    final var rawDataWithHash =
        RawGenesisDataWithHash.fromGenesisData(staticGenesisTestNetworkGenesisData());
    final var testGenesisDataFile =
        Paths.get("src", "test", "resources", "genesis", "test_genesis.bin").toFile();
    if (!testGenesisDataFile.exists()) {
      throw new RuntimeException("Please manually create resources/genesis/test_genesis.bin");
    }
    assert (testGenesisDataFile.delete());
    assert (testGenesisDataFile.createNewFile());
    System.out.printf("Outputting to: %s%n", testGenesisDataFile);
    try (FileOutputStream outputStream = new FileOutputStream(testGenesisDataFile)) {
      final var compressed = Compress.compress(rawDataWithHash.genesisData().value());
      outputStream.write(compressed);
    } catch (IOException e) {
      throw new RuntimeException("Couldn't write to the genesis data file", e);
    }
    System.out.printf(
        "The hash (for updating Network.java) is: %s%n",
        Hex.toHexString(rawDataWithHash.genesisDataHash().asBytes()));
  }

  private GenesisData staticGenesisTestNetworkGenesisData() {
    return genesisWithSingleValidator(ECKeyPair.fromSeed(new byte[] {1, 2, 3}));
  }

  @Test
  public void test_loading_genesis_from_properties_file() throws IOException, ParseException {
    final var storageDirectory = tmpFolder.newFolder();

    final var network = Network.LOCALNET;
    // Just a sanity check in case someone adds it later :)
    // This test requires a network without a fixed genesis
    assertTrue(network.fixedGenesis().isEmpty());

    final var genesisFile = tmpFolder.newFile();
    final var genesisData = GenesisData.testingDefaultEmpty();
    final var genesisDataBytes =
        NodeSborCodecs.encode(genesisData, NodeSborCodecs.resolveCodec(new TypeToken<>() {}));
    final var compressedGenesisDataBytes = Compress.compress(genesisDataBytes);
    try (var outputStream = new FileOutputStream(genesisFile)) {
      outputStream.write(compressedGenesisDataBytes);
    }

    final var properties =
        RuntimeProperties.defaultWithOverrides(
            Map.of(
                "network.genesis_data_file",
                genesisFile.getAbsolutePath(),
                "network.genesis_data",
                ""));
    final var genesisFileStore = new GenesisFileStore(new File(storageDirectory, "genesis"));
    final var nodeHandle =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties,
                new GenesisFromPropertiesLoader(properties),
                genesisFileStore,
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();

    assertTrue(nodeHandle instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);
  }

  @Test
  public void test_loading_genesis_from_genesis_data_file() throws IOException, ParseException {
    final var storageDirectory = tmpFolder.newFolder();

    final var network = Network.LOCALNET;
    // Just a sanity check in case someone adds it later :)
    // This test requires a network without a fixed genesis
    assertTrue(network.fixedGenesis().isEmpty());

    final var genesisFileStore = new GenesisFileStore(new File(storageDirectory, "genesis"));
    genesisFileStore.saveGenesisData(
        RawGenesisDataWithHash.fromGenesisData(GenesisData.testingDefaultEmpty()));

    // Explicitly no genesis configured, but there's one in the genesis store
    final var properties =
        RuntimeProperties.defaultWithOverrides(
            Map.of(
                "network.genesis_data", "",
                "network.genesis_data_file", ""));
    final var nodeHandle =
        new RadixNodeBootstrapper(
                ECKeyPair.generateNew().getPublicKey(),
                network,
                Addressing.ofNetwork(network),
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties,
                new GenesisFromPropertiesLoader(properties),
                genesisFileStore,
                storageDirectory.getAbsolutePath(),
                Mockito.mock(SeedNodesConfigParser.class))
            .bootstrapRadixNode();

    assertTrue(nodeHandle instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);
  }

  private GenesisData genesisWithSingleValidator(ECKeyPair key) {
    return new GenesisData(
        UInt64.fromNonNegativeLong(1),
        1L,
        GenesisConsensusManagerConfig.testingDefaultEmpty(),
        ImmutableList.of(
            new GenesisDataChunk.Validators(
                ImmutableList.of(
                    new GenesisValidator(
                        key.getPublicKey(),
                        true,
                        true,
                        Decimal.ONE,
                        ImmutableList.of(),
                        Address.virtualAccountAddress(key.getPublicKey()))))),
        GenesisData.DEFAULT_TEST_FAUCET_SUPPLY,
        GenesisData.NO_SCENARIOS);
  }

  private String encodeToCompressedBase64(GenesisData genesisData) throws IOException {
    final var encoded =
        NodeSborCodecs.encode(genesisData, NodeSborCodecs.resolveCodec(new TypeToken<>() {}));
    final var compressed = Compress.compress(encoded);
    return Base64.getEncoder().encodeToString(compressed);
  }
}
