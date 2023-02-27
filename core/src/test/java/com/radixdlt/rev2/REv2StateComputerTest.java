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

import com.google.inject.AbstractModule;
import com.google.inject.Guice;
import com.google.inject.Injector;
import com.google.inject.TypeLiteral;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.*;
import com.radixdlt.ledger.RoundDetails;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.UInt64;
import java.util.List;
import org.junit.Test;

public class REv2StateComputerTest {
  private Injector createInjector() {
    return Guice.createInjector(
        new CryptoModule(),
        REv2StateManagerModule.create(
            Network.INTEGRATIONTESTNET.getId(),
            10,
            10 * 1024 * 1024,
            50 * 1024 * 1024,
            REv2DatabaseConfig.inMemory(),
            Option.none()),
        new AbstractModule() {
          @Override
          protected void configure() {
            bind(LedgerAccumulator.class).to(SimpleLedgerAccumulatorAndVerifier.class);
            bind(new TypeLiteral<EventDispatcher<LedgerUpdate>>() {}).toInstance(e -> {});
            bind(new TypeLiteral<EventDispatcher<MempoolAddSuccess>>() {}).toInstance(e -> {});
            bind(new TypeLiteral<EventDispatcher<ConsensusByzantineEvent>>() {})
                .toInstance(e -> {});
            bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
          }
        });
  }

  private CommittedTransactionsWithProof buildGenesis(LedgerAccumulator accumulator) {
    var initialAccumulatorState = new AccumulatorState(0, HashUtils.zero256());
    var genesis =
        TransactionBuilder.createGenesisWithNumValidators(
            1, Decimal.of(1), UInt64.fromNonNegativeLong(10));
    var accumulatorState =
        accumulator.accumulate(initialAccumulatorState, genesis.getPayloadHash());
    // The accumulator state is computed correctly, but we cannot easily do the same for state hash
    var stateHash = HashUtils.random256();
    var validatorSet =
        BFTValidatorSet.from(
            PrivateKeys.numeric(1)
                .map(k -> BFTValidator.from(BFTValidatorId.create(k.getPublicKey()), UInt256.ONE))
                .limit(1));
    var proof = LedgerProof.genesis(accumulatorState, stateHash, validatorSet, 0, 0);
    return CommittedTransactionsWithProof.create(List.of(genesis), proof);
  }

  @Test
  public void test_valid_rev2_transaction_passes() {
    // Arrange
    var injector = createInjector();
    var stateComputer = injector.getInstance(StateComputerLedger.StateComputer.class);
    var accumulator = injector.getInstance(LedgerAccumulator.class);
    var genesis = buildGenesis(accumulator);
    stateComputer.commit(genesis, null);
    var validTransaction = REv2TestTransactions.constructValidRawTransaction(0, 0);

    // Act
    var roundDetails = new RoundDetails(1, 1, 0, BFTValidatorId.random(), 1000, 1000);
    var result =
        stateComputer.prepare(
            genesis.getProof().getAccumulatorState().getAccumulatorHash(),
            List.of(),
            List.of(validTransaction),
            roundDetails);

    // Assert
    assertThat(result.getFailedTransactions()).isEmpty();
  }

  @Test
  public void test_invalid_rev2_transaction_fails() {
    // Arrange
    var injector = createInjector();
    var stateComputer = injector.getInstance(StateComputerLedger.StateComputer.class);
    var accumulator = injector.getInstance(LedgerAccumulator.class);
    var genesis = buildGenesis(accumulator);
    stateComputer.commit(genesis, null);
    var invalidTransaction = RawNotarizedTransaction.create(new byte[1]);

    // Act
    var roundDetails = new RoundDetails(1, 1, 0, BFTValidatorId.random(), 1000, 1000);
    var result =
        stateComputer.prepare(
            genesis.getProof().getAccumulatorState().getAccumulatorHash(),
            List.of(),
            List.of(invalidTransaction),
            roundDetails);

    // Assert
    assertThat(result.getFailedTransactions()).hasSize(1);
  }
}
