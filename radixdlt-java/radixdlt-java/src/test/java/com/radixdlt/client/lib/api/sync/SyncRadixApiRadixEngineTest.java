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
package com.radixdlt.client.lib.api.sync;

import org.junit.Test;

import com.radixdlt.utils.functional.Result;

import java.io.IOException;

import okhttp3.Call;
import okhttp3.OkHttpClient;
import okhttp3.Response;
import okhttp3.ResponseBody;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import static com.radixdlt.client.lib.api.sync.RadixApi.DEFAULT_PRIMARY_PORT;
import static com.radixdlt.client.lib.api.sync.RadixApi.DEFAULT_SECONDARY_PORT;

public class SyncRadixApiRadixEngineTest {
	private static final String BASE_URL = "http://localhost/";

	private static final String CONFIGURATION = "{\"result\":{\"current_fork\":{\"name\":\"betanet1\",\"hash\":\"abcd\",\"minEpoch\":0,"
		+ "\"maxRounds\":1000},\"known_forks\":[{\"name\":\"betanet1\",\"hash\":\"abcd\",\"minEpoch\":0,"
		+ "\"maxRounds\":1000},{\"name\":\"betanet2\",\"hash\":\"cde\",\"minEpoch\":4,\"maxRounds\":1000},{\"name\":\"betanet3\",\"hash\":\"xyz\","
		+ "\"minEpoch\":8,\"maxRounds\":1000},{\"name\":\"betanet4\",\"hash\":\"www\",\"minEpoch\":10,\"maxRounds\":10000}],"
		+ "\"maxValidators\":100,\"minValidators\":1,\"maxTxnsPerProposal\":50},"
		+ "\"id\":\"1\",\"jsonrpc\":\"2.0\"}\n";
	private static final String DATA = "{\"result\":{\"invalidProposedCommands\":0,\"systemTransactions\":207536,"
		+ "\"userTransactions\":0},\"id\":\"1\",\"jsonrpc\":\"2.0\"}\n";

	private final OkHttpClient client = mock(OkHttpClient.class);

	@Test
	public void testConfiguration() throws IOException {
		prepareClient(CONFIGURATION)
			.map(RadixApi::withTrace)
			.onFailure(failure -> fail(failure.toString()))
			.onSuccess(client -> client.radixEngine().configuration()
				.onFailure(failure -> fail(failure.toString()))
				.onSuccess(configuration -> assertEquals(4, configuration.getKnownForks().size()))
				.onSuccess(configuration -> assertEquals("betanet4", configuration.getKnownForks().get(3).getName()))
			);
	}

	@Test
	public void testData() throws IOException {
		prepareClient(DATA)
			.map(RadixApi::withTrace)
			.onFailure(failure -> fail(failure.toString()))
			.onSuccess(
				client -> client.radixEngine().data()
					.onFailure(failure -> fail(failure.toString()))
					.onSuccess(data -> assertEquals(207536L, data.getSystemTransactions()))
			);
	}

	private Result<RadixApi> prepareClient(String responseBody) throws IOException {
		var call = mock(Call.class);
		var response = mock(Response.class);
		var body = mock(ResponseBody.class);

		when(client.newCall(any())).thenReturn(call);
		when(call.execute()).thenReturn(response);
		when(response.body()).thenReturn(body);
		when(body.string()).thenReturn(responseBody);

		return SyncRadixApi.connect(BASE_URL, DEFAULT_PRIMARY_PORT, DEFAULT_SECONDARY_PORT, client);
	}
}
