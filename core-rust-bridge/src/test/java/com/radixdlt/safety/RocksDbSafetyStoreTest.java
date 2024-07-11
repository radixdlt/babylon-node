package com.radixdlt.safety;

import com.radixdlt.environment.*;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.RustMempoolConfig;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.transaction.LedgerSyncLimitsConfig;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import java.io.IOException;

import static org.junit.Assert.*;

public class RocksDbSafetyStoreTest {
    @Rule
    public TemporaryFolder folder = new TemporaryFolder();

  @Test
  public void test_safety_state_can_be_saved_and_restored() throws Exception {
    try (var nodeRustEnvironment = createNodeRustEnvironment()) {
        var safetyStore = RocksDbSafetyStore.create(new MetricsInitializer().initialize(), nodeRustEnvironment);
        var originalSafetyState = new SafetyStateDTO(new byte[] {1, 2, 3});
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
            () -> {}, // A no-op fatal panic handler. Please note that a JNI-invoking test (like this one) will observe
            // panics as runtime exceptions propagated up the stack (through JNI), which will fail the test
            // gracefully anyway.
            config,
            new NodeConfig(nodeDbConfig));
    }
}
