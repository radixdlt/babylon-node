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

import com.google.common.base.Splitter;
import com.google.common.collect.Streams;
import com.google.inject.AbstractModule;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.ApiModule;
import com.radixdlt.api.CoreApiServerModule;
import com.radixdlt.consensus.MockedConsensusRecoveryModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochsConsensusModule;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.environment.rx.RxEnvironmentModule;
import com.radixdlt.keys.PersistedBFTKeyModule;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.mempool.MempoolReceiverModule;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.mempool.MempoolRelayerModule;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.messaging.MessagingModule;
import com.radixdlt.modules.*;
import com.radixdlt.networks.Network;
import com.radixdlt.networks.NetworkId;
import com.radixdlt.p2p.P2PModule;
import com.radixdlt.p2p.capability.LedgerSyncCapability;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.statemanager.CoreApiServerConfig;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.store.DatabasePropertiesModule;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.utils.BooleanUtils;
import com.radixdlt.utils.IOUtils;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.FileInputStream;
import java.io.IOException;
import java.util.Optional;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.json.JSONObject;

/** Module which manages everything in a single node */
public final class RadixNodeModule extends AbstractModule {
  private static final int DEFAULT_CORE_API_PORT = 3333;
  private static final int DEFAULT_SYSTEM_API_PORT = 3334;
  private static final String DEFAULT_CORE_API_BIND_ADDRESS = "0.0.0.0";
  private static final String DEFAULT_SYSTEM_API_BIND_ADDRESS = "127.0.0.1";

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
    if (Network.ofId(this.networkId).isEmpty()) {
      throw new IllegalStateException(
          "NetworkId " + networkId + " does not match any known networks");
    }

    var addressing = Addressing.ofNetworkId(networkId);
    bind(Addressing.class).toInstance(addressing);
    bindConstant().annotatedWith(NetworkId.class).to(networkId);

    bind(RuntimeProperties.class).toInstance(properties);

    // Consensus configuration
    // These cannot be changed without introducing possibilities of
    // going out of sync with consensus.
    bindConstant()
        .annotatedWith(BFTSyncPatienceMillis.class)
        .to(properties.get("bft.sync.patience", 200));

    // Default values mean that pacemakers will sync if they are within 5 rounds of each other.
    // 5 consecutive failing rounds will take 1*(2^6)-1 seconds = 63 seconds.
    bindConstant().annotatedWith(PacemakerBaseTimeoutMs.class).to(3000L);
    bindConstant().annotatedWith(PacemakerBackoffRate.class).to(1.1);
    bindConstant().annotatedWith(PacemakerMaxExponent.class).to(0);

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
    install(new MempoolRelayConfig(5, 60000, 60000, 100).asModule());
    install(new MempoolRelayerModule());

    // Sync
    final long syncPatience = properties.get("sync.patience", 5000L);
    install(new SyncServiceModule(SyncRelayConfig.of(syncPatience, 10, 3000L)));

    // Epochs - Consensus
    install(new EpochsConsensusModule());
    // Epochs - Sync
    install(new EpochsSyncModule());

    // State Computer
    var mempoolMaxSize = properties.get("mempool.maxSize", 10000);
    var mempoolConfig = new RustMempoolConfig(mempoolMaxSize);
    var databasePath = properties.get("db.location", ".//RADIXDB");
    var databaseConfig = new REv2DatabaseConfig.RocksDB(databasePath);
    install(REv2StateManagerModule.create(networkId, databaseConfig, Option.some(mempoolConfig)));
    install(new MockedPersistenceStoreModule());

    // Core API server
    final var coreApiBindAddress =
        properties.get("api.core.bind_address", DEFAULT_CORE_API_BIND_ADDRESS);
    final var coreApiPort = properties.get("api.core.port", DEFAULT_CORE_API_PORT);
    final var coreApiServerConfig =
        new CoreApiServerConfig(coreApiBindAddress, UInt32.fromNonNegativeInt(coreApiPort));
    install(new CoreApiServerModule(coreApiServerConfig));

    // Storage
    install(new DatabasePropertiesModule());
    // install(new PersistenceModule());
    // install(new ConsensusRecoveryModule());
    // install(new LedgerRecoveryModule());

    String genesisTxn;
    final var genesisFileProp = properties.get("network.genesis_file");
    if (genesisFileProp != null && !genesisFileProp.isBlank()) {
      log.info("Loading genesis from file: {}", genesisFileProp);
      genesisTxn = loadGenesisFromFile(genesisFileProp);
    } else {
      log.info("Loading genesis from genesis_txn property");
      genesisTxn = properties.get("network.genesis_txn");
    }

    log.info("Using genesis txn: {}", genesisTxn);

    final var initialVset =
        Streams.stream(
                Splitter.fixedLength(ECDSASecp256k1PublicKey.COMPRESSED_BYTES * 2)
                    .split(genesisTxn))
            .map(
                pubKeyBytes -> {
                  log.info("Initial vset validator: {}", pubKeyBytes);
                  try {
                    return BFTNode.create(ECDSASecp256k1PublicKey.fromHex(pubKeyBytes));
                  } catch (PublicKeyException e) {
                    throw new RuntimeException(e);
                  }
                })
            .toList();

    install(new MockedConsensusRecoveryModule.Builder().withNodes(initialVset).build());
    install(new MockedLedgerRecoveryModule());

    // System Info
    install(new SystemInfoModule());

    install(new MessagingModule(properties));

    install(new P2PModule(properties));

    // API
    final var systemApiBindAddress =
        properties.get("api.system.bind_address", DEFAULT_SYSTEM_API_BIND_ADDRESS);
    final var systemApiPort = properties.get("api.system.port", DEFAULT_SYSTEM_API_PORT);
    install(new ApiModule(systemApiBindAddress, systemApiPort));

    // Capabilities
    var capabilitiesLedgerSyncEnabled =
        properties.get("capabilities.ledger_sync.enabled", BooleanUtils::parseBoolean);
    LedgerSyncCapability.Builder builder =
        capabilitiesLedgerSyncEnabled
            .map(LedgerSyncCapability.Builder::new)
            .orElse(LedgerSyncCapability.Builder.asDefault());
    install(new CapabilitiesModule(builder.build()));
  }

  private String loadGenesisFromFile(String genesisFile) {
    try (var genesisJsonString = new FileInputStream(genesisFile)) {
      var genesisJson = new JSONObject(IOUtils.toString(genesisJsonString));
      return genesisJson.getString("genesis");
    } catch (IOException e) {
      throw new IllegalStateException(e);
    }
  }
}
