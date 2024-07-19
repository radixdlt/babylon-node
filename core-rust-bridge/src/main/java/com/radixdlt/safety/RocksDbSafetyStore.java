package com.radixdlt.safety;

import static com.radixdlt.lang.Tuple.*;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.lang.Option;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;
import com.radixdlt.sbor.NodeSborCodecs;
import com.radixdlt.sbor.codec.Codec;

public class RocksDbSafetyStore {
  static {
    System.loadLibrary("corerust");
  }

  private static native byte[] upsert(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] get(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Codec<SafetyStateDTO> dtoCodec;

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

    dtoCodec = NodeSborCodecs.resolveCodec(new TypeToken<>() {});
  }

  public void upsert(SafetyStateDTO state) {
    this.upsertFunc.call(state);
  }

  public Option<SafetyStateDTO> get() {
    return this.getFunc.call(tuple()).map(value -> NodeSborCodecs.decode(value, dtoCodec));
  }

  private final Natives.Call1<SafetyStateDTO, Tuple0> upsertFunc;
  private final Natives.Call1<Tuple0, Option<byte[]>> getFunc;
}
