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

import com.google.inject.*;
import com.google.inject.util.Modules;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.BrowseApiServer;
import com.radixdlt.api.CoreApiServer;
import com.radixdlt.api.prometheus.PrometheusApi;
import com.radixdlt.api.system.SystemApi;
import com.radixdlt.consensus.bft.SelfValidatorInfo;
import com.radixdlt.consensus.safety.PersistentSafetyStateStore;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.environment.Runners;
import com.radixdlt.genesis.GenesisProvider;
import com.radixdlt.modules.ModuleRunner;
import com.radixdlt.monitoring.MetricInstaller;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.addressbook.AddressBookPersistence;
import com.radixdlt.p2p.transport.PeerServerBootstrap;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class RadixNode {
  private static final Logger log = LogManager.getLogger();

  private final Injector injector;

  private RadixNode(Injector injector) {
    this.injector = injector;
  }

  public static RadixNode run(
      RuntimeProperties properties, Network network, GenesisProvider genesisProvider) {
    log.info("Using a genesis of hash {}", genesisProvider.genesisDataHash());

    final var injector =
        Guice.createInjector(
            Modules.requireAtInjectOnConstructorsModule(),
            Modules.disableCircularProxiesModule(),
            new RadixNodeModule(properties, network, genesisProvider));

    final var radixNode = new RadixNode(injector);

    log.info("Radix node {} is starting...", radixNode.selfDetailedString());

    final var metrics = injector.getInstance(Metrics.class);
    injector.getInstance(MetricInstaller.class).installAt(metrics);

    final var moduleRunners =
        injector.getInstance(Key.get(new TypeLiteral<Map<String, ModuleRunner>>() {}));

    final var moduleStartOrder =
        List.of(
            Runners.P2P_NETWORK,
            Runners.SYSTEM_INFO,
            Runners.SYNC,
            Runners.MEMPOOL,
            Runners.CONSENSUS);

    for (var module : moduleStartOrder) {
      final var moduleRunner = moduleRunners.get(module);
      moduleRunner.start(
          error -> {
            log.error("Uncaught exception in runner {}; exiting the process", module, error);
            System.exit(-1);
          });
    }

    final var peerServer = injector.getInstance(PeerServerBootstrap.class);
    peerServer.start();

    // Start the system API server
    final var systemApi = injector.getInstance(SystemApi.class);
    systemApi.start();

    // Start the prometheus API server
    final var prometheusApi = injector.getInstance(PrometheusApi.class);
    prometheusApi.start();

    // Start the Core API server
    final var coreApiServer = injector.getInstance(CoreApiServer.class);
    coreApiServer.start();

    // Start the Browse API server
    final var browseApiServer = injector.getInstance(BrowseApiServer.class);
    browseApiServer.start();

    return radixNode;
  }

  public Injector injector() {
    return injector;
  }

  public String selfDetailedString() {
    final var addressing = this.injector.getInstance(Addressing.class);
    return self().toDetailedString(addressing);
  }

  public SelfValidatorInfo self() {
    return this.injector.getInstance(SelfValidatorInfo.class);
  }

  public void reportSelfStartupTime(Duration startupTimeMs) {
    this.injector.getInstance(Metrics.class).misc().nodeStartup().observe(startupTimeMs);
  }

  public void shutdown() {
    // using System.out.printf as logger no longer works reliably in a shutdown hook
    System.out.printf("Node %s is shutting down...\n", this.selfDetailedString());

    injector
        .getInstance(Key.get(new TypeLiteral<Map<String, ModuleRunner>>() {}))
        .values()
        .forEach(
            moduleRunner ->
                catchAllAndLogShutdownError(
                    "ModuleRunner " + moduleRunner.threadName(), moduleRunner::stop));

    catchAllAndLogShutdownError(
        "AddressBookPersistence", () -> injector.getInstance(AddressBookPersistence.class).close());

    catchAllAndLogShutdownError(
        "PersistentSafetyStateStore",
        () -> injector.getInstance(PersistentSafetyStateStore.class).close());

    catchAllAndLogShutdownError(
        "PeerServerBootstrap", () -> injector.getInstance(PeerServerBootstrap.class).stop());

    catchAllAndLogShutdownError("SystemApi", () -> injector.getInstance(SystemApi.class).stop());

    catchAllAndLogShutdownError(
        "PrometheusApi", () -> injector.getInstance(PrometheusApi.class).stop());

    catchAllAndLogShutdownError(
        "CoreApiServer", () -> injector.getInstance(CoreApiServer.class).stop());

    catchAllAndLogShutdownError(
        "StateManager", () -> injector.getInstance(NodeRustEnvironment.class).shutdown());
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
}
