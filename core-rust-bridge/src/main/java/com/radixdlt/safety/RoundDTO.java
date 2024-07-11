package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record RoundDTO(long round) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        RoundDTO.class, codecs -> StructCodec.fromRecordComponents(RoundDTO.class, codecs));
  }
}
