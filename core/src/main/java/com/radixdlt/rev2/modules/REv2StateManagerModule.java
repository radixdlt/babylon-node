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

package com.radixdlt.rev2.modules;

import com.google.inject.*;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.BFTConfiguration;
import com.radixdlt.consensus.ProposalLimitsConfig;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.vertexstore.PersistentVertexStore;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.*;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.NodeAutoCloseable;
import com.radixdlt.environment.ProcessOnDispatch;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.StateComputerLedger;
import com.radixdlt.mempool.*;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.NewestProtocolVersion;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.RustProtocolUpdate;
import com.radixdlt.recovery.VertexStoreRecovery;
import com.radixdlt.rev2.*;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.store.NodeStorageLocation;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.NotarizedTransactionHash;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.io.File;

public final class REv2StateManagerModule extends AbstractModule {

  private final ProposalLimitsConfig proposalLimitsConfig;
  private final Option<VertexLimitsConfig> vertexLimitsConfigOpt;
  private final DatabaseFlags databaseFlags;
  private final Option<RustMempoolConfig> mempoolConfig;
  private final boolean debugLogging;
  private final StateHashTreeGcConfig stateHashTreeGcConfig;
  private final LedgerProofsGcConfig ledgerProofsGcConfig;
  private final LedgerSyncLimitsConfig ledgerSyncLimitsConfig;
  private final ProtocolConfig protocolConfig;
  private final boolean noFees;

  private REv2StateManagerModule(
      ProposalLimitsConfig proposalLimitsConfig,
      Option<VertexLimitsConfig> vertexLimitsConfigOpt,
      DatabaseFlags databaseFlags,
      Option<RustMempoolConfig> mempoolConfig,
      boolean debugLogging,
      StateHashTreeGcConfig stateHashTreeGcConfig,
      LedgerProofsGcConfig ledgerProofsGcConfig,
      LedgerSyncLimitsConfig ledgerSyncLimitsConfig,
      ProtocolConfig protocolConfig,
      boolean noFees) {
    this.proposalLimitsConfig = proposalLimitsConfig;
    this.vertexLimitsConfigOpt = vertexLimitsConfigOpt;
    this.databaseFlags = databaseFlags;
    this.mempoolConfig = mempoolConfig;
    this.debugLogging = debugLogging;
    this.stateHashTreeGcConfig = stateHashTreeGcConfig;
    this.ledgerProofsGcConfig = ledgerProofsGcConfig;
    this.ledgerSyncLimitsConfig = ledgerSyncLimitsConfig;
    this.protocolConfig = protocolConfig;
    this.noFees = noFees;
  }

  public static REv2StateManagerModule create(
      ProposalLimitsConfig proposalLimitsConfig,
      VertexLimitsConfig vertexLimitsConfig,
      DatabaseFlags databaseFlags,
      Option<RustMempoolConfig> mempoolConfig,
      StateHashTreeGcConfig stateHashTreeGcConfig,
      LedgerProofsGcConfig ledgerProofsGcConfig,
      LedgerSyncLimitsConfig ledgerSyncLimitsConfig) {
    return new REv2StateManagerModule(
        proposalLimitsConfig,
        Option.some(vertexLimitsConfig),
        databaseFlags,
        mempoolConfig,
        false,
        stateHashTreeGcConfig,
        ledgerProofsGcConfig,
        ledgerSyncLimitsConfig,
        ProtocolConfig.productionDefault(),
        false);
  }

  public static REv2StateManagerModule createForTesting(
      ProposalLimitsConfig proposalLimitsConfig,
      DatabaseFlags databaseFlags,
      Option<RustMempoolConfig> mempoolConfig,
      boolean debugLogging,
      StateHashTreeGcConfig stateHashTreeGcConfig,
      LedgerProofsGcConfig ledgerProofsGcConfig,
      LedgerSyncLimitsConfig ledgerSyncLimitsConfig,
      ProtocolConfig protocolConfig,
      boolean noFees) {
    return new REv2StateManagerModule(
        proposalLimitsConfig,
        Option.none(),
        databaseFlags,
        mempoolConfig,
        debugLogging,
        stateHashTreeGcConfig,
        ledgerProofsGcConfig,
        ledgerSyncLimitsConfig,
        protocolConfig,
        noFees);
  }

