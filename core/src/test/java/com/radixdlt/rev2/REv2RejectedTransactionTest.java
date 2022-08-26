package com.radixdlt.rev2;

import com.google.inject.*;
import com.radixdlt.consensus.MockedConsensusRecoveryModule;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.ExecutedVertex;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.environment.deterministic.DeterministicProcessor;
import com.radixdlt.environment.deterministic.network.DeterministicNetwork;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.environment.deterministic.network.MessageSelector;
import com.radixdlt.harness.deterministic.DeterministicEnvironmentModule;
import com.radixdlt.keys.InMemoryBFTKeyModule;
import com.radixdlt.lang.Tuple;
import com.radixdlt.ledger.MockedLedgerRecoveryModule;
import com.radixdlt.messaging.TestMessagingModule;
import com.radixdlt.modules.CryptoModule;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.modules.StateComputerConfig.REV2ProposerConfig;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.networks.Addressing;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.TestP2PModule;
import com.radixdlt.rev2.modules.MockedPersistenceStoreModule;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.TimeSupplier;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

public final class REv2RejectedTransactionTest {
    private static final ECKeyPair TEST_KEY = PrivateKeys.ofNumeric(1);
    private static final NetworkDefinition NETWORK_DEFINITION = NetworkDefinition.LOCAL_SIMULATOR;

    private final DeterministicNetwork network =
            new DeterministicNetwork(
                    List.of(BFTNode.create(TEST_KEY.getPublicKey())),
                    MessageSelector.firstSelector(),
                    MessageMutator.nothing());

    @Rule
    public TemporaryFolder folder = new TemporaryFolder();
    @Inject
    private DeterministicProcessor processor;
    @Inject private REv2TransactionAndProofStore transactionStoreReader;

    private Injector createInjector(ProposalGenerator proposalGenerator) {
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
                        ConsensusConfig.of(),
                        LedgerConfig.stateComputerNoSync(
                                StateComputerConfig.rev2(
                                        Network.LOCALSIMULATOR.getId(),
                                        new REv2DatabaseConfig.RocksDB(folder.getRoot().getAbsolutePath()),
                                        REV2ProposerConfig.transactionGenerator(proposalGenerator)))),
                new TestP2PModule.Builder().build(),
                new InMemoryBFTKeyModule(TEST_KEY),
                new DeterministicEnvironmentModule(
                        network.createSender(BFTNode.create(TEST_KEY.getPublicKey()))),
                new AbstractModule() {
                    @Override
                    protected void configure() {
                        bind(SystemCounters.class).to(SystemCountersImpl.class).in(Scopes.SINGLETON);
                        bind(Addressing.class).toInstance(Addressing.ofNetwork(Network.LOCALSIMULATOR));
                        bind(TimeSupplier.class).toInstance(System::currentTimeMillis);
                    }
                });
    }

    private static RawTransaction createRejectableTransaction() {
        var rejectableManifest = "CALL_METHOD ComponentAddress(\"account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064\") \"withdraw_by_amount\" Decimal(\"5.0\") ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\");";
        var header =
                TransactionHeader.defaults(NETWORK_DEFINITION, TEST_KEY.getPublicKey(), false);
        var intentBytes = TransactionBuilder.createIntent(NETWORK_DEFINITION, header, rejectableManifest);
        var hashedIntent = HashUtils.sha256Twice(intentBytes).asBytes();
        var intentSignature = TEST_KEY.sign(hashedIntent);
        var signedIntent =
                TransactionBuilder.createSignedIntentBytes(
                        intentBytes, List.of(Tuple.Tuple2.of(TEST_KEY.getPublicKey(), intentSignature)));
        var hashedSignedIntent = HashUtils.sha256Twice(signedIntent).asBytes();

        var notarySignature = TEST_KEY.sign(hashedSignedIntent);
        var transactionPayload = TransactionBuilder.createNotarizedBytes(signedIntent, notarySignature);
        return RawTransaction.create(transactionPayload);
    }

    private static class ControlledProposerGenerator implements ProposalGenerator {
        private RawTransaction nextTransaction = null;

        @Override
        public List<RawTransaction> getTransactionsForProposal(Round round, List<ExecutedVertex> prepared) {
            if (nextTransaction == null) {
                return List.of();
            } else {
                var txns = List.of(nextTransaction);
                this.nextTransaction = null;
                return txns;
            }
        }
    }

    @Test
    public void rejected_transaction_in_proposal_should_not_be_committed() {
        var proposalGenerator = new ControlledProposerGenerator();

        // Arrange: Start single node network
        createInjector(proposalGenerator).injectMembers(this);
        var newAccountTransaction = createRejectableTransaction();

        // Act: Submit transaction to mempool and run consensus
        processor.start();
        for (int i = 0; i < 1000; i++) {
            var msg = network.nextMessage().value();
            processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
        }
        proposalGenerator.nextTransaction = newAccountTransaction;
        for (int i = 0; i < 1000; i++) {
            var msg = network.nextMessage().value();
            processor.handleMessage(msg.origin(), msg.message(), msg.typeLiteral());
        }

        // Assert: Check transaction and post submission state
        assertThat(proposalGenerator.nextTransaction).isNull();
        assertThat(transactionStoreReader.getLastProof()).isEmpty();
    }
}
