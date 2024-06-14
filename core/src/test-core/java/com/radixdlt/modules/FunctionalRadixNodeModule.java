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
import com.google.inject.Module;
import com.google.inject.multibindings.Multibinder;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.ProposalLimitsConfig;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochsConsensusModule;
import com.radixdlt.consensus.liveness.PacemakerTimeoutCalculatorConfig;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.consensus.vertexstore.VertexStoreConfig;
import com.radixdlt.environment.NoEpochsConsensusModule;
import com.radixdlt.environment.NoEpochsSyncModule;
import com.radixdlt.environment.NodeAutoCloseable;
import com.radixdlt.environment.ScenariosExecutionConfig;
import com.radixdlt.genesis.RawGenesisDataWithHash;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.MockedLedgerModule;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.mempool.*;
import com.radixdlt.modules.StateComputerConfig.*;
import com.radixdlt.rev2.modules.*;
import com.radixdlt.statecomputer.MockedMempoolStateComputerModule;
import com.radixdlt.statecomputer.MockedStateComputerModule;
import com.radixdlt.statecomputer.MockedStateComputerWithEpochsModule;
import com.radixdlt.statecomputer.RandomTransactionGenerator;
import com.radixdlt.store.InMemoryCommittedReaderModule;
import com.radixdlt.sync.SyncRelayConfig;
import java.io.File;
import java.time.Duration;
import org.junit.rules.TemporaryFolder;

/** Manages the functional components of a node */
public final class FunctionalRadixNodeModule extends AbstractModule {
  public sealed interface NodeStorageConfig {
    /**
     * No storage configured. Can only be used if no other modules depend on @NodeStorageLocation.
     */
    record None() implements NodeStorageConfig {}

    /**
     * For tests that use a real storage. Either BerkeleyDB (on the Java side; I think it's just
     * BerkeleySafetyStore) or RocksRb (on the Rust side).
     */
    record FileStorage(File folder) implements NodeStorageConfig {}

    static NodeStorageConfig none() {
      return new None();
    }

    static NodeStorageConfig tempFolder(TemporaryFolder tempFolder) {
      return file(tempFolder.getRoot());
    }

    static NodeStorageConfig file(File folder) {
      return new FileStorage(folder);
    }
  }

  public enum SafetyRecoveryConfig {
    MOCKED,
    BERKELEY_DB,
  }

  public static final class ConsensusConfig {
    private final int bftSyncPatienceMillis;
    private final long pacemakerBaseTimeoutMs;
    private final double pacemakerBackoffRate;
    private final long additionalRoundTimeIfProposalReceivedMs;
    private final long timeoutQuorumResolutionDelayMs;
    private final VertexStoreConfig vertexStoreConfig;

    public ConsensusConfig(
        int bftSyncPatienceMillis,
        long pacemakerBaseTimeoutMs,
        double pacemakerBackoffRate,
        long additionalRoundTimeIfProposalReceivedMs,
        long timeoutQuorumResolutionDelayMs,
        VertexStoreConfig vertexStoreConfig) {
      this.bftSyncPatienceMillis = bftSyncPatienceMillis;
      this.pacemakerBaseTimeoutMs = pacemakerBaseTimeoutMs;
      this.pacemakerBackoffRate = pacemakerBackoffRate;
      this.additionalRoundTimeIfProposalReceivedMs = additionalRoundTimeIfProposalReceivedMs;
      this.timeoutQuorumResolutionDelayMs = timeoutQuorumResolutionDelayMs;
      this.vertexStoreConfig = vertexStoreConfig;
    }

    public static ConsensusConfig of(
        int bftSyncPatienceMillis,
        long pacemakerBaseTimeoutMs,
        double pacemakerBackoffRate,
        long additionalRoundTimeIfProposalReceivedMs,
        long timeoutQuorumResolutionDelayMs) {
      return new ConsensusConfig(
          bftSyncPatienceMillis,
          pacemakerBaseTimeoutMs,
          pacemakerBackoffRate,
          additionalRoundTimeIfProposalReceivedMs,
          timeoutQuorumResolutionDelayMs,
          VertexStoreConfig.testingDefault());
    }

    public static ConsensusConfig of(long pacemakerBaseTimeoutMs) {
      return ConsensusConfig.of(
          pacemakerBaseTimeoutMs,
          pacemakerBaseTimeoutMs /* double the timeout if proposal was received */);
    }

    public static ConsensusConfig of(
        long pacemakerBaseTimeoutMs, long additionalRoundTimeIfProposalReceivedMs) {
      return new ConsensusConfig(
          200,
          pacemakerBaseTimeoutMs,
          2.0,
          additionalRoundTimeIfProposalReceivedMs,
          pacemakerBaseTimeoutMs / 2,
          VertexStoreConfig.testingDefault());
    }

    public static ConsensusConfig of() {
      final var pacemakerBaseTimeoutMs = 12 * 50;
      return new ConsensusConfig(
          200,
          pacemakerBaseTimeoutMs,
          2.0,
          pacemakerBaseTimeoutMs /* double the timeout if proposal was received */,
          2000,
          VertexStoreConfig.testingDefault());
    }

