package com.radixdlt.p2p;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;

public record NodeIdDTO(ECDSASecp256k1PublicKey publicKey) {
    public static void registerCodec(CodecMap codecMap) {
        codecMap.register(
            NodeIdDTO.class,
            codecs -> StructCodec.fromRecordComponents(NodeIdDTO.class, codecs));
    }
}
