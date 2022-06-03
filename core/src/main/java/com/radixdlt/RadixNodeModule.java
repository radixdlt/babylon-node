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

import com.google.inject.AbstractModule;
import com.radixdlt.api.ApiModule;
import com.radixdlt.capability.CapabilitiesModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.rx.RxEnvironmentModule;
import com.radixdlt.keys.PersistedBFTKeyModule;
import com.radixdlt.mempool.MempoolConfig;
import com.radixdlt.mempool.MempoolReceiverModule;
import com.radixdlt.mempool.MempoolRelayerModule;
import com.radixdlt.modules.*;
import com.radixdlt.network.hostip.HostIpModule;
import com.radixdlt.network.messaging.MessageCentralModule;
import com.radixdlt.network.messaging.MessagingModule;
import com.radixdlt.network.p2p.P2PModule;
import com.radixdlt.network.p2p.PeerDiscoveryModule;
import com.radixdlt.network.p2p.PeerLivenessMonitorModule;
import com.radixdlt.networks.Addressing;
import com.radixdlt.networks.NetworkId;
import com.radixdlt.rev2.modules.InMemoryCommittedReaderModule;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.rev2.modules.MockedRecoveryModule;
import com.radixdlt.rev2.modules.REv2StateComputerModule;
import com.radixdlt.store.DatabasePropertiesModule;
import com.radixdlt.sync.SyncConfig;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.util.Optional;
import java.util.Set;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Module which manages everything in a single node */
public final class RadixNodeModule extends AbstractModule {
  private static final int DEFAULT_CORE_PORT = 3333;
  private static final String DEFAULT_BIND_ADDRESS = "0.0.0.0";
  private static final Logger log = LogManager.getLogger();

  private final RuntimeProperties properties;
  private final int networkId;

  public RadixNodeModule(RuntimeProperties properties) {
    this.properties = properties;
    this.networkId =
        Optional.ofNullable(properties.get("network.id"))
            .map(Integer::parseInt)
            .orElseThrow(() -> new IllegalStateException("Must specify network.id"));
  }

  @Override
  protected void configure() {
    if (this.networkId <= 0) {
      throw new IllegalStateException("Illegal networkId " + networkId);
    }

    var addressing = Addressing.ofNetworkId(networkId);
    bind(Addressing.class).toInstance(addressing);
    bindConstant().annotatedWith(NetworkId.class).to(networkId);

    // Use genesis to specify number of validators for now
    var numValidators = Integer.parseInt(properties.get("network.genesis_txn"));
    var initialVset =
        BFTValidatorSet.from(
            PrivateKeys.numeric(6)
                .limit(numValidators)
                .map(ECKeyPair::getPublicKey)
                .map(k -> BFTValidator.from(BFTNode.create(k), UInt256.ONE)));
    bind(BFTValidatorSet.class).toInstance(initialVset);
    bind(RuntimeProperties.class).toInstance(properties);

    // Consensus configuration
    // These cannot be changed without introducing possibilities of
    // going out of sync with consensus.
    bindConstant()
        .annotatedWith(BFTSyncPatienceMillis.class)
        .to(properties.get("bft.sync.patience", 200));

    // Default values mean that pacemakers will sync if they are within 5 views of each other.
    // 5 consecutive failing views will take 1*(2^6)-1 seconds = 63 seconds.
    bindConstant().annotatedWith(PacemakerTimeout.class).to(3000L);
    bindConstant().annotatedWith(PacemakerRate.class).to(1.1);
    bindConstant().annotatedWith(PacemakerMaxExponent.class).to(0);

    // Mempool configuration
    var mempoolMaxSize = properties.get("mempool.maxSize", 10000);
    install(MempoolConfig.asModule(mempoolMaxSize, 5, 60000, 60000, 100));

    // Sync configuration
    final long syncPatience = properties.get("sync.patience", 5000L);
    bind(SyncConfig.class).toInstance(SyncConfig.of(syncPatience, 10, 3000L));

    // System (e.g. time, random)
    install(new SystemModule());

    install(new RxEnvironmentModule());

    install(new EventLoggerModule());
    install(new DispatcherModule());

    // Consensus
    install(new PersistedBFTKeyModule());
    install(new CryptoModule());
    install(new ConsensusModule());

    // Ledger
    install(new LedgerModule());
    install(new MempoolReceiverModule());

    // Mempool Relay
    install(new MempoolRelayerModule());

    // Sync
    install(new SyncServiceModule());

    // Epochs - Consensus
    install(new EpochsConsensusModule());
    // Epochs - Sync
    install(new EpochsSyncModule());

    // State Computer
    install(new MockedPersistenceStoreModule());
    install(new REv2StateComputerModule());
    install(new InMemoryCommittedReaderModule());

    // Storage
    install(new DatabasePropertiesModule());
    // install(new PersistenceModule());
    // install(new ConsensusRecoveryModule());
    // install(new LedgerRecoveryModule());
    install(new MockedRecoveryModule());

    // System Info
    install(new SystemInfoModule());

    // Network
    install(new MessagingModule());
    install(new MessageCentralModule(properties));
    install(new HostIpModule(properties));
    install(new P2PModule(properties));
    install(new PeerDiscoveryModule());
    install(new PeerLivenessMonitorModule());

    install(new StateManagerModule());

    // API
    var bindAddress = properties.get("api.bind.address", DEFAULT_BIND_ADDRESS);
    var port = properties.get("api.port", DEFAULT_CORE_PORT);
    install(new ApiModule(bindAddress, port));

    var disabledCapabilities = Set.of(properties.get("disabled_capabilities", "").split(","));
    install(new CapabilitiesModule(disabledCapabilities));
  }
}
