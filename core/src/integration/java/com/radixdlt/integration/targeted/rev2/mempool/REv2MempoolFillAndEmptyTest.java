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

package com.radixdlt.integration.targeted.rev2.mempool;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.AbstractModule;
import com.google.inject.Guice;
import com.google.inject.Inject;
import com.google.inject.Injector;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.consensus.epoch.EpochRoundUpdate;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.deterministic.DeterministicProcessor;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicEnvironmentModule;
import com.radixdlt.integration.Slow;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.logger.EventLoggerConfig;
import com.radixdlt.logger.EventLoggerModule;
import com.radixdlt.mempool.MempoolFullException;
import com.radixdlt.mempool.MempoolInserter;
import com.radixdlt.mempool.MempoolReader;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.statemanager.REv2StateConfig;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.UInt64;
import java.util.List;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.junit.Test;
import org.junit.experimental.categories.Category;

/**
 * Test which fills a mempool and then empties it checking to make sure there are no stragglers left
 * behind.
 */
@Category(Slow.class)
public final class REv2MempoolFillAndEmptyTest {
  private static final Logger logger = LogManager.getLogger();
  private static final ECKeyPair TEST_KEY = PrivateKeys.ofNumeric(1);

  private final DeterministicNetwork network =
      new DeterministicNetwork(
          List.of(BFTNode.create(TEST_KEY.getPublicKey())),
          MessageSelector.firstSelector(),
          MessageMutator.nothing());
  private final REV2TransactionGenerator transactionGenerator =
      new REV2TransactionGenerator(NetworkDefinition.INT_TEST_NET);

  @Inject private Metrics metrics;
  @Inject private DeterministicProcessor processor;
  @Inject private MempoolInserter<RawNotarizedTransaction, RawNotarizedTransaction> mempoolInserter;
  @Inject private MempoolReader<RawNotarizedTransaction> mempoolReader;

  private Injector createInjector() {
    return Guice.createInjector(
        new CryptoModule(),
        new TestMessagingModule.Builder().withDefaultRateLimit().build(),
        new AbstractModule() {
          @Override
          protected void configure() {
            var validatorSet =
                BFTValidatorSet.from(
                    List.of(
                        BFTValidator.from(BFTNode.create(TEST_KEY.getPublicKey()), UInt256.ONE)));
            bind(BFTValidatorSet.class).toInstance(validatorSet);
          }
        },
        new FunctionalRadixNodeModule(
            false,
            SafetyRecoveryConfig.mocked(),
            ConsensusConfig.of(),
            LedgerConfig.stateComputerWithSyncRelay(
                StateComputerConfig.rev2(
                    Network.INTEGRATIONTESTNET.getId(),
                    new REv2StateConfig(UInt64.fromNonNegativeLong(1000)),
                    REv2DatabaseConfig.inMemory(),
                    StateComputerConfig.REV2ProposerConfig.mempool(
                        10, 1000, MempoolRelayConfig.of())),
                SyncRelayConfig.of(5000, 10, 3000L))),
        new TestP2PModule.Builder().build(),
        new InMemoryBFTKeyModule(TEST_KEY),
        new DeterministicEnvironmentModule(
            network.createSender(BFTNode.create(TEST_KEY.getPublicKey()))),
        new EventLoggerModule(
            EventLoggerConfig.addressed(Addressing.ofNetwork(Network.INTEGRATIONTESTNET))),
        new AbstractModule() {
          @Override
          protected void configure() {
            bind(Metrics.class).toInstance(new MetricsInitializer().initialize());
            bind(TimeSupplier.class).toInstance(System::currentTimeMillis);
          }
        });
  }

  private void fillAndEmptyMempool() throws Exception {

    var rateLimiter = RateLimiter.create(0.5);

    while (mempoolReader.getCount() < 1000) {
      if (rateLimiter.tryAcquire()) {
        logger.info("Filling Mempool...  Current Size: {}", mempoolReader.getCount());
      }
      ControlledMessage msg = network.nextMessage().value();
      processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
      if (msg.message() instanceof RoundUpdate || msg.message() instanceof EpochRoundUpdate) {
        for (int i = 0; i < 50; i++) {
          try {
            mempoolInserter.addTransaction(transactionGenerator.nextTransaction());
          } catch (MempoolFullException ignored) {
          }
        }
      }
    }

    for (int i = 0; i < 10000; i++) {
      if (rateLimiter.tryAcquire()) {
        logger.info("Emptying Mempool...  Current Size: {}", mempoolReader.getCount());
      }
      ControlledMessage msg = network.nextMessage().value();
      processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
      if (mempoolReader.getCount() == 0) {
        break;
      }
    }
  }

  @Test
  public void check_that_full_mempool_empties_itself() throws Exception {
    createInjector().injectMembers(this);
    processor.start();

    for (int i = 0; i < 10; i++) {
      fillAndEmptyMempool();
    }

    assertThat(metrics.v1RadixEngine().invalidProposedTransactions().get()).isZero();
  }
}
