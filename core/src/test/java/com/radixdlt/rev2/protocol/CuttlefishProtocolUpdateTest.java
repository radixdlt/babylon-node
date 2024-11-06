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

package com.radixdlt.rev2.protocol;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.harness.predicates.NodesPredicate.*;
import static org.junit.Assert.*;

import com.google.inject.Module;
import com.radixdlt.api.CoreApiHelper;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.*;
import com.radixdlt.statecomputer.RustStateComputer;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class CuttlefishProtocolUpdateTest {
  private static final long ENACTMENT_EPOCH = 8;
  private static final ProtocolConfig IMMEDIATELY_CUTTLEFISH =
      ProtocolConfig.launchAt(ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME);
  private static final ProtocolConfig CUTTLEFISH_AT_EPOCH =
      ProtocolConfig.enactAtEpoch(ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME, ENACTMENT_EPOCH);

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(ProtocolConfig protocolConfig, Module... extraModules) {
    var genesis =
        GenesisBuilder.createTestGenesisWithNumValidators(
            1, Decimal.ONE, GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(5));
    var test =
        DeterministicTest.builder()
            .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
            .messageSelector(firstSelector())
            .addModules(extraModules)
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    FunctionalRadixNodeModule.NodeStorageConfig.tempFolder(folder),
                    true,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.REAL,
                    FunctionalRadixNodeModule.ConsensusConfig.testDefault(),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                        StateComputerConfig.rev2()
                            .withGenesis(genesis)
                            .withProtocolConfig(protocolConfig))));
    test.startAllNodes();
    return test;
  }

  @Test
  public void transaction_v2_behaviour_across_cuttlefish() throws ApiException {
    final var coreApiHelper = new CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(CUTTLEFISH_AT_EPOCH, coreApiHelper.module())) {
      final var stateComputer = test.getInstance(0, RustStateComputer.class);
      test.runUntilState(allAtOrOverEpoch(ENACTMENT_EPOCH - 1));

      assertEquals(
          ProtocolConfig.BOTTLENOSE_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());
      assertEquals(
          ProtocolConfig.BOTTLENOSE_PROTOCOL_VERSION_NAME,
          coreApiHelper.getNetworkStatus().getCurrentProtocolVersion());

      // Act: Attempt to submit a TransactionV2 against bottlenose
      // ===> It rejects permanently (for now)... or at least as far as bottlenose is concerned
      // ===> But the status is NOTSEEN because it didn't actually get through preparation, so we
      // don't have
      //      a hash to assign against it even, and so can't record it in our cache
      var transactionA = TransactionV2Builder.forTests().prepare();
      try {
        coreApiHelper.submit(transactionA);
      } catch (Exception ignored) {
      }
      assertEquals(
          TransactionIntentStatus.NOTSEEN, coreApiHelper.getStatus(transactionA).getIntentStatus());

      // Arrange: Run the protocol update:
      test.runUntilState(
          allAtExactlyProtocolVersion(ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME));

      assertEquals(
          ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());
      assertEquals(
          ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME,
          coreApiHelper.getNetworkStatus().getCurrentProtocolVersion());

      // Act: Can now submit a new TransactionV2
      var transactionB = TransactionV2Builder.forTests().prepare();

      coreApiHelper.submit(transactionB);

      assertEquals(
          TransactionIntentStatus.INMEMPOOL,
          coreApiHelper.getStatus(transactionB).getIntentStatus());
      test.runUntilState(allCommittedTransactionSuccess(transactionB.raw()), 1000);
      assertEquals(
          TransactionIntentStatus.COMMITTEDSUCCESS,
          coreApiHelper.getStatus(transactionB).getIntentStatus());

      // ... and for what it's worth, we can now resubmit the first transaction
      // (it never even got to the pending transaction result cache)
      assertEquals(
          TransactionIntentStatus.NOTSEEN, coreApiHelper.getStatus(transactionA).getIntentStatus());
      coreApiHelper.submit(transactionA);
      assertEquals(
          TransactionIntentStatus.INMEMPOOL,
          coreApiHelper.getStatus(transactionA).getIntentStatus());
      test.runUntilState(allCommittedTransactionSuccess(transactionA.raw()), 1000);
      assertEquals(
          TransactionIntentStatus.COMMITTEDSUCCESS,
          coreApiHelper.getStatus(transactionA).getIntentStatus());
    }
  }

  @Test
  public void protocol_update_process_updates_status_summary() throws ApiException {
    final var coreApiHelper = new CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var ignored = createTest(IMMEDIATELY_CUTTLEFISH, coreApiHelper.module())) {
      var latestStateVersion =
          coreApiHelper.getNetworkStatus().getCurrentStateIdentifier().getStateVersion();
      var lastTransaction = coreApiHelper.getTransactionFromStream(latestStateVersion);
      var stateUpdates = lastTransaction.getReceipt().getStateUpdates();
      var latestStatus =
          (ProtocolUpdateStatusModuleFieldSummarySubstate)
              stateUpdates.getUpdatedSubstates().stream()
                  .filter(
                      updated ->
                          updated.getSubstateId().getEntityModule()
                              == EntityModule.PROTOCOLUPDATESTATUS)
                  .findFirst()
                  .get()
                  .getNewValue()
                  .getSubstateData();
      assertEquals(
          ProtocolConfig.CUTTLEFISH_PROTOCOL_VERSION_NAME, latestStatus.getProtocolVersion());
      assertEquals(ProtocolUpdateStatusType.COMPLETE, latestStatus.getUpdateStatus().getType());
    }
  }
}
