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
import com.radixdlt.environment.CoreApiServerFlags;
import com.radixdlt.environment.StartProcessorOnRunner;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.*;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.NodeStorageConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.*;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.utils.FreePortFinder;
import java.net.http.HttpClient;
import java.util.List;
import org.assertj.core.api.ThrowableAssert;
import org.junit.Rule;
import org.junit.rules.TemporaryFolder;

public abstract class DeterministicCoreApiTestBase {
  private static final DatabaseConfig TEST_DATABASE_CONFIG = new DatabaseConfig(true, false, false, false);

  @Rule public TemporaryFolder folder = new TemporaryFolder();
  public static NetworkDefinition networkDefinition = NetworkDefinition.INT_TEST_NET;
  public static Addressing addressing = Addressing.ofNetwork(NetworkDefinition.INT_TEST_NET);
  public static String networkLogicalName = networkDefinition.logical_name();

  private final int coreApiPort;
  private final ApiClient apiClient;

  static {
    ensureOpenApiModelsAreReady();
  }

  protected DeterministicCoreApiTestBase() {
    this.coreApiPort = FreePortFinder.findFreeLocalPort();
    this.apiClient = buildApiClient();
  }

  protected DeterministicTest buildRunningServerTest() {
    return buildRunningServerTest(
        1000000, TEST_DATABASE_CONFIG, GenesisData.NO_SCENARIOS, ProtocolConfig.testingDefault());
  }

  protected DeterministicTest buildRunningServerTestWithProtocolConfig(
      int roundsPerEpoch, ProtocolConfig protocolConfig) {
    return buildRunningServerTest(
        roundsPerEpoch, TEST_DATABASE_CONFIG, GenesisData.NO_SCENARIOS, protocolConfig);
  }

  protected DeterministicTest buildRunningServerTestWithScenarios(ImmutableList<String> scenarios) {
    return buildRunningServerTest(
        1000000, TEST_DATABASE_CONFIG, scenarios, ProtocolConfig.testingDefault());
  }

  protected DeterministicTest buildRunningServerTest(DatabaseConfig databaseConfig) {
    return buildRunningServerTest(
        1000000, databaseConfig, GenesisData.NO_SCENARIOS, ProtocolConfig.testingDefault());
  }

  protected DeterministicTest buildRunningServerTest(
      DatabaseConfig databaseConfig, StateHashTreeGcConfig stateHashTreeGcConfig) {
    return buildRunningServerTest(
        1000000,
        databaseConfig,
        GenesisData.NO_SCENARIOS,
        ProtocolConfig.testingDefault(),
        stateHashTreeGcConfig);
  }

  protected DeterministicTest buildRunningServerTest(int roundsPerEpoch) {
    return buildRunningServerTest(
        roundsPerEpoch,
        TEST_DATABASE_CONFIG,
        GenesisData.NO_SCENARIOS,
        ProtocolConfig.testingDefault());
  }

  protected DeterministicTest buildRunningServerTest(
      int roundsPerEpoch,
      DatabaseConfig databaseConfig,
      ImmutableList<String> scenariosToRun,
      ProtocolConfig protocolConfig,
      StateHashTreeGcConfig stateHashTreeGcConfig) {
    return buildRunningServerTest(
        StateComputerConfig.rev2(
            Network.INTEGRATIONTESTNET.getId(),
            GenesisBuilder.createTestGenesisWithNumValidators(
                1,
                Decimal.ONE,
                GenesisConsensusManagerConfig.Builder.testDefaults()
                    .epochExactRoundCount(roundsPerEpoch),
                scenariosToRun),
            databaseConfig,
            StateComputerConfig.REV2ProposerConfig.Mempool.defaults(),
            false,
            false,
            protocolConfig,
            stateHashTreeGcConfig));
  }

  protected DeterministicTest buildRunningServerTest(
      int roundsPerEpoch,
      DatabaseConfig databaseConfig,
      ImmutableList<String> scenariosToRun,
      ProtocolConfig protocolConfig) {
    return buildRunningServerTest(
        StateComputerConfig.rev2(
            Network.INTEGRATIONTESTNET.getId(),
            GenesisBuilder.createTestGenesisWithNumValidators(
                1,
                Decimal.ONE,
                GenesisConsensusManagerConfig.Builder.testDefaults()
                    .epochExactRoundCount(roundsPerEpoch),
                scenariosToRun),
            databaseConfig,
            StateComputerConfig.REV2ProposerConfig.Mempool.defaults(),
            false,
            false,
            protocolConfig));
  }

  protected DeterministicTest buildRunningServerTest(StateComputerConfig stateComputerConfig) {
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
                        stateComputerConfig, SyncRelayConfig.of(200, 10, 2000))));
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
    apiClient.setHttpClientBuilder(
        HttpClient.newBuilder().sslContext(DummySslContextFactory.create()));
    return apiClient;
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

  protected CoreApiHelper getApiHelper() {
    return new CoreApiHelper(networkDefinition, coreApiPort);
  }

  public ResourceAddress createFreeMintBurnNonFungibleResource(DeterministicTest test)
      throws Exception {
    var committedNewResourceTxn =
        getApiHelper()
            .submitAndWaitForSuccess(test, Manifest.createAllowAllNonFungibleResource(), List.of());

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
}
