/*
 * Rosetta
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

package com.radixdlt.api.mesh.generated.api;

import com.radixdlt.api.mesh.generated.client.ApiClient;
import com.radixdlt.api.mesh.generated.client.ApiException;
import com.radixdlt.api.mesh.generated.client.ApiResponse;
import com.radixdlt.api.mesh.generated.client.Pair;

import com.radixdlt.api.mesh.generated.models.Error;
import com.radixdlt.api.mesh.generated.models.MetadataRequest;
import com.radixdlt.api.mesh.generated.models.NetworkListResponse;
import com.radixdlt.api.mesh.generated.models.NetworkOptionsResponse;
import com.radixdlt.api.mesh.generated.models.NetworkRequest;
import com.radixdlt.api.mesh.generated.models.NetworkStatusResponse;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;

import java.util.ArrayList;
import java.util.StringJoiner;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Consumer;

@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NetworkApi {
  private final HttpClient memberVarHttpClient;
  private final ObjectMapper memberVarObjectMapper;
  private final String memberVarBaseUri;
  private final Consumer<HttpRequest.Builder> memberVarInterceptor;
  private final Duration memberVarReadTimeout;
  private final Consumer<HttpResponse<InputStream>> memberVarResponseInterceptor;
  private final Consumer<HttpResponse<String>> memberVarAsyncResponseInterceptor;

  public NetworkApi() {
    this(new ApiClient());
  }

  public NetworkApi(ApiClient apiClient) {
    memberVarHttpClient = apiClient.getHttpClient();
    memberVarObjectMapper = apiClient.getObjectMapper();
    memberVarBaseUri = apiClient.getBaseUri();
    memberVarInterceptor = apiClient.getRequestInterceptor();
    memberVarReadTimeout = apiClient.getReadTimeout();
    memberVarResponseInterceptor = apiClient.getResponseInterceptor();
    memberVarAsyncResponseInterceptor = apiClient.getAsyncResponseInterceptor();
  }

  protected ApiException getApiException(String operationId, HttpResponse<InputStream> response) throws IOException {
    String body = response.body() == null ? null : new String(response.body().readAllBytes());
    String message = formatExceptionMessage(operationId, response.statusCode(), body);
    return new ApiException(response.statusCode(), message, response.headers(), body);
  }

  private String formatExceptionMessage(String operationId, int statusCode, String body) {
    if (body == null || body.isEmpty()) {
      body = "[no body]";
    }
    return operationId + " call failed with: " + statusCode + " - " + body;
  }

  /**
   * Get List of Available Networks
   * This endpoint returns a list of NetworkIdentifiers that the Rosetta server supports. 
   * @param metadataRequest  (required)
   * @return NetworkListResponse
   * @throws ApiException if fails to make API call
   */
  public NetworkListResponse networkList(MetadataRequest metadataRequest) throws ApiException {
    ApiResponse<NetworkListResponse> localVarResponse = networkListWithHttpInfo(metadataRequest);
    return localVarResponse.getData();
  }

  /**
   * Get List of Available Networks
   * This endpoint returns a list of NetworkIdentifiers that the Rosetta server supports. 
   * @param metadataRequest  (required)
   * @return ApiResponse&lt;NetworkListResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<NetworkListResponse> networkListWithHttpInfo(MetadataRequest metadataRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = networkListRequestBuilder(metadataRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("networkList", localVarResponse);
        }
        return new ApiResponse<NetworkListResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<NetworkListResponse>() {}) // closes the InputStream
          
        );
      } finally {
      }
    } catch (IOException e) {
      throw new ApiException(e);
    }
    catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new ApiException(e);
    }
  }

  private HttpRequest.Builder networkListRequestBuilder(MetadataRequest metadataRequest) throws ApiException {
    // verify the required parameter 'metadataRequest' is set
    if (metadataRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'metadataRequest' when calling networkList");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/network/list";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(metadataRequest);
      localVarRequestBuilder.method("POST", HttpRequest.BodyPublishers.ofByteArray(localVarPostBody));
    } catch (IOException e) {
      throw new ApiException(e);
    }
    if (memberVarReadTimeout != null) {
      localVarRequestBuilder.timeout(memberVarReadTimeout);
    }
    if (memberVarInterceptor != null) {
      memberVarInterceptor.accept(localVarRequestBuilder);
    }
    return localVarRequestBuilder;
  }
  /**
   * Get Network Options
   * This endpoint returns the version information and allowed network-specific types for a NetworkIdentifier. Any NetworkIdentifier returned by /network/list should be accessible here.  Because options are retrievable in the context of a NetworkIdentifier, it is possible to define unique options for each network. 
   * @param networkRequest  (required)
   * @return NetworkOptionsResponse
   * @throws ApiException if fails to make API call
   */
  public NetworkOptionsResponse networkOptions(NetworkRequest networkRequest) throws ApiException {
    ApiResponse<NetworkOptionsResponse> localVarResponse = networkOptionsWithHttpInfo(networkRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Network Options
   * This endpoint returns the version information and allowed network-specific types for a NetworkIdentifier. Any NetworkIdentifier returned by /network/list should be accessible here.  Because options are retrievable in the context of a NetworkIdentifier, it is possible to define unique options for each network. 
   * @param networkRequest  (required)
   * @return ApiResponse&lt;NetworkOptionsResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<NetworkOptionsResponse> networkOptionsWithHttpInfo(NetworkRequest networkRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = networkOptionsRequestBuilder(networkRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("networkOptions", localVarResponse);
        }
        return new ApiResponse<NetworkOptionsResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<NetworkOptionsResponse>() {}) // closes the InputStream
          
        );
      } finally {
      }
    } catch (IOException e) {
      throw new ApiException(e);
    }
    catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new ApiException(e);
    }
  }

  private HttpRequest.Builder networkOptionsRequestBuilder(NetworkRequest networkRequest) throws ApiException {
    // verify the required parameter 'networkRequest' is set
    if (networkRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'networkRequest' when calling networkOptions");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/network/options";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(networkRequest);
      localVarRequestBuilder.method("POST", HttpRequest.BodyPublishers.ofByteArray(localVarPostBody));
    } catch (IOException e) {
      throw new ApiException(e);
    }
    if (memberVarReadTimeout != null) {
      localVarRequestBuilder.timeout(memberVarReadTimeout);
    }
    if (memberVarInterceptor != null) {
      memberVarInterceptor.accept(localVarRequestBuilder);
    }
    return localVarRequestBuilder;
  }
  /**
   * Get Network Status
   * This endpoint returns the current status of the network requested. Any NetworkIdentifier returned by /network/list should be accessible here. 
   * @param networkRequest  (required)
   * @return NetworkStatusResponse
   * @throws ApiException if fails to make API call
   */
  public NetworkStatusResponse networkStatus(NetworkRequest networkRequest) throws ApiException {
    ApiResponse<NetworkStatusResponse> localVarResponse = networkStatusWithHttpInfo(networkRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Network Status
   * This endpoint returns the current status of the network requested. Any NetworkIdentifier returned by /network/list should be accessible here. 
   * @param networkRequest  (required)
   * @return ApiResponse&lt;NetworkStatusResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<NetworkStatusResponse> networkStatusWithHttpInfo(NetworkRequest networkRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = networkStatusRequestBuilder(networkRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("networkStatus", localVarResponse);
        }
        return new ApiResponse<NetworkStatusResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<NetworkStatusResponse>() {}) // closes the InputStream
          
        );
      } finally {
      }
    } catch (IOException e) {
      throw new ApiException(e);
    }
    catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new ApiException(e);
    }
  }

  private HttpRequest.Builder networkStatusRequestBuilder(NetworkRequest networkRequest) throws ApiException {
    // verify the required parameter 'networkRequest' is set
    if (networkRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'networkRequest' when calling networkStatus");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/network/status";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(networkRequest);
      localVarRequestBuilder.method("POST", HttpRequest.BodyPublishers.ofByteArray(localVarPostBody));
    } catch (IOException e) {
      throw new ApiException(e);
    }
    if (memberVarReadTimeout != null) {
      localVarRequestBuilder.timeout(memberVarReadTimeout);
    }
    if (memberVarInterceptor != null) {
      memberVarInterceptor.accept(localVarRequestBuilder);
    }
    return localVarRequestBuilder;
  }
}
