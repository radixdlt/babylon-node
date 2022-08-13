package com.radixdlt.integration.targeted.transfer;

import com.google.common.hash.Hasher;
import com.google.common.primitives.Bytes;
import com.google.inject.*;
import com.radixdlt.address.ComponentAddress;
import com.radixdlt.consensus.MockedConsensusRecoveryModule;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.sync.BFTSyncPatienceMillis;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.environment.Environment;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.MockedCryptoModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.networks.Addressing;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.rev2.REv2StateReader;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import org.junit.Test;

import java.math.BigInteger;
import java.util.List;
import java.util.stream.Stream;

import static org.assertj.core.api.Assertions.assertThat;

public class REv2TransferTest {

    private static final ECKeyPair TEST_KEY = PrivateKeys.ofNumeric(1);
    private static final BigInteger ONE_TOKEN = BigInteger.TEN.pow(18);
    private static final BigInteger GENESIS_AMOUNT =
            BigInteger.valueOf(24).multiply(BigInteger.TEN.pow(9)).multiply(ONE_TOKEN);

    @Inject
    private REv2StateReader stateReader;

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

        var unsignedManifest = TransactionBuilder.buildNewAccountManifest(TEST_KEY.getPublicKey());
        var hashed = HashUtils.sha256(HashUtils.sha256(unsignedManifest).asBytes()).asBytes();
        var signedManifest = TEST_KEY.sign(hashed);
        var notaryToSignManifest = TransactionBuilder.combineForNotary(unsignedManifest, TEST_KEY.getPublicKey(), signedManifest);
    }
}
