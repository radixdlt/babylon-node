/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

package com.radixdlt.api.core.generated.api;

import com.radixdlt.api.core.generated.client.ApiClient;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.client.ApiResponse;
import com.radixdlt.api.core.generated.client.Pair;

import com.radixdlt.api.core.generated.models.BasicErrorResponse;
import com.radixdlt.api.core.generated.models.LtsStateAccountAllFungibleResourceBalancesRequest;
import com.radixdlt.api.core.generated.models.LtsStateAccountAllFungibleResourceBalancesResponse;
import com.radixdlt.api.core.generated.models.LtsStateAccountFungibleResourceBalanceRequest;
import com.radixdlt.api.core.generated.models.LtsStateAccountFungibleResourceBalanceResponse;
import com.radixdlt.api.core.generated.models.LtsStreamAccountTransactionOutcomesRequest;
import com.radixdlt.api.core.generated.models.LtsStreamAccountTransactionOutcomesResponse;
import com.radixdlt.api.core.generated.models.LtsStreamTransactionOutcomesRequest;
import com.radixdlt.api.core.generated.models.LtsStreamTransactionOutcomesResponse;
import com.radixdlt.api.core.generated.models.LtsTransactionConstructionRequest;
import com.radixdlt.api.core.generated.models.LtsTransactionConstructionResponse;
import com.radixdlt.api.core.generated.models.LtsTransactionStatusRequest;
import com.radixdlt.api.core.generated.models.LtsTransactionStatusResponse;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitRequest;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitResponse;
import com.radixdlt.api.core.generated.models.TransactionSubmitErrorResponse;

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
public class LtsApi {
  private final HttpClient memberVarHttpClient;
  private final ObjectMapper memberVarObjectMapper;
  private final String memberVarBaseUri;
  private final Consumer<HttpRequest.Builder> memberVarInterceptor;
  private final Duration memberVarReadTimeout;
  private final Consumer<HttpResponse<InputStream>> memberVarResponseInterceptor;
  private final Consumer<HttpResponse<String>> memberVarAsyncResponseInterceptor;

  public LtsApi() {
    this(new ApiClient());
  }

