package com.radixdlt.safety;

import com.radixdlt.lang.Option;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record SafetyStateDTO(BFTValidatorIdDTO validatorId, RoundDTO round, Option<VoteDTO> lastVote) {
    public static void registerCodec(CodecMap codecMap) {
        codecMap.register(
            SafetyStateDTO.class,
            codecs -> StructCodec.fromRecordComponents(SafetyStateDTO.class, codecs));
    }
}
