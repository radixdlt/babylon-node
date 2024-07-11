package com.radixdlt.safety;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

/** Representation of the validator suitable for Java-Rust exchange. */
public record BFTValidatorIdDTO(
    ECDSASecp256k1PublicKey key, ComponentAddress validatorAddress, String shortenedName) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        BFTValidatorIdDTO.class,
        codecs -> StructCodec.fromRecordComponents(BFTValidatorIdDTO.class, codecs));
  }
}
