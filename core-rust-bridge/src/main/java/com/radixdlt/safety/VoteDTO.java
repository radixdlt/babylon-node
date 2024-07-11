package com.radixdlt.safety;

import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.lang.Option;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record VoteDTO(
    BFTValidatorIdDTO author,
    HighQCDTO highQC,
    VoteDataDTO voteData,
    long timestamp,
    ECDSASecp256k1Signature signature,
    Option<ECDSASecp256k1Signature> timeoutSignature) {
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        VoteDTO.class, codecs -> StructCodec.fromRecordComponents(VoteDTO.class, codecs));
  }
}