    public AbstractModule asModule() {
      return new AbstractModule() {
        @Override
        protected void configure() {
          bindConstant().annotatedWith(BFTSyncPatienceMillis.class).to(bftSyncPatienceMillis);
          bind(PacemakerTimeoutCalculatorConfig.class)
              .toInstance(
                  new PacemakerTimeoutCalculatorConfig(
                      pacemakerBaseTimeoutMs,
                      pacemakerBackoffRate,
                      0,
                      additionalRoundTimeIfProposalReceivedMs,
                      0,
                      1));
          bindConstant()
              .annotatedWith(TimeoutQuorumResolutionDelayMs.class)
              .to(timeoutQuorumResolutionDelayMs);
          bind(VertexStoreConfig.class).toInstance(vertexStoreConfig);
        }
      };
    }
  }

  public sealed interface LedgerConfig {
    static LedgerConfig mocked(int numValidators) {
      return new MockedLedgerConfig(numValidators, ProposerElectionMode.WITH_DEFAULT_ROTATION);
    }

    static LedgerConfig mocked(int numValidators, ProposerElectionMode proposerElectionMode) {
      return new MockedLedgerConfig(numValidators, proposerElectionMode);
    }

    static LedgerConfig stateComputerNoSync(StateComputerConfig stateComputerConfig) {
      return new StateComputerLedgerConfig(stateComputerConfig, new SyncConfig.None());
    }

    static LedgerConfig stateComputerMockedSync(StateComputerConfig stateComputerConfig) {
      return new StateComputerLedgerConfig(stateComputerConfig, new SyncConfig.Mocked());
    }

    static LedgerConfig stateComputerWithSyncRelay(
        StateComputerConfig stateComputerConfig, SyncRelayConfig syncRelayConfig) {
      return new StateComputerLedgerConfig(
          stateComputerConfig, new SyncConfig.Relayed(syncRelayConfig));
    }

    default boolean isREV2() {
      if (this instanceof StateComputerLedgerConfig c) {
        return c.config instanceof REv2StateComputerConfig;
      }
      return false;
    }
  }

  public record MockedLedgerConfig(int numValidators, ProposerElectionMode proposerElectionMode)
      implements LedgerConfig {}

  public record StateComputerLedgerConfig(StateComputerConfig config, SyncConfig syncConfig)
      implements LedgerConfig {}

  public sealed interface SyncConfig {
    record Relayed(SyncRelayConfig config) implements SyncConfig {}

    record Mocked() implements SyncConfig {}

    record None() implements SyncConfig {}
  }

  private final NodeStorageConfig nodeStorageConfig;
  private final boolean epochs;
  private final SafetyRecoveryConfig safetyRecoveryConfig;
  private final ConsensusConfig consensusConfig;
  private final LedgerConfig ledgerConfig;

  // FIXME: This is required for now for shared syncing, remove after refactor
  private final Module mockedSyncServiceModule = new MockedSyncServiceModule();

  public FunctionalRadixNodeModule(
      NodeStorageConfig nodeStorageConfig,
      boolean epochs,
      SafetyRecoveryConfig safetyRecoveryConfig,
      ConsensusConfig consensusConfig,
      LedgerConfig ledgerConfig) {
    this.nodeStorageConfig = nodeStorageConfig;
    this.epochs = epochs;
    this.safetyRecoveryConfig = safetyRecoveryConfig;
    this.consensusConfig = consensusConfig;
    this.ledgerConfig = ledgerConfig;
  }

  public FunctionalRadixNodeModule(
      NodeStorageConfig nodeStorageConfig,
      ConsensusConfig consensusConfig,
      StateComputerConfig stateComputerConfig,
      SyncRelayConfig syncRelayConfig) {
    this(
        nodeStorageConfig,
        true,
        SafetyRecoveryConfig.MOCKED,
        consensusConfig,
        LedgerConfig.stateComputerWithSyncRelay(stateComputerConfig, syncRelayConfig));
  }

  public static FunctionalRadixNodeModule justLedgerWithNumValidators(int numValidators) {
    return new FunctionalRadixNodeModule(
        NodeStorageConfig.none(),
        false,
        SafetyRecoveryConfig.MOCKED,
        ConsensusConfig.of(),
        LedgerConfig.stateComputerNoSync(StateComputerConfig.mockedNoEpochs(numValidators)));
  }

  public boolean supportsEpochs() {
    return epochs;
  }

  public boolean supportsREv2() {
    return this.ledgerConfig.isREV2();
  }

