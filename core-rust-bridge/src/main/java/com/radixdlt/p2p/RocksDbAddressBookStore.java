package com.radixdlt.p2p;

import static com.radixdlt.lang.Tuple.*;

import com.google.common.collect.ImmutableList;
import com.google.common.reflect.TypeToken;
import com.radixdlt.environment.NodeRustEnvironment;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.sbor.Natives;
import com.radixdlt.sbor.NodeSborCodecs;
import com.radixdlt.sbor.codec.Codec;
import java.util.List;

public class RocksDbAddressBookStore {
  static {
    System.loadLibrary("corerust");
  }

  private static native byte[] removeOne(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] upsertOne(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] reset(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private static native byte[] getAll(NodeRustEnvironment nodeRustEnvironment, byte[] payload);

  private final Codec<AddressBookEntryDTO> dtoCodec;

  public static RocksDbAddressBookStore create(
      Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    return new RocksDbAddressBookStore(metrics, nodeRustEnvironment);
  }

  private RocksDbAddressBookStore(Metrics metrics, NodeRustEnvironment nodeRustEnvironment) {
    final var timer = metrics.stateManager().nativeCall();
    removeOneFunc =
        Natives.builder(nodeRustEnvironment, RocksDbAddressBookStore::removeOne)
            .measure(timer.label(new Metrics.MethodId(RocksDbAddressBookStore.class, "removeOne")))
            .build(new TypeToken<>() {});
    upsertOneFunc =
        Natives.builder(nodeRustEnvironment, RocksDbAddressBookStore::upsertOne)
            .measure(timer.label(new Metrics.MethodId(RocksDbAddressBookStore.class, "upsertOne")))
            .build(new TypeToken<>() {});
    resetFunc =
        Natives.builder(nodeRustEnvironment, RocksDbAddressBookStore::reset)
            .measure(timer.label(new Metrics.MethodId(RocksDbAddressBookStore.class, "reset")))
            .build(new TypeToken<>() {});
    getAllFunc =
        Natives.builder(nodeRustEnvironment, RocksDbAddressBookStore::getAll)
            .measure(timer.label(new Metrics.MethodId(RocksDbAddressBookStore.class, "getAll")))
            .build(new TypeToken<>() {});

    dtoCodec = NodeSborCodecs.resolveCodec(new TypeToken<>() {});
  }

  boolean upsertEntry(AddressBookEntryDTO entry) {
    return this.upsertOneFunc.call(entry);
  }

  boolean removeEntry(NodeIdDTO nodeId) {
    return this.removeOneFunc.call(nodeId);
  }

  void reset() {
    this.resetFunc.call(Tuple0.of());
  }

  ImmutableList<AddressBookEntryDTO> getAllEntries() {
    return this.getAllFunc.call(tuple()).stream()
        .map(value -> NodeSborCodecs.decode(value, dtoCodec))
        .collect(ImmutableList.toImmutableList());
  }

  private final Natives.Call1<NodeIdDTO, Boolean> removeOneFunc;
  private final Natives.Call1<AddressBookEntryDTO, Boolean> upsertOneFunc;
  private final Natives.Call1<Tuple0, Tuple0> resetFunc;
  private final Natives.Call1<Tuple0, List<byte[]>> getAllFunc;
}
