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
import java.util.Set;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RocksDbAddressBookStoreTest {
    @Rule
    public TemporaryFolder folder = new TemporaryFolder();

    @Test
    public void test_address_book_entry_can_be_saved_and_restored() throws Exception {
        try (var nodeRustEnvironment = createNodeRustEnvironment()) {
            var addressBookStore =
                RocksDbAddressBookStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);

            var empty = addressBookStore.getAllEntries();
            assertTrue(empty.isEmpty());

            var nodeId = new NodeIdDTO(ECKeyPair.generateNew().getPublicKey());
            var entry = new AddressBookEntryDTO(nodeId, some(123L), Set.of("addr1", "addr2"));

            addressBookStore.upsertEntry(entry);

            var allEntries = addressBookStore.getAllEntries();
            assertEquals(1L, allEntries.size());
            assertEquals(entry, allEntries.get(0));
        }
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
