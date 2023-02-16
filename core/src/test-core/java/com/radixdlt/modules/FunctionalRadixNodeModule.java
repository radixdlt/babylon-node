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
import com.google.inject.Provides;
import com.google.inject.multibindings.Multibinder;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.epoch.EpochsConsensusModule;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.NoEpochsConsensusModule;
import com.radixdlt.environment.NoEpochsSyncModule;
import com.radixdlt.environment.NodeAutoCloseable;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.ledger.MockedLedgerModule;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.mempool.MempoolReceiverModule;
import com.radixdlt.mempool.MempoolRelayerModule;
import com.radixdlt.modules.StateComputerConfig.*;
import com.radixdlt.rev2.modules.*;
import com.radixdlt.statecomputer.MockedMempoolStateComputerModule;
import com.radixdlt.statecomputer.MockedStateComputerModule;
import com.radixdlt.statecomputer.MockedStateComputerWithEpochsModule;
import com.radixdlt.statecomputer.REv2StatelessComputerModule;
import com.radixdlt.statecomputer.RandomTransactionGenerator;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.store.InMemoryCommittedReaderModule;
import com.radixdlt.store.LastEpochProof;
import com.radixdlt.sync.SyncRelayConfig;

/** Manages the functional components of a node */
public final class FunctionalRadixNodeModule extends AbstractModule {
  public static final class SafetyRecoveryConfig {
    private final Option<String> root;

    private SafetyRecoveryConfig(Option<String> root) {
      this.root = root;
    }

    public static SafetyRecoveryConfig mocked() {
      return new SafetyRecoveryConfig(Option.none());
    }

    public static SafetyRecoveryConfig berkeleyStore(String root) {
      return new SafetyRecoveryConfig(Option.some(root));
    }

    public Module asModule() {
      return root.fold(BerkeleySafetyStoreModule::new, MockedSafetyStoreModule::new);
    }
  }

  public static final class ConsensusConfig {
    private final int bftSyncPatienceMillis;
    private final long pacemakerBaseTimeoutMs;
    private final double pacemakerBackoffRate;
    private final long additionalRoundTimeIfProposalReceivedMs;

    private ConsensusConfig(
        int bftSyncPatienceMillis,
        long pacemakerBaseTimeoutMs,
        double pacemakerBackoffRate,
        long additionalRoundTimeIfProposalReceivedMs) {
      this.bftSyncPatienceMillis = bftSyncPatienceMillis;
      this.pacemakerBaseTimeoutMs = pacemakerBaseTimeoutMs;
      this.pacemakerBackoffRate = pacemakerBackoffRate;
      this.additionalRoundTimeIfProposalReceivedMs = additionalRoundTimeIfProposalReceivedMs;
    }

    public static ConsensusConfig of(
        int bftSyncPatienceMillis,
        long pacemakerBaseTimeoutMs,
        double pacemakerBackoffRate,
        long additionalRoundTimeIfProposalReceivedMs) {
      return new ConsensusConfig(
          bftSyncPatienceMillis,
          pacemakerBaseTimeoutMs,
          pacemakerBackoffRate,
          additionalRoundTimeIfProposalReceivedMs);
    }

    public static ConsensusConfig of(long pacemakerBaseTimeoutMs) {
      return ConsensusConfig.of(
          pacemakerBaseTimeoutMs,
          pacemakerBaseTimeoutMs /* double the timeout if proposal was received */);
    }

    public static ConsensusConfig of(
        long pacemakerBaseTimeoutMs, long additionalRoundTimeIfProposalReceivedMs) {
      return new ConsensusConfig(
          200, pacemakerBaseTimeoutMs, 2.0, additionalRoundTimeIfProposalReceivedMs);
    }

    public static ConsensusConfig of() {
      final var pacemakerBaseTimeoutMs = 12 * 50;
      return new ConsensusConfig(
          200,
          pacemakerBaseTimeoutMs,
          2.0,
          pacemakerBaseTimeoutMs /* double the timeout if proposal was received */);
    }

