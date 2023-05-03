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

package com.radixdlt;

import com.google.inject.Guice;
import com.google.inject.Injector;
import com.radixdlt.api.system.SystemApi;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisFromPropertiesLoader;
import com.radixdlt.genesis.PreGenesisNodeModule;
import com.radixdlt.genesis.olympia.GenesisFromOlympiaNodeModule;
import com.radixdlt.genesis.olympia.OlympiaGenesisConfig;
import com.radixdlt.genesis.olympia.OlympiaGenesisService;
import com.radixdlt.lang.Either;
import com.radixdlt.networks.FixedNetworkGenesis;
import com.radixdlt.networks.Network;
import com.radixdlt.utils.BooleanUtils;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class RadixNodeBootstrapper {
  private static final Logger log = LogManager.getLogger();

  public sealed interface RadixNodeBootstrapperHandle {
    CompletableFuture<RadixNode> radixNodeFuture();

    void shutdown();

    record Resolved(RadixNode radixNode) implements RadixNodeBootstrapperHandle {
      @Override
      public CompletableFuture<RadixNode> radixNodeFuture() {
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
      public CompletableFuture<RadixNode> radixNodeFuture() {
        return olympiaGenesisBootstrapper.radixNodeFuture();
      }

      @Override
      public void shutdown() {
        olympiaGenesisBootstrapper.cleanup();
      }
    }

    record Failed(Exception e) implements RadixNodeBootstrapperHandle {
      @Override
      public CompletableFuture<RadixNode> radixNodeFuture() {
        return CompletableFuture.failedFuture(e);
      }

      @Override
      public void shutdown() {
        // no-op
      }
    }
  }

  public static RadixNodeBootstrapperHandle bootstrapRadixNode(RuntimeProperties properties) {
    final var networkEither = readNetworkFromProperties(properties);
    final Network network;
    if (networkEither.isLeft()) {
      return new RadixNodeBootstrapperHandle.Failed(networkEither.unwrapLeft());
    } else {
      network = networkEither.unwrapRight();
    }

    final var fixedNetworkGenesis =
        network.fixedGenesis().map(RadixNodeBootstrapper::resolveFixedNetworkGenesis);
    final var useOlympiaFlagIsSet =
        properties.get("genesis.use_olympia", BooleanUtils::parseBoolean).orElse(false);

    /*
     The Radix bootstrapper utilizes two (or tree including the node itself) top-level Guice modules:
     - PreGenesisNodeModule:
        A basic module containing utils for loading the genesis transaction from properties.
        This module is always instantiated.

     - GenesisFromOlympiaNodeModule:
        A module used to acquire the genesis transaction from a running Olympia node.
        This is only instantiated if both are true:
        a) there's no genesis transaction configured in properties or already executed in the database
        b) Olympia-based genesis has been configured in properties
        This module is managed by an encapsulating utility class: OlympiaGenesisBootstrapper.

     - RadixNodeModule:
        A complete Radix node module that requires a known genesis transaction in order to start.
        This module is managed by the RadixNode class.
    */
    final var preGenesisInjector =
        Guice.createInjector(new PreGenesisNodeModule(properties, network));

    final Optional<GenesisData> configuredGenesis =
        preGenesisInjector
            .getInstance(GenesisFromPropertiesLoader.class)
            .loadGenesisDataFromProperties();

    /* We have three sources of a genesis transaction at this point:
     *  - a fixed genesis transaction associated with a given network
     * 	- a genesis transaction configured in properties (either raw bytes or loaded from a file)
     * 	- a genesis transaction stored in a database
     * We can use either one, but we need to make sure they match, if more than one
     * is configured (to protect against node misconfiguration).
     * TODO(genesis): the check that the configured genesis data matches the database state has been
     *                temporarily removed, bring it back at some point (or something similar)
     *
     * If neither genesis transaction is present, we may try to acquire it from a running
     * Olympia node, if it has been configured. This mode will be (or already was :)) used to
     * coordinate the death of the Olympia-based Radix network and its rebirth on the Babylon side.
     *
     * Finally, if neither the genesis transaction nor any means to obtain it (from the Olympia node)
     * have been configured, we fail with an error.
     * */
    if (configuredGenesis.isPresent()) {
      // The ledger isn't initialized, but there is a configured genesis transaction in properties
      // So just need to make sure it matches the fixed network genesis
      if (isPresentAndNotEqual(configuredGenesis.get(), fixedNetworkGenesis)) {
        return new RadixNodeBootstrapperHandle.Failed(
            new RuntimeException(
                String.format(
                    """
                    Network %s has a genesis configuration that doesn't match \
                    the genesis that has been configured for this node. \
                    Make sure your configuration is correct (`network.id` and/or \
                    `network.genesis_txn` and/or `network.genesis_file`).""",
                    network.getLogicalName())));
      }
      final var radixNode = RadixNode.start(properties, network, configuredGenesis);
      return new RadixNodeBootstrapperHandle.Resolved(radixNode);
    } else if (fixedNetworkGenesis.isPresent()) {
      // There's no configured genesis data in properties, so we can just use
      // the network fixed genesis without any additional validation
      final var radixNode = RadixNode.start(properties, network, fixedNetworkGenesis);
      return new RadixNodeBootstrapperHandle.Resolved(radixNode);
    } else if (useOlympiaFlagIsSet) {
      // No genesis transaction known beforehand was found, but we can get it from Olympia...
      final var olympiaBootstrapper = OlympiaGenesisBootstrapper.start(properties, network);
      return new RadixNodeBootstrapperHandle.AsyncFromOlympia(olympiaBootstrapper);
    } else {
      // TODO(genesis): for now just starting the node without any genesis configuration,
      //                the node will start if the database was already initialized
      //                or fail in the recovery module.
      final var radixNode = RadixNode.start(properties, network, Optional.empty());
      return new RadixNodeBootstrapperHandle.Resolved(radixNode);
      // TODO(genesis): bring back the error message once the genesis check
      //                against the database is brought back

      // TODO(post-babylon): remove Olympia ref from the message below
      /*
      return new RadixNodeBootstrapperHandle.Failed(
          new RuntimeException(
              """
        Radix node couldn't be initialized. No genesis transaction has been configured. Make \
        sure that either `network.genesis_txn` or `network.genesis_file` is set or that \
        you're using a well known network (`network.id`). If you're running an Olympia \
        node consider using it as your genesis source (`genesis.use_olympia`). Refer to \
        documentation for more details."""));
       */
    }
  }

  private static Either<Exception, Network> readNetworkFromProperties(
      RuntimeProperties properties) {
    return Optional.ofNullable(properties.get("network.id"))
        .map(Integer::parseInt)
        .map(
            networkId -> {
              if (networkId <= 0) {
                return Either.<Exception, Network>left(
                    new IllegalStateException(
                        String.format(
                            "Invalid networkId %s. Must be a positive value.", networkId)));
              }

              return Network.ofId(networkId)
                  .map(Either::<Exception, Network>right)
                  .orElseGet(
                      () ->
                          Either.left(
                              new IllegalStateException(
                                  String.format(
                                      "Network ID %s does not match any known networks",
                                      networkId))));
            })
        .orElseGet(
            () ->
                Either.left(
                    new IllegalStateException(
                        """
            Can't determine the Radix network \
            (missing or invalid network.id config).""")));
  }

  private static GenesisData resolveFixedNetworkGenesis(FixedNetworkGenesis fixedNetworkGenesis) {
    // TODO(genesis): read genesis data from resources or parse from raw bytes
    throw new RuntimeException("Not implemented yet");
  }

  /** @return true if `valueToCheck` is present and not equal to `baseValue` */
  @SuppressWarnings("OptionalUsedAsFieldOrParameterType")
  private static <T> boolean isPresentAndNotEqual(T baseValue, Optional<T> valueToCheck) {
    return valueToCheck.isPresent() && !valueToCheck.get().equals(baseValue);
  }

  /** A utility class encapsulating the Olympia-based genesis functionality */
  private static class OlympiaGenesisBootstrapper {
    private final RuntimeProperties properties;
    private final Network network;
    private final Injector injector;
    private final CompletableFuture<RadixNode> radixNodeFuture;

    private OlympiaGenesisBootstrapper(RuntimeProperties properties, Network network) {
      this.properties = properties;
      this.network = network;
      this.injector = Guice.createInjector(new GenesisFromOlympiaNodeModule(properties, network));
      this.radixNodeFuture = new CompletableFuture<>();
    }

    static OlympiaGenesisBootstrapper start(RuntimeProperties properties, Network network) {
      final var bootstrapper = new OlympiaGenesisBootstrapper(properties, network);
      bootstrapper.startInternal();
      return bootstrapper;
    }

    private void startInternal() {
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
              radixNodeFuture.complete(
                  RadixNode.start(properties, network, Optional.of(genesisData)));
            }
          });
    }

    CompletableFuture<RadixNode> radixNodeFuture() {
      return this.radixNodeFuture;
    }

    void cleanup() {
      injector.getInstance(SystemApi.class).stop();
      injector.getInstance(OlympiaGenesisService.class).shutdown();
    }
  }
}
