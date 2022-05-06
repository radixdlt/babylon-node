package com.radixdlt.transaction;

public record TransactionHeader(
    byte TransactionSchemaVersion,
    byte Network,
    long MinEpoch,
    long MaxEpoch,
    byte[] NotaryCurveAndKey // Needs to change
) {}