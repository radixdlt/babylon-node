/*
 * (C) Copyright 2021 Radix DLT Ltd
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

package com.radixdlt.client.api;

import org.json.JSONObject;

import com.radixdlt.utils.UInt256;

import static org.bouncycastle.util.encoders.Hex.toHexString;
import static org.radix.api.jsonrpc.JsonRpcUtil.jsonObject;

public class PreparedTransaction {
	private final byte[] blob;
	private final byte[] hashToSign;
	private final UInt256 fee;

	public PreparedTransaction(byte[] blob, byte[] hashToSign, UInt256 fee) {
		this.blob = blob;
		this.hashToSign = hashToSign;
		this.fee = fee;
	}

	public JSONObject asJson() {
		return jsonObject()
			.put(
				"transaction",
				jsonObject()
					.put("blob", toHexString(blob))
					.put("hashOfBlobToSign", toHexString(hashToSign))
			)
			.put("fee", fee.toString());
	}
}
