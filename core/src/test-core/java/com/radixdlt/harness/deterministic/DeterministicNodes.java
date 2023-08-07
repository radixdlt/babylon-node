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

package com.radixdlt.harness.deterministic;

import com.google.common.collect.Streams;
import com.google.inject.*;
import com.google.inject.Module;
import com.google.inject.util.Modules;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.environment.Environment;
import com.radixdlt.environment.NodeAutoCloseable;
import com.radixdlt.environment.deterministic.DeterministicProcessor;
import com.radixdlt.environment.deterministic.network.ControlledDispatcher;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.keys.SelfValidatorInfoFromGenesisModule;
import com.radixdlt.keys.SelfValidatorInfoModule;
import com.radixdlt.logger.EventLoggerConfig;
import com.radixdlt.logger.EventLoggerModule;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.statemanager.FatalPanicHandler;
import com.radixdlt.utils.Pair;
import com.radixdlt.utils.TimeSupplier;
import io.reactivex.rxjava3.schedulers.Timed;
import java.util.*;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.ThreadContext;

/** BFT Nodes treated as a single unit where one message is processed at a time. */
public final class DeterministicNodes implements AutoCloseable {
  private static final Logger log = LogManager.getLogger();

  // Nodes
  private final List<Injector> nodeInstances;
  private final ControlledAddressBook addressBook;
  private final Map<Integer, PhysicalNodeConfig> nodeConfigs;
  private final Module baseModule;
  private final Module overrideModule;

  // Network
  private final DeterministicNetwork network;

  public DeterministicNodes(
      List<PhysicalNodeConfig> nodeConfigs,
      DeterministicNetwork network,
      Module baseModule,
      Module overrideModule) {
    this.baseModule = baseModule;
    this.overrideModule = overrideModule;
    this.network = network;
    this.addressBook = new ControlledAddressBook();
    this.nodeConfigs =
        Streams.mapWithIndex(nodeConfigs.stream(), (node, index) -> Pair.of((int) index, node))
            .collect(Collectors.toMap(Pair::getFirst, Pair::getSecond));
    this.nodeInstances =
        Stream.generate(() -> (Injector) null)
            .limit(nodeConfigs.size())
            .collect(Collectors.toList());

    for (int nodeIndex = 0; nodeIndex < this.nodeInstances.size(); nodeIndex++) {
      var nodeId = NodeId.fromPublicKey(this.nodeConfigs.get(nodeIndex).key());
      this.addressBook.addressBook.put(nodeId, nodeIndex);
    }
  }

  private static class ControlledTimeSupplier implements TimeSupplier {
    private long time;

    ControlledTimeSupplier(long time) {
      this.time = time;
    }

    private void setTime(long time) {
      this.time = time;
    }

    @Override
    public long currentTime() {
      return time;
    }
  }

  private static class ControlledAddressBook implements Function<NodeId, Integer> {

    private final Map<NodeId, Integer> addressBook;

    public ControlledAddressBook() {
      this.addressBook = new HashMap<>();
    }

    @Override
    public Integer apply(NodeId nodeId) {
      return this.addressBook.get(nodeId);
    }
  }

  private Injector createBFTInstance(
      int nodeIndex, Module baseModule, Module overrideModule, long time) {
    var config = this.nodeConfigs.get(nodeIndex);

    Module module =
        Modules.combine(
            new AbstractModule() {
              @Override
              public void configure() {
                install(
                    new EventLoggerModule(
                        new EventLoggerConfig(
                            k -> "Node" + addressBook.apply(NodeId.fromPublicKey(k)))));
                bind(ECDSASecp256k1PublicKey.class)
                    .annotatedWith(Self.class)
                    .toInstance(config.key());
                bind(NodeId.class)
                    .annotatedWith(Self.class)
                    .toInstance(NodeId.fromPublicKey(config.key()));

                switch (config.validatorIdSource()) {
                  case PhysicalNodeConfig.ValidatorIdSource.Provided provided -> {
                    install(
                        new SelfValidatorInfoModule(
                            Optional.of(provided.validatorId().getValidatorAddress())));
                  }
                  case PhysicalNodeConfig.ValidatorIdSource.LoadFromGenesis loadFromGenesis -> {
                    install(new SelfValidatorInfoFromGenesisModule());
                  }
                }

                install(
                    new TestP2PModule.Builder()
                        .withAllNodes(addressBook.addressBook.keySet().stream().toList())
                        .build());
                bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
                bind(ControlledTimeSupplier.class).toInstance(new ControlledTimeSupplier(time));
                bind(TimeSupplier.class).to(ControlledTimeSupplier.class);
                bind(FatalPanicHandler.class).toInstance(() -> shutdownNode(nodeIndex));
                bind(DeterministicProcessor.class).in(Scopes.SINGLETON);
              }

              @Provides
              @Singleton
              Environment sender(@Self NodeId self) {
                return new ControlledDispatcher(addressBook, network, self, nodeIndex);
              }
            },
            baseModule);
    if (overrideModule != null) {
      module = Modules.override(module).with(overrideModule);
    }
    return Guice.createInjector(module);
  }