    private AbstractModule asModule() {
      return new AbstractModule() {
        @Override
        protected void configure() {
          bindConstant().annotatedWith(BFTSyncPatienceMillis.class).to(bftSyncPatienceMillis);
          bindConstant().annotatedWith(PacemakerBaseTimeoutMs.class).to(pacemakerBaseTimeoutMs);
          bindConstant().annotatedWith(PacemakerBackoffRate.class).to(pacemakerBackoffRate);
          bindConstant()
              .annotatedWith(AdditionalRoundTimeIfProposalReceivedMs.class)
              .to(additionalRoundTimeIfProposalReceivedMs);
          bindConstant().annotatedWith(PacemakerMaxExponent.class).to(0);
        }
      };
    }
  }

  public sealed interface LedgerConfig {
    static LedgerConfig mocked(int numValidators) {
      return new MockedLedgerConfig(numValidators);
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

    default boolean hasSyncRelay() {
      if (this instanceof StateComputerLedgerConfig c) {
        return c.syncConfig instanceof SyncConfig.Relayed;
      }
      return false;
    }

    default boolean isREV2() {
      if (this instanceof StateComputerLedgerConfig c) {
        return c.config instanceof REv2StateComputerConfig;
      }
      return false;
    }
  }

  public record MockedLedgerConfig(int numValidators) implements LedgerConfig {}

  public record StateComputerLedgerConfig(StateComputerConfig config, SyncConfig syncConfig)
      implements LedgerConfig {}

  public sealed interface SyncConfig {
    record Relayed(SyncRelayConfig config) implements SyncConfig {}

    record Mocked() implements SyncConfig {}

    record None() implements SyncConfig {}
  }

  private final boolean epochs;
  private final SafetyRecoveryConfig safetyRecoveryConfig;
  private final ConsensusConfig consensusConfig;
  private final LedgerConfig ledgerConfig;

  // FIXME: This is required for now for shared syncing, remove after refactor
  private final Module mockedSyncServiceModule = new MockedSyncServiceModule();

  public FunctionalRadixNodeModule(
      boolean epochs,
      SafetyRecoveryConfig safetyRecoveryConfig,
      ConsensusConfig consensusConfig,
      LedgerConfig ledgerConfig) {
    this.epochs = epochs;
    this.safetyRecoveryConfig = safetyRecoveryConfig;
    this.consensusConfig = consensusConfig;
    this.ledgerConfig = ledgerConfig;
  }

  public FunctionalRadixNodeModule(
      ConsensusConfig consensusConfig,
      StateComputerConfig stateComputerConfig,
      SyncRelayConfig syncRelayConfig) {
    this(
        true,
        SafetyRecoveryConfig.mocked(),
        consensusConfig,
        LedgerConfig.stateComputerWithSyncRelay(stateComputerConfig, syncRelayConfig));
  }

  public static FunctionalRadixNodeModule justLedgerWithNumValidators(int numValidators) {
    return new FunctionalRadixNodeModule(
        false,
        SafetyRecoveryConfig.mocked(),
        ConsensusConfig.of(),
        LedgerConfig.stateComputerNoSync(
            StateComputerConfig.mockedNoEpochs(
                numValidators, new MockedMempoolConfig.NoMempool())));
  }

  public boolean supportsEpochs() {
    return epochs;
  }

  public boolean supportsREv2() {
    return this.ledgerConfig.isREV2();
  }

  public boolean supportsSync() {
    return this.ledgerConfig.hasSyncRelay();
  }

  @Override
  public void configure() {
    install(new DispatcherModule());

    // Consensus
    install(consensusConfig.asModule());
    install(new ConsensusModule());
    install(safetyRecoveryConfig.asModule());
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
        install(new MockedNoEpochsConsensusRecoveryModule(config.numValidators));
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
                install(new MempoolRelayerModule(10000));
                install(new MockedMempoolStateComputerModule(relayed.mempoolSize()));
              }
            }

