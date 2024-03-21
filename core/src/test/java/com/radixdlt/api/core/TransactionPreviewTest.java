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

package com.radixdlt.api.core;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.MoreCollectors;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.DatabaseConfig;
import com.radixdlt.environment.StateHashTreeGcConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Functions;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.utils.Bytes;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Predicate;
import java.util.stream.Stream;
import org.awaitility.Awaitility;
import org.junit.Test;

public class TransactionPreviewTest extends DeterministicCoreApiTestBase {

  // a known amount coming from faucet; for assert purposes
  private static final double FAUCET_AMOUNT = 10000;

  @Test
  public void transaction_preview_executes_at_historical_version() throws Exception {
    try (var test = buildTest(true, 20L)) {
      test.suppressUnusedWarning();

      // Prepare a simple manifest which just gets some amount from faucet, once:
      var accountKeyPair = ECKeyPair.generateNew();
      var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());
      var manifest = Manifest.depositFromFaucet(accountAddress);

      // Execute it once, to initialize the account and learn its XRD vault address:
      var firstCommit =
          getApiHelper().submitAndWaitForSuccess(test, manifest, List.of(accountKeyPair));
      var initialVaultBalance =
          this.getStateApi()
              .stateAccountPost(
                  new StateAccountRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress)))
              .getVaults()
              .stream()
              .collect(MoreCollectors.onlyElement());
      var vaultAddress = initialVaultBalance.getVaultEntity().getEntityAddress();

      // Sanity check - the vault has a known balance (the "1x from Faucet" amount):
      var initialAmount = (FungibleResourceAmount) initialVaultBalance.getResourceAmount();
      assertThat(Double.parseDouble(initialAmount.getAmount())).isEqualTo(FAUCET_AMOUNT);

      // Sanity check - a preview of the same manifest at the current version should result in the
      // vault having "2x from Faucet" amount:
      var previewedDirectlyAfterFirstCommit = previewAtVersion(manifest, Optional.empty());
      assertThat(getVaultBalance(previewedDirectlyAfterFirstCommit, vaultAddress))
          .isEqualTo(2 * FAUCET_AMOUNT);

      // Execute precisely the deposit that was just previewed:
      var secondCommit =
          getApiHelper().submitAndWaitForSuccess(test, manifest, List.of(accountKeyPair));
      assertThat(secondCommit.stateVersion()).isGreaterThan(firstCommit.stateVersion()); // (sanity)

      // Sanity check - a preview now should give "3x from Faucet" amount:
      var previewedAfterSecondCommit = previewAtVersion(manifest, Optional.empty());
      assertThat(getVaultBalance(previewedAfterSecondCommit, vaultAddress))
          .isEqualTo(3 * FAUCET_AMOUNT);

      // And a true assert: a preview executed *at version* of the first commit returns exactly the
      // same receipt as the preview executed *directly* on top of the first commit.
      var previewedAsOfAfterFirstCommit =
          previewAtVersion(manifest, Optional.of(firstCommit.stateVersion()));
      assertThat(previewedAsOfAfterFirstCommit).isEqualTo(previewedDirectlyAfterFirstCommit);
    }
  }

  @Test
  public void transaction_preview_refuses_too_old_state_version() throws Exception {
    final var historyLength = 7L;
    final var inspectedAtVersion = 23L;
    final var oldestAvailableVersion = inspectedAtVersion - historyLength;
    try (var test = buildTest(true, historyLength)) {
      test.suppressUnusedWarning();

      // Reach a known state version:
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(inspectedAtVersion));

      // Wait for the async GC to catch up its target:
      Awaitility.await()
          .until(
              test.getInstance(0, TestStateReader.class)::getLeastStaleStateHashTreeVersion,
              Predicate.isEqual(oldestAvailableVersion));

      // Assert that the oldest available version is fine:
      var atOldestVersion = previewAtVersion(Manifest.valid(), Optional.of(oldestAvailableVersion));
      assertThat(atOldestVersion.getStatus()).isEqualTo(TransactionStatus.SUCCEEDED);

      // ... but the 1-too-old is not:
      var atTooOldVersion =
          assertErrorResponseOfType(
              () -> previewAtVersion(Manifest.valid(), Optional.of(oldestAvailableVersion - 1)),
              BasicErrorResponse.class);
      assertThat(atTooOldVersion.getMessage()).containsIgnoringCase("cannot request state version");
      assertThat(atTooOldVersion.getMessage()).contains(String.valueOf(oldestAvailableVersion));
    }
  }

  @Test
  public void transaction_preview_refuses_future_state_version() throws Exception {
    try (var test = buildTest(true, 100L)) {
      test.suppressUnusedWarning();

      // Reach a known state version:
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(10L));

      // Assert that a future version is not available:
      var atTooOldVersion =
          assertErrorResponseOfType(
              () -> previewAtVersion(Manifest.valid(), Optional.of(11L)), BasicErrorResponse.class);
      assertThat(atTooOldVersion.getMessage())
          .containsIgnoringCase("state version ahead of the current top-of-ledger 10");
    }
  }

  @Test
  public void transaction_preview_does_not_support_state_version_when_disabled() throws Exception {
    try (var test = buildTest(false, 100L)) {
      test.suppressUnusedWarning();

      // Reach a known state version:
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(10L));

      // Assert that a historical state version is not available
      var atTooOldVersion =
          assertErrorResponseOfType(
              () -> previewAtVersion(Manifest.valid(), Optional.of(9L)), BasicErrorResponse.class);
      assertThat(atTooOldVersion.getMessage()).containsIgnoringCase("feature must be enabled");

      // The current version can still be requested explicitly, though:
      var atOldestVersion = previewAtVersion(Manifest.valid(), Optional.of(10L));
      assertThat(atOldestVersion.getStatus()).isEqualTo(TransactionStatus.SUCCEEDED);
    }
  }

  @SuppressWarnings("DataFlowIssue") // Suppress invalid null reference warnings
  @Test
  public void transaction_previewed_with_message_consumes_more_cost_units() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // Prepare a base request (no message)
      var manifest = Manifest.valid().apply(new Manifest.Parameters(networkDefinition));
      var baseRequest =
          new TransactionPreviewRequest()
              .network(networkLogicalName)
              .startEpochInclusive(0L)
              .endEpochExclusive(100L)
              .tipPercentage(1)
              .nonce(10L)
              .flags(
                  new TransactionPreviewRequestFlags()
                      .useFreeCredit(true)
                      .assumeAllSignatureProofs(true)
                      .skipEpochCheck(true))
              .manifest(manifest);

      // Prepare a complex message separately
      var largeEncryptedMessage =
          new EncryptedTransactionMessage()
              .encryptedHex(Bytes.toHexString(new byte[1000]))
              .addCurveDecryptorSetsItem(
                  new EncryptedMessageCurveDecryptorSet()
                      .dhEphemeralPublicKey(
                          new EcdsaSecp256k1PublicKey()
                              .keyHex(
                                  Bytes.toHexString(
                                      PrivateKeys.ofNumeric(1).getPublicKey().getBytes())))
                      .addDecryptorsItem(
                          new EncryptedMessageDecryptor()
                              .publicKeyFingerprintHex(Bytes.toHexString(new byte[8]))
                              .aesWrappedKeyHex(Bytes.toHexString(new byte[24]))));

      // Ask for costing of a base request
      var noMessageCost =
          getTransactionApi()
              .transactionPreviewPost(baseRequest)
              .getReceipt()
              .getFeeSummary()
              .getExecutionCostUnitsConsumed();

      // Ask for costing of a preview request with a large message
      var largeEncryptedMessageCost =
          getTransactionApi()
              .transactionPreviewPost(baseRequest.message(largeEncryptedMessage))
              .getReceipt()
              .getFeeSummary()
              .getExecutionCostUnitsConsumed();

      // Message should add some cost
      assertThat(largeEncryptedMessageCost).isGreaterThan(noMessageCost);
    }
  }

  private TransactionReceipt previewAtVersion(
      Functions.Func1<Manifest.Parameters, String> manifest, Optional<Long> atStateVersion)
      throws ApiException {
    return getTransactionApi()
        .transactionPreviewPost(
            new TransactionPreviewRequest()
                .network(networkLogicalName)
                .startEpochInclusive(1L)
                .endEpochExclusive(100L)
                .tipPercentage(1)
                .nonce(10L)
                .atLedgerState(
                    atStateVersion
                        .map(
                            version ->
                                new VersionLedgerStateCoordinate()
                                    .stateVersion(version)
                                    .type(LedgerStateCoordinateType.BYSTATEVERSION))
                        .orElse(null))
                .flags(
                    new TransactionPreviewRequestFlags()
                        .useFreeCredit(false)
                        .skipEpochCheck(false)
                        .assumeAllSignatureProofs(true))
                .manifest(manifest.apply(new Manifest.Parameters(networkDefinition))))
        .getReceipt();
  }

  private static double getVaultBalance(TransactionReceipt receipt, String vaultAddress) {
    assertThat(receipt.getErrorMessage()).isNull();
    var changes = receipt.getStateUpdates();
    var upserts =
        Stream.concat(
            changes.getCreatedSubstates().stream()
                .map(created -> Map.entry(created.getSubstateId(), created.getValue())),
            changes.getUpdatedSubstates().stream()
                .map(updated -> Map.entry(updated.getSubstateId(), updated.getNewValue())));
    var balanceSubstate =
        (FungibleVaultFieldBalanceSubstate)
            upserts
                .filter(entry -> entry.getKey().getEntityAddress().equals(vaultAddress))
                .filter(
                    entry ->
                        entry.getKey().getSubstateType() == SubstateType.FUNGIBLEVAULTFIELDBALANCE)
                .collect(MoreCollectors.onlyElement())
                .getValue()
                .getSubstateData();
    return Double.parseDouble(balanceSubstate.getValue().getAmount());
  }

  private DeterministicTest buildTest(boolean stateHistoryEnabled, long historyLength) {
    return buildRunningServerTest(
        new DatabaseConfig(true, false, stateHistoryEnabled, false),
        new StateHashTreeGcConfig(
            UInt32.fromNonNegativeInt(1), UInt64.fromNonNegativeLong(historyLength)));
  }
}
