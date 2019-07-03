package com.radixdlt.serialization;

import org.radix.network.messages.PeerPingMessage;

/**
 * Check serialization of PeerPingMessage
 */
public class PeerPingMessageSerializeTest extends SerializeMessageObject<PeerPingMessage> {
	public PeerPingMessageSerializeTest() {
		super(PeerPingMessage.class, PeerPingMessageSerializeTest::get);
	}

	private static PeerPingMessage get() {
		return new PeerPingMessage();
	}
}
