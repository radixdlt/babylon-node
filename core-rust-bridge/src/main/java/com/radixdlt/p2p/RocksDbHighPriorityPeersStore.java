package com.radixdlt.p2p;

import static com.radixdlt.lang.Tuple.*;

import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;
import java.util.List;

public class RocksDbHighPriorityPeersStore {
    static {
        System.loadLibrary("corerust");
    }

    private static native byte[] upsertAllHighPriorityPeers(NodeRustEnvironment nodeRustEnvironment, byte[] payload);
    private static native byte[] getAllHighPriorityPeers(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

    public static RocksDbHighPriorityPeersStore create(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
        return new RocksDbHighPriorityPeersStore(metrics, nodeRustEnvironment);
    }

    private RocksDbHighPriorityPeersStore(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
        final var timer = metrics.stateManager().nativeCall();
        upsertAllHighPriorityPeersFunc =
                Natives.builder(nodeRustEnvironment, RocksDbHighPriorityPeersStore::upsertAllHighPriorityPeers)
                       .measure(timer.label(new Metrics.MethodId(RocksDbHighPriorityPeersStore.class, "upsertAllHighPriorityPeers")))
                       .build(new TypeToken<>() {});
        getAllHighPriorityPeersFunc =
                Natives.builder(nodeRustEnvironment, RocksDbHighPriorityPeersStore::getAllHighPriorityPeers)
                        .measure(timer.label(new Metrics.MethodId(RocksDbHighPriorityPeersStore.class, "getAllHighPriorityPeers")))
                        .build(new TypeToken<>() {});
    }

    void storeHighPriorityPeers(List<NodeIdDTO> ids) {
        this.upsertAllHighPriorityPeersFunc.call(ids);
    }

    List<NodeIdDTO> getHighPriorityPeers() {
        return this.getAllHighPriorityPeersFunc.call(Tuple0.of());
    }

    private final Natives.Call1<List<NodeIdDTO>, Tuple0> upsertAllHighPriorityPeersFunc;
    private final Natives.Call1<Tuple0, List<NodeIdDTO>> getAllHighPriorityPeersFunc;
}