  public LtsApi(ApiClient apiClient) {
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
   * Get All Account Balances
   * Returns balances for all resources associated with an account
   * @param ltsStateAccountAllFungibleResourceBalancesRequest  (required)
   * @return LtsStateAccountAllFungibleResourceBalancesResponse
   * @throws ApiException if fails to make API call
   */
  public LtsStateAccountAllFungibleResourceBalancesResponse ltsStateAccountAllFungibleResourceBalancesPost(LtsStateAccountAllFungibleResourceBalancesRequest ltsStateAccountAllFungibleResourceBalancesRequest) throws ApiException {
    ApiResponse<LtsStateAccountAllFungibleResourceBalancesResponse> localVarResponse = ltsStateAccountAllFungibleResourceBalancesPostWithHttpInfo(ltsStateAccountAllFungibleResourceBalancesRequest);
    return localVarResponse.getData();
  }

  /**
   * Get All Account Balances
   * Returns balances for all resources associated with an account
   * @param ltsStateAccountAllFungibleResourceBalancesRequest  (required)
   * @return ApiResponse&lt;LtsStateAccountAllFungibleResourceBalancesResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsStateAccountAllFungibleResourceBalancesResponse> ltsStateAccountAllFungibleResourceBalancesPostWithHttpInfo(LtsStateAccountAllFungibleResourceBalancesRequest ltsStateAccountAllFungibleResourceBalancesRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsStateAccountAllFungibleResourceBalancesPostRequestBuilder(ltsStateAccountAllFungibleResourceBalancesRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsStateAccountAllFungibleResourceBalancesPost", localVarResponse);
        }
        return new ApiResponse<LtsStateAccountAllFungibleResourceBalancesResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsStateAccountAllFungibleResourceBalancesResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsStateAccountAllFungibleResourceBalancesPostRequestBuilder(LtsStateAccountAllFungibleResourceBalancesRequest ltsStateAccountAllFungibleResourceBalancesRequest) throws ApiException {
    // verify the required parameter 'ltsStateAccountAllFungibleResourceBalancesRequest' is set
    if (ltsStateAccountAllFungibleResourceBalancesRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsStateAccountAllFungibleResourceBalancesRequest' when calling ltsStateAccountAllFungibleResourceBalancesPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/state/account-all-fungible-resource-balances";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsStateAccountAllFungibleResourceBalancesRequest);
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
   * Get Single Account Balance
   * Returns balance of a single fungible resource in an account
   * @param ltsStateAccountFungibleResourceBalanceRequest  (required)
   * @return LtsStateAccountFungibleResourceBalanceResponse
   * @throws ApiException if fails to make API call
   */
  public LtsStateAccountFungibleResourceBalanceResponse ltsStateAccountFungibleResourceBalancePost(LtsStateAccountFungibleResourceBalanceRequest ltsStateAccountFungibleResourceBalanceRequest) throws ApiException {
    ApiResponse<LtsStateAccountFungibleResourceBalanceResponse> localVarResponse = ltsStateAccountFungibleResourceBalancePostWithHttpInfo(ltsStateAccountFungibleResourceBalanceRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Single Account Balance
   * Returns balance of a single fungible resource in an account
   * @param ltsStateAccountFungibleResourceBalanceRequest  (required)
   * @return ApiResponse&lt;LtsStateAccountFungibleResourceBalanceResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsStateAccountFungibleResourceBalanceResponse> ltsStateAccountFungibleResourceBalancePostWithHttpInfo(LtsStateAccountFungibleResourceBalanceRequest ltsStateAccountFungibleResourceBalanceRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsStateAccountFungibleResourceBalancePostRequestBuilder(ltsStateAccountFungibleResourceBalanceRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsStateAccountFungibleResourceBalancePost", localVarResponse);
        }
        return new ApiResponse<LtsStateAccountFungibleResourceBalanceResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsStateAccountFungibleResourceBalanceResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsStateAccountFungibleResourceBalancePostRequestBuilder(LtsStateAccountFungibleResourceBalanceRequest ltsStateAccountFungibleResourceBalanceRequest) throws ApiException {
    // verify the required parameter 'ltsStateAccountFungibleResourceBalanceRequest' is set
    if (ltsStateAccountFungibleResourceBalanceRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsStateAccountFungibleResourceBalanceRequest' when calling ltsStateAccountFungibleResourceBalancePost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/state/account-fungible-resource-balance";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsStateAccountFungibleResourceBalanceRequest);
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
   * Get Account Transaction Outcomes
   * Returns a list of committed transaction outcomes (containing balance changes) from a given state version, filtered to only transactions which involved the given account. 
   * @param ltsStreamAccountTransactionOutcomesRequest  (required)
   * @return LtsStreamAccountTransactionOutcomesResponse
   * @throws ApiException if fails to make API call
   */
  public LtsStreamAccountTransactionOutcomesResponse ltsStreamAccountTransactionOutcomesPost(LtsStreamAccountTransactionOutcomesRequest ltsStreamAccountTransactionOutcomesRequest) throws ApiException {
    ApiResponse<LtsStreamAccountTransactionOutcomesResponse> localVarResponse = ltsStreamAccountTransactionOutcomesPostWithHttpInfo(ltsStreamAccountTransactionOutcomesRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Account Transaction Outcomes
   * Returns a list of committed transaction outcomes (containing balance changes) from a given state version, filtered to only transactions which involved the given account. 
   * @param ltsStreamAccountTransactionOutcomesRequest  (required)
   * @return ApiResponse&lt;LtsStreamAccountTransactionOutcomesResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsStreamAccountTransactionOutcomesResponse> ltsStreamAccountTransactionOutcomesPostWithHttpInfo(LtsStreamAccountTransactionOutcomesRequest ltsStreamAccountTransactionOutcomesRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsStreamAccountTransactionOutcomesPostRequestBuilder(ltsStreamAccountTransactionOutcomesRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsStreamAccountTransactionOutcomesPost", localVarResponse);
        }
        return new ApiResponse<LtsStreamAccountTransactionOutcomesResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsStreamAccountTransactionOutcomesResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsStreamAccountTransactionOutcomesPostRequestBuilder(LtsStreamAccountTransactionOutcomesRequest ltsStreamAccountTransactionOutcomesRequest) throws ApiException {
    // verify the required parameter 'ltsStreamAccountTransactionOutcomesRequest' is set
    if (ltsStreamAccountTransactionOutcomesRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsStreamAccountTransactionOutcomesRequest' when calling ltsStreamAccountTransactionOutcomesPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/stream/account-transaction-outcomes";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsStreamAccountTransactionOutcomesRequest);
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
   * Get Transaction Outcomes
   * Returns a list of committed transaction outcomes (containing balance changes) from a given state version. 
   * @param ltsStreamTransactionOutcomesRequest  (required)
   * @return LtsStreamTransactionOutcomesResponse
   * @throws ApiException if fails to make API call
   */
  public LtsStreamTransactionOutcomesResponse ltsStreamTransactionOutcomesPost(LtsStreamTransactionOutcomesRequest ltsStreamTransactionOutcomesRequest) throws ApiException {
    ApiResponse<LtsStreamTransactionOutcomesResponse> localVarResponse = ltsStreamTransactionOutcomesPostWithHttpInfo(ltsStreamTransactionOutcomesRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Transaction Outcomes
   * Returns a list of committed transaction outcomes (containing balance changes) from a given state version. 
   * @param ltsStreamTransactionOutcomesRequest  (required)
   * @return ApiResponse&lt;LtsStreamTransactionOutcomesResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsStreamTransactionOutcomesResponse> ltsStreamTransactionOutcomesPostWithHttpInfo(LtsStreamTransactionOutcomesRequest ltsStreamTransactionOutcomesRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsStreamTransactionOutcomesPostRequestBuilder(ltsStreamTransactionOutcomesRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsStreamTransactionOutcomesPost", localVarResponse);
        }
        return new ApiResponse<LtsStreamTransactionOutcomesResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsStreamTransactionOutcomesResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsStreamTransactionOutcomesPostRequestBuilder(LtsStreamTransactionOutcomesRequest ltsStreamTransactionOutcomesRequest) throws ApiException {
    // verify the required parameter 'ltsStreamTransactionOutcomesRequest' is set
    if (ltsStreamTransactionOutcomesRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsStreamTransactionOutcomesRequest' when calling ltsStreamTransactionOutcomesPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/stream/transaction-outcomes";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsStreamTransactionOutcomesRequest);
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
   * Get Construction Metadata
   * Returns information necessary to build a transaction
   * @param ltsTransactionConstructionRequest  (required)
   * @return LtsTransactionConstructionResponse
   * @throws ApiException if fails to make API call
   */
  public LtsTransactionConstructionResponse ltsTransactionConstructionPost(LtsTransactionConstructionRequest ltsTransactionConstructionRequest) throws ApiException {
    ApiResponse<LtsTransactionConstructionResponse> localVarResponse = ltsTransactionConstructionPostWithHttpInfo(ltsTransactionConstructionRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Construction Metadata
   * Returns information necessary to build a transaction
   * @param ltsTransactionConstructionRequest  (required)
   * @return ApiResponse&lt;LtsTransactionConstructionResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsTransactionConstructionResponse> ltsTransactionConstructionPostWithHttpInfo(LtsTransactionConstructionRequest ltsTransactionConstructionRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsTransactionConstructionPostRequestBuilder(ltsTransactionConstructionRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsTransactionConstructionPost", localVarResponse);
        }
        return new ApiResponse<LtsTransactionConstructionResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsTransactionConstructionResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsTransactionConstructionPostRequestBuilder(LtsTransactionConstructionRequest ltsTransactionConstructionRequest) throws ApiException {
    // verify the required parameter 'ltsTransactionConstructionRequest' is set
    if (ltsTransactionConstructionRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsTransactionConstructionRequest' when calling ltsTransactionConstructionPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/transaction/construction";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsTransactionConstructionRequest);
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
   * Get Transaction Status
   * Shares the node&#39;s knowledge of any payloads associated with the given intent hash. Generally there will be a single payload for a given intent, but it&#39;s theoretically possible there may be multiple. This knowledge is summarised into a status for the intent. This summarised status in the response is likely sufficient for most clients. 
   * @param ltsTransactionStatusRequest  (required)
   * @return LtsTransactionStatusResponse
   * @throws ApiException if fails to make API call
   */
  public LtsTransactionStatusResponse ltsTransactionStatusPost(LtsTransactionStatusRequest ltsTransactionStatusRequest) throws ApiException {
    ApiResponse<LtsTransactionStatusResponse> localVarResponse = ltsTransactionStatusPostWithHttpInfo(ltsTransactionStatusRequest);
    return localVarResponse.getData();
  }

  /**
   * Get Transaction Status
   * Shares the node&#39;s knowledge of any payloads associated with the given intent hash. Generally there will be a single payload for a given intent, but it&#39;s theoretically possible there may be multiple. This knowledge is summarised into a status for the intent. This summarised status in the response is likely sufficient for most clients. 
   * @param ltsTransactionStatusRequest  (required)
   * @return ApiResponse&lt;LtsTransactionStatusResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsTransactionStatusResponse> ltsTransactionStatusPostWithHttpInfo(LtsTransactionStatusRequest ltsTransactionStatusRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsTransactionStatusPostRequestBuilder(ltsTransactionStatusRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsTransactionStatusPost", localVarResponse);
        }
        return new ApiResponse<LtsTransactionStatusResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsTransactionStatusResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsTransactionStatusPostRequestBuilder(LtsTransactionStatusRequest ltsTransactionStatusRequest) throws ApiException {
    // verify the required parameter 'ltsTransactionStatusRequest' is set
    if (ltsTransactionStatusRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsTransactionStatusRequest' when calling ltsTransactionStatusPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/transaction/status";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsTransactionStatusRequest);
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
   * Submit Transaction
   * Submits a notarized transaction to the network. Returns whether the transaction submission was already included in the node&#39;s mempool. 
   * @param ltsTransactionSubmitRequest  (required)
   * @return LtsTransactionSubmitResponse
   * @throws ApiException if fails to make API call
   */
  public LtsTransactionSubmitResponse ltsTransactionSubmitPost(LtsTransactionSubmitRequest ltsTransactionSubmitRequest) throws ApiException {
    ApiResponse<LtsTransactionSubmitResponse> localVarResponse = ltsTransactionSubmitPostWithHttpInfo(ltsTransactionSubmitRequest);
    return localVarResponse.getData();
  }

  /**
   * Submit Transaction
   * Submits a notarized transaction to the network. Returns whether the transaction submission was already included in the node&#39;s mempool. 
   * @param ltsTransactionSubmitRequest  (required)
   * @return ApiResponse&lt;LtsTransactionSubmitResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<LtsTransactionSubmitResponse> ltsTransactionSubmitPostWithHttpInfo(LtsTransactionSubmitRequest ltsTransactionSubmitRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = ltsTransactionSubmitPostRequestBuilder(ltsTransactionSubmitRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("ltsTransactionSubmitPost", localVarResponse);
        }
        return new ApiResponse<LtsTransactionSubmitResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<LtsTransactionSubmitResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder ltsTransactionSubmitPostRequestBuilder(LtsTransactionSubmitRequest ltsTransactionSubmitRequest) throws ApiException {
    // verify the required parameter 'ltsTransactionSubmitRequest' is set
    if (ltsTransactionSubmitRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'ltsTransactionSubmitRequest' when calling ltsTransactionSubmitPost");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/lts/transaction/submit";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(ltsTransactionSubmitRequest);
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
