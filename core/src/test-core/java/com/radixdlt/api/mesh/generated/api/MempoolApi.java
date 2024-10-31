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
import com.radixdlt.api.mesh.generated.models.MempoolResponse;
import com.radixdlt.api.mesh.generated.models.MempoolTransactionRequest;
import com.radixdlt.api.mesh.generated.models.MempoolTransactionResponse;
import com.radixdlt.api.mesh.generated.models.NetworkRequest;

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
public class MempoolApi {
  private final HttpClient memberVarHttpClient;
  private final ObjectMapper memberVarObjectMapper;
  private final String memberVarBaseUri;
  private final Consumer<HttpRequest.Builder> memberVarInterceptor;
  private final Duration memberVarReadTimeout;
  private final Consumer<HttpResponse<InputStream>> memberVarResponseInterceptor;
  private final Consumer<HttpResponse<String>> memberVarAsyncResponseInterceptor;

  public MempoolApi() {
    this(new ApiClient());
  }

  public MempoolApi(ApiClient apiClient) {
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
   * Get All Mempool Transactions
   * Get all Transaction Identifiers in the mempool
   * @param networkRequest  (required)
   * @return MempoolResponse
   * @throws ApiException if fails to make API call
   */
  public MempoolResponse mempool(NetworkRequest networkRequest) throws ApiException {
    ApiResponse<MempoolResponse> localVarResponse = mempoolWithHttpInfo(networkRequest);
    return localVarResponse.getData();
  }

  /**
   * Get All Mempool Transactions
   * Get all Transaction Identifiers in the mempool
   * @param networkRequest  (required)
   * @return ApiResponse&lt;MempoolResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<MempoolResponse> mempoolWithHttpInfo(NetworkRequest networkRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = mempoolRequestBuilder(networkRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("mempool", localVarResponse);
        }
        return new ApiResponse<MempoolResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<MempoolResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder mempoolRequestBuilder(NetworkRequest networkRequest) throws ApiException {
    // verify the required parameter 'networkRequest' is set
    if (networkRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'networkRequest' when calling mempool");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/mempool";

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
   * Get a Mempool Transaction
   * Get a transaction in the mempool by its Transaction Identifier. This is a separate request than fetching a block transaction (/block/transaction) because some blockchain nodes need to know that a transaction query is for something in the mempool instead of a transaction in a block.  Transactions may not be fully parsable until they are in a block (ex: may not be possible to determine the fee to pay before a transaction is executed). On this endpoint, it is ok that returned transactions are only estimates of what may actually be included in a block. 
   * @param mempoolTransactionRequest  (required)
   * @return MempoolTransactionResponse
   * @throws ApiException if fails to make API call
   */
  public MempoolTransactionResponse mempoolTransaction(MempoolTransactionRequest mempoolTransactionRequest) throws ApiException {
    ApiResponse<MempoolTransactionResponse> localVarResponse = mempoolTransactionWithHttpInfo(mempoolTransactionRequest);
    return localVarResponse.getData();
  }

  /**
   * Get a Mempool Transaction
   * Get a transaction in the mempool by its Transaction Identifier. This is a separate request than fetching a block transaction (/block/transaction) because some blockchain nodes need to know that a transaction query is for something in the mempool instead of a transaction in a block.  Transactions may not be fully parsable until they are in a block (ex: may not be possible to determine the fee to pay before a transaction is executed). On this endpoint, it is ok that returned transactions are only estimates of what may actually be included in a block. 
   * @param mempoolTransactionRequest  (required)
   * @return ApiResponse&lt;MempoolTransactionResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<MempoolTransactionResponse> mempoolTransactionWithHttpInfo(MempoolTransactionRequest mempoolTransactionRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = mempoolTransactionRequestBuilder(mempoolTransactionRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("mempoolTransaction", localVarResponse);
        }
        return new ApiResponse<MempoolTransactionResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<MempoolTransactionResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder mempoolTransactionRequestBuilder(MempoolTransactionRequest mempoolTransactionRequest) throws ApiException {
    // verify the required parameter 'mempoolTransactionRequest' is set
    if (mempoolTransactionRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'mempoolTransactionRequest' when calling mempoolTransaction");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/mempool/transaction";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(mempoolTransactionRequest);
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
