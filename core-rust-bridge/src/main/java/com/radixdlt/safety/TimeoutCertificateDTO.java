package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record TimeoutCertificateDTO(
    long epoch, RoundDTO round, TimestampedECDSASignaturesDTO signatures) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        TimeoutCertificateDTO.class,
        codecs -> StructCodec.fromRecordComponents(TimeoutCertificateDTO.class, codecs));
  }
}
