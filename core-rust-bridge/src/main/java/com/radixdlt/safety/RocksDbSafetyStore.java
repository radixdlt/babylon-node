package com.radixdlt.safety;

import static com.radixdlt.lang.Tuple.*;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;
import java.util.Optional;

public class RocksDbSafetyStore {
  static {
    System.loadLibrary("corerust");
  }

  private static native byte[] upsert(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] get(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  public static RocksDbSafetyStore create(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RocksDbSafetyStore(metrics, nodeRustEnvironment);
  }

  private RocksDbSafetyStore(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    final var timer = metrics.stateManager().nativeCall();
    upsertFunc =
        Natives.builder(nodeRustEnvironment, RocksDbSafetyStore::upsert)
            .measure(timer.label(new Metrics.MethodId(RocksDbSafetyStore.class, "upsert")))
            .build(new TypeToken<>() {});
    getFunc =
        Natives.builder(nodeRustEnvironment, RocksDbSafetyStore::get)
            .measure(timer.label(new Metrics.MethodId(RocksDbSafetyStore.class, "get")))
            .build(new TypeToken<>() {});
  }

  public void upsert(SafetyStateDTO state) {
    this.upsertFunc.call(state);
  }

  public Optional<SafetyStateDTO> get() {
    return Optional.ofNullable(this.getFunc.call(Tuple0.of()));
  }

  private final Natives.Call1<SafetyStateDTO, Tuple0> upsertFunc;
  private final Natives.Call1<Tuple0, SafetyStateDTO> getFunc;
}
