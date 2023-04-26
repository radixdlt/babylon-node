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

import static com.radixdlt.rev2.REv2TestTransactions.*;
import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.hash.HashCode;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Option;
import com.radixdlt.rev2.*;
import com.radixdlt.utils.Bytes;
import java.math.BigDecimal;
import java.util.List;
import java.util.Optional;
import org.junit.Test;

public class LtsTransactionOutcomesTest extends DeterministicCoreApiTestBase {
  @Test
  public void test_multiple_transactions_have_correct_outcomes() throws Exception {
    try (var test = buildRunningServerTest(true)) {

      var faucetAddress = ScryptoConstants.FAUCET_COMPONENT_ADDRESS;
      var faucetAddressStr = addressing.encodeNormalComponentAddress(faucetAddress);
      var account1KeyPair = ECKeyPair.generateNew();
      var account1Address = Address.virtualAccountAddress(account1KeyPair.getPublicKey());
      var account1AddressStr = addressing.encodeAccountAddress(account1Address);

      var account2KeyPair = ECKeyPair.generateNew();
      var account2Address = Address.virtualAccountAddress(account2KeyPair.getPublicKey());
      var account2AddressStr = addressing.encodeAccountAddress(account2Address);

      var account1FaucetClaim =
          submitAndWaitForSuccess(
              test,
              constructDepositFromFaucetManifest(networkDefinition, account1Address),
              List.of());
      var account2FaucetClaim =
          submitAndWaitForSuccess(
              test,
              constructDepositFromFaucetManifest(networkDefinition, account2Address),
              List.of());

      var account1SelfXrdTransferAmount = 1L;
      var account1SelfXrdTransfer =
          submitAndWaitForSuccess(
              test,
              constructTransferBetweenAccountsFeeFromSender(
                  networkDefinition,
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.of(account1SelfXrdTransferAmount),
                  account1Address),
              List.of(account1KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromAccount1Amount = 5;
      var account1ToAccount2XrdTransferWithFeeFromAccount1 =
          submitAndWaitForSuccess(
              test,
              constructTransferBetweenAccountsFeeFromSender(
                  networkDefinition,
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.of(account1ToAccount2XrdTransferWithFeeFromAccount1Amount),
                  account2Address),
              List.of(account1KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromAccount2Amount = 31;
      var account1ToAccount2XrdTransferWithFeeFromAccount2 =
          submitAndWaitForSuccess(
              test,
              constructTransferBetweenAccountsFeeFromReceiver(
                  networkDefinition,
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.of(account1ToAccount2XrdTransferWithFeeFromAccount2Amount),
                  account2Address),
              List.of(account1KeyPair, account2KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromFaucetAmount = 6;
      var account1ToAccount2XrdTransferWithFeeFromFaucet =
          submitAndWaitForSuccess(
              test,
              constructTransferBetweenAccountsFeeFromFaucet(
                  networkDefinition,
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.of(account1ToAccount2XrdTransferWithFeeFromFaucetAmount),
                  account2Address),
              List.of(account1KeyPair));

      validateAccountTransactions(
          account1Address,
          List.of(
              account1FaucetClaim.intentHash,
              account1SelfXrdTransfer.intentHash,
              account1ToAccount2XrdTransferWithFeeFromAccount1.intentHash,
              account1ToAccount2XrdTransferWithFeeFromAccount2.intentHash,
              account1ToAccount2XrdTransferWithFeeFromFaucet.intentHash));
      validateAccountTransactions(
          account2Address,
          List.of(
              account2FaucetClaim.intentHash,
              account1ToAccount2XrdTransferWithFeeFromAccount1.intentHash,
              account1ToAccount2XrdTransferWithFeeFromAccount2.intentHash,
              account1ToAccount2XrdTransferWithFeeFromFaucet.intentHash));

      var faucetFreeXrdAmount = 10000L;
      assertNonFeeXrdBalanceChange(
          account1FaucetClaim.stateVersion, faucetAddressStr, -faucetFreeXrdAmount);
      assertNonFeeXrdBalanceChange(
          account1FaucetClaim.stateVersion, account1AddressStr, faucetFreeXrdAmount);
      assertNoNonFeeXrdBalanceChange(account1FaucetClaim.stateVersion, account2AddressStr);

      assertNonFeeXrdBalanceChange(
          account2FaucetClaim.stateVersion, faucetAddressStr, -faucetFreeXrdAmount);
      assertNoNonFeeXrdBalanceChange(account2FaucetClaim.stateVersion, account1AddressStr);
      assertNonFeeXrdBalanceChange(
          account2FaucetClaim.stateVersion, account2AddressStr, faucetFreeXrdAmount);

      // In the self-transfer, there was no net balance transfer
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion, faucetAddressStr);
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion, account1AddressStr);
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion, account2AddressStr);

      // Check
      assertNoNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion, faucetAddressStr);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion,
          account1AddressStr,
          -account1ToAccount2XrdTransferWithFeeFromAccount1Amount);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion,
          account2AddressStr,
          account1ToAccount2XrdTransferWithFeeFromAccount1Amount);

      // Slightly weird transaction, the fee payment calculation guesses wrong for now.
      // TODO: Uncomment this test when fee calculation is fixed to make fewer assumptions
      //
      // assertNoNonFeeXrdBalanceChange(account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion, faucetAddressStr);
      //
      // assertNonFeeXrdBalanceChange(account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion,
      // account1AddressStr, -account1ToAccount2XrdTransferWithFeeFromAccount2Amount);
      //
      // assertNonFeeXrdBalanceChange(account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion,
      // account2AddressStr, account1ToAccount2XrdTransferWithFeeFromAccount2Amount);

      // Even though the faucet paid the fee, it didn't have any other balance transfers
      assertNoNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion, faucetAddressStr);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion,
          account1AddressStr,
          -account1ToAccount2XrdTransferWithFeeFromFaucetAmount);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion,
          account2AddressStr,
          account1ToAccount2XrdTransferWithFeeFromFaucetAmount);
    }
  }

  private void assertNonFeeXrdBalanceChange(
      long stateVersion, String entityAddress, long balanceChange) throws Exception {
    assertNonFeeBalanceChange(
        stateVersion,
        entityAddress,
        ScryptoConstants.XRD_RESOURCE_ADDRESS,
        Option.some(BigDecimal.valueOf(balanceChange)));
  }

  private void assertNoNonFeeXrdBalanceChange(long stateVersion, String entityAddress)
      throws Exception {
    assertNonFeeBalanceChange(
        stateVersion, entityAddress, ScryptoConstants.XRD_RESOURCE_ADDRESS, Option.none());
  }

  private void assertNonFeeBalanceChange(
      long stateVersion,
      String entityAddress,
      ResourceAddress resourceAddress,
      Option<BigDecimal> balanceChange)
      throws Exception {
    if (balanceChange.isEmpty()) {
      assertThat(
              getTransactionOutcomeNonFeeBalanceChange(
                  stateVersion, entityAddress, resourceAddress))
          .isEqualTo(Optional.empty());
    } else {
      var loadedChange =
          getTransactionOutcomeNonFeeBalanceChange(stateVersion, entityAddress, resourceAddress);
      assertThat(loadedChange).isPresent();
      assertThat(new BigDecimal(loadedChange.get().getBalanceChange()))
          .isEqualTo(balanceChange.unwrap());
    }
  }

  private Optional<LtsFungibleResourceBalanceChange> getTransactionOutcomeNonFeeBalanceChange(
      long stateVersion, String entityAddress, ResourceAddress resourceAddress) throws Exception {
    var outcomeResponse =
        getLtsApi()
            .ltsStreamTransactionOutcomesPost(
                new LtsStreamTransactionOutcomesRequest()
                    .network(networkLogicalName)
                    .fromStateVersion(stateVersion)
                    .limit(1));
    assertThat(outcomeResponse.getCommittedTransactionOutcomes().size()).isEqualTo(1);
    var outcome = outcomeResponse.getCommittedTransactionOutcomes().get(0);
    return outcome.getFungibleEntityBalanceChanges().stream()
        .filter(item -> item.getEntityAddress().equals(entityAddress))
        .findFirst()
        .flatMap(
            ltsEntityFungibleBalanceChanges ->
                ltsEntityFungibleBalanceChanges.getNonFeeBalanceChanges().stream()
                    .filter(
                        item ->
                            item.getResourceAddress()
                                .equals(addressing.encodeResourceAddress(resourceAddress)))
                    .findFirst());
  }

  private void validateAccountTransactions(
      ComponentAddress accountAddress, List<HashCode> intentHashes) throws Exception {
    var accountOutcomesResponse =
        getLtsApi()
            .ltsStreamAccountTransactionOutcomesPost(
                new LtsStreamAccountTransactionOutcomesRequest()
                    .network(networkLogicalName)
                    .fromStateVersion(1L)
                    .limit(1000)
                    .accountAddress(addressing.encodeAccountAddress(accountAddress)));
    var outcomes = accountOutcomesResponse.getCommittedTransactionOutcomes();
    assertThat(outcomes.size()).isEqualTo(intentHashes.size());
    for (var i = 0; i < outcomes.size(); i++) {
      var outcome = outcomes.get(i);
      if (outcome.getStatus() != LtsCommittedTransactionStatus.SUCCESS) {
        throw new RuntimeException("Status is not success");
      }
      var transactionIdentifiers = outcome.getUserTransactionIdentifiers();
      assertThat(transactionIdentifiers).isNotNull();
      assertThat(transactionIdentifiers.getIntentHash())
          .isEqualTo(Bytes.toHexString(intentHashes.get(i).asBytes()));
    }
  }

  public record CommittedResult(HashCode intentHash, long stateVersion) {}

  public CommittedResult submitAndWaitForSuccess(
      DeterministicTest test, String manifest, List<ECKeyPair> signatories) throws Exception {
    var metadata =
        getLtsApi()
            .ltsTransactionConstructionPost(
                new LtsTransactionConstructionRequest().network(networkLogicalName));

    var transactionBuilder =
        buildTransactionWithDefaultNotary(
            networkDefinition, manifest, metadata.getCurrentEpoch(), 0, signatories);

    var intentHash = transactionBuilder.hashedIntent();
    var payload = transactionBuilder.constructRawTransaction().getPayload();

    var submitResponse =
        getLtsApi()
            .ltsTransactionSubmitPost(
                new LtsTransactionSubmitRequest()
                    .network(networkLogicalName)
                    .notarizedTransactionHex(Bytes.toHexString(payload)));

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
                      .intentHash(Bytes.toHexString(intentHash.asBytes())));
      switch (statusResponse.getIntentStatus()) {
        case COMMITTEDSUCCESS -> {
          var stateVersion = statusResponse.getCommittedStateVersion();
          if (stateVersion == null) {
            throw new RuntimeException(
                "Transaction got committed as success without state version on response");
          }
          return new CommittedResult(intentHash, stateVersion);
        }
        case COMMITTEDFAILURE -> throw new RuntimeException("Transaction got committed as failure");
        case PERMANENTREJECTION -> throw new RuntimeException(
            "Transaction got permanently rejected");
        default -> test.runForCount(messagesProcessedPerAttempt);
      }
    }
    throw new RuntimeException(
        String.format(
            "Transaction submit didn't complete in after running for count of %s. Status still: %s",
            attempts * messagesProcessedPerAttempt, statusResponse.getIntentStatus()));
  }
}
