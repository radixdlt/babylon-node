package com.radixdlt.db;

import static org.junit.Assert.*;

import com.radixdlt.environment.*;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import java.io.IOException;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RocksDbMigrationStoreTest {

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  @Test
  public void migration_status_is_properly_reported() throws Exception {
    var migrationStore =
        RocksDbMigrationStore.create(
            new MetricsInitializer().initialize(), createNodeRustEnvironment());

    assertFalse(migrationStore.isMigrated(StoreId.SAFETY_STORE));
    assertFalse(migrationStore.isMigrated(StoreId.ADDRESS_BOOK));

    migrationStore.migrationDone(StoreId.SAFETY_STORE);
    assertTrue(migrationStore.isMigrated(StoreId.SAFETY_STORE));

    migrationStore.migrationDone(StoreId.ADDRESS_BOOK);
    assertTrue(migrationStore.isMigrated(StoreId.ADDRESS_BOOK));
  }

  private NodeRustEnvironment createNodeRustEnvironment() throws IOException {
    final var mempoolMaxTotalTransactionsSize = 10 * 1024 * 1024;
    final var mempoolMaxTransactionCount = 20;
    final var stateManagerDbConfig = new DatabaseBackendConfig(folder.newFolder().getPath());
    final var nodeDbConfig = new DatabaseBackendConfig(folder.newFolder().getPath());

    final var config =
        new StateManagerConfig(
            NetworkDefinition.INT_TEST_NET,
            Option.some(
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
