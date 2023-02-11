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
import com.google.common.base.Strings;
import com.google.common.collect.Streams;
import com.google.inject.AbstractModule;
import com.google.inject.Key;
import com.google.inject.multibindings.OptionalBinder;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.CoreApiServerModule;
import com.radixdlt.api.prometheus.PrometheusApiModule;
import com.radixdlt.api.system.SystemApiModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochsConsensusModule;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.environment.rx.RxEnvironmentModule;
import com.radixdlt.identifiers.Address;
import com.radixdlt.keys.BFTValidatorIdFromGenesisModule;
import com.radixdlt.keys.BFTValidatorIdModule;
import com.radixdlt.keys.PersistedBFTKeyModule;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.logger.EventLoggerConfig;
import com.radixdlt.logger.EventLoggerModule;
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
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.modules.BerkeleySafetyStoreModule;
import com.radixdlt.rev2.modules.REv2ConsensusRecoveryModule;
import com.radixdlt.rev2.modules.REv2LedgerRecoveryModule;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.utils.BooleanUtils;
import com.radixdlt.utils.IOUtils;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt64;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.FileInputStream;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.json.JSONObject;

/** Module which manages everything in a single node */
public final class RadixNodeModule extends AbstractModule {
  private static final int DEFAULT_CORE_API_PORT = 3333;
  private static final int DEFAULT_SYSTEM_API_PORT = 3334;
  private static final int DEFAULT_PROMETHEUS_API_PORT = 3335;

  // APIs are only exposed on localhost by default
  private static final String DEFAULT_CORE_API_BIND_ADDRESS = "127.0.0.1";
  private static final String DEFAULT_SYSTEM_API_BIND_ADDRESS = "127.0.0.1";
  private static final String DEFAULT_PROMETHEUS_API_BIND_ADDRESS = "127.0.0.1";