  @Override
  public void configure() {
    bind(StateComputerLedger.StateComputer.class).to(REv2StateComputer.class);
    bind(REv2TransactionsAndProofReader.class).in(Scopes.SINGLETON);
    bind(TransactionsAndProofReader.class).to(REv2TransactionsAndProofReader.class);
    bind(DatabaseFlags.class).toInstance(databaseFlags);
    bind(LedgerSyncLimitsConfig.class).toInstance(ledgerSyncLimitsConfig);
    install(proposalLimitsConfig.asModule());

    install(
        new AbstractModule() {
          @Provides
          @Singleton
          DatabaseBackendConfig databaseBackendConfig(
              @NodeStorageLocation String nodeStorageLocation) {
            return new DatabaseBackendConfig(
                new File(nodeStorageLocation, "state_manager").getPath());
          }

          @Provides
          @Singleton
          private NodeRustEnvironment stateManager(
              MempoolRelayDispatcher<RawNotarizedTransaction> mempoolRelayDispatcher,
              FatalPanicHandler fatalPanicHandler,
              Network network,
              DatabaseBackendConfig databaseBackendConfig,
              DatabaseFlags databaseFlags) {
            return new NodeRustEnvironment(
                mempoolRelayDispatcher,
                fatalPanicHandler,
                new StateManagerConfig(
                    NetworkDefinition.from(network),
                    mempoolConfig,
                    vertexLimitsConfigOpt,
                    databaseBackendConfig,
                    databaseFlags,
                    getLoggingConfig(),
                    stateHashTreeGcConfig,
                    ledgerProofsGcConfig,
                    ledgerSyncLimitsConfig,
                    protocolConfig,
                    noFees));
          }

          @Provides
          @Singleton
          REv2StateComputer rEv2StateComputer(
              RustStateComputer stateComputer,
              RustMempool mempool,
              RustProtocolUpdate rustProtocolUpdate,
              EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher,
              Hasher hasher,
              EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher,
              Serialization serialization,
              BFTConfiguration initialBftConfiguration,
              Metrics metrics,
              SelfValidatorInfo selfValidatorInfo) {
            return new REv2StateComputer(
                stateComputer,
                mempool,
                rustProtocolUpdate,
                proposalLimitsConfig,
                hasher,
                ledgerUpdateEventDispatcher,
                mempoolAddSuccessEventDispatcher,
                serialization,
                initialBftConfiguration.getProposerElection(),
                metrics,
                selfValidatorInfo);
          }

          @Provides
          REv2TransactionAndProofStore transactionAndProofStore(
              Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
            return new REv2TransactionAndProofStore(metrics, nodeRustEnvironment);
          }

          @Provides
          VertexStoreRecovery rEv2VertexStoreRecovery(
              Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
            return new VertexStoreRecovery(metrics, nodeRustEnvironment);
          }

          @Provides
          TestStateReader testStateReader(NodeRustEnvironment nodeRustEnvironment) {
            return new TestStateReader(nodeRustEnvironment);
          }

          @Provides
          PersistentVertexStore vertexStore(
              VertexStoreRecovery recovery, Metrics metrics, Serialization serialization) {
            return s -> {
              metrics.misc().vertexStoreSaved().inc();
              var vertexStoreBytes = serialization.toDson(s.toSerialized(), DsonOutput.Output.ALL);
              recovery.saveVertexStore(vertexStoreBytes);
            };
          }

          @ProvidesIntoSet
          @ProcessOnDispatch
          EventProcessor<BFTHighQCUpdate> onQCUpdatePersistVertexStore(
              PersistentVertexStore persistentVertexStore) {
            return update -> persistentVertexStore.save(update.getVertexStoreState());
          }

          @ProvidesIntoSet
          @ProcessOnDispatch
          EventProcessor<BFTInsertUpdate> onInsertUpdatePersistVertexStore(
              PersistentVertexStore persistentVertexStore) {
            return update -> persistentVertexStore.save(update.getVertexStoreState());
          }
        });

    if (mempoolConfig.isPresent()) {
      bind(new Key<MempoolReader<PreparedNotarizedTransaction, NotarizedTransactionHash>>() {})
          .to(RustMempool.class);
      bind(new Key<MempoolInserter<RawNotarizedTransaction>>() {}).to(RustMempool.class);
      bind(MempoolReevaluator.class).to(RustMempool.class);
    }
  }

  @ProvidesIntoSet
  NodeAutoCloseable closeable(NodeRustEnvironment nodeRustEnvironment) {
    return nodeRustEnvironment::shutdown;
  }

  public LoggingConfig getLoggingConfig() {
    return debugLogging ? LoggingConfig.getDebug() : LoggingConfig.getDefault();
  }

  @Provides
  @Singleton
  private RustStateComputer rustStateComputer(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RustStateComputer(metrics, nodeRustEnvironment);
  }

  @Provides
  @Singleton
  private RustProtocolUpdate rustProtocolUpdate(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RustProtocolUpdate(metrics, nodeRustEnvironment);
  }

  @Provides
  @Singleton
  private RustMempool rustMempool(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RustMempool(metrics, nodeRustEnvironment);
  }

  @Provides
  @NewestProtocolVersion
  private String newestProtocolVersion(RustStateComputer rustStateComputer) {
    return rustStateComputer.newestProtocolVersion();
  }
}
