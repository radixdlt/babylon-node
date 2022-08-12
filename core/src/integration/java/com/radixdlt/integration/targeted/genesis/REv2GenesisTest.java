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

package com.radixdlt.integration.targeted.genesis;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.inject.*;
import com.radixdlt.address.ComponentAddress;
import com.radixdlt.consensus.MockedConsensusRecoveryModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.Environment;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.*;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.networks.Addressing;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.rev2.REv2StateReader;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import java.math.BigInteger;
import java.util.List;
import java.util.stream.Stream;
import org.junit.Test;

public final class REv2GenesisTest {
  private static final ECKeyPair TEST_KEY = PrivateKeys.ofNumeric(1);
  private static final BigInteger ONE_TOKEN = BigInteger.TEN.pow(18);
  private static final BigInteger GENESIS_AMOUNT = BigInteger.valueOf(24).multiply(BigInteger.TEN.pow(9)).multiply(ONE_TOKEN);

  @Inject private REv2StateReader stateReader;

  private Injector createInjector() {
    return Guice.createInjector(
        new MockedCryptoModule(),
        new TestMessagingModule.Builder().withDefaultRateLimit().build(),
        new MockedLedgerRecoveryModule(),
        new MockedConsensusRecoveryModule.Builder()
            .withNodes(List.of(BFTNode.create(TEST_KEY.getPublicKey())))
            .build(),
        new MockedPersistenceStoreModule(),
        new FunctionalRadixNodeModule(
            false,
            FunctionalRadixNodeModule.LedgerConfig.stateComputer(
                StateComputerConfig.rev2(
                    StateComputerConfig.REV2ProposerConfig.halfCorrectProposer()),
                false)),
        new TestP2PModule.Builder().build(),
        new InMemoryBFTKeyModule(TEST_KEY),
        new AbstractModule() {
          @Override
          protected void configure() {
            bind(SystemCounters.class).to(SystemCountersImpl.class).in(Scopes.SINGLETON);
            bind(Addressing.class).toInstance(Addressing.ofNetwork(Network.INTEGRATIONTESTNET));
            bind(TimeSupplier.class).toInstance(System::currentTimeMillis);
            bindConstant().annotatedWith(BFTSyncPatienceMillis.class).to(200);
            bindConstant().annotatedWith(PacemakerBaseTimeoutMs.class).to(100L);
            bindConstant().annotatedWith(PacemakerBackoffRate.class).to(2.0);
            bindConstant()
                .annotatedWith(PacemakerMaxExponent.class)
                .to(0); // Use constant timeout for now
          }

          @Provides
          @Singleton
          public DeterministicNetwork network(@Self BFTNode self, PeersView peersView) {
            return new DeterministicNetwork(
                Stream.concat(Stream.of(self), peersView.peers().map(PeersView.PeerInfo::bftNode))
                    .toList(),
                MessageSelector.firstSelector(),
                MessageMutator.nothing());
          }

          @Provides
          @Singleton
          Environment environment(@Self BFTNode self, DeterministicNetwork network) {
            return network.createSender(self);
          }
        });
  }

  @Test
  public void state_reader_on_genesis_returns_correct_amount() {
    createInjector().injectMembers(this);

    var amount = this.stateReader.getComponentResources(ComponentAddress.SYSTEM_COMPONENT_ADDRESS);

    assertThat(amount).isEqualTo(GENESIS_AMOUNT);
  }
}
