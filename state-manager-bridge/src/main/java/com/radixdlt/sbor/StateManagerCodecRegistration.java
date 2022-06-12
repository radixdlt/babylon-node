package com.radixdlt.sbor;

import com.radixdlt.mempool.MempoolError;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.statemanager.StateManagerRuntimeError;

public final class StateManagerCodecRegistration {
    public static void registerCodecs(CodecMap codecMap) {
        StateManagerRuntimeError.registerCodec(codecMap);
        MempoolError.registerCodec(codecMap);
    }
}
