package com.radixdlt.genesis;

import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import com.radixdlt.utils.UInt32;

public record GenesisResourceAllocation(UInt32 accountIndex, Decimal amount) {

	public static void registerCodec(CodecMap codecMap) {
		codecMap.register(
				GenesisResourceAllocation.class,
				codecs -> StructCodec.fromRecordComponents(GenesisResourceAllocation.class, codecs));

	}
}
