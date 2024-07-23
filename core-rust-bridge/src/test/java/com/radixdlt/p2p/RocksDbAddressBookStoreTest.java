package com.radixdlt.p2p;

import static com.radixdlt.lang.Option.none;
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
    public void test_address_book_entries_can_be_saved_and_restored() throws Exception {
        try (var nodeRustEnvironment = createNodeRustEnvironment()) {
            var addressBookStore =
                RocksDbAddressBookStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);

            // New store is empty
            var empty = addressBookStore.getAllEntries();
            assertTrue(empty.isEmpty());

            // Ensure keys are repeatable to make test deterministic
            var nodeId1 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {1}).getPublicKey());
            var entry1 = new AddressBookEntryDTO(nodeId1, some(123L), Set.of("addr1", "addr2"));

            addressBookStore.upsertEntry(entry1);

            // Store now contains one entry
            var allEntries = addressBookStore.getAllEntries();
            assertEquals(1L, allEntries.size());
            assertEquals(entry1, allEntries.get(0));

            // Ensure keys are repeatable to make test deterministic
            var nodeId2 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {2}).getPublicKey());
            var entry2 = new AddressBookEntryDTO(nodeId2, none(), Set.of("addr3", "addr4", "addr5"));

            // Add another entry
            addressBookStore.upsertEntry(entry2);

            allEntries = addressBookStore.getAllEntries();
            assertEquals(2L, allEntries.size());
            assertEquals(entry1, allEntries.get(0));
            assertEquals(entry2, allEntries.get(1));
        }
    }

    @Test
    public void test_address_book_entry_can_be_added_and_removed() throws Exception {
        try (var nodeRustEnvironment = createNodeRustEnvironment()) {
            var addressBookStore =
                RocksDbAddressBookStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);

            // New store is empty
            var empty = addressBookStore.getAllEntries();
            assertTrue(empty.isEmpty());

            // Ensure keys are repeatable to make test deterministic
            var nodeId1 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {1}).getPublicKey());
            var entry1 = new AddressBookEntryDTO(nodeId1, some(123L), Set.of("addr1", "addr2"));
            var nodeId2 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {2}).getPublicKey());
            var entry2 = new AddressBookEntryDTO(nodeId2, none(), Set.of("addr3", "addr4", "addr5"));

            addressBookStore.upsertEntry(entry1);
            addressBookStore.upsertEntry(entry2);

            // Check that entries were added
            var allEntries = addressBookStore.getAllEntries();
            assertEquals(2L, allEntries.size());
            assertEquals(entry1, allEntries.get(0));
            assertEquals(entry2, allEntries.get(1));

            // Remove entry1
            var removed = addressBookStore.removeEntry(nodeId1);
            assertTrue(removed);

            // Check that entry1 was removed
            allEntries = addressBookStore.getAllEntries();
            assertEquals(1L, allEntries.size());
            assertEquals(entry2, allEntries.get(0));
        }
    }

    @Test
    public void test_address_book_can_be_reset() throws Exception {
        try (var nodeRustEnvironment = createNodeRustEnvironment()) {
            var addressBookStore =
                RocksDbAddressBookStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);

            // New store is empty
            var empty = addressBookStore.getAllEntries();
            assertTrue(empty.isEmpty());

            // Ensure keys are repeatable to make test deterministic
            var nodeId1 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {1}).getPublicKey());
            var entry1 = new AddressBookEntryDTO(nodeId1, some(123L), Set.of("addr1", "addr2"));
            var nodeId2 = new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {2}).getPublicKey());
            var entry2 = new AddressBookEntryDTO(nodeId2, none(), Set.of("addr3", "addr4", "addr5"));

            addressBookStore.upsertEntry(entry1);
            addressBookStore.upsertEntry(entry2);

            // Check that entries were added
            var allEntries = addressBookStore.getAllEntries();
            assertEquals(2L, allEntries.size());
            assertEquals(entry1, allEntries.get(0));
            assertEquals(entry2, allEntries.get(1));

            // Reset store
            addressBookStore.reset();

            // Check that entry1 was removed
            empty = addressBookStore.getAllEntries();
            assertTrue(empty.isEmpty());
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
