package com.radixdlt.statemanager;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.EnumCodec;
import com.radixdlt.sbor.codec.EnumEntry;

public sealed interface REv2DatabaseConfig {
    static void registerCodec(CodecMap codecMap) {
        codecMap.register(
                REv2DatabaseConfig.class,
                (codecs) ->
                        EnumCodec.fromEntries(
                                EnumEntry.noFields(
                                        REv2DatabaseConfig.InMemory.class,
                                        REv2DatabaseConfig.InMemory::new)

                                        ));
    }

    record InMemory() implements REv2DatabaseConfig {}
}
