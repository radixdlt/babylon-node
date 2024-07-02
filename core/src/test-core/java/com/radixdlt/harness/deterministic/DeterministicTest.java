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

import com.google.common.collect.ImmutableList;
import com.google.inject.*;
import com.google.inject.Module;
import com.google.inject.util.Modules;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochRound;
import com.radixdlt.consensus.epoch.EpochRoundUpdate;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.invariants.MessageMonitor;
import com.radixdlt.harness.deterministic.invariants.StateMonitor;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.MockedCryptoModule;
import com.radixdlt.modules.MockedKeyModule;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.ScryptoConstants;
import io.reactivex.rxjava3.schedulers.Timed;
import java.util.*;
import java.util.function.Predicate;
import java.util.stream.Stream;

/**
 * A deterministic test where each event that occurs in the network is emitted and processed
 * synchronously by the caller.
 */
public final class DeterministicTest implements AutoCloseable {
  private final DeterministicNodes nodes;
  private final DeterministicNetwork network;
  private final StateMonitor stateMonitor;

  private int numMessagesProcessed = 0;

  private DeterministicTest(
      List<PhysicalNodeConfig> nodes,
      MessageSelector messageSelector,
      MessageMutator messageMutator,
      MessageMonitor messageMonitor,
      StateMonitor stateMonitor,
      Module baseModule,
      Module overrideModule) {
    this.network = new DeterministicNetwork(messageSelector, messageMutator, messageMonitor);
    this.nodes = new DeterministicNodes(nodes, this.network, baseModule, overrideModule);
    this.stateMonitor = stateMonitor;
  }

  public void suppressUnusedWarning() {}

  @Override
  public void close() {
    this.nodes.close();
  }

  public static class Builder {
    private final List<PhysicalNodeConfig> nodes = new ArrayList<>();
    private MessageSelector messageSelector = MessageSelector.firstSelector();
    private MessageMutator messageMutator = MessageMutator.nothing();
    private Module overrideModule = null;
    private final ImmutableList.Builder<Module> modules = ImmutableList.builder();
    private final ImmutableList.Builder<Module> testModules = ImmutableList.builder();

    private Builder() {
      // Nothing to do here
    }

    public Builder addPhysicalNodes(List<PhysicalNodeConfig> nodeConfigs) {
      this.nodes.addAll(nodeConfigs);
      return this;
    }

    /**
     * Override with an incorrect module which should cause a test to fail. TODO: Refactor to make
     * the link between incorrect module and failing test more explicit.
     *
     * @param module the incorrect module
     * @return the current builder
     */
    public Builder overrideWithIncorrectModule(Module module) {
      this.overrideModule = module;
      return this;
    }

    public Builder addModule(Module module) {
      this.modules.add(module);
      return this;
    }

    public Builder addModules(Module... modules) {
      this.modules.addAll(Arrays.asList(modules));
      return this;
    }

    public DeterministicTest functionalNodeModule(FunctionalRadixNodeModule module) {
      modules.add(module);
      return build();
    }

    public Builder messageSelector(MessageSelector messageSelector) {
      this.messageSelector = Objects.requireNonNull(messageSelector);
      return this;
    }

    public Builder messageMutator(MessageMutator messageMutator) {
      this.messageMutator = Objects.requireNonNull(messageMutator);
      return this;
    }

    public Builder messageMutators(MessageMutator... messageMutators) {
      final var combinedMutator = Stream.of(messageMutators).reduce(MessageMutator::andThen).get();
      return this.messageMutator(combinedMutator);
    }

    public Builder addMonitors(Module... modules) {
      for (var module : modules) {
        this.testModules.add(module);
      }
      return this;
    }

    private DeterministicTest build() {
      modules.add(
          new AbstractModule() {
            @Override
            public void configure() {
              bind(Network.class).toInstance(Network.INTEGRATIONTESTNET);
              bind(Addressing.class).toInstance(Addressing.ofNetwork(Network.INTEGRATIONTESTNET));
              bind(Random.class).toInstance(new Random(123456));
            }
          });
      modules.add(new MockedKeyModule());
      modules.add(new MockedCryptoModule());
      modules.add(new TestMessagingModule.Builder().build());

      // Retrieve monitors
      var monitorInjector =
          Guice.createInjector(
              new DeterministicMonitorsModule(), Modules.combine(testModules.build()));
      var messageMonitor = monitorInjector.getInstance(MessageMonitor.class);
      var stateMonitor = monitorInjector.getInstance(StateMonitor.class);

      return new DeterministicTest(
          this.nodes,
          this.messageSelector,
          this.messageMutator,
          messageMonitor,
          stateMonitor,
          Modules.combine(modules.build()),
          overrideModule);
    }
  }

