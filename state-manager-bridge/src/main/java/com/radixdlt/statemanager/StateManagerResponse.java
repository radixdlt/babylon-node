package com.radixdlt.statemanager;

import com.google.common.reflect.TypeToken;
import com.radixdlt.exceptions.StateManagerRuntimeException;
import com.radixdlt.lang.Result;
import com.radixdlt.sbor.TypedSbor;

public final class StateManagerResponse {

    public static <T> T decode(byte[] stateManagerResponse, TypeToken<Result<T, StateManagerRuntimeError>> type) {
        var result = TypedSbor.decode(stateManagerResponse, type);

        // Handle System/Runtime Errors
        if (result.isErr()) {
            throw new StateManagerRuntimeException(result.unwrapErr());
        }

        return result.unwrap();
    }
}
