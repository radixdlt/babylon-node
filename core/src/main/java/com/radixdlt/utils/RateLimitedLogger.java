package com.radixdlt.utils;

import com.google.common.util.concurrent.RateLimiter;
import org.apache.logging.log4j.Logger;

import java.util.function.BiConsumer;
import java.util.function.BiFunction;

@SuppressWarnings("UnstableApiUsage")
public final class RateLimitedLogger {
    public interface REs {}
    record Permitted(int rejectionsSinceLastPermit) implements REs {}
    record Forbidden() implements REs {}

    private final Logger log;
    private final RateLimiter rateLimiter;
    private final int numSinceLastLog = 0;

    public RateLimitedLogger(Logger log, RateLimiter rateLimiter) {
        this.log = log;
        this.rateLimiter = rateLimiter;
    }

    public RateLimitedLogger(Logger log, double permitsPerSecond) {
        this(log, RateLimiter.create(permitsPerSecond));
    }


    public void logRateLimited(BiConsumer<Logger, Integer> thunk) {
    }


}
