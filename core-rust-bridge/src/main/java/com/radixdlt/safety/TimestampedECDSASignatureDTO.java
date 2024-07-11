package com.radixdlt.safety;

import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record TimestampedECDSASignatureDTO(long timestamp, ECDSASecp256k1Signature signature) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        TimestampedECDSASignatureDTO.class,
        codecs -> StructCodec.fromRecordComponents(TimestampedECDSASignatureDTO.class, codecs));
  }
}
