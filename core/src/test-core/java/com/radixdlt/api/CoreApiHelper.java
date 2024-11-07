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

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.catchThrowableOfType;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.google.common.reflect.ClassPath;
import com.google.inject.AbstractModule;
import com.google.inject.Module;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.core.generated.api.*;
import com.radixdlt.api.core.generated.client.ApiClient;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.CoreApiServerFlags;
import com.radixdlt.environment.StartProcessorOnRunner;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.lang.Either;
import com.radixdlt.lang.Functions;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.rev2.NetworkDefinition;
import com.radixdlt.rev2.TransactionBuilder;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import com.radixdlt.transactions.TransactionIntentHash;
import com.radixdlt.utils.FreePortFinder;
import java.net.http.HttpClient;
import java.util.*;
import org.assertj.core.api.ThrowableAssert;

public class CoreApiHelper {

  private final int coreApiPort;
  private final Network network;
  private final NetworkDefinition networkDefinition;
  private final Addressing addressing;
  private final ApiClient apiClient;

  static {
    ensureOpenApiModelsAreReady();
  }

  public static CoreApiHelper forTests() {
    return new CoreApiHelper(Network.INTEGRATIONTESTNET);
  }

  public CoreApiHelper(Network network) {
    this.coreApiPort = FreePortFinder.findFreeLocalPort();
    this.addressing = Addressing.ofNetwork(network);
    this.network = network;
    this.networkDefinition = NetworkDefinition.from(network);
    final var apiClient = new ApiClient();
    apiClient.updateBaseUri("http://127.0.0.1:" + coreApiPort + "/core");
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
          .filter(clazz -> clazz.getPackageName().equals("com.radixdlt.api.core.generated.models"))
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
        install(new CoreApiServerModule("127.0.0.1", coreApiPort, new CoreApiServerFlags(true)));
      }

