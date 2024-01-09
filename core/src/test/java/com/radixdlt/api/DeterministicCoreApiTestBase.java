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

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static org.assertj.core.api.Assertions.*;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.google.common.collect.ImmutableList;
import com.google.common.reflect.ClassPath;
import com.google.inject.AbstractModule;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.core.generated.api.*;
import com.radixdlt.api.core.generated.client.ApiClient;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.CoreApiServerFlags;
import com.radixdlt.environment.DatabaseFlags;
import com.radixdlt.environment.StartProcessorOnRunner;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.lang.Functions;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.*;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.transactions.IntentHash;
import com.radixdlt.utils.FreePortFinder;
import java.net.http.HttpClient;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import javax.net.ssl.SSLContext;
import org.assertj.core.api.ThrowableAssert;
import org.junit.Rule;
import org.junit.rules.TemporaryFolder;

public abstract class DeterministicCoreApiTestBase {
  @Rule public TemporaryFolder folder = new TemporaryFolder();
  public static NetworkDefinition networkDefinition = NetworkDefinition.INT_TEST_NET;
  public static Addressing addressing = Addressing.ofNetwork(NetworkDefinition.INT_TEST_NET);
  public static String networkLogicalName = networkDefinition.logical_name();
  protected int coreApiPort = FreePortFinder.findFreeLocalPort();

  protected ApiClient apiClient = buildApiClient();

  static {
    ensureOpenApiModelsAreReady();
  }

  protected DeterministicCoreApiTestBase() {}

  protected DeterministicTest buildRunningServerTest() {
    return buildRunningServerTest(
        1000000, new DatabaseFlags(true, false), GenesisData.NO_SCENARIOS);
  }

  protected DeterministicTest buildRunningServerTestWithScenarios(ImmutableList<String> scenarios) {
    return buildRunningServerTest(1000000, new DatabaseFlags(true, false), scenarios);
  }

  protected DeterministicTest buildRunningServerTest(DatabaseFlags databaseFlags) {
    return buildRunningServerTest(1000000, databaseFlags, GenesisData.NO_SCENARIOS);
  }

  protected DeterministicTest buildRunningServerTest(int roundsPerEpoch) {
    return buildRunningServerTest(
        roundsPerEpoch, new DatabaseFlags(true, false), GenesisData.NO_SCENARIOS);
  }

  protected DeterministicTest buildRunningServerTest(
      int roundsPerEpoch, DatabaseFlags databaseConfig, ImmutableList<String> scenariosToRun) {
    var test =
        DeterministicTest.builder()
            .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
            .messageSelector(firstSelector())
            .addMonitors()
            .addModule(
                new CoreApiServerModule("127.0.0.1", coreApiPort, new CoreApiServerFlags(true)))
            .addModule(
                new AbstractModule() {
                  @ProvidesIntoSet
                  private StartProcessorOnRunner startCoreApi(CoreApiServer coreApiServer) {
                    // This is a slightly hacky way to run something on node start-up in a
                    // Deterministic test.
                    // Stop is called by the AutoClosable binding in CoreApiServerModule
                    return new StartProcessorOnRunner("N/A", coreApiServer::start);
                  }
                })
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    NodeStorageConfig.tempFolder(folder),
                    true,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                    FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                        StateComputerConfig.rev2(
                            Network.INTEGRATIONTESTNET.getId(),
                            GenesisBuilder.createTestGenesisWithNumValidators(
                                1,
                                Decimal.ONE,
                                GenesisConsensusManagerConfig.Builder.testDefaults()
                                    .epochExactRoundCount(roundsPerEpoch),
                                scenariosToRun),
                            databaseConfig,
                            StateComputerConfig.REV2ProposerConfig.Mempool.defaults()),
                        SyncRelayConfig.of(200, 10, 2000))));
    try {
      test.startAllNodes();
    } catch (Exception ex) {
      test.close();
      throw ex;
    }
    return test;
  }

  private static void ensureOpenApiModelsAreReady() {
    /* The generated Open API models are rubbish and requires that static initializers run on models before
     * deserialization to work correctly... But that doesn't happen in eg models under the response model in
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

  protected ApiClient buildApiClient() {
    final var apiClient = new ApiClient();
    apiClient.updateBaseUri("http://127.0.0.1:" + coreApiPort + "/core");
    // Create a dummy SSLContext to avoid the "NoSuchAlgorithmException" when
    // the default HttpClient fails to load a trust store. We don't need SSL anyway.
    try {
      // SNYK - this file is ignored in .snyk file
      // Raised issue: Inadequate Encryption Strength
      // Explanation: This is just a test, it doesn't matter.
      final var dummySSLContext = SSLContext.getInstance("TLS");
      dummySSLContext.init(null, null, null);
      apiClient.setHttpClientBuilder(HttpClient.newBuilder().sslContext(dummySSLContext));
      return apiClient;
    } catch (Exception ex) {
      throw new RuntimeException(ex);
    }
  }

  public <Response> Response assertErrorResponseOfType(
      ThrowableAssert.ThrowingCallable apiCall, Class<Response> responseClass)
      throws JsonProcessingException {
    var apiException = catchThrowableOfType(apiCall, ApiException.class);
    return apiClient.getObjectMapper().readValue(apiException.getResponseBody(), responseClass);
  }

  public MempoolApi getMempoolApi() {
    return new MempoolApi(apiClient);
  }

  protected StatusApi getStatusApi() {
    return new StatusApi(apiClient);
  }

  protected TransactionApi getTransactionApi() {
    return new TransactionApi(apiClient);
  }

  protected StreamApi getStreamApi() {
    return new StreamApi(apiClient);
  }

  protected StateApi getStateApi() {
    return new StateApi(apiClient);
  }

  protected LtsApi getLtsApi() {
    return new LtsApi(apiClient);
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
        getLtsApi()
            .ltsTransactionConstructionPost(
                new LtsTransactionConstructionRequest().network(networkLogicalName));

    var transaction =
        TransactionBuilder.forNetwork(networkDefinition)
            .manifest(manifest)
            .fromEpoch(metadata.getCurrentEpoch())
            .signatories(signatories)
            .prepare();

    var submitResponse =
        getLtsApi()
            .ltsTransactionSubmitPost(
                new LtsTransactionSubmitRequest()
                    .network(networkLogicalName)
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));

    assertThat(submitResponse.getDuplicate()).isFalse();

    int messagesProcessedPerAttempt = 20;
    long attempts = 50;

    LtsTransactionStatusResponse statusResponse = null;
    for (long i = 0; i < attempts; i++) {
      statusResponse =
          getLtsApi()
              .ltsTransactionStatusPost(
                  new LtsTransactionStatusRequest()
                      .network(networkLogicalName)
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

  public ResourceAddress createFreeMintBurnNonFungibleResource(DeterministicTest test)
      throws Exception {
    var committedNewResourceTxn =
        submitAndWaitForSuccess(test, Manifest.createAllowAllNonFungibleResource(), List.of());

    final var receipt =
        getTransactionApi()
            .transactionReceiptPost(
                new TransactionReceiptRequest()
                    .network(networkLogicalName)
                    .intentHash(committedNewResourceTxn.intentHash().hex()));

    final var newResourceAddressStr =
        receipt
            .getCommitted()
            .getReceipt()
            .getStateUpdates()
            .getNewGlobalEntities()
            .get(0)
            .getEntityAddress();

    return addressing.decodeResourceAddress(newResourceAddressStr);
  }

  public record CommittedResult(
      IntentHash intentHash, long stateVersion, Optional<String> errorMessage) {}
}
