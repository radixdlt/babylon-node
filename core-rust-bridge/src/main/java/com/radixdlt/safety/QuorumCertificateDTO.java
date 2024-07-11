package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record QuorumCertificateDTO(TimestampedECDSASignaturesDTO signatures, VoteDataDTO voteData) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        QuorumCertificateDTO.class,
        codecs -> StructCodec.fromRecordComponents(QuorumCertificateDTO.class, codecs));
  }
}
