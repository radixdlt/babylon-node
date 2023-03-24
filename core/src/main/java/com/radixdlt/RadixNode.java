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

import com.google.common.base.Stopwatch;
import com.google.inject.Guice;
import com.google.inject.Injector;
import com.google.inject.Key;
import com.google.inject.TypeLiteral;
import com.radixdlt.api.CoreApiServer;
import com.radixdlt.api.prometheus.PrometheusApi;
import com.radixdlt.api.system.SystemApi;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.environment.Runners;
import com.radixdlt.genesis.GenesisConfig;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisFromPropertiesLoader;
import com.radixdlt.genesis.PreGenesisNodeModule;
import com.radixdlt.genesis.olympia.GenesisFromOlympiaNodeModule;
import com.radixdlt.genesis.olympia.OlympiaGenesisConfig;
import com.radixdlt.genesis.olympia.OlympiaGenesisService;
import com.radixdlt.modules.ModuleRunner;
import com.radixdlt.monitoring.MetricInstaller;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.networks.FixedNetworkGenesis;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.addressbook.AddressBookPersistence;
import com.radixdlt.p2p.transport.PeerServerBootstrap;
import com.radixdlt.statemanager.StateManager;
import com.radixdlt.store.berkeley.BerkeleyDatabaseEnvironment;
import com.radixdlt.transaction.ExecutedTransaction;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.utils.BooleanUtils;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public final class RadixNode {
  private static final Logger log = LogManager.getLogger();

  private final RuntimeProperties properties;

  private Optional<Injector> maybeGenesisFromOlympiaInjector = Optional.empty();
  private Optional<Injector> maybeRadixNodeInjector = Optional.empty();

  public RadixNode(RuntimeProperties properties) {
    this.properties = properties;
  }

  public void start() {
    log.info("Starting the Radix node...");

    final var network = readNetworkFromProperties();
    final var fixedNetworkGenesis = network.fixedGenesis().map(this::resolveFixedNetworkGenesis);
    final var useOlympiaFlagIsSet =
        properties.get("genesis.use_olympia", BooleanUtils::parseBoolean).orElse(false);

    final var nodeBootStopwatch = Stopwatch.createStarted();

    /*
     The Radix node utilizes three top-level Guice modules:
     - PreGenesisNodeModule:
        A basic module containing utils for loading the genesis transaction from properties
        and a simplified/mocked StateManager instance used to read the executed genesis transaction
        from the database. This module is always instantiated.

     - GenesisFromOlympiaNodeModule:
        A module used to acquire the genesis transaction from a running Olympia node.
        This is only instantiated if both are true:
        a) there's no genesis transaction configured in properties or already executed in the database
        b) Olympia-based genesis has been configured in properties

     - RadixNodeModule:
        A complete Radix node module that requires a known genesis transaction in order to start.
    */
    final var preGenesisInjector =
        Guice.createInjector(new PreGenesisNodeModule(properties, network));

    // We only need REv2 state manager to be able to read the genesis transaction from the database
    // once we read it we can immediately shut it down, so that the database lock and any other
    // resources are released and can be re-used by the real
    // state manager instantiated by the RadixNodeModule.
    final var executedGenesisTransaction =
        preGenesisInjector
            .getInstance(REv2TransactionAndProofStore.class)
            .getTransactionAtStateVersion(1)
            .map(ExecutedTransaction::rawTransaction)
            .toOptional();
    preGenesisInjector.getInstance(StateManager.class).shutdown();

    final var genesisFromPropertiesLoader =
        preGenesisInjector.getInstance(GenesisFromPropertiesLoader.class);
    final var configuredGenesisTransaction =
        genesisFromPropertiesLoader
            .loadGenesisDataFromProperties()
            .map(gd -> gd.toGenesisTransaction(GenesisConfig.babylonDefault()));

    /* We have three sources of a genesis transaction at this point:
     *  - a fixed genesis transaction associated with a given network
     * 	- a genesis transaction configured in properties (either raw bytes or loaded from a file)
     * 	- a genesis transaction stored in a database
     * We can use either one, but we need to make sure they match, if more than one
     * is configured (to protect against node misconfiguration).
     *
     * If neither genesis transaction is present, we may try to acquire it from a running
     * Olympia node, if it has been configured. This mode will be (or already was :)) used to
     * coordinate the death of the Olympia-based Radix network and its rebirth on the Babylon side.
     *
     * Finally, if neither the genesis transaction nor any means to obtain it (from the Olympia node)
     * have been configured, we fail with an error.
     * */
    if (executedGenesisTransaction.isPresent()) {
      // The ledger is already initialized, let's make sure the genesis
      // transaction matches the properties and/or network.
      final var executedTx = executedGenesisTransaction.get();

      if (isPresentAndNotEqual(executedTx, configuredGenesisTransaction)) {
        throw new RuntimeException(
            String.format(
                """
                    Configured genesis transaction (%s) doesn't match the genesis transaction that has \
                    already been executed and stored on ledger (%s). Make sure your \
                    `network.genesis_txn` and/or `network.genesis_file` config options are set \
                    correctly (or clear them).""",
                configuredGenesisTransaction.orElseThrow().getPayloadHash(),
                executedTx.getPayloadHash()));
      }

      if (isPresentAndNotEqual(executedTx, fixedNetworkGenesis)) {
        throw new RuntimeException(
            String.format(
                """
                    Network %s has a genesis transaction (%s) that doesn't match the genesis" \
                    transaction that has previously been executed and stored on ledger (%s)." \
                    Make sure your configuration is correct (`network.id` and/or" \
                    `db.location`).""",
                network.getLogicalName(),
                fixedNetworkGenesis.orElseThrow().getPayloadHash(),
                executedTx.getPayloadHash()));
      }

      startRadixNodeModule(executedTx, nodeBootStopwatch);
    } else if (configuredGenesisTransaction.isPresent()) {
      // The ledger isn't initialized, but there is a configured genesis transaction in properties
      // So just need to make sure it matches the fixed network genesis
      final var genesisTx = configuredGenesisTransaction.get();

      if (isPresentAndNotEqual(genesisTx, fixedNetworkGenesis)) {
        throw new RuntimeException(
            String.format(
                """
                    Network %s has a genesis transaction (%s) that doesn't match \
                    the genesis transaction that has been configured for this node (%s). \
                    Make sure your configuration is correct (`network.id` and/or \
                    `network.genesis_txn` and/or `network.genesis_file`).""",
                network.getLogicalName(),
                fixedNetworkGenesis.orElseThrow().getPayloadHash(),
                genesisTx.getPayloadHash()));
      }

      startRadixNodeModule(genesisTx, nodeBootStopwatch);
    } else if (fixedNetworkGenesis.isPresent()) {
      // There's nothing on ledger and/or properties, so we can just use
      // the network fixed genesis without any additional validation
      startRadixNodeModule(fixedNetworkGenesis.get(), nodeBootStopwatch);
    } else if (useOlympiaFlagIsSet) {
      // No genesis transaction known beforehand was found, but we can get it from Olympia...
      startOlympiaGenesisModule(network, nodeBootStopwatch);
    } else {
      // TODO(post-babylon): remove Olympia ref from the message below
      throw new RuntimeException(
          """
              Radix node couldn't be initialized. No genesis transaction has been configured. Make \
              sure that either `network.genesis_txn` or `network.genesis_file` is set or that \
              you're using a well known network (`network.id`). If you're running an Olympia \
              node consider using it as your genesis source (`genesis.use_olympia`). Refer to \
              documentation for more details.""");
    }
  }

  private void startOlympiaGenesisModule(Network network, Stopwatch nodeBootStopwatch) {
    final var genesisFromOlympiaInjector =
        Guice.createInjector(new GenesisFromOlympiaNodeModule(properties, network));
    this.maybeGenesisFromOlympiaInjector = Optional.of(genesisFromOlympiaInjector);

    log.info(
        "Olympia-based genesis was configured ({}). Using core API URL of {}",
        network.getLogicalName(),
        genesisFromOlympiaInjector.getInstance(OlympiaGenesisConfig.class).nodeCoreApiUrl());

    final var systemApi = genesisFromOlympiaInjector.getInstance(SystemApi.class);
    systemApi.start();

    final var olympiaGenesisService =
        genesisFromOlympiaInjector.getInstance(OlympiaGenesisService.class);

    olympiaGenesisService.start(
        genesisData ->
            proceedWithGenesisFromOlympia(
                nodeBootStopwatch, genesisFromOlympiaInjector, genesisData),
        ex -> {
          log.warn(
              """
                  Radix node couldn't be initialized. The Olympia-based genesis was configured but \
                  an error occurred.""",
              ex);
          this.shutdown();
        });
  }

  private void proceedWithGenesisFromOlympia(
      Stopwatch nodeBootStopwatch, Injector genesisFromOlympiaInjector, GenesisData genesisData) {
    log.info(
        """
            Genesis data has been successfully received from the Olympia node \
            ({} accounts, {} validators). Initializing the Babylon node...""",
        genesisData.accountXrdAllocations().size(),
        genesisData.validatorSetAndStakeOwners().size());
    cleanupOlympiaGenesisModule(genesisFromOlympiaInjector);
    final var genesisTxn = genesisData.toGenesisTransaction(GenesisConfig.babylonDefault());
    this.startRadixNodeModule(genesisTxn, nodeBootStopwatch);
  }

  private void cleanupOlympiaGenesisModule(Injector genesisFromOlympiaInjector) {
    genesisFromOlympiaInjector.getInstance(SystemApi.class).stop();
    genesisFromOlympiaInjector.getInstance(OlympiaGenesisService.class).shutdown();
  }

  private void startRadixNodeModule(RawLedgerTransaction genesisTxn, Stopwatch nodeBootStopwatch) {
    log.info("Starting Radix node (genesis transaction: {})", genesisTxn.getPayloadHash());

    final var network = readNetworkFromProperties();
    final var radixNodeInjector =
        Guice.createInjector(
            new RadixNodeModule(properties, network, genesisTxn, nodeBootStopwatch));
    this.maybeRadixNodeInjector = Optional.of(radixNodeInjector);

    final var metrics = radixNodeInjector.getInstance(Metrics.class);
    radixNodeInjector.getInstance(MetricInstaller.class).installAt(metrics);

    final var moduleRunners =
        radixNodeInjector.getInstance(Key.get(new TypeLiteral<Map<String, ModuleRunner>>() {}));

    final var moduleStartOrder =
        List.of(
            Runners.P2P_NETWORK,
            Runners.SYSTEM_INFO,
            Runners.SYNC,
            Runners.MEMPOOL,
            Runners.CONSENSUS);

    for (var module : moduleStartOrder) {
      final var moduleRunner = moduleRunners.get(module);
      moduleRunner.start();
    }

    final var peerServer = radixNodeInjector.getInstance(PeerServerBootstrap.class);
    try {
      peerServer.start();
    } catch (InterruptedException e) {
      log.error("Cannot start p2p server", e);
    }

    // Start the system API server
    final var systemApi = radixNodeInjector.getInstance(SystemApi.class);
    systemApi.start();

    // Start the prometheus API server
    final var prometheusApi = radixNodeInjector.getInstance(PrometheusApi.class);
    prometheusApi.start();

    // Start the core API server
    final var coreApiServer = radixNodeInjector.getInstance(CoreApiServer.class);
    coreApiServer.start();

    final var self = radixNodeInjector.getInstance(Key.get(BFTValidatorId.class, Self.class));
    log.info("Radix node {} started (took {} ms)", self, nodeBootStopwatch.elapsed().toMillis());

    metrics.misc().timeUntilRadixNodeModuleStarted().observe(nodeBootStopwatch.elapsed());
  }

  public void shutdown() {
    this.maybeGenesisFromOlympiaInjector.ifPresent(this::cleanupOlympiaGenesisModule);

    this.maybeRadixNodeInjector.ifPresent(
        radixNodeInjector -> {
          // using System.out.printf as logger no longer works reliably in a shutdown hook
          final var self = radixNodeInjector.getInstance(Key.get(BFTValidatorId.class, Self.class));
          System.out.printf("Node %s is shutting down...\n", self);

          radixNodeInjector
              .getInstance(Key.get(new TypeLiteral<Map<String, ModuleRunner>>() {}))
              .forEach(
                  (k, moduleRunner) -> {
                    try {
                      moduleRunner.stop();
                    } catch (Exception e) {
                      logShutdownError("ModuleRunner " + moduleRunner.threadName(), e.getMessage());
                    }
                  });

          catchAllAndLogShutdownError(
              "AddressBookPersistence",
              () -> radixNodeInjector.getInstance(AddressBookPersistence.class).close());

          catchAllAndLogShutdownError(
              "PersistentSafetyStateStore",
              () -> radixNodeInjector.getInstance(PersistentSafetyStateStore.class).close());

          catchAllAndLogShutdownError(
              "BerkeleyDatabaseEnvironment",
              () -> radixNodeInjector.getInstance(BerkeleyDatabaseEnvironment.class).stop());

          catchAllAndLogShutdownError(
              "PeerServerBootstrap",
              () -> radixNodeInjector.getInstance(PeerServerBootstrap.class).stop());

          catchAllAndLogShutdownError(
              "SystemApi", () -> radixNodeInjector.getInstance(SystemApi.class).stop());

          catchAllAndLogShutdownError(
              "PrometheusApi", () -> radixNodeInjector.getInstance(PrometheusApi.class).stop());

          catchAllAndLogShutdownError(
              "CoreApiServer", () -> radixNodeInjector.getInstance(CoreApiServer.class).stop());

          catchAllAndLogShutdownError(
              "StateManager", () -> radixNodeInjector.getInstance(StateManager.class).shutdown());
        });
  }

  private void catchAllAndLogShutdownError(String what, Runnable thunk) {
    try {
      thunk.run();
    } catch (Exception e) {
      logShutdownError(what, e.getMessage());
    }
  }

  private void logShutdownError(String what, String why) {
    System.out.printf("Could not stop %s because of %s, continuing...\n", what, why);
  }

  private Network readNetworkFromProperties() {
    final var networkId =
        Optional.ofNullable(properties.get("network.id"))
            .map(Integer::parseInt)
            .orElseThrow(
                () ->
                    new IllegalStateException(
                        """
                        Can't determine the Radix network \
                        (missing or invalid network.id config)."""));

    if (networkId <= 0) {
      throw new IllegalStateException(
          String.format("Invalid networkId %s. Must be a positive value.", networkId));
    }

    return Network.ofId(networkId)
        .orElseThrow(
            () ->
                new IllegalStateException(
                    String.format("Network ID %s does not match any known networks", networkId)));
  }

  private RawLedgerTransaction resolveFixedNetworkGenesis(FixedNetworkGenesis fixedNetworkGenesis) {
    // TODO: read genesis data from resources or parse from raw bytes
    throw new RuntimeException("Not implemented yet");
  }

  /** @return true if `valueToCheck` is present and not equal to `baseValue` */
  private <T> boolean isPresentAndNotEqual(T baseValue, Optional<T> valueToCheck) {
    return valueToCheck.isPresent() && !valueToCheck.get().equals(baseValue);
  }
}
