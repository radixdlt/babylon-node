package com.radixdlt.p2p;

import static com.radixdlt.lang.Option.some;
import static org.junit.Assert.*;

import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.*;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import java.io.IOException;
import java.util.List;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RocksDbHighPriorityPeersStoreTest {
    @Rule
    public TemporaryFolder folder = new TemporaryFolder();

    @Test
    public void test_high_priority_peers_can_be_saved_and_restored() throws Exception {
        try (var nodeRustEnvironment = createNodeRustEnvironment()) {
            var highPriorityPeersStore =
                RocksDbHighPriorityPeersStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);

            // New store is empty
            var empty = highPriorityPeersStore.getHighPriorityPeers();
            assertTrue(empty.isEmpty());

            var inputList1 = List.of(newNodeId(), newNodeId(), newNodeId());

            // Store new list
            highPriorityPeersStore.storeHighPriorityPeers(inputList1);

            // Retrieve same list back
            var peers = highPriorityPeersStore.getHighPriorityPeers();
            assertEquals(3L, peers.size());
            assertEquals(inputList1, peers);

            // Overwrite with a new list
            var inputList2 = List.of(newNodeId(), newNodeId(), newNodeId());

            // Ensure lists are different
            assertNotEquals(inputList1, inputList2);

            // Store new list
            highPriorityPeersStore.storeHighPriorityPeers(inputList2);

            peers = highPriorityPeersStore.getHighPriorityPeers();
            assertEquals(3L, peers.size());
            assertEquals(inputList2, peers);
        }
    }

    private static NodeIdDTO newNodeId() {
        return new NodeIdDTO(ECKeyPair.generateNew().getPublicKey());
    }

    private NodeRustEnvironment createNodeRustEnvironment() throws IOException {
        final var mempoolMaxTotalTransactionsSize = 10 * 1024 * 1024;
        final var mempoolMaxTransactionCount = 20;
        final var stateManagerDbConfig = new DatabaseBackendConfig(folder.newFolder().getPath());
        final var nodeDbConfig = new DatabaseBackendConfig(folder.newFolder().getPath());

        final var config =
            new StateManagerConfig(
                NetworkDefinition.INT_TEST_NET,
                some(
                    new RustMempoolConfig(mempoolMaxTotalTransactionsSize, mempoolMaxTransactionCount)),
                Option.none(),
                stateManagerDbConfig,
                new DatabaseConfig(false, false, false, false),
                LoggingConfig.getDefault(),
                StateTreeGcConfig.forTesting(),
                LedgerProofsGcConfig.forTesting(),
                LedgerSyncLimitsConfig.defaults(),
                ProtocolConfig.testingDefault(),
                false,
                ScenariosExecutionConfig.NONE);

        return new NodeRustEnvironment(
            tx -> {}, // A no-op dispatcher of transactions to be relayed.
            () -> {}, // A no-op fatal panic handler. Please note that a JNI-invoking test (like this
            // one) will observe
            // panics as runtime exceptions propagated up the stack (through JNI), which will fail the
            // test
            // gracefully anyway.
            config,
            new NodeConfig(nodeDbConfig));
    }
}
