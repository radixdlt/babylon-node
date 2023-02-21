package com.radixdlt.crypto;

import com.radixdlt.SecurityCritical;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

/** Collection of Hashing methods using BLAKE-2B family. */
@SecurityCritical(SecurityCritical.SecurityKind.HASHING)
public final class Blake2BHashHandler {
    Blake2BHashHandler() {
        // Nothing to do here
    }

    private final ThreadLocal<MessageDigest> blake2BDigester =
            ThreadLocal.withInitial(() -> getDigester("BLAKE2B-256"));

    public byte[] blake2b256(byte[] data, int offset, int length) {
        final MessageDigest blake2bDigesterLocal = blake2BDigester.get();
        blake2bDigesterLocal.reset();
        blake2bDigesterLocal.update(data, offset, length);
        return blake2bDigesterLocal.digest();
    }

    private static MessageDigest getDigester(String algorithm) {
        try {
            return MessageDigest.getInstance(algorithm);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalArgumentException("No such algorithm: " + algorithm, e);
        }
    }
}