            switch (c) {
              case MockedStateComputerConfigNoEpochs noEpochs -> {
                install(new MockedNoEpochsConsensusRecoveryModule(noEpochs.numValidators()));
              }
              case MockedStateComputerConfigWithEpochs withEpochs -> {
                install(
                    new MockedEpochsConsensusRecoveryModule(
                        withEpochs.epochMaxRound(),
                        withEpochs.mapping(),
                        withEpochs.preGenesisAccumulatorHash(),
                        withEpochs.preGenesisStateHash()));
              }
            }
          }
          case REv2StateComputerConfig rev2Config -> {
            var initialAccumulatorState = new AccumulatorState(0, HashUtils.zero256());

            if (REv2DatabaseConfig.isNone(rev2Config.databaseConfig())) {
              install(new REv2StatelessComputerModule());
              install(new MockedLedgerRecoveryModule());
              install(
                  new AbstractModule() {
                    @Provides
                    private RoundUpdate initialRoundUpdate(
                        BFTConfiguration configuration, ProposerElection proposerElection) {
                      var highQC = configuration.getVertexStoreState().getHighQC();
                      var round = highQC.getHighestRound().next();
                      var leader = proposerElection.getProposer(round);
                      var nextLeader = proposerElection.getProposer(round.next());

                      return RoundUpdate.create(round, highQC, leader, nextLeader);
                    }

                    @Provides
                    private BFTConfiguration configuration(
                        @LastEpochProof LedgerProof proof,
                        BFTValidatorSet validatorSet,
                        Hasher hasher) {
                      var genesisVertex =
                          Vertex.createInitialEpochVertex(
                                  LedgerHeader.genesis(
                                      initialAccumulatorState,
                                      HashUtils.zero256(),
                                      validatorSet,
                                      0,
                                      0))
                              .withId(hasher);
                      var nextLedgerHeader =
                          LedgerHeader.create(
                              proof.getNextEpoch().orElseThrow().getEpoch(),
                              Round.genesis(),
                              proof.getAccumulatorState(),
                              proof.getStateHash(),
                              proof.consensusParentRoundTimestamp(),
                              proof.proposerTimestamp());
                      var initialEpochQC =
                          QuorumCertificate.createInitialEpochQC(genesisVertex, nextLedgerHeader);
                      var proposerElection = new WeightedRotatingLeaders(validatorSet);
                      return new BFTConfiguration(
                          proposerElection,
                          validatorSet,
                          VertexStoreState.create(
                              HighQC.ofInitialEpochQc(initialEpochQC), genesisVertex, hasher));
                    }
                  });
            } else {
              install(new REv2LedgerRecoveryModule(initialAccumulatorState, rev2Config.genesis()));
              install(new REv2ConsensusRecoveryModule());
            }

            switch (rev2Config.proposerConfig()) {
              case REV2ProposerConfig.Generated generated -> {
                bind(ProposalGenerator.class).toInstance(generated.generator());
                install(
                    REv2StateManagerModule.createForTesting(
                        rev2Config.networkId(),
                        0,
                        0,
                        rev2Config.databaseConfig(),
                        Option.none(),
                        rev2Config.debugLogging()));
              }
              case REV2ProposerConfig.Mempool mempool -> {
                install(new MempoolRelayerModule(10000));
                install(new MempoolReceiverModule());
                install(mempool.relayConfig().asModule());
                install(
                    REv2StateManagerModule.createForTesting(
                        rev2Config.networkId(),
                        mempool.maxNumTransactionsPerProposal(),
                        mempool.maxProposalTotalTxnsPayloadSize(),
                        rev2Config.databaseConfig(),
                        Option.some(mempool.mempoolConfig()),
                        rev2Config.debugLogging()));
              }
            }
          }
        }
      }
    }
  }
}
