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

package com.radixdlt.api;

import static org.assertj.core.api.Assertions.catchThrowableOfType;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.google.common.reflect.ClassPath;
import com.google.inject.AbstractModule;
import com.google.inject.Module;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.api.mesh.generated.api.*;
import com.radixdlt.api.mesh.generated.client.ApiClient;
import com.radixdlt.api.mesh.generated.client.ApiException;
import com.radixdlt.api.mesh.generated.models.*;
import com.radixdlt.environment.StartProcessorOnRunner;
import com.radixdlt.monitoring.ApplicationVersion;
import com.radixdlt.networks.Network;
import com.radixdlt.utils.FreePortFinder;
import java.net.http.HttpClient;
import org.assertj.core.api.ThrowableAssert;

public class MeshApiHelper {

  private final int meshApiPort;
  private final Network network;
  private final ApiClient apiClient;

  static {
    ensureOpenApiModelsAreReady();
  }

  public static MeshApiHelper forTests() {
    return new MeshApiHelper(Network.INTEGRATIONTESTNET);
  }

  public MeshApiHelper(Network network) {
    this.meshApiPort = FreePortFinder.findFreeLocalPort();
    this.network = network;
    final var apiClient = new ApiClient();
    apiClient.updateBaseUri("http://127.0.0.1:" + meshApiPort + "/mesh");
    apiClient.setHttpClientBuilder(
        HttpClient.newBuilder().sslContext(DummySslContextFactory.create()));
    this.apiClient = apiClient;
  }

  private static void ensureOpenApiModelsAreReady() {
    /* The generated Open API models are rubbish and requires that static initializers run on models before
     * deserialization to work correctly... But that doesn't happen in e.g. models under the response model in
     * assertErrorResponseOfType.
     * As a workaround for now, let's go through all the types and explicitly ensure their static initializers run
     * by using the Class.forName method.
     */
    try {
      ClassPath.from(ClassLoader.getSystemClassLoader()).getAllClasses().stream()
          .filter(clazz -> clazz.getPackageName().equals("com.radixdlt.api.mesh.generated.models"))
          .forEach(
              clazz -> {
                try {
                  Class.forName(clazz.getName());
                } catch (Exception ex) {
                  throw new RuntimeException(ex);
                }
              });
    } catch (Exception ex) {
      throw new RuntimeException(ex);
    }
  }

  public Module module() {
    return new AbstractModule() {
      @Override
      protected void configure() {
        install(
            new MeshApiServerModule(
                "127.0.0.1", meshApiPort, ApplicationVersion.INSTANCE.display()));
      }

      @ProvidesIntoSet
      private StartProcessorOnRunner startCoreApi(MeshApiServer meshApiServer) {
        // This is a slightly hacky way to run something on node start-up in a Deterministic test.
        // Stop is called by the AutoClosable binding in CoreApiServerModule
        return new StartProcessorOnRunner("meshApi", meshApiServer::start);
      }
    };
  }

  public ApiClient client() {
    return this.apiClient;
  }

  public MempoolApi mempoolApi() {
    return new MempoolApi(client());
  }

  public NetworkIdentifier networkIdentifier() {
    return new NetworkIdentifier().blockchain("radix").network(this.network.getLogicalName());
  }

  public <Response> Response assertErrorResponseOfType(
      ThrowableAssert.ThrowingCallable apiCall, Class<Response> responseClass) {
    var apiException = catchThrowableOfType(apiCall, ApiException.class);
    try {
      return apiClient.getObjectMapper().readValue(apiException.getResponseBody(), responseClass);
    } catch (JsonProcessingException ex) {
      throw new RuntimeException(ex);
    }
  }
}
