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

import com.google.inject.*;
import com.radixdlt.consensus.MockedConsensusRecoveryModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.environment.deterministic.DeterministicProcessor;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicEnvironmentModule;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.mempool.MempoolConfig;
import com.radixdlt.mempool.MempoolInserter;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.networks.Addressing;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import java.util.List;
import org.junit.Test;

public final class REv2ConsensusTransferTest {

  private static final ECKeyPair TEST_KEY = PrivateKeys.ofNumeric(1);
  private static final Decimal GENESIS_AMOUNT = Decimal.of(24_000_000_000L);

  private final DeterministicNetwork network =
      new DeterministicNetwork(
          List.of(BFTNode.create(TEST_KEY.getPublicKey())),
          MessageSelector.firstSelector(),
          MessageMutator.nothing());

  @Inject private DeterministicProcessor processor;
  @Inject private MempoolInserter<RawTransaction> mempoolInserter;
  @Inject private REv2TransactionAndProofStore transactionStoreReader;
  @Inject private REv2StateReader stateReader;

  private Injector createInjector() {
    return Guice.createInjector(
        new CryptoModule(),
        new TestMessagingModule.Builder().withDefaultRateLimit().build(),
        new MockedLedgerRecoveryModule(),
        new MockedConsensusRecoveryModule.Builder()
            .withNodes(List.of(BFTNode.create(TEST_KEY.getPublicKey())))
            .build(),
        new MockedPersistenceStoreModule(),
        new FunctionalRadixNodeModule(
            false,
            FunctionalRadixNodeModule.ConsensusConfig.of(),
            FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                StateComputerConfig.rev2(
                    StateComputerConfig.REV2ProposerConfig.mempool(MempoolConfig.of(1))))),
        new TestP2PModule.Builder().build(),
        new InMemoryBFTKeyModule(TEST_KEY),
        new DeterministicEnvironmentModule(
            network.createSender(BFTNode.create(TEST_KEY.getPublicKey()))),
        new AbstractModule() {
          @Override
          protected void configure() {
            bind(SystemCounters.class).to(SystemCountersImpl.class).in(Scopes.SINGLETON);
            bind(Addressing.class).toInstance(Addressing.ofNetwork(Network.INTEGRATIONTESTNET));
            bind(TimeSupplier.class).toInstance(System::currentTimeMillis);
          }
        });
  }

  private static RawTransaction createNewAccountTransaction() {
    var unsignedManifest = TransactionBuilder.buildNewAccountManifest(TEST_KEY.getPublicKey());
    var hashedManifest = HashUtils.sha256Twice(unsignedManifest).asBytes();

    var intentSignature = TEST_KEY.sign(hashedManifest);
    var signedIntent =
        TransactionBuilder.createSignedIntentBytes(
            unsignedManifest, TEST_KEY.getPublicKey(), intentSignature);
    var hashedSignedIntent = HashUtils.sha256Twice(signedIntent).asBytes();

    var notarySignature = TEST_KEY.sign(hashedSignedIntent);
    var transactionPayload = TransactionBuilder.createNotarizedBytes(signedIntent, notarySignature);
    return RawTransaction.create(transactionPayload);
  }

  @Test
  public void new_account_creates_transfer_of_xrd_to_account() throws Exception {
    // Arrange: Start single node network
    createInjector().injectMembers(this);
    var newAccountTransaction = createNewAccountTransaction();

    // Act: Submit transaction to mempool and run consensus
    processor.start();
    for (int i = 0; i < 1000; i++) {
      var msg = network.nextMessage().value();
      processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
    }
    mempoolInserter.addTransaction(newAccountTransaction);
    for (int i = 0; i < 1000; i++) {
      var msg = network.nextMessage().value();
      processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
    }

    // Assert: Check transaction and post submission state
    var receipt = transactionStoreReader.getTransactionAtStateVersion(1);
    var componentAddress = receipt.getNewComponentAddresses().get(0);
    var accountAmount = stateReader.getComponentXrdAmount(componentAddress);
    assertThat(accountAmount).isEqualTo(Decimal.of(1_000_000L));
    var systemAmount = stateReader.getComponentXrdAmount(ComponentAddress.SYSTEM_COMPONENT_ADDRESS);
    assertThat(systemAmount).isLessThan(GENESIS_AMOUNT);
  }
}
