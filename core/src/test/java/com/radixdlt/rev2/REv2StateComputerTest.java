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

package com.radixdlt.rev2;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.ImmutableList;
import com.google.inject.*;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.genesis.RawGenesisDataWithHash;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.*;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.mempool.MempoolRelayDispatcher;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.rev2.modules.REv2LedgerInitializerModule;
import com.radixdlt.rev2.modules.REv2LedgerInitializerToken;
import com.radixdlt.rev2.modules.REv2LedgerRecoveryModule;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.statecomputer.commit.ActiveValidatorInfo;
import com.radixdlt.statecomputer.commit.LedgerHeader;
import com.radixdlt.statemanager.DatabaseFlags;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public class REv2StateComputerTest {

  private static final BFTValidatorId ONLY_VALIDATOR_ID = BFTValidatorId.random();

  private Injector createInjector() {
    return Guice.createInjector(
        new CryptoModule(),
        REv2StateManagerModule.create(
            10,
            10 * 1024 * 1024,
            50 * 1024 * 1024,
            REv2StateManagerModule.DatabaseType.IN_MEMORY,
            new DatabaseFlags(false, false),
            Option.none()),
        new REv2LedgerInitializerModule(
            RawGenesisDataWithHash.fromGenesisData(
                GenesisBuilder.createGenesisWithValidatorsAndXrdBalances(
                    ImmutableList.of(ONLY_VALIDATOR_ID.getKey()),
                    Decimal.of(1),
                    Address.virtualAccountAddress(ONLY_VALIDATOR_ID.getKey()),
                    Map.of(),
                    GenesisConsensusManagerConfig.Builder.testDefaults()))),
        new REv2LedgerRecoveryModule(),
        new AbstractModule() {
          @Override
          protected void configure() {
            bind(Network.class).toInstance(Network.INTEGRATIONTESTNET);
            bind(LedgerAccumulator.class).to(SimpleLedgerAccumulatorAndVerifier.class);
            bind(new TypeLiteral<EventDispatcher<LedgerUpdate>>() {}).toInstance(e -> {});
            bind(new TypeLiteral<EventDispatcher<MempoolAddSuccess>>() {}).toInstance(e -> {});
            bind(new TypeLiteral<MempoolRelayDispatcher<RawNotarizedTransaction>>() {})
                .toInstance(e -> {});
            bind(new TypeLiteral<EventDispatcher<ConsensusByzantineEvent>>() {})
                .toInstance(e -> {});
            bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
            bind(NodeId.class)
                .annotatedWith(Self.class)
                .toInstance(NodeId.fromPublicKey(ONLY_VALIDATOR_ID.getKey()));
          }

          @Provides
          public ProposerElection proposerElection() {
            return round -> ONLY_VALIDATOR_ID;
          }
        });
  }

  @Test
  public void test_valid_rev2_transaction_passes() {
    // Arrange
    var injector = createInjector();
    var stateComputer = injector.getInstance(StateComputerLedger.StateComputer.class);
    var unused = injector.getInstance(REv2LedgerInitializerToken.class);
    var genesisEpochProof =
        injector.getInstance(REv2TransactionAndProofStore.class).getFirstEpochProof().orElseThrow();
    var accumulatorHash = genesisEpochProof.ledgerHeader().accumulatorState().accumulatorHash();
    var validTransaction = TransactionBuilder.forTests().prepare().raw();

    // Act
    var roundDetails =
        new RoundDetails(
            1,
            1,
            false,
            0,
            getValidatorFromEpochHeader(genesisEpochProof.ledgerHeader(), 0),
            1000,
            1000);
    var result =
        stateComputer.prepare(accumulatorHash, List.of(), List.of(validTransaction), roundDetails);

    // Assert
    assertThat(result.getRejectedTransactionCount()).isZero();
  }

  @Test
  public void test_invalid_rev2_transaction_fails() {
    // Arrange
    var injector = createInjector();
    var stateComputer = injector.getInstance(StateComputerLedger.StateComputer.class);
    var unused = injector.getInstance(REv2LedgerInitializerToken.class);
    var genesisEpochProof =
        injector.getInstance(REv2TransactionAndProofStore.class).getFirstEpochProof().orElseThrow();
    var accumulatorHash = genesisEpochProof.ledgerHeader().accumulatorState().accumulatorHash();
    var invalidTransaction = RawNotarizedTransaction.create(new byte[1]);

    // Act
    var roundDetails =
        new RoundDetails(
            1,
            1,
            false,
            0,
            getValidatorFromEpochHeader(genesisEpochProof.ledgerHeader(), 0),
            1000,
            1000);
    var result =
        stateComputer.prepare(
            accumulatorHash, List.of(), List.of(invalidTransaction), roundDetails);

    // Assert
    assertThat(result.getRejectedTransactionCount()).isEqualTo(1);
  }

  private BFTValidatorId getValidatorFromEpochHeader(LedgerHeader epochHeader, int validatorIndex) {
    final var validator =
        epochHeader.nextEpoch().unwrap().validators().stream()
            .sorted(Comparator.comparing(ActiveValidatorInfo::stake).reversed())
            .skip(validatorIndex)
            .findFirst()
            .orElseThrow(() -> new IllegalStateException("some validator expected"));
    return BFTValidatorId.create(validator.address(), validator.key());
  }
}
