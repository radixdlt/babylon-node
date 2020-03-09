/*
 * (C) Copyright 2020 Radix DLT Ltd
 *
 * Radix DLT Ltd licenses this file to you under the Apache License,
 * Version 2.0 (the "License"); you may not use this file except in
 * compliance with the License.  You may obtain a copy of the
 * License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied.  See the License for the specific
 * language governing permissions and limitations under the License.
 */

package org.radix.serialization;

import com.radixdlt.common.AID;
import com.radixdlt.common.Atom;
import com.radixdlt.common.EUID;
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.Round;
import com.radixdlt.consensus.Vertex;
import com.radixdlt.consensus.VertexMetadata;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.messages.VertexMessage;
import com.radixdlt.utils.Ints;

public class VertexMessageSerializeTest extends SerializeMessageObject<VertexMessage> {
	public VertexMessageSerializeTest() {
		super(VertexMessage.class, VertexMessageSerializeTest::get);
	}

	private static VertexMessage get() {
		Round parentRound = Round.of(1234567890L);
		Round round = parentRound.next();
		AID parentAid = aidOf(12345);
		AID aid = aidOf(23456);
		EUID author = EUID.TWO;
		Atom atom = new Atom();

		VertexMetadata vertexMetadata = new VertexMetadata(round, aid, parentRound, parentAid);
		Vote vote = new Vote(author, vertexMetadata, signature);
		QuorumCertificate qc = new QuorumCertificate(vote, vertexMetadata);
		Vertex vertex = new Vertex(qc, round, atom);
		return new VertexMessage(1, vertex);
	}

	private static AID aidOf(int id) {
		byte[] bytes = new byte[AID.BYTES];
		Ints.copyTo(id, bytes, AID.BYTES - Integer.BYTES);
		return AID.from(bytes);
	}
}
