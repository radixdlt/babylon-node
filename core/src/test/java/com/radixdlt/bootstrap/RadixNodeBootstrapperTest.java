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

import static org.junit.Assert.assertTrue;

import com.google.common.collect.ImmutableList;
import com.google.common.reflect.TypeToken;
import com.radixdlt.consensus.Blake2b256Hasher;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.genesis.*;
import com.radixdlt.identifiers.Address;
import com.radixdlt.networks.Network;
import com.radixdlt.sbor.StateManagerSbor;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.serialization.TestSetupUtils;
import com.radixdlt.utils.Compress;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.BufferedWriter;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.util.Base64;
import java.util.Map;
import org.apache.commons.cli.ParseException;
import org.junit.BeforeClass;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class RadixNodeBootstrapperTest {
  private static final Hasher HASHER = new Blake2b256Hasher(DefaultSerialization.getInstance());

  @Rule public TemporaryFolder tmpFolder = new TemporaryFolder();

  @BeforeClass
  public static void beforeClass() {
    TestSetupUtils.installBouncyCastleProvider();
  }

  @Test
  public void test_genesis_hash_verification_from_properties() throws ParseException, IOException {
    final var network = Network.LOCALNET;
    final var genesisStore = new GenesisFileStore(new File(tmpFolder.getRoot(), "genesis.bin"));

    final var genesisData1 = encodeToString(genesisWithSingleValidator(ECKeyPair.generateNew()));
    final var properties1 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData1));
    final var nodeHandle1 =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties1,
                new GenesisFromPropertiesLoader(properties1),
                genesisStore)
            .bootstrapRadixNode();
    assertTrue(nodeHandle1 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);

    // Let's try again, same genesis store but a different genesis txn
    final var genesisData2 = encodeToString(genesisWithSingleValidator(ECKeyPair.generateNew()));
    final var properties2 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData2));
    final var nodeHandle2 =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties2,
                new GenesisFromPropertiesLoader(properties2),
                genesisStore)
            .bootstrapRadixNode();
    assertTrue(nodeHandle2 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Failed);
  }

  @Test
  public void test_network_genesis_must_match_properties() throws ParseException, IOException {
    // This network has a fixed genesis, so property (if set) must match
    final var network = Network.GENESIS_TEST;

    // This transaction matches the genesis of GENESIS_TEST network
    final var genesisData1 =
        encodeToString(genesisWithSingleValidator(ECKeyPair.fromSeed(new byte[] {1, 2, 3})));
    final var properties1 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData1));
    final var nodeHandle1 =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties1,
                new GenesisFromPropertiesLoader(properties1),
                new GenesisFileStore(new File(tmpFolder.newFolder(), "genesis.bin")))
            .bootstrapRadixNode();
    assertTrue(nodeHandle1 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);

    // This transaction doesn't match the genesis of GENESIS_TEST network
    final var genesisData2 =
        encodeToString(genesisWithSingleValidator(ECKeyPair.fromSeed(new byte[] {9, 9, 9})));
    final var properties2 =
        RuntimeProperties.defaultWithOverrides(Map.of("network.genesis_data", genesisData2));
    final var nodeHandle2 =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties2,
                new GenesisFromPropertiesLoader(properties2),
                new GenesisFileStore(new File(tmpFolder.newFolder(), "genesis.bin")))
            .bootstrapRadixNode();
    assertTrue(nodeHandle2 instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Failed);
  }

  @Test
  public void test_loading_genesis_from_properties_file() throws IOException, ParseException {
    final var network = Network.LOCALNET;
    // Just a sanity check in case someone adds it later :)
    // This test requires a network without a fixed genesis
    assertTrue(network.fixedGenesis().isEmpty());

    final var genesisFile = tmpFolder.newFile();
    final var genesisData = GenesisData.testingDefaultEmpty();
    final var genesisDataBytes =
        StateManagerSbor.encode(genesisData, StateManagerSbor.resolveCodec(new TypeToken<>() {}));
    final var compressedGenesisDataBytes = Compress.compress(genesisDataBytes);
    final var genesisDataBase64 = Base64.getEncoder().encodeToString(compressedGenesisDataBytes);
    try (var writer = new BufferedWriter(new FileWriter(genesisFile))) {
      writer.write(genesisDataBase64);
    }

    final var properties =
        RuntimeProperties.defaultWithOverrides(
            Map.of("network.genesis_file", genesisFile.getAbsolutePath()));
    final var genesisFileStore = new GenesisFileStore(tmpFolder.newFile());
    final var nodeHandle =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties,
                new GenesisFromPropertiesLoader(properties),
                genesisFileStore)
            .bootstrapRadixNode();

    assertTrue(nodeHandle instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);
  }

  @Test
  public void test_loading_genesis_from_genesis_file() throws IOException, ParseException {
    final var network = Network.LOCALNET;
    // Just a sanity check in case someone adds it later :)
    // This test requires a network without a fixed genesis
    assertTrue(network.fixedGenesis().isEmpty());

    final var genesisFileStore = new GenesisFileStore(tmpFolder.newFile());
    genesisFileStore.saveGenesisData(
        RawGenesisDataWithHash.fromGenesisData(GenesisData.testingDefaultEmpty(), HASHER));

    // Explicitly no genesis configured, but there's one in the genesis store
    final var properties =
        RuntimeProperties.defaultWithOverrides(
            Map.of(
                "network.genesis_data", "",
                "network.genesis_file", ""));
    final var nodeHandle =
        new RadixNodeBootstrapper(
                network,
                new Blake2b256Hasher(DefaultSerialization.getInstance()),
                properties,
                new GenesisFromPropertiesLoader(properties),
                genesisFileStore)
            .bootstrapRadixNode();

    assertTrue(nodeHandle instanceof RadixNodeBootstrapper.RadixNodeBootstrapperHandle.Resolved);
  }

  private GenesisData genesisWithSingleValidator(ECKeyPair key) {
    return new GenesisData(
        UInt64.fromNonNegativeLong(1),
        UInt32.fromNonNegativeInt(1),
        UInt64.fromNonNegativeLong(1),
        UInt64.fromNonNegativeLong(1),
        1,
        ImmutableList.of(
            new GenesisDataChunk.Validators(
                ImmutableList.of(
                    new GenesisValidator(
                        key.getPublicKey(),
                        true,
                        true,
                        ImmutableList.of(),
                        Address.virtualAccountAddress(key.getPublicKey()))))));
  }

  private String encodeToString(GenesisData genesisData) throws IOException {
    final var encoded =
        StateManagerSbor.encode(genesisData, StateManagerSbor.resolveCodec(new TypeToken<>() {}));
    final var compressed = Compress.compress(encoded);
    return Base64.getEncoder().encodeToString(compressed);
  }
}
