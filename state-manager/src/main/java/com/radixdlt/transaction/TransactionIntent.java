package com.radixdlt.transaction;

public record TransactionIntent(
    TransactionHeader TransactionHeader,
    byte[] CompiledManifest,
    byte[][] BinaryBlobs
) {}