  public static Builder builder() {
    return new Builder();
  }

  public DeterministicNetwork getNetwork() {
    return this.network;
  }

  public DeterministicNodes getNodes() {
    return this.nodes;
  }

  public int numNodesLive() {
    return this.nodes.numNodesLive();
  }

  public List<Injector> getNodeInjectors() {
    return this.nodes.getNodeInjectors();
  }

  private void handleMessage(Timed<ControlledMessage> nextMessage) {
    this.stateMonitor.next(nodes.getNodeInjectors(), nextMessage.time(), this.numMessagesProcessed);
    this.nodes.handleMessage(nextMessage);
    this.numMessagesProcessed++;
  }

  public interface DeterministicManualExecutor {
    void start();

    void processNext(int senderIndex, int receiverIndex, Class<?> eventClass);
  }

  public DeterministicManualExecutor createExecutor() {
    return new DeterministicManualExecutor() {
      @Override
      public void start() {
        nodes.startAllNodes(network.currentTime());
      }

      @Override
      public void processNext(int senderIndex, int receiverIndex, Class<?> eventClass) {
        Timed<ControlledMessage> nextMsg =
            network.nextMessage(
                msg ->
                    msg.channelId().senderIndex() == senderIndex
                        && msg.channelId().receiverIndex() == receiverIndex
                        && eventClass.isInstance(msg.message()));

        handleMessage(nextMsg);
      }
    };
  }

  public void startAllNodes() {
    this.nodes.startAllNodes(this.network.currentTime());
  }

  public void shutdownNode(int nodeIndex) {
    // Drop local messages
    this.network.dropMessages(
        m ->
            m.channelId().receiverIndex() == nodeIndex && m.channelId().senderIndex() == nodeIndex);
    this.nodes.shutdownNode(nodeIndex);
  }

  public void startNode(int nodeIndex) {
    this.nodes.startNode(nodeIndex, this.network.currentTime());
  }

  public void restartNode(int nodeIndex) {
    this.shutdownNode(nodeIndex);
    this.startNode(nodeIndex);
  }

  public void restartNodeWithConfig(int nodeIndex, PhysicalNodeConfig config) {
    this.shutdownNode(nodeIndex);
    this.nodes.setNodeConfig(nodeIndex, config);
    this.startNode(nodeIndex);
  }

  public void restartNodeWithOverrideModule(int nodeIndex, Module overrideModule) {
    this.shutdownNode(nodeIndex);
    // Keep the current module
    final var origOverrideModule = this.nodes.getOverrideModule();
    // Use the provided module for starting this node
    this.nodes.setOverrideModule(overrideModule);
    this.startNode(nodeIndex);
    // Restore the original module
    this.nodes.setOverrideModule(origOverrideModule);
  }

  public static class NeverReachedStateException extends IllegalStateException {
    private NeverReachedStateException(int max) {
      super("Never reached state after " + max + " messages");
    }
  }

  @SuppressWarnings("OptionalUsedAsFieldOrParameterType")
  public DeterministicTest runUntilState(
      Predicate<List<Injector>> nodeStatePredicate,
      int max,
      Optional<Predicate<ControlledMessage>> predicate) {
    int count = 0;

    while (!nodeStatePredicate.test(getNodeInjectors())) {
      if (count == max) {
        throw new NeverReachedStateException(max);
      }
      // TODO: this is a bit of a hack, but `nextMessage` with predicate doesn't
      // respect messageSelector, so not using it if not needed
      final var nextMsg =
          predicate.map(this.network::nextMessage).orElseGet(this.network::nextMessage);
      handleMessage(nextMsg);
      count++;
    }

    return this;
  }

  public DeterministicTest runUntilState(
      Predicate<List<Injector>> nodeStatePredicate,
      int max,
      Predicate<ControlledMessage> predicate) {
    return runUntilState(nodeStatePredicate, max, Optional.of(predicate));
  }

  public DeterministicTest runUntilState(
      Predicate<List<Injector>> nodeStatePredicate, Predicate<ControlledMessage> predicate) {
    return runUntilState(nodeStatePredicate, 10000, Optional.of(predicate));
  }

  public DeterministicTest runUntilState(Predicate<List<Injector>> nodeStatePredicate, int max) {
    return this.runUntilState(nodeStatePredicate, max, Optional.empty());
  }

  public DeterministicTest runUntilState(Predicate<List<Injector>> nodeStatePredicate) {
    return this.runUntilState(nodeStatePredicate, 10000, Optional.empty());
  }

