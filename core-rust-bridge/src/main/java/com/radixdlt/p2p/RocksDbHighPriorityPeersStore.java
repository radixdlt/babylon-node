package com.radixdlt.p2p;

import static com.radixdlt.lang.Tuple.*;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.lang.Option;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;
import com.radixdlt.sbor.NodeSborCodecs;
import com.radixdlt.sbor.codec.Codec;
import java.util.List;

public class RocksDbHighPriorityPeersStore {
  static {
    System.loadLibrary("corerust");
  }

  private static native byte[] upsertAllHighPriorityPeers(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] getAllHighPriorityPeers(
      NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Codec<List<NodeIdDTO>> dtoCodec;

  public static RocksDbHighPriorityPeersStore create(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RocksDbHighPriorityPeersStore(metrics, nodeRustEnvironment);
  }

  private RocksDbHighPriorityPeersStore(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    final var timer = metrics.stateManager().nativeCall();
    upsertAllHighPriorityPeersFunc =
        Natives.builder(
                nodeRustEnvironment, RocksDbHighPriorityPeersStore::upsertAllHighPriorityPeers)
            .measure(
                timer.label(
                    new Metrics.MethodId(
                        RocksDbHighPriorityPeersStore.class, "upsertAllHighPriorityPeers")))
            .build(new TypeToken<>() {});
    getAllHighPriorityPeersFunc =
        Natives.builder(nodeRustEnvironment, RocksDbHighPriorityPeersStore::getAllHighPriorityPeers)
            .measure(
                timer.label(
                    new Metrics.MethodId(
                        RocksDbHighPriorityPeersStore.class, "getAllHighPriorityPeers")))
            .build(new TypeToken<>() {});

    dtoCodec = NodeSborCodecs.resolveCodec(new TypeToken<>() {});
  }

  void storeHighPriorityPeers(List<NodeIdDTO> ids) {
    this.upsertAllHighPriorityPeersFunc.call(ids);
  }

  List<NodeIdDTO> getHighPriorityPeers() {
    return this.getAllHighPriorityPeersFunc
        .call(tuple())
        .map(value -> NodeSborCodecs.decode(value, dtoCodec))
        .or(List.of());
  }

  private final Natives.Call1<List<NodeIdDTO>, Tuple0> upsertAllHighPriorityPeersFunc;
  private final Natives.Call1<Tuple0, Option<byte[]>> getAllHighPriorityPeersFunc;
}
