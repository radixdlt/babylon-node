package com.radixdlt.rev2;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.utils.KeyComparator;
import com.radixdlt.utils.PrivateKeys;

import java.util.List;

public final class ValidatorList {
    public static List<ECDSASecp256k1PublicKey> create(long num) {
        return PrivateKeys.numeric(1)
                .limit(num)
                .map(ECKeyPair::getPublicKey)
                .sorted(KeyComparator.instance())
                .toList();
    }
}
