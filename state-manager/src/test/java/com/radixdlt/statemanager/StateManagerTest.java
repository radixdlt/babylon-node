package com.radixdlt.statemanager;

import com.radixdlt.crypto.ECKeyPair;
import org.junit.Test;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;

public final class StateManagerTest {

	@Test
	public void test_rust_interop() {
		final var key1 = ECKeyPair.generateNew().getPublicKey();
		final var stateManagerNode1 = StateManager.create(key1);

		final var key2 = ECKeyPair.generateNew().getPublicKey();
		final var stateManagerNode2 = StateManager.create(key2);

		assertEquals(key1, stateManagerNode1.getPublicKey());
		assertEquals(key1, stateManagerNode1.getPublicKey());
		assertEquals(key2, stateManagerNode2.getPublicKey());

		stateManagerNode1.insertTransaction(1L, new byte[] { 1, 2, 3 });
		assertArrayEquals(new byte[] { 1, 2, 3 }, stateManagerNode1.getTransactionAtStateVersion(1L));
	}
}
