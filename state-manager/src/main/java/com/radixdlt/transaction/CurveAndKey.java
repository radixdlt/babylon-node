package com.radixdlt.transaction;

public record CurveAndKey(
    Curve Curve,
    byte[] Key
) {}