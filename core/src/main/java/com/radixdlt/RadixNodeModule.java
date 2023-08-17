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

import com.google.common.base.Preconditions;
import com.google.common.base.Strings;
import com.google.inject.AbstractModule;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.CoreApiServerModule;
import com.radixdlt.api.prometheus.PrometheusApiModule;
import com.radixdlt.api.system.SystemApiModule;
import com.radixdlt.consensus.ProposalLimitsConfig;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochsConsensusModule;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.environment.DatabaseFlags;
import com.radixdlt.environment.NodeConstants;
import com.radixdlt.environment.VertexLimitsConfig;
import com.radixdlt.environment.rx.RxEnvironmentModule;
import com.radixdlt.genesis.GenesisProvider;
import com.radixdlt.keys.PersistedBFTKeyModule;
import com.radixdlt.keys.SelfValidatorInfoFromGenesisModule;
import com.radixdlt.keys.SelfValidatorInfoModule;
import com.radixdlt.lang.Option;
import com.radixdlt.logger.EventLoggerConfig;
import com.radixdlt.logger.EventLoggerModule;
import com.radixdlt.mempool.*;
import com.radixdlt.messaging.MessagingModule;
import com.radixdlt.modules.*;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.P2PModule;
import com.radixdlt.p2p.capability.LedgerSyncCapability;
import com.radixdlt.rev2.modules.*;
import com.radixdlt.store.NodeStorageLocationFromPropertiesModule;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.utils.BooleanUtils;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.time.Duration;
import java.util.Optional;

/** Module which manages everything in a single node */
public final class RadixNodeModule extends AbstractModule {
  private static final int DEFAULT_CORE_API_PORT = 3333;
  private static final int DEFAULT_SYSTEM_API_PORT = 3334;
  private static final int DEFAULT_PROMETHEUS_API_PORT = 3335;

  // APIs are only exposed on localhost by default
  private static final String DEFAULT_CORE_API_BIND_ADDRESS = "127.0.0.1";
  private static final String DEFAULT_SYSTEM_API_BIND_ADDRESS = "127.0.0.1";
  private static final String DEFAULT_PROMETHEUS_API_BIND_ADDRESS = "127.0.0.1";

  private final RuntimeProperties properties;
  private final Network network;

  private final GenesisProvider genesisProvider;

  public RadixNodeModule(
      RuntimeProperties properties, Network network, GenesisProvider genesisProvider) {
    this.properties = properties;
    this.network = network;
    this.genesisProvider = genesisProvider;
  }

  @Override
  protected void configure() {
    bind(RuntimeProperties.class).toInstance(properties);

    final var addressing = Addressing.ofNetwork(network);
    bind(Network.class).toInstance(network);
    bind(Addressing.class).toInstance(addressing);

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
    bindConstant().annotatedWith(AdditionalRoundTimeIfProposalReceivedMs.class).to(30_000L);
    bindConstant().annotatedWith(TimeoutQuorumResolutionDelayMs.class).to(1500L);

    // System (e.g. time, random)
    install(new SystemModule());

    install(new RxEnvironmentModule());

    install(new DispatcherModule());

    // Consensus
    install(new EventLoggerModule(EventLoggerConfig.addressed(addressing)));

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
      install(
          new SelfValidatorInfoModule(
              Optional.of(addressing.decodeValidatorAddress(validatorAddress))));
    } else if (useGenesis.isEmpty() || (useGenesis.isPresent() && useGenesis.unwrap())) {
      install(new SelfValidatorInfoFromGenesisModule());
    } else {
      // No validator address provided, and use genesis explicitly disabled
      install(new SelfValidatorInfoModule(Optional.empty()));
    }

    install(new PersistedBFTKeyModule());
    install(new CryptoModule());

    // Ledger
    install(new LedgerModule());
    install(new MempoolReceiverModule());

    // Mempool Relay
    install(new MempoolReceiverConfig(5).asModule());
    var mempoolRelayerIntervalMs =
        properties.get("mempool.relayer.interval_ms", MempoolRelayerConfig.DEFAULT_INTERVAL_MS);
    var mempoolRelayerMaxPeers =
        properties.get("mempool.relayer.max_peers", MempoolRelayerConfig.DEFAULT_MAX_PEERS);
    var mempoolRelayerMaxRelayedSize =
        properties.get(
            "mempool.relayer.max_relayed_size", MempoolRelayerConfig.DEFAULT_MAX_RELAYED_SIZE);
    var mempoolRelayerMaxMessageTransactionCount =
        properties.get(
            "mempool.relayer.max_message_transaction_count",
            MempoolRelayerConfig.DEFAULT_MAX_MESSAGE_TRANSACTION_COUNT);
    var mempoolRelayerMaxMessagePayloadSize =
        properties.get(
            "mempool.relayer.max_message_payload_size",
            MempoolRelayerConfig.DEFAULT_MAX_MESSAGE_PAYLOAD_SIZE);
    Preconditions.checkArgument(
        mempoolRelayerMaxPeers > 0,
        "Invalid configuration: mempool.relayer.max_peers ("
            + mempoolRelayerMaxPeers
            + ") must be a non zero positive number.");
    Preconditions.checkArgument(
        mempoolRelayerMaxMessageTransactionCount > 0,
        "Invalid configuration: mempool.relayer.max_message_transaction_count (%s) must be a non"
            + " zero positive number.",
        mempoolRelayerMaxMessageTransactionCount);
    Preconditions.checkArgument(
        mempoolRelayerMaxMessagePayloadSize >= NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE,
        "Invalid configuration: mempool.relayer.max_message_payload_size (%s) must be at least the"
            + " maximum transaction size (%s).",
        mempoolRelayerMaxMessagePayloadSize,
        NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE);
    Preconditions.checkArgument(
        mempoolRelayerMaxRelayedSize >= mempoolRelayerMaxMessagePayloadSize,
        "Invalid configuration: mempool.relayer.max_relayed_size (%s) must be at least"
            + " mempool.relayer.max_message_payload_size (%s).",
        mempoolRelayerMaxRelayedSize,
        mempoolRelayerMaxMessagePayloadSize);
    install(
        new MempoolRelayerModule(
            new MempoolRelayerConfig(
                mempoolRelayerIntervalMs,
                mempoolRelayerMaxPeers,
                mempoolRelayerMaxRelayedSize,
                mempoolRelayerMaxMessageTransactionCount,
                mempoolRelayerMaxMessagePayloadSize)));

