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

import com.radixdlt.api.core.generated.api.*;
import com.radixdlt.api.core.generated.client.ApiClient;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.lang.Functions;
import com.radixdlt.rev2.*;
import com.radixdlt.transactions.IntentHash;

import java.net.http.HttpClient;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

import static org.assertj.core.api.Assertions.assertThat;

public class CoreApiHelper {

  private final NetworkDefinition networkDefinition;

  private final ApiClient apiClient;

  public CoreApiHelper(NetworkDefinition networkDefinition, int coreApiPort) {
    this.networkDefinition = networkDefinition;
    this.apiClient = new ApiClient();
    apiClient.updateBaseUri("http://127.0.0.1:" + coreApiPort + "/core");
    apiClient.setHttpClientBuilder(HttpClient.newBuilder().sslContext(DummySslContextFactory.create()));
  }

  public NetworkConfigurationResponseWellKnownAddresses getWellKnownAddresses() throws ApiException {
    return new StatusApi(apiClient).statusNetworkConfigurationPost().getWellKnownAddresses();
  }

  public enum TransactionOutcome {
    CommittedSuccess,
    CommittedFailure,
    PermanentRejection,
  }

  public <T> T submitAndWait(
      DeterministicTest test,
      Functions.Func1<Manifest.Parameters, String> manifest,
      List<ECKeyPair> signatories,
      Functions.Func3<IntentHash, TransactionOutcome, LtsTransactionStatusResponse, T>
          outcomeMapper)
      throws Exception {
    var metadata =
        new LtsApi(apiClient)
            .ltsTransactionConstructionPost(
                new LtsTransactionConstructionRequest().network(networkDefinition.logical_name()));

    var transaction =
        TransactionBuilder.forNetwork(networkDefinition)
            .manifest(manifest)
            .fromEpoch(metadata.getCurrentEpoch())
            .signatories(signatories)
            .prepare();

    var submitResponse =
        new LtsApi(apiClient)
            .ltsTransactionSubmitPost(
                new LtsTransactionSubmitRequest()
                    .network(networkDefinition.logical_name())
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));

    assertThat(submitResponse.getDuplicate()).isFalse();

    int messagesProcessedPerAttempt = 20;
    long attempts = 50;

    LtsTransactionStatusResponse statusResponse = null;
    for (long i = 0; i < attempts; i++) {
      statusResponse =
          new LtsApi(apiClient)
              .ltsTransactionStatusPost(
                  new LtsTransactionStatusRequest()
                      .network(networkDefinition.logical_name())
                      .intentHash(transaction.hexIntentHash()));
      switch (statusResponse.getIntentStatus()) {
        case COMMITTEDSUCCESS -> {
          return outcomeMapper.apply(
              transaction.intentHash(), TransactionOutcome.CommittedSuccess, statusResponse);
        }
        case COMMITTEDFAILURE -> {
          return outcomeMapper.apply(
              transaction.intentHash(), TransactionOutcome.CommittedFailure, statusResponse);
        }
        case PERMANENTREJECTION -> {
          return outcomeMapper.apply(
              transaction.intentHash(), TransactionOutcome.PermanentRejection, statusResponse);
        }
        default -> test.runForCount(messagesProcessedPerAttempt);
      }
    }
    throw new RuntimeException(
        String.format(
            "Transaction submit didn't complete in after running for count of %s. Status still: %s",
            attempts * messagesProcessedPerAttempt, statusResponse.getIntentStatus()));
  }

  public CommittedResult submitAndWaitForSuccess(
      DeterministicTest test,
      Functions.Func1<Manifest.Parameters, String> manifest,
      List<ECKeyPair> signatories)
      throws Exception {
    return this.submitAndWait(
        test,
        manifest,
        signatories,
        (intentHash, outcome, response) -> {
          switch (outcome) {
            case CommittedSuccess -> {
              var stateVersion = response.getCommittedStateVersion();
              if (stateVersion == null) {
                throw new RuntimeException(
                    "Transaction got committed as success without state version on response");
              }
              return new CommittedResult(intentHash, stateVersion, Optional.empty());
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
        });
  }

  public CommittedResult submitAndWaitForCommittedFailure(
      DeterministicTest test,
      Functions.Func1<Manifest.Parameters, String> manifest,
      List<ECKeyPair> signatories)
      throws Exception {
    return this.submitAndWait(
        test,
        manifest,
        signatories,
        (intentHash, outcome, response) -> {
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
              return new CommittedResult(intentHash, stateVersion, Optional.of(errorMessage));
            }
            case PermanentRejection -> throw new RuntimeException(
                String.format(
                    "Transaction got permanently rejected: %s",
                    response.getKnownPayloads().get(0).getErrorMessage()));
          }
          throw new IllegalStateException("Shouldn't be able to get here");
        });
  }

  public record CommittedResult(
      IntentHash intentHash, long stateVersion, Optional<String> errorMessage) {}
}
