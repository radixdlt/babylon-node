package com.radixdlt.rev2;

import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.harness.simulation.application.TransactionGenerator;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawTransaction;
import com.radixdlt.utils.PrivateKeys;

public class REv2OneMBTransactionGenerator implements TransactionGenerator {
    private int currentKey = 1;

    @Override
    public RawTransaction nextTransaction() {
        final ECKeyPair key = PrivateKeys.numeric(currentKey++).findFirst().orElseThrow();
        var manifest = TransactionBuilder.build1MBManifest(key.getPublicKey());
        var hashedManifest = HashUtils.sha256Twice(manifest);
        var signedIntent =
                TransactionBuilder.createSignedIntentBytes(
                        manifest, key.getPublicKey(), key.sign(hashedManifest.asBytes()));
        var hashedSignedIntent = HashUtils.sha256Twice(signedIntent);
        var notarized =
                TransactionBuilder.createNotarizedBytes(
                        signedIntent, key.sign(hashedSignedIntent.asBytes()));
        return RawTransaction.create(notarized);
    }
}
