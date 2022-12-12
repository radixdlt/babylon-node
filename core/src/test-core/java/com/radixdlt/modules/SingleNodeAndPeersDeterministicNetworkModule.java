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

package com.radixdlt.modules;

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Singleton;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.Environment;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.logger.EventLoggerConfig;
import com.radixdlt.logger.EventLoggerModule;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.rev1.modules.REv1PersistenceModule;
import com.radixdlt.rev1.modules.RadixEngineStoreModule;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.UInt256;
import java.util.List;
import java.util.stream.Stream;

/** Module which injects a full one node network */
public final class SingleNodeAndPeersDeterministicNetworkModule extends AbstractModule {
  private final ECKeyPair self;
  private final FunctionalRadixNodeModule radixNodeModule;

  public static SingleNodeAndPeersDeterministicNetworkModule rev1(
      ECKeyPair self, int maxMempoolSize, String databasePath) {
    return new SingleNodeAndPeersDeterministicNetworkModule(
        self,
        new FunctionalRadixNodeModule(
            true,
            FunctionalRadixNodeModule.SafetyRecoveryConfig.berkeleyStore(databasePath),
            FunctionalRadixNodeModule.ConsensusConfig.of(),
            FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                StateComputerConfig.rev1(maxMempoolSize),
                new SyncRelayConfig(500, 10, 3000, 10, Long.MAX_VALUE))));
  }

  public SingleNodeAndPeersDeterministicNetworkModule(
      ECKeyPair self, FunctionalRadixNodeModule radixNodeModule) {
    this.self = self;
    this.radixNodeModule = radixNodeModule;
  }

  @Override
  protected void configure() {
    // System
    bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
    bind(TimeSupplier.class).toInstance(System::currentTimeMillis);

    var addressing = Addressing.ofNetwork(Network.INTEGRATIONTESTNET);
    bind(Addressing.class).toInstance(addressing);
    install(new EventLoggerModule(EventLoggerConfig.addressed(addressing)));
    install(new InMemoryBFTKeyModule(self));
    install(new CryptoModule());
    install(radixNodeModule);
    if (radixNodeModule.supportsREv2()) {
      // FIXME: a hack for tests that use rev2 (api); fix once ledger/consensus recovery are
      // hooked up
      bind(BFTValidatorSet.class)
          .toInstance(
              BFTValidatorSet.from(
                  List.of(BFTValidator.from(BFTNode.create(self.getPublicKey()), UInt256.ONE))));
    }
    if (radixNodeModule.supportsREv1()) {

      install(new REv1PersistenceModule());
      install(new RadixEngineStoreModule());
    }
  }

  @Provides
  public List<BFTNode> nodes(@Self BFTNode self) {
    return List.of(self);
  }

  @Provides
  @Singleton
  public DeterministicNetwork network(@Self BFTNode self, PeersView peersView) {
    return new DeterministicNetwork(
        Stream.concat(Stream.of(self), peersView.peers().map(PeersView.PeerInfo::bftNode)).toList(),
        MessageSelector.firstSelector(),
        MessageMutator.nothing());
  }

  @Provides
  @Singleton
  Environment environment(@Self BFTNode self, DeterministicNetwork network) {
    return network.createSender(self);
  }
}