    // Mempool Reevaluation
    var mempoolReevaluationIntervalMs = properties.get("mempool.reevaluation.interval_ms", 10000);
    var mempoolReevaluationMaxCount = properties.get("mempool.reevaluation.max_count", 5);
    install(
        new MempoolReevaluationModule(
            Duration.ofMillis(mempoolReevaluationIntervalMs), mempoolReevaluationMaxCount));

    // Ledger Sync
    final long syncPatience = properties.get("sync.patience", 5000L);
    install(new SyncServiceModule(SyncRelayConfig.of(syncPatience, 10, 3000L)));

    // Epochs - Consensus
    install(new EpochsConsensusModule());
    // Epochs - Sync
    install(new EpochsSyncModule());

    // Storage directory
    install(new NodeStorageLocationFromPropertiesModule());
    // State Computer
    var mempoolMaxMemory =
        properties.get(
            "mempool.max_memory",
            (int)
                (NodeConstants.DEFAULT_MEMPOOL_MAX_TOTAL_TRANSACTIONS_SIZE
                    * NodeConstants.MEMPOOL_TRANSACTION_OVERHEAD_FACTOR));
    var mempoolMaxTotalTransactionsSize =
        (int) (mempoolMaxMemory / NodeConstants.MEMPOOL_TRANSACTION_OVERHEAD_FACTOR);
    var mempoolMaxTransactionCount =
        properties.get(
            "mempool.max_transaction_count", NodeConstants.DEFAULT_MEMPOOL_MAX_TRANSACTION_COUNT);
    Preconditions.checkArgument(
        mempoolMaxTransactionCount > 0,
        "Invalid configuration: mempool.max_transaction_count (%s) must be a non zero positive"
            + " number.",
        mempoolMaxTransactionCount);
    Preconditions.checkArgument(
        mempoolMaxTotalTransactionsSize >= NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE,
        "Invalid configuration: Computed mempool total transactions size is lower than the maximum"
            + " transaction size (%s). Please increase mempool.max_memory.",
        NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE);
    var mempoolConfig =
        new RustMempoolConfig(mempoolMaxTotalTransactionsSize, mempoolMaxTransactionCount);
    var enableLocalTransactionExecutionIndex =
        properties.get("db.local_transaction_execution_index.enable", true);
    var enableAccountChangeIndex = properties.get("db.account_change_index.enable", true);
    var databaseFlags =
        new DatabaseFlags(enableLocalTransactionExecutionIndex, enableAccountChangeIndex);

    install(new REv2LedgerInitializerModule(genesisProvider));

    var vertexMaxTransactionCount =
        properties.get(
            "protocol.vertex.max_transaction_count",
            NodeConstants.DEFAULT_MAX_VERTEX_TRANSACTION_COUNT);
    var vertexMaxTotalTransactionsSize =
        properties.get(
            "protocol.vertex.max_total_transactions_size",
            (int) NodeConstants.DEFAULT_MAX_TOTAL_VERTEX_TRANSACTIONS_SIZE);
    var vertexMaxTotalExecutionCostUnitsConsumed =
        properties.get(
            "protocol.vertex.max_total_execution_cost_units_consumed",
            NodeConstants.DEFAULT_MAX_TOTAL_VERTEX_EXECUTION_COST_UNITS_CONSUMED);
    Preconditions.checkArgument(
        vertexMaxTransactionCount > 0,
        "Invalid configuration: protocol.vertex.max_transaction_count (%s) must be a non zero"
            + " positive number.",
        vertexMaxTransactionCount);
    Preconditions.checkArgument(
        vertexMaxTotalTransactionsSize >= NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE,
        "Invalid configuration: protocol.vertex.max_total_transactions_size (%s) must be at least"
            + " the maximum transaction size (%s).",
        vertexMaxTotalTransactionsSize,
        NodeConstants.DEFAULT_MAX_TRANSACTION_SIZE);
    Preconditions.checkArgument(
        vertexMaxTotalExecutionCostUnitsConsumed >= NodeConstants.DEFAULT_EXECUTION_COST_UNIT_LIMIT,
        "Invalid configuration: protocol.vertex.max_total_execution_cost_units_consumed (%s) must"
            + " be at least the transaction cost unit limit (%s).",
        vertexMaxTotalExecutionCostUnitsConsumed,
        NodeConstants.DEFAULT_EXECUTION_COST_UNIT_LIMIT);
    var vertexLimitsConfig =
        new VertexLimitsConfig(
            vertexMaxTransactionCount,
            vertexMaxTotalTransactionsSize,
            vertexMaxTotalExecutionCostUnitsConsumed);
    install(
        REv2StateManagerModule.create(
            ProposalLimitsConfig.from(vertexLimitsConfig),
            vertexLimitsConfig,
            REv2StateManagerModule.DatabaseType.ROCKS_DB,
            databaseFlags,
            Option.some(mempoolConfig)));

    // Recovery
    install(new BerkeleySafetyStoreModule());
    install(new EpochsSafetyRecoveryModule());
    install(new REv2LedgerRecoveryModule());
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
}
