package com.radixdlt.genesis;

import com.google.common.collect.ImmutableList;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.rev2.ComponentAddress2;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record GenesisValidator(
	ECDSASecp256k1PublicKey key,
	boolean acceptDelegatedStake,
	boolean isRegistered,
	ImmutableList<Tuple2<String, String>> metadata,
	ComponentAddress2 owner) {
	public static void registerCodec(CodecMap codecMap) {
		codecMap.register(
			GenesisValidator.class,
			codecs -> StructCodec.fromRecordComponents(GenesisValidator.class, codecs));
	}
}
