package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import java.util.Map;

public record TimestampedECDSASignaturesDTO(
    Map<BFTValidatorIdDTO, TimestampedECDSASignatureDTO> nodeToTimestampedSignature) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        TimestampedECDSASignaturesDTO.class,
        codecs -> StructCodec.fromRecordComponents(TimestampedECDSASignaturesDTO.class, codecs));
  }
}
