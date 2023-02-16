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

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Scopes;
import com.google.inject.Singleton;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.vertexstore.PersistentVertexStore;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.Hasher;
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
import com.radixdlt.recovery.VertexStoreRecovery;
import com.radixdlt.rev2.*;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statemanager.*;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.RawNotarizedTransaction;

public final class REv2StateManagerModule extends AbstractModule {
  private final int networkId;
  private final int maxNumTransactionsPerProposal;
  private final int maxProposalTotalTxnsPayloadSize;
  private final int maxUncommittedUserTransactionsTotalPayloadSize;
  private final REv2DatabaseConfig databaseConfig;
  private final Option<RustMempoolConfig> mempoolConfig;
  private final boolean testing;
  private final boolean debugLogging;

  private REv2StateManagerModule(
      int networkId,
      int maxNumTransactionsPerProposal,
      int maxProposalTotalTxnsPayloadSize,
      int maxUncommittedUserTransactionsTotalPayloadSize,
      boolean prefixDatabase,
      REv2DatabaseConfig databaseConfig,
      Option<RustMempoolConfig> mempoolConfig,
      boolean debugLogging) {
    this.networkId = networkId;
    this.maxNumTransactionsPerProposal = maxNumTransactionsPerProposal;
    this.maxProposalTotalTxnsPayloadSize = maxProposalTotalTxnsPayloadSize;
    this.maxUncommittedUserTransactionsTotalPayloadSize =
        maxUncommittedUserTransactionsTotalPayloadSize;
    this.testing = prefixDatabase;
    this.databaseConfig = databaseConfig;
    this.mempoolConfig = mempoolConfig;
    this.debugLogging = debugLogging;
  }

  public static REv2StateManagerModule create(
      int networkId,
      int maxNumTransactionsPerProposal,
      int maxProposalTotalTxnsPayloadSize,
      int maxUncommittedUserTransactionsTotalPayloadSize,
      REv2DatabaseConfig databaseConfig,
      Option<RustMempoolConfig> mempoolConfig) {
    return new REv2StateManagerModule(
        networkId,
        maxNumTransactionsPerProposal,
        maxProposalTotalTxnsPayloadSize,
        maxUncommittedUserTransactionsTotalPayloadSize,
        false,
        databaseConfig,
        mempoolConfig,
        false);
  }

  public static REv2StateManagerModule createForTesting(
      int networkId,
      int maxNumTransactionsPerProposal,
      int maxProposalTotalTxnsPayloadSize,
      REv2DatabaseConfig databaseConfig,
      Option<RustMempoolConfig> mempoolConfig,
      boolean debugLogging) {
    return new REv2StateManagerModule(
        networkId,
        maxNumTransactionsPerProposal,
        maxProposalTotalTxnsPayloadSize,
        maxProposalTotalTxnsPayloadSize * 5,
        true,
        databaseConfig,
        mempoolConfig,
        debugLogging);
  }

  @Override
  public void configure() {
    if (testing && databaseConfig instanceof REv2DatabaseConfig.RocksDB rocksDB) {
      install(
          new AbstractModule() {
            @Provides
            @Singleton
            private StateManager stateManager(@Self ECDSASecp256k1PublicKey key) {
              var network = Network.ofId(networkId).orElseThrow();
              final REv2DatabaseConfig databaseConfigToUse;
              var databasePath = rocksDB.databasePath() + key.toString();
              databaseConfigToUse = REv2DatabaseConfig.rocksDB(databasePath);
              return StateManager.createAndInitialize(
                  new StateManagerConfig(
                      NetworkDefinition.from(network),
                      mempoolConfig,
                      databaseConfigToUse,
                      getLoggingConfig()));
            }
          });
    } else {
      install(
          new AbstractModule() {
            @Provides
            @Singleton
            private StateManager stateManager() {
              var network = Network.ofId(networkId).orElseThrow();
              return StateManager.createAndInitialize(
                  new StateManagerConfig(
                      NetworkDefinition.from(network),
                      mempoolConfig,
                      databaseConfig,
                      getLoggingConfig()));
            }
          });
    }

    if (!REv2DatabaseConfig.isNone(this.databaseConfig)) {
      bind(StateComputerLedger.StateComputer.class).to(REv2StateComputer.class);
      bind(REv2TransactionsAndProofReader.class).in(Scopes.SINGLETON);
      bind(TransactionsAndProofReader.class).to(REv2TransactionsAndProofReader.class);
      install(
          new AbstractModule() {
            @Provides
            @Singleton
            REv2StateComputer rEv2StateComputer(
                RustStateComputer stateComputer,
                EventDispatcher<LedgerUpdate> ledgerUpdateEventDispatcher,
                Hasher hasher,
                EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher,
                EventDispatcher<ConsensusByzantineEvent> byzantineEventEventDispatcher,
                Serialization serialization,
                Metrics metrics) {
              return new REv2StateComputer(
                  stateComputer,
                  maxNumTransactionsPerProposal,
                  maxProposalTotalTxnsPayloadSize,
                  maxUncommittedUserTransactionsTotalPayloadSize,
                  hasher,
                  ledgerUpdateEventDispatcher,
                  mempoolAddSuccessEventDispatcher,
                  byzantineEventEventDispatcher,
                  serialization,
                  metrics);
            }

            @Provides
            REv2TransactionAndProofStore transactionAndProofStore(RustStateComputer stateComputer) {
              return stateComputer.getTransactionAndProofStore();
            }

            @Provides
            VertexStoreRecovery rEv2VertexStoreRecovery(RustStateComputer stateComputer) {
              return stateComputer.getVertexStoreRecovery();
            }

            @Provides
            REv2StateReader stateReader(RustStateComputer stateComputer) {
              return new REv2StateReader() {
                @Override
                public Decimal getComponentXrdAmount(ComponentAddress componentAddress) {
                  return stateComputer.getComponentXrdAmount(componentAddress);
                }

                @Override
                public ValidatorInfo getValidatorInfo(ComponentAddress systemAddress) {
                  return stateComputer.getValidatorInfo(systemAddress);
                }

                @Override
                public long getEpoch() {
                  return stateComputer
                      .getEpoch()
                      .toNonNegativeLong()
                      .unwrap(() -> new IllegalStateException("Epoch is not non-negative"));
                }
              };
            }

            @Provides
            PersistentVertexStore vertexStore(
                RustStateComputer stateComputer, Metrics metrics, Serialization serialization) {
              return s -> {
                metrics.misc().vertexStoreSaved().inc();
                var vertexStoreBytes =
                    serialization.toDson(s.toSerialized(), DsonOutput.Output.ALL);
                stateComputer.saveVertexStore(vertexStoreBytes);
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
    }

    if (mempoolConfig.isPresent()) {
      install(
          new AbstractModule() {
            @Provides
            private MempoolReader<RawNotarizedTransaction> mempoolReader(
                RustStateComputer stateComputer) {
              return stateComputer.getMempoolReader();
            }

            @Provides
            private MempoolInserter<RawNotarizedTransaction, RawNotarizedTransaction>
                mempoolInserter(RustStateComputer stateComputer) {
              return stateComputer.getMempoolInserter();
            }
          });
    }
  }

  @ProvidesIntoSet
  NodeAutoCloseable closeable(StateManager stateManager) {
    return stateManager::shutdown;
  }

  public LoggingConfig getLoggingConfig() {
    return debugLogging ? LoggingConfig.getDebug() : LoggingConfig.getDefault();
  }

  @Provides
  @Singleton
  private RustStateComputer rustStateComputer(Metrics metrics, StateManager stateManager) {
    return new RustStateComputer(metrics, stateManager);
  }
}
