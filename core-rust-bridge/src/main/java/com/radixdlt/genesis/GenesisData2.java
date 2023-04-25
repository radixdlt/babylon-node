package com.radixdlt.genesis;

import com.google.common.collect.ImmutableList;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record GenesisData2(ImmutableList<GenesisDataChunk> chunks) {
	public static void registerCodec(CodecMap codecMap) {
		codecMap.register(
			GenesisData2.class,
			codecs -> StructCodec.fromRecordComponents(GenesisData2.class, codecs));
	}
}
