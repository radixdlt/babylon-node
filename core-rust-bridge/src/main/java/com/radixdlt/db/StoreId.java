package com.radixdlt.db;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.EnumCodec;

/**
 * Java side representation of the StoreId enum in Rust
  */
public sealed interface StoreId {
  record AddressBookStoreId() implements StoreId {}

  record SafetyStoreId() implements StoreId {}

  AddressBookStoreId ADDRESS_BOOK = new AddressBookStoreId();
  SafetyStoreId SAFETY_STORE = new SafetyStoreId();

  static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        StoreId.class, codecs -> EnumCodec.fromPermittedRecordSubclasses(StoreId.class, codecs));
  }
}
