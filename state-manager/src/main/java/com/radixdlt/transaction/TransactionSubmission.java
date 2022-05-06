package com.radixdlt.transaction;

import com.radixdlt.identifiers.AID;

public record TransactionSubmission(
    AID TransactionSubmissionIdentifierHash,
    SignedTransaction SignedTransaction,
    CurveAndKey NotarySignature
) {}