  public int numNodes() {
    return this.nodeInstances.size();
  }

  public int numNodesLive() {
    return (int) this.nodeInstances.stream().filter(Objects::nonNull).count();
  }

  public void startAllNodes(long time) {
    for (int nodeIndex = 0; nodeIndex < this.nodeInstances.size(); nodeIndex++) {
      this.startNode(nodeIndex, time);
    }
  }

  public void shutdownNode(int nodeIndex) {
    if (!isNodeLive(nodeIndex)) {
      return;
    }

    var closeables =
        this.nodeInstances
            .get(nodeIndex)
            .getInstance(Key.get(new TypeLiteral<Set<NodeAutoCloseable>>() {}));
    for (var c : closeables) {
      c.close();
    }

    this.nodeInstances.set(nodeIndex, null);
  }

  public static class EventHandleException extends RuntimeException {
    private final ControlledMessage message;

    EventHandleException(ControlledMessage message, Exception e) {
      super("Exception: " + e + "\nOn message: " + message.toString(), e);
      this.message = message;
    }
  }

  public void startNode(int nodeIndex, long time) {
    if (isNodeLive(nodeIndex)) {
      return;
    }

    ThreadContext.put("self", " Node" + nodeIndex);

    var injector = createBFTInstance(nodeIndex, baseModule, overrideModule, time);
    try {
      var processor = injector.getInstance(DeterministicProcessor.class);
      processor.start();
    } finally {
      ThreadContext.remove("self");
    }

    this.nodeInstances.set(nodeIndex, injector);
  }

  public void handleMessage(Timed<ControlledMessage> timedNextMsg) {
    var nextMsg = timedNextMsg.value();
    var sender = nextMsg.origin();
    int receiverIndex = nextMsg.channelId().receiverIndex();
    var injector = nodeInstances.get(receiverIndex);

    if (injector == null) {
      return;
    }

    var time = timedNextMsg.time();
    injector.getInstance(ControlledTimeSupplier.class).setTime(time);

    ThreadContext.put("self", " " + injector.getInstance(Key.get(String.class, Self.class)));
    try {
      log.debug("{}: Receive message {}", timedNextMsg.time(), nextMsg);
      nodeInstances
          .get(receiverIndex)
          .getInstance(DeterministicProcessor.class)
          .handleMessage(sender, nextMsg.message(), nextMsg.typeLiteral());
    } catch (Exception e) {
      throw new EventHandleException(nextMsg, e);
    } finally {
      ThreadContext.remove("self");
    }
  }

  public List<Injector> getNodeInjectors() {
    return this.nodeInstances;
  }

  public void setNodeConfig(int nodeIndex, PhysicalNodeConfig config) {
    this.nodeConfigs.put(nodeIndex, config);
  }

  public int getNode(Predicate<Injector> injectorPredicate) {
    return Streams.mapWithIndex(this.nodeInstances.stream(), Pair::of)
        .filter(p -> injectorPredicate.test(p.getFirst()))
        .map(Pair::getSecond)
        .findFirst()
        .orElseThrow()
        .intValue();
  }

  public boolean isNodeLive(int nodeIndex) {
    return this.nodeInstances.get(nodeIndex) != null;
  }

  public List<Integer> getNodeIndices() {
    return IntStream.range(0, this.numNodes()).boxed().toList();
  }

  public List<Integer> getLiveNodeIndices() {
    return IntStream.range(0, this.numNodes()).filter(this::isNodeLive).boxed().toList();
  }

  public <T> T getInstance(int nodeIndex, Class<T> instanceClass) {
    return this.nodeInstances.get(nodeIndex).getInstance(instanceClass);
  }

  public <T> T getInstance(int nodeIndex, Key<T> key) {
    return this.nodeInstances.get(nodeIndex).getInstance(key);
  }

  @Override
  public void close() {
    for (int i = 0; i < this.nodeInstances.size(); i++) {
      this.shutdownNode(i);
    }
  }
}
