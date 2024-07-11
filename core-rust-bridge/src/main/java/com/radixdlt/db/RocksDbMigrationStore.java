package com.radixdlt.db;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;

public class RocksDbMigrationStore {
  static {
    System.loadLibrary("corerust");
  }

  private static native byte[] isMigrated(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] migrationDone(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  /**
   * Stores a pointer to the rust core api server across JNI calls. In the JNI model, this is
   * equivalent to the CoreApiServer "owning" the rust core api server memory. On each call into
   * Rust, we map the rustCoreApiServerPointer onto a concrete implementation in Rust land, and it
   * uses that to access all state and make calls.
   */
  @SuppressWarnings("unused")
  private final long rustCoreApiServerPointer = 0;

  public static RocksDbMigrationStore create(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RocksDbMigrationStore(metrics, nodeRustEnvironment);
  }

  private RocksDbMigrationStore(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    final var timer = metrics.stateManager().nativeCall();
    isMigratedFunc =
        Natives.builder(nodeRustEnvironment, RocksDbMigrationStore::isMigrated)
            .measure(timer.label(new Metrics.MethodId(RocksDbMigrationStore.class, "isMigrated")))
            .build(new TypeToken<>() {});
    migrationDoneFunc =
        Natives.builder(nodeRustEnvironment, RocksDbMigrationStore::migrationDone)
            .measure(
                timer.label(new Metrics.MethodId(RocksDbMigrationStore.class, "migrationDone")))
            .build(new TypeToken<>() {});
  }

  public boolean isMigrated(StoreId storeId) {
    return this.isMigratedFunc.call(storeId);
  }

  public void migrationDone(StoreId storeId) {
    this.migrationDoneFunc.call(storeId);
  }

  private final Natives.Call1<StoreId, Boolean> isMigratedFunc;
  private final Natives.Call1<StoreId, Void> migrationDoneFunc;
}
