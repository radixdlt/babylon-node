package com.radixdlt.transaction;

import com.google.common.primitives.Bytes;
import com.google.common.reflect.TypeToken;
import com.radixdlt.crypto.ECDSASignature;
import com.radixdlt.crypto.ECPublicKey;
import com.radixdlt.exceptions.StateManagerRuntimeError;
import com.radixdlt.lang.Result;
import com.radixdlt.statemanager.StateManagerResponse;

public final class TransactionBuilder {
    private static final TypeToken<Result<byte[], StateManagerRuntimeError>> byteArrayType =
            new TypeToken<>() {};

    public static byte[] buildNewAccountManifest(ECPublicKey publicKey) {
        var encodedResponse = account(publicKey.getCompressedBytes());
        return StateManagerResponse.decode(encodedResponse, byteArrayType);
    }

    public static byte[] combineForNotary(byte[] manifest, ECPublicKey publicKey, ECDSASignature signature) {
        var signatureBytes = Bytes.concat(
                signature.getR().toByteArray(),
                signature.getS().toByteArray()
        );
        return combine(manifest, publicKey.getCompressedBytes(), signatureBytes);
    }

    private static native byte[] account(byte[] publicKey);
    private static native byte[] combine(byte[] manifest, byte[] publicKey, byte[] signature);
}
