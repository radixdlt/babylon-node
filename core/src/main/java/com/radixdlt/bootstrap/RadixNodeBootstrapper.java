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

import com.google.common.collect.ImmutableSet;
import com.google.common.reflect.TypeToken;
import com.google.inject.Guice;
import com.google.inject.Inject;
import com.google.inject.Injector;
import com.radixdlt.UnstartedRadixNode;
import com.radixdlt.api.system.SystemApi;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.genesis.FixedGenesisLoader;
import com.radixdlt.genesis.GenesisFromPropertiesLoader;
import com.radixdlt.genesis.RawGenesisDataWithHash;
import com.radixdlt.genesis.olympia.GenesisFromOlympiaNodeModule;
import com.radixdlt.genesis.olympia.OlympiaGenesisConfig;
import com.radixdlt.genesis.olympia.OlympiaGenesisService;
import com.radixdlt.networks.FixedNetworkGenesis;
import com.radixdlt.networks.Network;
import com.radixdlt.sbor.StateManagerSbor;
import com.radixdlt.utils.WrappedByteArray;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Stream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class RadixNodeBootstrapper {
  private static final Logger log = LogManager.getLogger();

  public sealed interface RadixNodeBootstrapperHandle {
    CompletableFuture<UnstartedRadixNode> radixNodeFuture();

    void shutdown();

    record Resolved(UnstartedRadixNode radixNode) implements RadixNodeBootstrapperHandle {
      @Override
      public CompletableFuture<UnstartedRadixNode> radixNodeFuture() {
        return CompletableFuture.completedFuture(radixNode);
      }

      @Override
      public void shutdown() {
        // no-op
      }
    }

    record AsyncFromOlympia(OlympiaGenesisBootstrapper olympiaGenesisBootstrapper)
        implements RadixNodeBootstrapperHandle {
      @Override
      public CompletableFuture<UnstartedRadixNode> radixNodeFuture() {
        return olympiaGenesisBootstrapper.radixNodeFuture();
      }

      @Override
      public void shutdown() {
        olympiaGenesisBootstrapper.cleanup();
      }
    }

    record Failed(Exception e) implements RadixNodeBootstrapperHandle {
      @Override
      public CompletableFuture<UnstartedRadixNode> radixNodeFuture() {
        return CompletableFuture.failedFuture(e);
      }

      @Override
      public void shutdown() {
        // no-op
      }
    }
  }

  private final Network network;
  private final Hasher hasher;
  private final RuntimeProperties properties;
  private final GenesisFromPropertiesLoader genesisFromPropertiesLoader;
  private final GenesisStore genesisStore;

  @Inject
  public RadixNodeBootstrapper(
      Network network,
      Hasher hasher,
      RuntimeProperties properties,
      GenesisFromPropertiesLoader genesisFromPropertiesLoader,
      GenesisStore genesisStore) {
    this.network = network;
    this.hasher = hasher;
    this.properties = properties;
    this.genesisFromPropertiesLoader = genesisFromPropertiesLoader;
    this.genesisStore = genesisStore;
  }

  public RadixNodeBootstrapperHandle bootstrapRadixNode() {
    // Genesis source #1: node configuration parameters / genesis file
    // If there is one configured, we always need to read it to memory
    // and calculate its hash.
    final var configuredGenesis = genesisFromPropertiesLoader.loadGenesisDataFromProperties();
    final var configuredGenesisHash = configuredGenesis.map(hasher::hash);

    // Genesis source #2: a fixed genesis associated with the given network
    // We only need its hash at this point.
    final var fixedNetworkGenesisHash =
        network.fixedGenesis().map(FixedNetworkGenesis::genesisDataHash);

    // The genesis stored from previous runs
    final var storedGenesisHash = genesisStore.readGenesisDataHash();

    final var distinctGenesisHashes =
        Stream.of(configuredGenesisHash, fixedNetworkGenesisHash, storedGenesisHash)
            .filter(Optional::isPresent)
            .map(Optional::get)
            .collect(ImmutableSet.toImmutableSet());

    if (distinctGenesisHashes.size() == 0) {
      // No genesis was configured, use Olympia (if configured) or error
      final var useOlympiaFlagIsSet = properties.get("genesis.use_olympia", false);
      if (useOlympiaFlagIsSet) {
        final var olympiaBootstrapper = new OlympiaGenesisBootstrapper();
        olympiaBootstrapper.start();
        return new RadixNodeBootstrapperHandle.AsyncFromOlympia(olympiaBootstrapper);
      } else {
        // TODO(post-babylon): remove Olympia ref from the message below
        return new RadixNodeBootstrapperHandle.Failed(
            new RuntimeException(
                """
                  Radix node couldn't be initialized. No genesis transaction has been configured. Make \
                  sure that either `network.genesis_data` or `network.genesis_data_file` is set or that \
                  you're using a well known network (`network.id`). If you're running an Olympia \
                  node consider using it as your genesis source (`genesis.use_olympia`). Refer to \
                  documentation for more details."""));
      }
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
          rawGenesisDataWithHash =
              new RawGenesisDataWithHash(
                  FixedGenesisLoader.loadGenesisData(network.fixedGenesis().orElseThrow()),
                  fixedNetworkGenesisHash.orElseThrow());
        }
        genesisStore.saveGenesisData(rawGenesisDataWithHash);
      }

      // Start the node using the genesisStore as a (lazy) genesis source
      return new RadixNodeBootstrapperHandle.Resolved(
          new UnstartedRadixNode(properties, network, genesisStore));
    } else {
      // There was more than one genesis configured and the hashes don't match
      return new RadixNodeBootstrapperHandle.Failed(
          new RuntimeException(
              String.format(
                  """
                    Inconsistent genesis configuration. The following genesis sources were read:
                    - properties genesis hash (based on network.genesis_data or network.genesis_data_file): %s
                    - expected network genesis hash (based on network.id): %s
                    - genesis hash stored from previous runs: %s
                    Make sure your configuration is correct (check `network.id` and/or \
                    `network.genesis_data` and/or `network.genesis_data_file`).""",
                  configuredGenesisHash, fixedNetworkGenesisHash, storedGenesisHash)));
    }
  }

  /** A utility class encapsulating the Olympia-based genesis functionality */
  private class OlympiaGenesisBootstrapper {
    private final Injector injector;
    private final CompletableFuture<UnstartedRadixNode> radixNodeFuture;

    OlympiaGenesisBootstrapper() {
      this.injector = Guice.createInjector(new GenesisFromOlympiaNodeModule(properties, network));
      this.radixNodeFuture = new CompletableFuture<>();
    }

    void start() {
      log.info(
          "Olympia-based genesis was configured ({}). Using core API URL of {}",
          network.getLogicalName(),
          injector.getInstance(OlympiaGenesisConfig.class).nodeCoreApiUrl());

      final var systemApi = injector.getInstance(SystemApi.class);
      systemApi.start();

      final var olympiaGenesisService = injector.getInstance(OlympiaGenesisService.class);

      final var genesisDataFuture = olympiaGenesisService.start();

      genesisDataFuture.whenComplete(
          (genesisData, ex) -> {
            this.cleanup();

            if (ex != null) {
              log.warn(
                  """
              Radix node couldn't be initialized. The Olympia-based genesis was configured but \
              an error occurred.""",
                  ex);
              radixNodeFuture.completeExceptionally(ex);
            } else {
              log.info(
                  """
              Genesis data has been successfully received from the Olympia node \
              ({} data chunks). Initializing the Babylon node...""",
                  genesisData.chunks().size());
              final var encodedGenesisData =
                  StateManagerSbor.encode(
                      genesisData, StateManagerSbor.resolveCodec(new TypeToken<>() {}));
              final var genesisDataHash = hasher.hashBytes(encodedGenesisData);
              genesisStore.saveGenesisData(
                  new RawGenesisDataWithHash(
                      new WrappedByteArray(encodedGenesisData), genesisDataHash));
              radixNodeFuture.complete(new UnstartedRadixNode(properties, network, genesisStore));
            }
          });
    }

    CompletableFuture<UnstartedRadixNode> radixNodeFuture() {
      return this.radixNodeFuture;
    }

    void cleanup() {
      injector.getInstance(SystemApi.class).stop();
      injector.getInstance(OlympiaGenesisService.class).shutdown();
    }
  }
}