  public void runUntilOutOfMessagesOfType(int count, Predicate<ControlledMessage> predicate) {
    for (int i = 0; i < count; i++) {
      var nextMsg = this.network.nextMessageIfExists(predicate);
      if (nextMsg.isEmpty()) {
        return;
      } else {
        handleMessage(nextMsg.get());
      }
    }
    throw new IllegalStateException(
        String.format("Run for %s messages, but didn't run out", count));
  }

  public DeterministicTest runUntilMessage(
      Predicate<Timed<ControlledMessage>> stopPredicate, boolean inclusive, int count) {
    for (int i = 0; i < count; i++) {
      Timed<ControlledMessage> nextMsg = this.network.nextMessage();
      if (stopPredicate.test(nextMsg)) {
        if (inclusive) {
          handleMessage(nextMsg);
        }
        return this;
      }

      handleMessage(nextMsg);
    }

    throw new IllegalStateException(
        String.format("Run for %s messages, but didn't find message matching predicate", count));
  }

  public DeterministicTest runUntilMessage(
      Predicate<Timed<ControlledMessage>> stopPredicate, int count) {
    return runUntilMessage(stopPredicate, false, count);
  }

  public DeterministicTest runForCount(int count) {
    for (int i = 0; i < count; i++) {
      Timed<ControlledMessage> nextMsg = this.network.nextMessage();
      handleMessage(nextMsg);
    }

    return this;
  }

  public void runNext(Predicate<ControlledMessage> predicate) {
    Timed<ControlledMessage> nextMsg = this.network.nextMessage(predicate);
    handleMessage(nextMsg);
  }

  public void runForCount(int count, Predicate<ControlledMessage> predicate) {
    for (int i = 0; i < count; i++) {
      Timed<ControlledMessage> nextMsg = this.network.nextMessage(predicate);
      handleMessage(nextMsg);
    }
  }

  /**
   * Returns a predicate that stops processing messages after a specified number of epochs and
   * rounds.
   *
   * @param maxEpochRound the last epoch and round to process
   * @return a predicate that halts processing after the specified number of epochs and rounds
   */
  public static Predicate<Timed<ControlledMessage>> hasReachedEpochRound(EpochRound maxEpochRound) {
    return timedMsg -> {
      ControlledMessage message = timedMsg.value();
      if (!(message.message() instanceof Proposal proposal)) {
        return false;
      }
      EpochRound nev = EpochRound.of(proposal.getEpoch(), proposal.getRound());
      return (nev.compareTo(maxEpochRound) > 0);
    };
  }

  /**
   * Returns a predicate that stops processing messages after a specified number of rounds.
   *
   * @param round the last round to process
   * @return a predicate that return true after the specified number of rounds
   */
  public static Predicate<Timed<ControlledMessage>> hasReachedRound(Round round) {
    final long maxRoundNumber = round.previous().number();
    return timedMsg -> {
      ControlledMessage message = timedMsg.value();
      if (!(message.message() instanceof Proposal p)) {
        return false;
      }
      return (p.getRound().number() > maxRoundNumber);
    };
  }

  public static Predicate<Timed<ControlledMessage>> roundUpdateOnNode(Round round, int nodeIndex) {
    return timedMsg -> {
      final var message = timedMsg.value();
      // This method works with both epoched and non-epoched consensus tests
      if (message.message() instanceof final EpochRoundUpdate epochRoundUpdate) {
        return message.channelId().receiverIndex() == nodeIndex
            && epochRoundUpdate.roundUpdate().currentRound().gte(round);
      } else if (message.message() instanceof final RoundUpdate roundUpdate) {
        return message.channelId().receiverIndex() == nodeIndex
            && roundUpdate.currentRound().gte(round);
      } else {
        return false;
      }
    };
  }

  public static Predicate<Timed<ControlledMessage>> epochLedgerUpdate(long epoch) {
    return timedMsg -> {
      final var message = timedMsg.value();
      if (message.message() instanceof final LedgerUpdate ledgerUpdate) {
        var epochChange = ledgerUpdate.epochChange();
        return epochChange.isPresent() && epochChange.orElseThrow().nextEpoch() == epoch;
      }

      return false;
    };
  }

  public <T> T getInstance(int nodeIndex, Class<T> instanceClass) {
    return this.nodes.getInstance(nodeIndex, instanceClass);
  }

  public <T> T getInstance(int nodeIndex, Key<T> key) {
    return this.nodes.getInstance(nodeIndex, key);
  }

  public int numNodes() {
    return this.nodes.numNodes();
  }

  public List<Integer> getNodeIndices() {
    return this.nodes.getNodeIndices();
  }

  public List<Integer> getLiveNodeIndices() {
    return this.nodes.getLiveNodeIndices();
  }

  public ComponentAddress faucetAddress() {
    return ScryptoConstants.FAUCET_ADDRESS;
  }
}