  @Override
  public void configure() {
    install(new DispatcherModule());

    switch (this.nodeStorageConfig) {
      case NodeStorageConfig.None none -> {}
      case NodeStorageConfig.FileStorage fileStorage -> {
        final var tempFolderPath = fileStorage.folder.getAbsolutePath();
        install(new PrefixedNodeStorageLocationModule(tempFolderPath));
      }
    }

    switch (this.safetyRecoveryConfig) {
      case MOCKED -> install(new MockedSafetyStoreModule());
      case BERKELEY_DB -> install(new BerkeleySafetyStoreModule());
    }

    // Consensus
    install(consensusConfig.asModule());
    if (this.epochs) {
      install(new EpochsConsensusModule());
      install(new EpochsSafetyRecoveryModule());
    } else {
      install(new NoEpochsConsensusModule());
      install(new NoEpochsSafetyRecoveryModule());
    }

    Multibinder.newSetBinder(binder(), NodeAutoCloseable.class);

    // Ledger
    switch (this.ledgerConfig) {
      case MockedLedgerConfig config -> {
        install(new MockedLedgerRecoveryModule());
        install(new MockedLedgerModule());
        install(
            new MockedNoEpochsConsensusRecoveryModule(
                config.numValidators, config.proposerElectionMode));
      }
      case StateComputerLedgerConfig stateComputerLedgerConfig -> {
        install(new LedgerModule());

        // Sync
        switch (stateComputerLedgerConfig.syncConfig) {
          case SyncConfig.Relayed relayed -> {
            install(new SyncServiceModule(relayed.config()));
            if (this.epochs) {
              install(new EpochsSyncModule());
            } else {
              install(new NoEpochsSyncModule());
            }
          }
          case SyncConfig.Mocked ignored -> {
            install(mockedSyncServiceModule);
          }
          case SyncConfig.None ignored -> {}
        }

        // State Computer
        switch (stateComputerLedgerConfig.config) {
          case MockedStateComputerConfig c -> {
            install(new MockedLedgerRecoveryModule());
            install(new InMemoryCommittedReaderModule());

            switch (c.mempoolConfig()) {
              case MockedMempoolConfig.NoMempool ignored -> {
                bind(ProposalGenerator.class).to(RandomTransactionGenerator.class);
                if (!this.epochs) {
                  install(new MockedStateComputerModule());
                } else {
                  install(new MockedStateComputerWithEpochsModule());
                }
              }
              case MockedMempoolConfig.LocalOnly localOnly -> {
                install(new MempoolReceiverModule());
                install(new MockedMempoolStateComputerModule(localOnly.mempoolSize()));
              }
              case MockedMempoolConfig.Relayed relayed -> {
                install(new MempoolReceiverModule());
                install(
                    new MempoolRelayerModule(
                        MempoolRelayerConfig.defaults().withIntervalMs(10000)));
                install(new MempoolReevaluationModule(Duration.ofSeconds(1), 1));
                install(new MockedMempoolStateComputerModule(relayed.mempoolSize()));
              }
            }

            switch (c) {
              case MockedStateComputerConfigNoEpochs noEpochs -> {
                install(
                    new MockedNoEpochsConsensusRecoveryModule(
                        noEpochs.numValidators(), noEpochs.proposerElectionMode()));
              }
              case MockedStateComputerConfigWithEpochs withEpochs -> {
                install(
                    new MockedEpochsConsensusRecoveryModule(
                        withEpochs.epochMaxRound(),
                        withEpochs.mapping(),
                        withEpochs.preGenesisLedgerHashes(),
                        withEpochs.proposerElectionMode()));
              }
            }
          }
          case REv2StateComputerConfig rev2Config -> {
            final var genesisProvider =
                RawGenesisDataWithHash.fromGenesisData(rev2Config.genesis());
            install(new REv2LedgerInitializerModule(genesisProvider));
            install(new REv2LedgerRecoveryModule());
            install(new REv2ConsensusRecoveryModule());

            switch (rev2Config.proposerConfig()) {
              case REV2ProposerConfig.Generated generated -> {
                bind(ProposalGenerator.class).toInstance(generated.generator());
                install(
                    REv2StateManagerModule.createForTesting(
                        ProposalLimitsConfig.testDefaults(),
                        rev2Config.databaseConfig(),
                        Option.none(),
                        rev2Config.debugLogging(),
                        rev2Config.stateTreeGcConfig(),
                        rev2Config.ledgerProofsGcConfig(),
                        rev2Config.ledgerSyncLimitsConfig(),
                        rev2Config.protocolConfig(),
                        rev2Config.noFees(),
                        ScenariosExecutionConfig.ALL));
              }
              case REV2ProposerConfig.Mempool mempool -> {
                install(new MempoolRelayerModule(mempool.mempoolRelayerConfig()));
                install(new MempoolReevaluationModule(Duration.ofSeconds(1), 1));
                install(new MempoolReceiverModule());
                install(mempool.mempoolReceiverConfig().asModule());
                install(
                    REv2StateManagerModule.createForTesting(
                        mempool.proposalLimitsConfig(),
                        rev2Config.databaseConfig(),
                        Option.some(mempool.mempoolConfig()),
                        rev2Config.debugLogging(),
                        rev2Config.stateTreeGcConfig(),
                        rev2Config.ledgerProofsGcConfig(),
                        rev2Config.ledgerSyncLimitsConfig(),
                        rev2Config.protocolConfig(),
                        rev2Config.noFees(),
                        ScenariosExecutionConfig.ALL));
              }
            }
          }
        }
      }
    }
  }
}
