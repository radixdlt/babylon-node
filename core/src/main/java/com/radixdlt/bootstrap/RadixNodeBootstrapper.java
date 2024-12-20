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

import static com.radixdlt.lang.Unit.unit;

import com.google.common.collect.ImmutableSet;
import com.google.inject.Guice;
import com.google.inject.Inject;
import com.google.inject.util.Modules;
import com.radixdlt.RadixNode;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.genesis.FixedGenesisLoader;
import com.radixdlt.genesis.GenesisFromPropertiesLoader;
import com.radixdlt.genesis.RawGenesisDataWithHash;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Unit;
import com.radixdlt.networks.FixedNetworkGenesis;
import com.radixdlt.networks.Network;
import com.radixdlt.store.NodeStorageLocation;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.File;
import java.io.IOException;
import java.util.Optional;
import java.util.stream.Stream;

public final class RadixNodeBootstrapper {
  private final Network network;
  private final Hasher hasher;
  private final RuntimeProperties properties;
  private final GenesisFromPropertiesLoader genesisFromPropertiesLoader;
  private final GenesisStore genesisStore;
  private final File nodeStorageDir;

  public static RadixNode runNewNode(RuntimeProperties properties) {
    final var bootstrapperModule =
        Guice.createInjector(
            Modules.requireAtInjectOnConstructorsModule(),
            Modules.disableCircularProxiesModule(),
            new RadixNodeBootstrapperModule(properties));
    final var bootstrapper = bootstrapperModule.getInstance(RadixNodeBootstrapper.class);
    return bootstrapper.bootstrapRadixNode();
  }

  @Inject
  RadixNodeBootstrapper(
      Network network,
      Hasher hasher,
      RuntimeProperties properties,
      GenesisFromPropertiesLoader genesisFromPropertiesLoader,
      GenesisStore genesisStore,
      @NodeStorageLocation String nodeStorageLocation) {
    this.network = network;
    this.hasher = hasher;
    this.properties = properties;
    this.genesisFromPropertiesLoader = genesisFromPropertiesLoader;
    this.genesisStore = genesisStore;
    this.nodeStorageDir = new File(nodeStorageLocation);
  }

  private RadixNode bootstrapRadixNode() {
    // An early check for storage misconfiguration
    final var storageVerifyResult = verifyNodeStorageDirIsWritable();
    if (storageVerifyResult.isError()) {
      throw new RuntimeException(storageVerifyResult.unwrapError());
    }

    // Genesis source #1: node configuration parameters / genesis file
    // If there is one configured, we always need to read it to memory
    // and calculate its hash.
    final var configuredGenesis = genesisFromPropertiesLoader.loadGenesisDataFromProperties();
    final var configuredGenesisHash = configuredGenesis.map(hasher::hash);

    // Genesis source #2: a fixed genesis associated with the given network
    // We only need its hash at this point.
    // Note that to save work on start-up with a large genesis, we verify this hash is correct only
    // when the genesis data is used.
    final var fixedNetworkGenesisHardcodedHash =
        network.fixedGenesis().map(FixedNetworkGenesis::genesisDataHash);

    // The genesis stored from previous runs
    final var storedGenesisHash = genesisStore.readGenesisDataHash();

    final var distinctGenesisHashes =
        Stream.of(configuredGenesisHash, fixedNetworkGenesisHardcodedHash, storedGenesisHash)
            .filter(Optional::isPresent)
            .map(Optional::get)
            .collect(ImmutableSet.toImmutableSet());

    if (distinctGenesisHashes.isEmpty()) {
      // No genesis was configured
      throw new RuntimeException(
          """
              Radix node couldn't be initialized. No genesis transaction has been configured. Make \
              sure that either `network.genesis_data` or `network.genesis_data_file` is set or that \
              you're using a well known network (`network.id`).""");
    } else if (distinctGenesisHashes.size() == 1) {
      // All genesis sources agree, we can proceed
      // Store the genesis, if needed
      if (storedGenesisHash.isEmpty()) {
        RawGenesisDataWithHash rawGenesisDataWithHash;
        if (configuredGenesis.isPresent()) {
          // If configured genesis is set it must have been already loaded to memory,
          // so prioritise if over the fixed network genesis.
          rawGenesisDataWithHash =
              new RawGenesisDataWithHash(
                  configuredGenesis.orElseThrow(), configuredGenesisHash.orElseThrow());
        } else {
          // No configured genesis, so load the fixed network genesis.
          var genesisData =
              FixedGenesisLoader.loadGenesisData(network.fixedGenesis().orElseThrow());
          var genesisDataHash = hasher.hash(genesisData);
          var hardcodedHash = fixedNetworkGenesisHardcodedHash.orElse(null);
          // Sanity check that the hardcoded hash is correct
          if (!genesisDataHash.equals(hardcodedHash)) {
            throw new RuntimeException(
                String.format(
                    "The fixed genesis definition for network %s is inconsistent. It claims to have"
                        + " hash (%s) but actually has hash (%s).",
                    network, hardcodedHash, genesisDataHash));
          }
          rawGenesisDataWithHash =
              new RawGenesisDataWithHash(
                  FixedGenesisLoader.loadGenesisData(network.fixedGenesis().orElseThrow()),
                  fixedNetworkGenesisHardcodedHash.orElseThrow());
        }
        genesisStore.saveGenesisData(rawGenesisDataWithHash);
      }

      // Start the node using the genesisStore as a (lazy) genesis source
      return RadixNode.run(properties, network, genesisStore);
    } else {
      // There was more than one genesis configured and the hashes don't match
      throw new RuntimeException(
          String.format(
              """
                    Inconsistent genesis configuration. The following genesis sources were read:
                    - properties genesis hash (based on network.genesis_data or network.genesis_data_file): %s
                    - expected network genesis hash (based on network.id): %s
                    - genesis hash stored from previous runs: %s
                    Make sure your configuration is correct (check `network.id` and/or \
                    `network.genesis_data` and/or `network.genesis_data_file`).""",
              configuredGenesisHash, fixedNetworkGenesisHardcodedHash, storedGenesisHash));
    }
  }

  private Result<Unit, String> verifyNodeStorageDirIsWritable() {
    if (!nodeStorageDir.exists()) {
      if (!nodeStorageDir.mkdirs()) {
        return Result.error(
            String.format(
                "Node storage directory (%s) doesn't exist and it couldn't be created. Make sure"
                    + " that the directory specified in `db.location` is writeable and accessible"
                    + " by the current user.",
                nodeStorageDir));
      }
    }
    final var testFile = new File(nodeStorageDir, ".storage_test");
    final var writeErrorMsg =
        String.format(
            "Couldn't write to node storage directory (%s). Make sure that the directory specified"
                + " in `db.location` is writeable and accessible by the current user.",
            nodeStorageDir);
    try {
      if (!testFile.exists()) {
        if (!testFile.createNewFile()) {
          return Result.error(writeErrorMsg);
        }
      }
      if (!testFile.delete()) {
        return Result.error(writeErrorMsg);
      }
    } catch (IOException e) {
      return Result.error(writeErrorMsg);
    }

    return Result.success(unit());
  }
}
