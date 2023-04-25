package com.radixdlt.genesis;

import com.google.common.collect.ImmutableList;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.rev2.ComponentAddress2;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record GenesisResource(
	byte[] addressBytesWithoutEntityId,
	Decimal initialSupply,
	ImmutableList<Tuple2<String, String>> metadata,
	Option<ComponentAddress2> owner) {
	public static void registerCodec(CodecMap codecMap) {
		codecMap.register(
				GenesisResource.class,
				codecs -> StructCodec.fromRecordComponents(GenesisResource.class, codecs));
	}
}
