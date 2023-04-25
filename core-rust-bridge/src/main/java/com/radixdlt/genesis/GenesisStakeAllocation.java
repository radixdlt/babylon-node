package com.radixdlt.genesis;

import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import com.radixdlt.utils.UInt32;

public record GenesisStakeAllocation(UInt32 accountIndex, Decimal xrdAmount) {
	public static void registerCodec(CodecMap codecMap) {
		codecMap.register(
				GenesisStakeAllocation.class,
				codecs -> StructCodec.fromRecordComponents(GenesisStakeAllocation.class, codecs));

	}
}
