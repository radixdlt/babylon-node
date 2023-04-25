package com.radixdlt.genesis;

import com.google.common.collect.ImmutableList;
import com.google.common.reflect.TypeToken;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.ComponentAddress2;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.ResourceAddress;
import com.radixdlt.rev2.ResourceAddress2;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.EnumCodec;
import com.radixdlt.sbor.codec.EnumEntry;
import com.radixdlt.sbor.codec.Field;

public sealed interface GenesisDataChunk {
	static void registerCodec(CodecMap codecMap) {
		codecMap.registerForSealedClassAndSubclasses(
			GenesisDataChunk.class,
			(codecs) ->
				EnumCodec.fromEntries(
					EnumEntry.fromFields(
						Validators.class,
						Validators::new,
						Field.of(Validators::value, codecs.of(new TypeToken<>() { }))),
					EnumEntry.fromFields(
						Stakes.class,
						Stakes::new,
						Field.of(Stakes::accounts, codecs.of(new TypeToken<>() { })),
						Field.of(Stakes::allocations, codecs.of(new TypeToken<>() { }))),
					EnumEntry.fromFields(
						Resources.class,
						Resources::new,
						Field.of(Resources::value, codecs.of(new TypeToken<>() {}))),
					EnumEntry.fromFields(
						ResourceBalances.class,
						ResourceBalances::new,
						Field.of(ResourceBalances::accounts, codecs.of(new TypeToken<>() { })),
						Field.of(ResourceBalances::allocations, codecs.of(new TypeToken<>() { }))),
					EnumEntry.fromFields(
						XrdBalances.class,
						XrdBalances::new,
						Field.of(XrdBalances::value, codecs.of(new TypeToken<>() { })))));
	}

	record Validators(
		ImmutableList<GenesisValidator> value) implements GenesisDataChunk { }

	record Stakes(
		ImmutableList<ComponentAddress2> accounts,
		ImmutableList<Tuple2<ECDSASecp256k1PublicKey, ImmutableList<GenesisStakeAllocation>>> allocations
	) implements GenesisDataChunk { }

	record Resources(
		ImmutableList<GenesisResource> value) implements GenesisDataChunk { }

	record ResourceBalances(
			ImmutableList<ComponentAddress2> accounts,
			ImmutableList<Tuple2<ResourceAddress2, ImmutableList<GenesisResourceAllocation>>> allocations
	) implements GenesisDataChunk { }

	record XrdBalances(
			ImmutableList<Tuple2<ComponentAddress2, Decimal>> value
	) implements GenesisDataChunk { }
}
