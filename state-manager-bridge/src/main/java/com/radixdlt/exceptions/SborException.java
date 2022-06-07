package com.radixdlt.exceptions;

public class SborException extends RuntimeException {
    /**
     * @param clazz - The class being encoded/decoded
     * @param isJavaSide - If the error happened on java side or rust side
     * @param isEncoding - If the error happened whilst encoding or decoding
     * @param message - The message from the Cause
     */
    public SborException(Class clazz, boolean isJavaSide, boolean isEncoding, String message) {
        super(message);
    }
}
