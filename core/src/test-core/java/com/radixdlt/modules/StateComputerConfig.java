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

import com.radixdlt.consensus.EpochNodeWeightMapping;
import com.radixdlt.consensus.LedgerHashes;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.harness.simulation.application.TransactionGenerator;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.statemanager.DatabaseFlags;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/** Configuration options for the state computer */
public sealed interface StateComputerConfig {
  static StateComputerConfig mockedWithEpochs(
      Round epochMaxRound, EpochNodeWeightMapping mapping, MockedMempoolConfig mempoolType) {
    return mockedWithEpochs(epochMaxRound, mapping, LedgerHashes.zero(), mempoolType);
  }

  static StateComputerConfig mockedWithEpochs(
      Round epochMaxRound,
      EpochNodeWeightMapping mapping,
      LedgerHashes preGenesisLedgerHashes,
      MockedMempoolConfig mempoolType) {
    return mockedWithEpochs(
        epochMaxRound,
        mapping,
        preGenesisLedgerHashes,
        mempoolType,
        ProposerElectionMode.WITH_INITIAL_ROUNDS_ITERATION);
  }

  static StateComputerConfig mockedWithEpochs(
      Round epochMaxRound,
      EpochNodeWeightMapping mapping,
      LedgerHashes preGenesisLedgerHashes,
      MockedMempoolConfig mempoolType,
      ProposerElectionMode proposerElectionMode) {
    return new MockedStateComputerConfigWithEpochs(
        epochMaxRound, mapping, preGenesisLedgerHashes, mempoolType, proposerElectionMode);
  }

  static StateComputerConfig mockedNoEpochs(int numValidators, MockedMempoolConfig mempoolType) {
    return new MockedStateComputerConfigNoEpochs(numValidators, mempoolType);
  }

  static StateComputerConfig rev2(
      int networkId,
      GenesisData genesis,
      REv2StateManagerModule.DatabaseType databaseType,
      DatabaseFlags databaseFlags,
      REV2ProposerConfig proposerConfig,
      boolean debugLogging) {
    return new REv2StateComputerConfig(
        networkId, genesis, databaseType, databaseFlags, proposerConfig, debugLogging);
  }

  static StateComputerConfig rev2(
      int networkId,
      GenesisData genesis,
      REv2StateManagerModule.DatabaseType databaseType,
      DatabaseFlags databaseFlags,
      REV2ProposerConfig proposerConfig) {
    return new REv2StateComputerConfig(
        networkId, genesis, databaseType, databaseFlags, proposerConfig, false);
  }

  static StateComputerConfig rev2(
      int networkId,
      GenesisData genesis,
      REv2StateManagerModule.DatabaseType databaseType,
      REV2ProposerConfig proposerConfig) {
    return new REv2StateComputerConfig(
        networkId, genesis, databaseType, new DatabaseFlags(true, false), proposerConfig, false);
  }

  sealed interface MockedMempoolConfig {
    static MockedMempoolConfig noMempool() {
      return new NoMempool();
    }

    record NoMempool() implements MockedMempoolConfig {}

    record LocalOnly(int mempoolSize) implements MockedMempoolConfig {}

    record Relayed(int mempoolSize) implements MockedMempoolConfig {}
  }

  sealed interface MockedStateComputerConfig extends StateComputerConfig {
    MockedMempoolConfig mempoolConfig();
  }

  record MockedStateComputerConfigWithEpochs(
      Round epochMaxRound,
      EpochNodeWeightMapping mapping,
      LedgerHashes preGenesisLedgerHashes,
      MockedMempoolConfig mempoolType,
      ProposerElectionMode proposerElectionMode)
      implements MockedStateComputerConfig {
    @Override
    public MockedMempoolConfig mempoolConfig() {
      return mempoolType;
    }
  }

  /** A choice of {@link com.radixdlt.consensus.liveness.ProposerElection} implementation. */
  enum ProposerElectionMode {
    /** Use the {@link com.radixdlt.consensus.liveness.WeightedRotatingLeaders} only. */
    ONLY_WEIGHTED_BY_STAKE,

    /** Use the newer {@link com.radixdlt.consensus.liveness.ProposerElections#defaultRotation}. */
    WITH_INITIAL_ROUNDS_ITERATION,
  }

  record MockedStateComputerConfigNoEpochs(int numValidators, MockedMempoolConfig mempoolType)
      implements MockedStateComputerConfig {
    @Override
    public MockedMempoolConfig mempoolConfig() {
      return mempoolType;
    }
  }

  record REv2StateComputerConfig(
      int networkId,
      GenesisData genesis,
      REv2StateManagerModule.DatabaseType databaseType,
      DatabaseFlags databaseFlags,
      REV2ProposerConfig proposerConfig,
      boolean debugLogging)
      implements StateComputerConfig {}

  sealed interface REV2ProposerConfig {
    static REV2ProposerConfig transactionGenerator(
        TransactionGenerator<RawNotarizedTransaction> transactionGenerator, long count) {
      return new Generated(
          (round, prepared) ->
              Stream.generate(transactionGenerator::nextTransaction)
                  .limit(count)
                  .collect(Collectors.toList()));
    }

    static REV2ProposerConfig transactionGenerator(ProposalGenerator proposalGenerator) {
      return new Generated(proposalGenerator);
    }

    static REV2ProposerConfig noUserTransactions() {
      return new Generated((round, prepared) -> List.of());
    }

    static REV2ProposerConfig mempool(
        int maxNumTransactionsPerProposal,
        int maxProposalTotalTxnsPayloadSize,
        RustMempoolConfig mempoolConfig,
        MempoolRelayConfig config) {
      return new Mempool(
          maxNumTransactionsPerProposal, maxProposalTotalTxnsPayloadSize, mempoolConfig, config);
    }

    static REV2ProposerConfig zeroMempool() {
      return new Mempool(0, 0, new RustMempoolConfig(0, 0), MempoolRelayConfig.of());
    }

    static REV2ProposerConfig singleTransactionMempool() {
      return new Mempool(
          1, 1024 * 1024, new RustMempoolConfig(1024 * 1024, 1), new MempoolRelayConfig(0, 100));
    }

    static REV2ProposerConfig defaultMempool() {
      return defaultMempool(MempoolRelayConfig.of());
    }

    static REV2ProposerConfig defaultMempool(MempoolRelayConfig relayConfig) {
      return new Mempool(
          10, 10 * 1024 * 1024, new RustMempoolConfig(100 * 1024 * 1024, 100), relayConfig);
    }

    record Generated(ProposalGenerator generator) implements REV2ProposerConfig {}

    record Mempool(
        int maxNumTransactionsPerProposal,
        int maxProposalTotalTxnsPayloadSize,
        RustMempoolConfig mempoolConfig,
        MempoolRelayConfig relayConfig)
        implements REV2ProposerConfig {}
  }
}
