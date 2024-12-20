/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

package com.radixdlt.api.engine_state.generated.api;

import com.radixdlt.api.engine_state.generated.client.ApiClient;
import com.radixdlt.api.engine_state.generated.client.ApiException;
import com.radixdlt.api.engine_state.generated.client.ApiResponse;
import com.radixdlt.api.engine_state.generated.client.Pair;

import com.radixdlt.api.engine_state.generated.models.ErrorResponse;
import com.radixdlt.api.engine_state.generated.models.ObjectCollectionEntryRequest;
import com.radixdlt.api.engine_state.generated.models.ObjectCollectionEntryResponse;
import com.radixdlt.api.engine_state.generated.models.ObjectCollectionIteratorRequest;
import com.radixdlt.api.engine_state.generated.models.ObjectCollectionIteratorResponse;
import com.radixdlt.api.engine_state.generated.models.ObjectFieldRequest;
import com.radixdlt.api.engine_state.generated.models.ObjectFieldResponse;

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
public class ObjectsApi {
  private final HttpClient memberVarHttpClient;
  private final ObjectMapper memberVarObjectMapper;
  private final String memberVarBaseUri;
  private final Consumer<HttpRequest.Builder> memberVarInterceptor;
  private final Duration memberVarReadTimeout;
  private final Consumer<HttpResponse<InputStream>> memberVarResponseInterceptor;
  private final Consumer<HttpResponse<String>> memberVarAsyncResponseInterceptor;

  public ObjectsApi() {
    this(new ApiClient());
  }

  public ObjectsApi(ApiClient apiClient) {
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
   * Get Object Collection Entry
   * Reads the current value of a specific entry from an Object&#39;s Collection. 
   * @param objectCollectionEntryRequest  (required)
   * @return ObjectCollectionEntryResponse
   * @throws ApiException if fails to make API call
   */
  public ObjectCollectionEntryResponse objectCollectionEntryPost(ObjectCollectionEntryRequest objectCollectionEntryRequest) throws ApiException {
    ApiResponse<ObjectCollectionEntryResponse> localVarResponse = objectCollectionEntryPostWithHttpInfo(objectCollectionEntryRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Object Collection Entry
   * Reads the current value of a specific entry from an Object&#39;s Collection. 
   * @param objectCollectionEntryRequest  (required)
   * @return ApiResponse&lt;ObjectCollectionEntryResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<ObjectCollectionEntryResponse> objectCollectionEntryPostWithHttpInfo(ObjectCollectionEntryRequest objectCollectionEntryRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = objectCollectionEntryPostRequestBuilder(objectCollectionEntryRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("objectCollectionEntryPost", localVarResponse);
        }
        return new ApiResponse<ObjectCollectionEntryResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<ObjectCollectionEntryResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder objectCollectionEntryPostRequestBuilder(ObjectCollectionEntryRequest objectCollectionEntryRequest) throws ApiException {
    // verify the required parameter 'objectCollectionEntryRequest' is set
    if (objectCollectionEntryRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'objectCollectionEntryRequest' when calling objectCollectionEntryPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/object/collection/entry";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(objectCollectionEntryRequest);
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
   * List Object Collection
   * Lists keys of all entries from a particular object&#39;s collection, in an iterator-like paged fashion
   * @param objectCollectionIteratorRequest  (required)
   * @return ObjectCollectionIteratorResponse
   * @throws ApiException if fails to make API call
   */
  public ObjectCollectionIteratorResponse objectCollectionIteratorPost(ObjectCollectionIteratorRequest objectCollectionIteratorRequest) throws ApiException {
    ApiResponse<ObjectCollectionIteratorResponse> localVarResponse = objectCollectionIteratorPostWithHttpInfo(objectCollectionIteratorRequest);
    return localVarResponse.getData();
  }

  /**
   * List Object Collection
   * Lists keys of all entries from a particular object&#39;s collection, in an iterator-like paged fashion
   * @param objectCollectionIteratorRequest  (required)
   * @return ApiResponse&lt;ObjectCollectionIteratorResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<ObjectCollectionIteratorResponse> objectCollectionIteratorPostWithHttpInfo(ObjectCollectionIteratorRequest objectCollectionIteratorRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = objectCollectionIteratorPostRequestBuilder(objectCollectionIteratorRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("objectCollectionIteratorPost", localVarResponse);
        }
        return new ApiResponse<ObjectCollectionIteratorResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<ObjectCollectionIteratorResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder objectCollectionIteratorPostRequestBuilder(ObjectCollectionIteratorRequest objectCollectionIteratorRequest) throws ApiException {
    // verify the required parameter 'objectCollectionIteratorRequest' is set
    if (objectCollectionIteratorRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'objectCollectionIteratorRequest' when calling objectCollectionIteratorPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/object/collection/iterator";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(objectCollectionIteratorRequest);
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
   * Get Object Field
   * Reads the current value of an object&#39;s field, given an entity address, a module (&#x60;Main&#x60; by default) and either a field index or its human-readable name (if applicable). 
   * @param objectFieldRequest  (required)
   * @return ObjectFieldResponse
   * @throws ApiException if fails to make API call
   */
  public ObjectFieldResponse objectFieldPost(ObjectFieldRequest objectFieldRequest) throws ApiException {
    ApiResponse<ObjectFieldResponse> localVarResponse = objectFieldPostWithHttpInfo(objectFieldRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Object Field
   * Reads the current value of an object&#39;s field, given an entity address, a module (&#x60;Main&#x60; by default) and either a field index or its human-readable name (if applicable). 
   * @param objectFieldRequest  (required)
   * @return ApiResponse&lt;ObjectFieldResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<ObjectFieldResponse> objectFieldPostWithHttpInfo(ObjectFieldRequest objectFieldRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = objectFieldPostRequestBuilder(objectFieldRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("objectFieldPost", localVarResponse);
        }
        return new ApiResponse<ObjectFieldResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<ObjectFieldResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder objectFieldPostRequestBuilder(ObjectFieldRequest objectFieldRequest) throws ApiException {
    // verify the required parameter 'objectFieldRequest' is set
    if (objectFieldRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'objectFieldRequest' when calling objectFieldPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/object/field";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(objectFieldRequest);
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
