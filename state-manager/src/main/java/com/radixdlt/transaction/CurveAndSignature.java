package com.radixdlt.transaction;

public record CurveAndSignature(
    Curve Curve,
    byte[] Signature
) {}