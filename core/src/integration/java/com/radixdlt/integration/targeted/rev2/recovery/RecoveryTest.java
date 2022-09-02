package com.radixdlt.integration.targeted.rev2.recovery;

import com.google.common.collect.Streams;
import com.radixdlt.consensus.ConsensusEvent;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.REV2TransactionGenerator;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.sync.messages.local.SyncCheckTrigger;
import com.radixdlt.sync.messages.remote.StatusRequest;
import com.radixdlt.sync.messages.remote.SyncRequest;
import com.radixdlt.sync.messages.remote.SyncResponse;
import com.radixdlt.utils.Pair;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;

public class RecoveryTest {
    @Rule
    public TemporaryFolder folder = new TemporaryFolder();

    private DeterministicTest createTest() {
        return DeterministicTest.builder()
                .numNodes(2, 0)
                .messageSelector(firstSelector())
                .messageMutator(MessageMutator.dropTimeouts())
                .functionalNodeModule(
                        new FunctionalRadixNodeModule(
                                false,
                                FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                                FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                                        StateComputerConfig.rev2(
                                                Network.INTEGRATIONTESTNET.getId(),
                                                REv2DatabaseConfig.rocksDB(folder.getRoot().getAbsolutePath()),
                                                StateComputerConfig.REV2ProposerConfig.transactionGenerator(
                                                        new REV2TransactionGenerator(), 1)),
                                        SyncRelayConfig.of(5000, 10, 3000L))));
    }

    @Test
    public void relayer_fills_mempool_of_all_nodes() throws Exception {
        try (var test = createTest()) {
            test.startAllNodes();

            test.runUntil(n ->
                n.stream()
                .allMatch(i -> i.getInstance(TransactionsAndProofReader.class).getLastProof().map(p -> p.getStateVersion() == 3).orElse(false))
            , 1000, msg -> msg.message() instanceof ConsensusEvent || (msg.channelId().isLocal() && !(msg.message() instanceof SyncCheckTrigger)));

            test.runUntil(n ->
                n.stream().anyMatch(i -> i.getInstance(TransactionsAndProofReader.class).getLastProof().map(p -> p.getStateVersion() == 4).orElse(false))
            , 1000, msg -> msg.message() instanceof ConsensusEvent || msg.channelId().isLocal());

            var behindNodeIndex = Streams.mapWithIndex(test.getNodeInjectors().stream(), Pair::of)
                    .filter(p -> p.getFirst().getInstance(TransactionsAndProofReader.class).getLastProof().orElseThrow().getStateVersion() == 3)
                    .map(Pair::getSecond)
                    .findFirst().orElseThrow().intValue();

            test.runUntil(n ->
                    n.stream()
                            .allMatch(i -> i.getInstance(TransactionsAndProofReader.class).getLastProof().map(p -> p.getStateVersion() == 4).orElse(false)),
                    1000, msg -> !(msg.message() instanceof ConsensusEvent) && !(msg.message() instanceof GetVerticesRequest)
                    );

            test.restartNode(behindNodeIndex);
            test.runForCount(1, msg -> msg.message() instanceof Proposal);
            test.runUntil(n ->
                            n.stream()
                                    .allMatch(i -> i.getInstance(TransactionsAndProofReader.class).getLastProof().map(p -> p.getStateVersion() == 5).orElse(false)),
                    100000, msg -> !(msg.message() instanceof ConsensusEvent) && !(msg.message() instanceof SyncRequest)
                            && !(msg.message() instanceof SyncResponse) && !(msg.message() instanceof StatusRequest) && !(msg.message() instanceof SyncCheckTrigger)
            );
        }
    }
}