  // Genesis parameters for XRD allocation for testnets
  private static final Set<Network> GENESIS_NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT =
      Set.of(
          Network.GILGANET,
          Network.ENKINET,
          Network.HAMMUNET,
          Network.MARDUNET,
          Network.NERGALNET,
          Network.NEBUNET);
  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE =
      Decimal.of(700_000_000_000L); // 70% XRD_MAX_SUPPLY
  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1_000_000_000L); // 0.1% XRD_MAX_SUPPLY
  private static final ECDSASecp256k1PublicKey GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY =
      ECDSASecp256k1PublicKey.tryFromHex(
              "026f08db98ef1d0231eb15580da9123db8e25aa1747c8c32e5fd2ec47b8db73d5c")
          .unwrap();
  private static final Decimal GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1); // Allow it to be easily changed in eg tests

  // Proposal constants
  // Up to 20 txns and 5 MiB total payload size for a proposal.
  // With a txn limit of 1 MiB, a proposal can fit at least 5 txns.
  public static final int MAX_TRANSACTIONS_PER_PROPOSAL = 20;
  public static final int MAX_PROPOSAL_TOTAL_TXNS_PAYLOAD_SIZE = 5 * 1024 * 1024;

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
    final var network =
        Network.ofId(this.networkId)
            .orElseThrow(
                () ->
                    new IllegalStateException(
                        "NetworkId " + networkId + " does not match any known networks"));

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

    var addressing = Addressing.ofNetworkId(networkId);
    bind(Addressing.class).toInstance(addressing);
    install(new EventLoggerModule(EventLoggerConfig.addressed(addressing)));
    install(new DispatcherModule());

    // Consensus
    final String useGenesisProperty = properties.get("consensus.use_genesis_for_validator_address");
    final Option<Boolean> useGenesis =
        Strings.isNullOrEmpty(useGenesisProperty)
            ? Option.none()
            : Option.some(Boolean.parseBoolean(useGenesisProperty));
    final String validatorAddress = properties.get("consensus.validator_address", (String) null);
    if (useGenesis.isPresent() && useGenesis.unwrap() && !Strings.isNullOrEmpty(validatorAddress)) {
      throw new IllegalArgumentException(
          "Invalid configuration. Using both consensus.use_genesis_for_validator_address=true and"
              + " consensus.validator_address. Please use one.");
    } else if (!Strings.isNullOrEmpty(validatorAddress)) {
      OptionalBinder.newOptionalBinder(binder(), Key.get(ComponentAddress.class, Self.class))
          .setBinding()
          .toInstance(addressing.decodeValidatorAddress(validatorAddress));
      install(new BFTValidatorIdModule());
    } else if (useGenesis.isEmpty() || (useGenesis.isPresent() && useGenesis.unwrap())) {
      install(new BFTValidatorIdFromGenesisModule());
    } else {
      // No validator address provided, and use genesis explicitly disabled
      OptionalBinder.newOptionalBinder(binder(), Key.get(ComponentAddress.class, Self.class));
      install(new BFTValidatorIdModule());
    }
    install(new PersistedBFTKeyModule());
    install(new CryptoModule());
    install(new ConsensusModule());

    // Ledger
    install(new LedgerModule());
    install(new MempoolReceiverModule());

    // Mempool Relay
    install(new MempoolRelayConfig(5, 100).asModule());
    install(new MempoolRelayerModule(10000));

    // Ledger Sync
    final long syncPatience = properties.get("sync.patience", 5000L);
    install(new SyncServiceModule(SyncRelayConfig.of(syncPatience, 10, 3000L)));

    // Epochs - Consensus
    install(new EpochsConsensusModule());
    // Epochs - Sync
    install(new EpochsSyncModule());

    // State Computer
    var databasePath = properties.get("db.location", ".//RADIXDB");
    var mempoolMaxSize = properties.get("mempool.maxSize", 50);
    var mempoolConfig = new RustMempoolConfig(mempoolMaxSize);
    var databaseConfig = new REv2DatabaseConfig.RocksDB(databasePath);

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
                    return ECDSASecp256k1PublicKey.fromHex(pubKeyBytes);
                  } catch (PublicKeyException e) {
                    throw new RuntimeException(e);
                  }
                })
            .toList();
    var validatorSet =
        new HashMap<ECDSASecp256k1PublicKey, Tuple.Tuple2<Decimal, ComponentAddress>>();
    final var usePowerfulStakingAccount =
        GENESIS_NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT.contains(network);

    final var stakingAccount =
        usePowerfulStakingAccount
            ? Address.virtualAccountAddress(GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY)
            : Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    final var stakeAmount =
        usePowerfulStakingAccount
            ? GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR
            : GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR;

    initialVset.forEach(k -> validatorSet.put(k, Tuple.tuple(stakeAmount, stakingAccount)));

    final Map<ECDSASecp256k1PublicKey, Decimal> xrdAllocations =
        usePowerfulStakingAccount
            ? Map.of(
                GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY,
                GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE)
            : Map.of();

    log.info("Genesis XRD allocations: {}", xrdAllocations.isEmpty() ? "(empty)" : "");
    xrdAllocations.forEach((k, v) -> log.info("{}: {}", k, v));

    var genesis =
        TransactionBuilder.createGenesis(
            validatorSet,
            xrdAllocations,
            UInt64.fromNonNegativeLong(1),
            UInt64.fromNonNegativeLong(1800), // approximately 5 minutes per epoch
            UInt64.fromNonNegativeLong(1));

    install(
        REv2StateManagerModule.create(
            networkId,
            MAX_TRANSACTIONS_PER_PROPOSAL,
            MAX_PROPOSAL_TOTAL_TXNS_PAYLOAD_SIZE,
            databaseConfig,
            Option.some(mempoolConfig)));

    // Recovery
    install(new BerkeleySafetyStoreModule(databasePath));
    var initialAccumulatorState = new AccumulatorState(0, HashUtils.zero256());
    install(new REv2LedgerRecoveryModule(initialAccumulatorState, genesis));
    install(new REv2ConsensusRecoveryModule());

    install(new MetricsModule());

    // System Info
    install(new SystemInfoModule());

    install(new MessagingModule(properties));

    install(new P2PModule(properties));

    // APIs
    final var coreApiBindAddress =
        properties.get("api.core.bind_address", DEFAULT_CORE_API_BIND_ADDRESS);
    final var coreApiPort = properties.get("api.core.port", DEFAULT_CORE_API_PORT);
    install(new CoreApiServerModule(coreApiBindAddress, coreApiPort));

    final var systemApiBindAddress =
        properties.get("api.system.bind_address", DEFAULT_SYSTEM_API_BIND_ADDRESS);
    final var systemApiPort = properties.get("api.system.port", DEFAULT_SYSTEM_API_PORT);
    install(new SystemApiModule(systemApiBindAddress, systemApiPort));

    final var metricsApiBindAddress =
        properties.get("api.prometheus.bind_address", DEFAULT_PROMETHEUS_API_BIND_ADDRESS);
    final var metricsApiPort = properties.get("api.prometheus.port", DEFAULT_PROMETHEUS_API_PORT);
    install(new PrometheusApiModule(metricsApiBindAddress, metricsApiPort));

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
