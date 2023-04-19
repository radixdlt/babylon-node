/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

package com.radixdlt.genesis.olympia;

import com.google.inject.Inject;
import com.jcabi.http.request.JdkRequest;
import com.radixdlt.networks.Network;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.net.HttpURLConnection;
import java.net.MalformedURLException;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import org.json.JSONException;
import org.json.JSONObject;

public final class OlympiaEndStateApiClient {
  private static final String END_STATE_API_PATH = "/olympia-end-state";

  private static final String STATUS_FIELD = "status";
  private static final String STATUS_READY = "ready";
  private static final String STATUS_NOT_READY = "not_ready";

  public sealed interface OlympiaEndStateResponse {
    record Ready(String signature, String hash, String base64Contents)
        implements OlympiaEndStateResponse {}

    record NotReady(String signature, String placeholderHash) implements OlympiaEndStateResponse {}
  }

  private final URL endStateApiUrl;
  private final Network network;

  @Inject
  public OlympiaEndStateApiClient(OlympiaGenesisConfig config, Network network) {
    try {
      this.endStateApiUrl = new URL(config.nodeCoreApiUrl(), END_STATE_API_PATH);
    } catch (MalformedURLException e) {
      throw new RuntimeException(
          // Shouldn't really happen, nodeCoreApiUrl is verified earlier
          String.format(
              """
                  Failed to create OlympiaEndStateApiClient. \
                  Invalid endStateApiUrl (coreApiUrl=%s path=%s)""",
              config.nodeCoreApiUrl(), END_STATE_API_PATH));
    }
    this.network = network;
  }

  public OlympiaEndStateResponse getOlympiaEndState() throws IOException, JSONException {
    final var requestData =
        new JSONObject()
            .put(
                "network_identifier",
                new JSONObject().put("network", toOlympiaNetworkIdentifier(network)));

    final var response = postForJson(endStateApiUrl, requestData);

    final var status = response.getString(STATUS_FIELD);

    switch (status) {
      case STATUS_READY -> {
        return new OlympiaEndStateResponse.Ready(
            response.getString("signature"),
            response.getString("hash"),
            response.getString("contents"));
      }
      case STATUS_NOT_READY -> {
        return new OlympiaEndStateResponse.NotReady(
            response.getString("signature"), response.getString("placeholder_hash"));
      }
      default -> throw new RuntimeException("Unexpected response status " + status);
    }
  }

  private static JSONObject postForJson(URL url, JSONObject request) throws IOException {
    // TODO: this may take a longer while, may need to change the default request timeouts?
    final var response =
        new JdkRequest(url)
            .header("Content-Type", "application/json")
            .method(JdkRequest.POST)
            .fetch(new ByteArrayInputStream(request.toString().getBytes(StandardCharsets.UTF_8)));

    if (response.status() != HttpURLConnection.HTTP_OK) {
      throw new RuntimeException(
          String.format(
              "The request didn't succeed. Expected status code %s but got %s",
              HttpURLConnection.HTTP_OK, response.status()));
    }

    return new JSONObject(response.body());
  }

  private String toOlympiaNetworkIdentifier(Network network) {
    return network.getLogicalName();
  }
}
