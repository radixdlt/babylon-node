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
import static com.radixdlt.protocol.ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch;
import static org.junit.Assert.*;

import com.google.common.collect.ImmutableList;
import com.google.inject.Module;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.core.generated.api.StreamApi;
import com.radixdlt.api.core.generated.api.TransactionApi;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.api.core.generated.models.TransactionStatus;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import com.radixdlt.rev2.*;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.sync.TransactionsAndProofReader;
import java.util.Arrays;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class BottlenoseProtocolUpdateTest {

  private static final long BOTTLENOSE_EPOCH = 8;

  private static final ProtocolConfig PROTOCOL_CONFIG =
      new ProtocolConfig(
          ImmutableList.of(
              // Update to Anemone at some arbitrary earlier moment (in case of dependencies):
              new ProtocolUpdateTrigger(
                  ProtocolConfig.ANEMONE_PROTOCOL_VERSION_NAME,
                  unconditionallyAtEpoch(BOTTLENOSE_EPOCH - 3)),
              new ProtocolUpdateTrigger(
                  ProtocolConfig.BOTTLENOSE_PROTOCOL_VERSION_NAME,
                  unconditionallyAtEpoch(BOTTLENOSE_EPOCH))));

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(Module... extraModules) {
    var genesis =
        GenesisBuilder.createTestGenesisWithNumValidators(
            1, Decimal.ONE, GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(5));
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
        .messageSelector(firstSelector())
        .messageMutator(MessageMutator.dropTimeouts())
        .addModules(extraModules)
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                FunctionalRadixNodeModule.NodeStorageConfig.tempFolder(folder),
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.REAL,
                FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2()
                        .withGenesis(genesis)
                        .withProtocolConfig(PROTOCOL_CONFIG))));
  }

  @Test
  public void example_bottlenose_feature_is_available_only_after_update() throws ApiException {
    // The easiest "new feature" to assert on is the AccountLocker package being published:
    final var addressing = Addressing.ofNetwork(Network.INTEGRATIONTESTNET);
    final var accountLockerCall =
        new TransactionCallPreviewRequest()
            .network(Network.INTEGRATIONTESTNET.getLogicalName())
            .target(
                new BlueprintFunctionTargetIdentifier()
                    .packageAddress(addressing.encode(ScryptoConstants.LOCKER_PACKAGE_ADDRESS))
                    .blueprintName("AccountLocker")
                    .functionName("instantiate_simple")
                    .type(TargetIdentifierType.FUNCTION))
            .addArgumentsItem("4d0101"); // hex-encoded SBOR `true` (for `allow_recover` parameter)

    final var coreApiHelper = new ProtocolUpdateTestUtils.CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(coreApiHelper.module())) {
      // Arrange: Start a single node network, reach state just before Bottlenose:
      test.startAllNodes();
      final var stateComputer = test.getInstance(0, RustStateComputer.class);
      test.runUntilState(allAtOrOverEpoch(BOTTLENOSE_EPOCH - 1));
      assertNotEquals(
          ProtocolConfig.BOTTLENOSE_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());

      // Act: Preview a transaction trying to create an AccountLocker:
      final var callBeforeBottlenose =
          new TransactionApi(coreApiHelper.client()).transactionCallPreviewPost(accountLockerCall);

      // Assert: It is rightfully rejected, since the package referenced in the manifest does not
      // exist
      assertEquals(TransactionStatus.REJECTED, callBeforeBottlenose.getStatus());

      // Arrange: Run the Bottlenose protocol update:
      test.runUntilState(allAtOrOverEpoch(BOTTLENOSE_EPOCH));
      assertEquals(
          ProtocolConfig.BOTTLENOSE_PROTOCOL_VERSION_NAME,
          stateComputer.protocolState().currentProtocolVersion());

      // Act: Preview the same transaction again:
      final var callAfterBottlenose =
          new TransactionApi(coreApiHelper.client()).transactionCallPreviewPost(accountLockerCall);

      // Assert: It rightfully fails, since the call returns a bucket (with a badge)
      assertEquals(TransactionStatus.FAILED, callAfterBottlenose.getStatus());
      assertTrue(callAfterBottlenose.getErrorMessage().contains("DropNonEmptyBucket"));
    }
  }

  @Test
  public void core_api_streams_bottlenose_flash_transactions() throws Exception {
    final var coreApiHelper = new ProtocolUpdateTestUtils.CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(coreApiHelper.module())) {
      // Arrange: Start a single Node network and capture the state version right before Bottlenose:
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(BOTTLENOSE_EPOCH - 1));
      final var preBottlenoseStateVersion =
          test.getInstance(0, TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .get()
              .resultantStateVersion();

      // Act: Run the Bottlenose update and fetch flash transactions executed by it:
      test.runUntilState(allAtOrOverEpoch(BOTTLENOSE_EPOCH));
      final var committedFlashTransactions =
          new StreamApi(coreApiHelper.client())
                  .streamTransactionsPost(
                      new StreamTransactionsRequest()
                          .network(Network.INTEGRATIONTESTNET.getLogicalName())
                          .limit(1000)
                          .fromStateVersion(preBottlenoseStateVersion))
                  .getTransactions()
                  .stream()
                  .filter(txn -> txn.getLedgerTransaction() instanceof FlashLedgerTransaction)
                  .toList();

      // Assert: We know the names of these flash transactions:
      assertEquals(
          Arrays.asList(
              "bottlenose-owner-role-getter",
              "bottlenose-locker-package",
              "bottlenose-account-try-deposit-or-refund",
              "bottlenose-protocol-params-to-state",
              "bottlenose-access-controller-xrd-fee-vault",
              "bottlenose-transaction-processor-blob-limits",
              "bottlenose-add-deferred-reference-check-cost",
              "bottlenose-restrict-role-assignment-reserved-role-key"),
          committedFlashTransactions.stream()
              .map(txn -> ((FlashLedgerTransaction) txn.getLedgerTransaction()).getName())
              .toList());

      // Assert: we know the contents of their receipts:
      ProtocolUpdateTestUtils.verifyFlashTransactionReceipts(committedFlashTransactions);
    }
  }
}