      @ProvidesIntoSet
      private StartProcessorOnRunner startCoreApi(CoreApiServer coreApiServer) {
        // This is a slightly hacky way to run something on node start-up in a Deterministic test.
        // Stop is called by the AutoClosable binding in CoreApiServerModule
        return new StartProcessorOnRunner("coreApi", coreApiServer::start);
      }
    };
  }

  public ApiClient client() {
    return this.apiClient;
  }

  public TransactionApi transactionApi() {
    return new TransactionApi(client());
  }

  public StreamApi streamApi() {
    return new StreamApi(client());
  }

  public StatusApi statusApi() {
    return new StatusApi(client());
  }

  public StateApi stateApi() {
    return new StateApi(client());
  }

  public LtsApi ltsApi() {
    return new LtsApi(client());
  }

  public MempoolApi mempoolApi() {
    return new MempoolApi(client());
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

  public NetworkConfigurationResponseWellKnownAddresses getWellKnownAddresses()
      throws ApiException {
    return new StatusApi(apiClient).statusNetworkConfigurationPost().getWellKnownAddresses();
  }

  public TransactionSubmitResponse submit(PreparedNotarizedTransaction transaction)
      throws ApiException {
    return transactionApi()
        .transactionSubmitPost(
            new TransactionSubmitRequest()
                .network(network.getLogicalName())
                .notarizedTransactionHex(transaction.hexPayloadBytes()));
  }

  public void ltsSubmit(PreparedNotarizedTransaction transaction) throws ApiException {
    var submitResponse =
        new LtsApi(apiClient)
            .ltsTransactionSubmitPost(
                new LtsTransactionSubmitRequest()
                    .network(networkDefinition.logical_name())
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));

    assertThat(submitResponse.getDuplicate()).isFalse();
  }

  public TransactionSubmitRejectedErrorDetails forceRecalculateSubmitExpectingRejection(
      PreparedNotarizedTransaction transaction) {
    var response =
        assertErrorResponseOfType(
            () ->
                transactionApi()
                    .transactionSubmitPost(
                        new TransactionSubmitRequest()
                            .network(network.getLogicalName())
                            .forceRecalculate(true)
                            .notarizedTransactionHex(transaction.hexPayloadBytes())),
            TransactionSubmitErrorResponse.class);
    return (TransactionSubmitRejectedErrorDetails) response.getDetails();
  }

  public TransactionSubmitRejectedErrorDetails submitExpectingRejection(
      PreparedNotarizedTransaction transaction) {
    var response =
        assertErrorResponseOfType(
            () ->
                transactionApi()
                    .transactionSubmitPost(
                        new TransactionSubmitRequest()
                            .network(network.getLogicalName())
                            .notarizedTransactionHex(transaction.hexPayloadBytes())),
            TransactionSubmitErrorResponse.class);
    return (TransactionSubmitRejectedErrorDetails) response.getDetails();
  }

  public TransactionSubmitResponse forceRecalculateSubmit(PreparedNotarizedTransaction transaction)
      throws ApiException {
    return transactionApi()
        .transactionSubmitPost(
            new TransactionSubmitRequest()
                .network(network.getLogicalName())
                .forceRecalculate(true)
                .notarizedTransactionHex(transaction.hexPayloadBytes()));
  }

  public PreparedNotarizedTransaction buildTransaction(
      Functions.Func1<Manifest.Parameters, String> manifest, List<ECKeyPair> signatories)
      throws ApiException {
    var metadata =
        new LtsApi(apiClient)
            .ltsTransactionConstructionPost(
                new LtsTransactionConstructionRequest().network(network.getLogicalName()));
    return TransactionBuilder.forNetwork(networkDefinition)
        .manifest(manifest)
        .fromEpoch(metadata.getCurrentEpoch())
        .signatories(signatories)
        .prepare();
  }

  public TransactionResult submitAndWaitForResult(
      DeterministicTest test, PreparedNotarizedTransaction transaction) throws ApiException {
    ltsSubmit(transaction);
    return waitForResult(test, transaction.transactionIntentHash());
  }

  public TransactionResult waitForResult(
      DeterministicTest test, PreparedNotarizedTransaction transaction) throws ApiException {
    return waitForResult(test, transaction.transactionIntentHash());
  }

  public Either<TransactionResult, LtsTransactionIntentStatus> permanentResult(
      TransactionIntentHash intentHash) throws ApiException {
    var statusResponse = ltsTransactionStatus(intentHash);
    return switch (statusResponse.getIntentStatus()) {
      case COMMITTEDSUCCESS -> Either.left(
          new TransactionResult(intentHash, TransactionOutcome.CommittedSuccess, statusResponse));
      case COMMITTEDFAILURE -> Either.left(
          new TransactionResult(intentHash, TransactionOutcome.CommittedFailure, statusResponse));
      case PERMANENTREJECTION -> Either.left(
          new TransactionResult(intentHash, TransactionOutcome.PermanentRejection, statusResponse));
      default -> Either.right(statusResponse.getIntentStatus());
    };
  }

  public TransactionResult waitForResult(DeterministicTest test, TransactionIntentHash intentHash)
      throws ApiException {

    int messagesProcessedPerAttempt = 20;
    long attempts = 50;

    LtsTransactionIntentStatus latestIntentStatus = null;
    for (long i = 0; i < attempts; i++) {
      var result = permanentResult(intentHash);
      if (result.isLeft()) {
        return result.unwrapLeft();
      } else {
        latestIntentStatus = result.unwrapRight();
      }
      test.runForCount(messagesProcessedPerAttempt);
    }
    throw new RuntimeException(
        String.format(
            "Transaction submit didn't complete in after running for count of %s messages. Status"
                + " still: %s",
            attempts * messagesProcessedPerAttempt, latestIntentStatus));
  }

  public TransactionResult waitForFirstResult(
      DeterministicTest test, TransactionIntentHash... intentHashes) throws ApiException {
    int messagesProcessedPerAttempt = 20;
    long attempts = 50;

    var latestIntentStatuses =
        new LinkedHashMap<TransactionIntentHash, LtsTransactionIntentStatus>();

    for (long i = 0; i < attempts; i++) {
      for (TransactionIntentHash intentHash : intentHashes) {
        var result = permanentResult(intentHash);
        if (result.isLeft()) {
          return result.unwrapLeft();
        } else {
          latestIntentStatuses.put(intentHash, result.unwrapRight());
        }
      }
      test.runForCount(messagesProcessedPerAttempt);
    }
    throw new RuntimeException(
        String.format(
            "Transaction submit didn't complete in after running for count of %s messages. Statuses"
                + " still: %s",
            attempts * messagesProcessedPerAttempt, latestIntentStatuses));
  }

  public CommittedResult submitAndWaitForSuccess(
      DeterministicTest test,
      Functions.Func1<Manifest.Parameters, String> manifest,
      List<ECKeyPair> signatories)
      throws ApiException {
    final var transaction = buildTransaction(manifest, signatories);
    return submitAndWaitForSuccess(test, transaction);
  }

  public CommittedResult submitAndWaitForSuccess(
      DeterministicTest test, PreparedNotarizedTransaction transaction) throws ApiException {
    return this.submitAndWaitForResult(test, transaction).assertCommittedSuccess();
  }

  public CommittedResult submitAndWaitForCommittedFailure(
      DeterministicTest test,
      Functions.Func1<Manifest.Parameters, String> manifest,
      List<ECKeyPair> signatories)
      throws ApiException {
    final var transaction = buildTransaction(manifest, signatories);
    return submitAndWaitForResult(test, transaction).assertCommittedFailure();
  }

  public CommittedResult submitAndWaitForCommittedFailure(
      DeterministicTest test, PreparedNotarizedTransaction transaction) throws ApiException {
    return submitAndWaitForResult(test, transaction).assertCommittedFailure();
  }

  public String submitAndWaitForPermanentRejection(
      DeterministicTest test, PreparedNotarizedTransaction transaction) throws ApiException {
    return submitAndWaitForResult(test, transaction).assertPermanentRejection();
  }

  public TransactionStatusResponse getStatus(PreparedNotarizedTransaction transaction)
      throws ApiException {
    return transactionApi()
        .transactionStatusPost(
            new TransactionStatusRequest()
                .network(network.getLogicalName())
                .intentHash(addressing.encode(transaction.transactionIntentHash())));
  }

  public LtsTransactionStatusResponse ltsTransactionStatus(PreparedNotarizedTransaction transaction)
      throws ApiException {
    return ltsTransactionStatus(transaction.transactionIntentHash());
  }

  public LtsTransactionStatusResponse ltsTransactionStatus(TransactionIntentHash intentHash)
      throws ApiException {
    return ltsApi()
        .ltsTransactionStatusPost(
            new LtsTransactionStatusRequest()
                .network(networkDefinition.logical_name())
                .intentHash(intentHash.hex()));
  }

  public NetworkStatusResponse getNetworkStatus() throws ApiException {
    return statusApi()
        .statusNetworkStatusPost(new NetworkStatusRequest().network(network.getLogicalName()));
  }

  public CommittedTransaction getTransactionFromStream(long stateVersion) throws ApiException {
    return streamApi()
        .streamTransactionsPost(
            new StreamTransactionsRequest()
                .network(network.getLogicalName())
                .fromStateVersion(stateVersion)
                .limit(1))
        .getTransactions()
        .get(0);
  }

  public record TransactionResult(
      TransactionIntentHash transactionIntentHash,
      TransactionOutcome outcome,
      LtsTransactionStatusResponse response) {
    public CommittedResult assertCommittedSuccess() {
      switch (outcome) {
        case CommittedSuccess -> {
          var stateVersion = response.getCommittedStateVersion();
          if (stateVersion == null) {
            throw new RuntimeException(
                "Transaction got committed as success without state version on response");
          }
          return new CommittedResult(transactionIntentHash, stateVersion, Optional.empty());
        }
        case CommittedFailure -> throw new RuntimeException(
            String.format(
                "Transaction got committed as failure: %s",
                response.getKnownPayloads().get(0).getErrorMessage()));
        case PermanentRejection -> throw new RuntimeException(
            String.format(
                "Transaction got permanently rejected: %s",
                response.getKnownPayloads().get(0).getErrorMessage()));
      }
      throw new IllegalStateException("Shouldn't be able to get here");
    }

    public CommittedResult assertCommittedFailure() {
      switch (outcome) {
        case CommittedSuccess -> throw new RuntimeException(
            "Transaction got committed as success, but was expecting committed failure");
        case CommittedFailure -> {
          var stateVersion = response.getCommittedStateVersion();
          if (stateVersion == null) {
            throw new RuntimeException(
                "Transaction got committed as failure without state version on response");
          }
          var errorMessage =
              Objects.requireNonNull(response.getKnownPayloads().get(0).getErrorMessage());
          return new CommittedResult(
              transactionIntentHash, stateVersion, Optional.of(errorMessage));
        }
        case PermanentRejection -> throw new RuntimeException(
            String.format(
                "Transaction got permanently rejected: %s",
                response.getKnownPayloads().get(0).getErrorMessage()));
      }
      throw new IllegalStateException("Shouldn't be able to get here");
    }

    public String assertPermanentRejection() {
      switch (outcome) {
        case CommittedSuccess -> throw new RuntimeException(
            "Transaction got committed as success, but was expecting a permanent rejection");
        case CommittedFailure -> throw new RuntimeException(
            "Transaction got committed as failure, but was expecting a permanent rejection");
        case PermanentRejection -> {
          return response.getKnownPayloads().get(0).getErrorMessage();
        }
      }
      throw new IllegalStateException("Shouldn't be able to get here");
    }

    public <T> T apply(
        Functions.Func3<TransactionIntentHash, TransactionOutcome, LtsTransactionStatusResponse, T>
            resultMapper) {
      return resultMapper.apply(transactionIntentHash, outcome, response);
    }
  }

  public enum TransactionOutcome {
    CommittedSuccess,
    CommittedFailure,
    PermanentRejection,
  }

  public record CommittedResult(
      TransactionIntentHash transactionIntentHash,
      long stateVersion,
      Optional<String> errorMessage) {}
}
