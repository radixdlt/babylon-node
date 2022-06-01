package com.radixdlt.lang;

/**
 * Basic interface for failure cause types.
 */
public interface Cause {
    /**
     * Message associated with the failure.
     */
    String message();

    /**
     * Represent cause as a failure {@link Result} instance.
     *
     * @return cause converted into {@link Result} with necessary type.
     */
    default <T> Result<T> result() {
        return Result.failure(this);
    }
}
