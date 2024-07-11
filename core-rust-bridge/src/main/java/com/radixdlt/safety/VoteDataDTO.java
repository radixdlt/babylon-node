package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record VoteDataDTO(BFTHeaderDTO proposed, BFTHeaderDTO parent, BFTHeaderDTO committed) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        VoteDataDTO.class, codecs -> StructCodec.fromRecordComponents(VoteDataDTO.class, codecs));
  }
}
