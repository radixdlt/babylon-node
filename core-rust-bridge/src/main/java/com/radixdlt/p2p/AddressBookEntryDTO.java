package com.radixdlt.p2p;

import com.radixdlt.lang.Option;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

import java.util.Set;

public record AddressBookEntryDTO(NodeIdDTO nodeId, Option<Long> bannedUntil, Set<String> knownAddresses) {
    public static void registerCodec(CodecMap codecMap) {
        codecMap.register(
            AddressBookEntryDTO.class,
            codecs -> StructCodec.fromRecordComponents(AddressBookEntryDTO.class, codecs));
    }
}
