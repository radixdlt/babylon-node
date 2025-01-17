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

import com.radixdlt.api.mesh.generated.models.AccountBalanceRequest;
import com.radixdlt.api.mesh.generated.models.AccountBalanceResponse;
import com.radixdlt.api.mesh.generated.models.AccountCoinsRequest;
import com.radixdlt.api.mesh.generated.models.AccountCoinsResponse;
import com.radixdlt.api.mesh.generated.models.Error;

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
public class AccountApi {
  private final HttpClient memberVarHttpClient;
  private final ObjectMapper memberVarObjectMapper;
  private final String memberVarBaseUri;
  private final Consumer<HttpRequest.Builder> memberVarInterceptor;
  private final Duration memberVarReadTimeout;
  private final Consumer<HttpResponse<InputStream>> memberVarResponseInterceptor;
  private final Consumer<HttpResponse<String>> memberVarAsyncResponseInterceptor;

  public AccountApi() {
    this(new ApiClient());
  }

  public AccountApi(ApiClient apiClient) {
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
   * Get an Account&#39;s Balance
   * Get an array of all AccountBalances for an AccountIdentifier and the BlockIdentifier at which the balance lookup was performed. The BlockIdentifier must always be returned because some consumers of account balance data need to know specifically at which block the balance was calculated to compare balances they compute from operations with the balance returned by the node.  It is important to note that making a balance request for an account without populating the SubAccountIdentifier should not result in the balance of all possible SubAccountIdentifiers being returned. Rather, it should result in the balance pertaining to no SubAccountIdentifiers being returned (sometimes called the liquid balance). To get all balances associated with an account, it may be necessary to perform multiple balance requests with unique AccountIdentifiers.  It is also possible to perform a historical balance lookup (if the server supports it) by passing in an optional BlockIdentifier. 
   * @param accountBalanceRequest  (required)
   * @return AccountBalanceResponse
   * @throws ApiException if fails to make API call
   */
  public AccountBalanceResponse accountBalance(AccountBalanceRequest accountBalanceRequest) throws ApiException {
    ApiResponse<AccountBalanceResponse> localVarResponse = accountBalanceWithHttpInfo(accountBalanceRequest);
    return localVarResponse.getData();
  }

  /**
   * Get an Account&#39;s Balance
   * Get an array of all AccountBalances for an AccountIdentifier and the BlockIdentifier at which the balance lookup was performed. The BlockIdentifier must always be returned because some consumers of account balance data need to know specifically at which block the balance was calculated to compare balances they compute from operations with the balance returned by the node.  It is important to note that making a balance request for an account without populating the SubAccountIdentifier should not result in the balance of all possible SubAccountIdentifiers being returned. Rather, it should result in the balance pertaining to no SubAccountIdentifiers being returned (sometimes called the liquid balance). To get all balances associated with an account, it may be necessary to perform multiple balance requests with unique AccountIdentifiers.  It is also possible to perform a historical balance lookup (if the server supports it) by passing in an optional BlockIdentifier. 
   * @param accountBalanceRequest  (required)
   * @return ApiResponse&lt;AccountBalanceResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<AccountBalanceResponse> accountBalanceWithHttpInfo(AccountBalanceRequest accountBalanceRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = accountBalanceRequestBuilder(accountBalanceRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("accountBalance", localVarResponse);
        }
        return new ApiResponse<AccountBalanceResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<AccountBalanceResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder accountBalanceRequestBuilder(AccountBalanceRequest accountBalanceRequest) throws ApiException {
    // verify the required parameter 'accountBalanceRequest' is set
    if (accountBalanceRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'accountBalanceRequest' when calling accountBalance");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/account/balance";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(accountBalanceRequest);
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
   * Get an Account&#39;s Unspent Coins
   * Get an array of all unspent coins for an AccountIdentifier and the BlockIdentifier at which the lookup was performed. If your implementation does not support coins (i.e. it is for an account-based blockchain), you do not need to implement this endpoint. If you implementation does support coins (i.e. it is fro a UTXO-based blockchain), you MUST also complete the &#x60;/account/balance&#x60; endpoint.  It is important to note that making a coins request for an account without populating the SubAccountIdentifier should not result in the coins of all possible SubAccountIdentifiers being returned. Rather, it should result in the coins pertaining to no SubAccountIdentifiers being returned. To get all coins associated with an account, it may be necessary to perform multiple coin requests with unique AccountIdentifiers.  Optionally, an implementation may choose to support updating an AccountIdentifier&#39;s unspent coins based on the contents of the mempool. Note, using this functionality breaks any guarantee of idempotency. 
   * @param accountCoinsRequest  (required)
   * @return AccountCoinsResponse
   * @throws ApiException if fails to make API call
   */
  public AccountCoinsResponse accountCoins(AccountCoinsRequest accountCoinsRequest) throws ApiException {
    ApiResponse<AccountCoinsResponse> localVarResponse = accountCoinsWithHttpInfo(accountCoinsRequest);
    return localVarResponse.getData();
  }

  /**
   * Get an Account&#39;s Unspent Coins
   * Get an array of all unspent coins for an AccountIdentifier and the BlockIdentifier at which the lookup was performed. If your implementation does not support coins (i.e. it is for an account-based blockchain), you do not need to implement this endpoint. If you implementation does support coins (i.e. it is fro a UTXO-based blockchain), you MUST also complete the &#x60;/account/balance&#x60; endpoint.  It is important to note that making a coins request for an account without populating the SubAccountIdentifier should not result in the coins of all possible SubAccountIdentifiers being returned. Rather, it should result in the coins pertaining to no SubAccountIdentifiers being returned. To get all coins associated with an account, it may be necessary to perform multiple coin requests with unique AccountIdentifiers.  Optionally, an implementation may choose to support updating an AccountIdentifier&#39;s unspent coins based on the contents of the mempool. Note, using this functionality breaks any guarantee of idempotency. 
   * @param accountCoinsRequest  (required)
   * @return ApiResponse&lt;AccountCoinsResponse&gt;
   * @throws ApiException if fails to make API call
   */
  public ApiResponse<AccountCoinsResponse> accountCoinsWithHttpInfo(AccountCoinsRequest accountCoinsRequest) throws ApiException {
    HttpRequest.Builder localVarRequestBuilder = accountCoinsRequestBuilder(accountCoinsRequest);
    try {
      HttpResponse<InputStream> localVarResponse = memberVarHttpClient.send(
          localVarRequestBuilder.build(),
          HttpResponse.BodyHandlers.ofInputStream());
      if (memberVarResponseInterceptor != null) {
        memberVarResponseInterceptor.accept(localVarResponse);
      }
      try {
        if (localVarResponse.statusCode()/ 100 != 2) {
          throw getApiException("accountCoins", localVarResponse);
        }
        return new ApiResponse<AccountCoinsResponse>(
          localVarResponse.statusCode(),
          localVarResponse.headers().map(),
          memberVarObjectMapper.readValue(localVarResponse.body(), new TypeReference<AccountCoinsResponse>() {}) // closes the InputStream
          
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

  private HttpRequest.Builder accountCoinsRequestBuilder(AccountCoinsRequest accountCoinsRequest) throws ApiException {
    // verify the required parameter 'accountCoinsRequest' is set
    if (accountCoinsRequest == null) {
      throw new ApiException(400, "Missing the required parameter 'accountCoinsRequest' when calling accountCoins");
    }

    HttpRequest.Builder localVarRequestBuilder = HttpRequest.newBuilder();

    String localVarPath = "/account/coins";

    localVarRequestBuilder.uri(URI.create(memberVarBaseUri + localVarPath));

    localVarRequestBuilder.header("Content-Type", "application/json");
    localVarRequestBuilder.header("Accept", "application/json");

    try {
      byte[] localVarPostBody = memberVarObjectMapper.writeValueAsBytes(accountCoinsRequest);
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
