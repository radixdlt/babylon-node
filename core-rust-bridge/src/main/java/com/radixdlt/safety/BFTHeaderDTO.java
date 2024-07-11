package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import com.radixdlt.statecomputer.commit.LedgerHeader;

public record BFTHeaderDTO(RoundDTO round, VertexIdDTO vertexId, LedgerHeader ledgerHeader) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        BFTHeaderDTO.class, codecs -> StructCodec.fromRecordComponents(BFTHeaderDTO.class, codecs));
  }
}
