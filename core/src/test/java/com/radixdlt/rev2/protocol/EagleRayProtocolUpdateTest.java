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
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static org.junit.Assert.assertEquals;

import com.google.inject.Module;
import com.radixdlt.api.CoreApiHelper;
import com.radixdlt.api.core.generated.api.StreamApi;
import com.radixdlt.api.core.generated.models.BootLoaderModuleFieldSystemBootSubstate;
import com.radixdlt.api.core.generated.models.FlashLedgerTransaction;
import com.radixdlt.api.core.generated.models.ProtocolUpdateStatusModuleFieldSummarySubstate;
import com.radixdlt.api.core.generated.models.ProtocolUpdateStatusType;
import com.radixdlt.api.core.generated.models.StreamTransactionsRequest;
import com.radixdlt.api.core.generated.models.SystemVersion;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.sync.TransactionsAndProofReader;
import java.util.List;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

/** Verifies Eagle Ray enactment through the node protocol-update integration. */
public final class EagleRayProtocolUpdateTest {
  /** Keeps the update late enough to observe Cuttlefish Part 2 first. */
  private static final long EAGLE_RAY_EPOCH = 8;

  /** Launches earlier updates immediately and schedules Eagle Ray at the test epoch. */
  private static final ProtocolConfig EAGLE_RAY_AT_EPOCH =
      ProtocolConfig.enactAtEpoch(ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME, EAGLE_RAY_EPOCH);

  /** Provides isolated storage for the node used by each test. */
  @Rule public final TemporaryFolder folder = new TemporaryFolder();

  /** Creates a one-node network using the Eagle Ray test schedule. */
  private DeterministicTest createTest(Module... extraModules) {
    final var genesis =
        GenesisBuilder.createTestGenesisWithNumValidators(
            1, Decimal.ONE, GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(5));
    return DeterministicTest.builder()
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
                        .withProtocolConfig(EAGLE_RAY_AT_EPOCH))));
  }

  /** Verifies the direct CF2-to-Eagle-Ray transition and its committed flashes. */
  @Test
  public void eagle_ray_enacts_after_cuttlefish_part2_with_expected_flashes() throws Exception {
    // Arrange
    final var coreApiHelper = new CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(coreApiHelper.module())) {
      test.startAllNodes();
      final var stateComputer = test.getInstance(0, RustStateComputer.class);
      test.runUntilState(allAtOrOverEpoch(EAGLE_RAY_EPOCH - 1));
      assertEquals(
          ProtocolConfig.CUTTLEFISH_PART2_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());
      assertEquals(
          ProtocolConfig.CUTTLEFISH_PART2_PROTOCOL_VERSION_NAME,
          coreApiHelper.getNetworkStatus().getCurrentProtocolVersion());
      final var preEagleRayStateVersion =
          test.getInstance(0, TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .orElseThrow()
              .resultantStateVersion();

      // Act
      test.runUntilState(allAtOrOverEpoch(EAGLE_RAY_EPOCH));
      final var committedFlashTransactions =
          new StreamApi(coreApiHelper.client())
                  .streamTransactionsPost(
                      new StreamTransactionsRequest()
                          .network(Network.INTEGRATIONTESTNET.getLogicalName())
                          .limit(1000)
                          .fromStateVersion(preEagleRayStateVersion))
                  .getTransactions()
                  .stream()
                  .filter(
                      transaction ->
                          transaction.getLedgerTransaction() instanceof FlashLedgerTransaction)
                  .toList();

      // Assert
      assertEquals(
          ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());
      assertEquals(
          ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME,
          coreApiHelper.getNetworkStatus().getCurrentProtocolVersion());
      final var postProtocolUpdateProof =
          test.getInstance(0, TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .orElseThrow();
      assertEquals(
          ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME,
          postProtocolUpdateProof
              .latestProofWhichInitiatedOneOrMoreProtocolUpdates()
              .unwrap()
              .ledgerHeader()
              .nextProtocolVersion()
              .unwrap());
      assertEquals(
          List.of("eagle-ray-system-version-update", "status-summary", "status-summary"),
          committedFlashTransactions.stream()
              .map(
                  transaction ->
                      ((FlashLedgerTransaction) transaction.getLedgerTransaction()).getName())
              .toList());
      ProtocolUpdateTestUtils.verifyFlashTransactionReceipts(committedFlashTransactions);

      final var systemBoot =
          committedFlashTransactions
              .get(0)
              .getReceipt()
              .getStateUpdates()
              .getUpdatedSubstates()
              .stream()
              .map(updatedSubstate -> updatedSubstate.getNewValue().getSubstateData())
              .filter(BootLoaderModuleFieldSystemBootSubstate.class::isInstance)
              .map(BootLoaderModuleFieldSystemBootSubstate.class::cast)
              .findFirst()
              .orElseThrow();
      assertEquals(SystemVersion.V5, systemBoot.getValue().getSystemVersion());

      final var latestStatus =
          committedFlashTransactions
              .get(2)
              .getReceipt()
              .getStateUpdates()
              .getUpdatedSubstates()
              .stream()
              .map(updatedSubstate -> updatedSubstate.getNewValue().getSubstateData())
              .filter(ProtocolUpdateStatusModuleFieldSummarySubstate.class::isInstance)
              .map(ProtocolUpdateStatusModuleFieldSummarySubstate.class::cast)
              .findFirst()
              .orElseThrow();
      assertEquals(
          ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME, latestStatus.getProtocolVersion());
      assertEquals(ProtocolUpdateStatusType.COMPLETE, latestStatus.getUpdateStatus().getType());
    }
  }
}
