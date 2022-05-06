package com.radixdlt.transaction;

import com.radixdlt.identifiers.AID;

public record SignedTransaction(
    AID TransactionIntentIdentifierHash,
    byte[] TransactionIntent,
    CurveAndSignature[] Signatures
) {}