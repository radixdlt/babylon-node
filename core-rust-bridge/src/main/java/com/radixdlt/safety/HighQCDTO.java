package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record HighQCDTO(
    QuorumCertificateDTO highestQC,
    QuorumCertificateDTO highestCommittedQC,
    TimeoutCertificateDTO highestTC) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        HighQCDTO.class, codecs -> StructCodec.fromRecordComponents(HighQCDTO.class, codecs));
  }
}
