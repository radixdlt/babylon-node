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

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.DatabaseFlags;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Option;
import com.radixdlt.rev2.*;
import com.radixdlt.transactions.IntentHash;
import java.math.BigDecimal;
import java.util.List;
import java.util.Optional;
import org.junit.Test;

public class LtsTransactionOutcomesTest extends DeterministicCoreApiTestBase {
  @Test
  public void test_non_fungible_entity_changes() throws Exception {
    try (var test = buildRunningServerTest(new DatabaseFlags(true, true, false))) {
      test.suppressUnusedWarning();

      var accountKeyPair = ECKeyPair.generateNew();
      var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());
      var resourceAddress = createFreeMintBurnNonFungibleResource(test);

      // Single NF mint
      var tx1Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
                      resourceAddress, accountAddress, List.of(1), List.of()),
                  List.of(accountKeyPair)));
      assertThat(tx1Result.getNonFungibleEntityBalanceChanges().size()).isEqualTo(1);
      var tx1Changes = tx1Result.getNonFungibleEntityBalanceChanges().get(0);
      assertThat(tx1Changes.getAdded()).containsExactlyInAnyOrderElementsOf(List.of("#1#"));
      assertThat(tx1Changes.getRemoved()).isEmpty();

      // Transient token is not reported
      var tx2Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
                      resourceAddress, accountAddress, List.of(2), List.of(2)),
                  List.of(accountKeyPair)));
      assertThat(tx2Result.getNonFungibleEntityBalanceChanges()).isEmpty();

      // Multiple NF mint
      var tx3Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
                      resourceAddress, accountAddress, List.of(3, 4, 5), List.of()),
                  List.of(accountKeyPair)));
      assertThat(tx3Result.getNonFungibleEntityBalanceChanges().size()).isEqualTo(1);
      var tx3Changes = tx3Result.getNonFungibleEntityBalanceChanges().get(0);
      assertThat(tx3Changes.getRemoved()).isEmpty();
      assertThat(tx3Changes.getAdded())
          .containsExactlyInAnyOrderElementsOf(List.of("#3#", "#4#", "#5#"));

      // Multiple NF burn
      var tx4Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
                      resourceAddress, accountAddress, List.of(), List.of(3, 4, 5, 1)),
                  List.of(accountKeyPair)));
      assertThat(tx4Result.getNonFungibleEntityBalanceChanges().size()).isEqualTo(1);
      var tx4Changes = tx4Result.getNonFungibleEntityBalanceChanges().get(0);
      assertThat(tx4Changes.getAdded()).isEmpty();
      assertThat(tx4Changes.getRemoved())
          .containsExactlyInAnyOrderElementsOf(List.of("#1#", "#3#", "#4#", "#5#"));
    }
  }

  @Test
  public void test_resultant_account_balances() throws Exception {
    // We run all scenarios for the case when RE decides to change invariants (i.e. no vault
    // substate is deleted).
    try (var test = buildRunningServerTestWithScenarios(GenesisData.ALL_SCENARIOS)) {
      test.suppressUnusedWarning();

      var account1KeyPair = ECKeyPair.generateNew();
      var account1Address = Address.virtualAccountAddress(account1KeyPair.getPublicKey());

      var account2KeyPair = ECKeyPair.generateNew();
      var account2Address = Address.virtualAccountAddress(account2KeyPair.getPublicKey());

      var account1ExpectedAmount = 10000;
      var account2ExpectedAmount = 10000;
      var XRD = ScryptoConstants.XRD_RESOURCE_ADDRESS;

      var tx1Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test, Manifest.depositFromFaucet(account1Address), List.of()));
      assertThat(tx1Result.getResultantAccountFungibleBalances())
          .isEqualTo(
              List.of(account(account1Address, List.of(balance(account1ExpectedAmount, XRD)))));

      var tx2Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test, Manifest.depositFromFaucet(account2Address), List.of()));
      assertThat(tx2Result.getResultantAccountFungibleBalances())
          .isEqualTo(
              List.of(account(account2Address, List.of(balance(account2ExpectedAmount, XRD)))));

      var tx3Amount = 100;
      account1ExpectedAmount -= tx3Amount;
      account2ExpectedAmount += tx3Amount;
      var tx3Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.transferBetweenAccountsFeeFromFaucet(
                      account1Address, XRD, Decimal.ofNonNegative(tx3Amount), account2Address),
                  List.of(account1KeyPair)));
      assertThat(
              findAccount(tx3Result.getResultantAccountFungibleBalances(), account1Address)
                  .getResultantBalances())
          .isEqualTo(List.of(balance(account1ExpectedAmount, XRD)));
      assertThat(
              findAccount(tx3Result.getResultantAccountFungibleBalances(), account2Address)
                  .getResultantBalances())
          .isEqualTo(List.of(balance(account2ExpectedAmount, XRD)));

      var tx4Result =
          getSingleCommittedTransactionOutcome(
              submitAndWaitForSuccess(
                  test,
                  Manifest.transferBetweenAccountsFeeFromFaucet(
                      account2Address, XRD, Decimal.ofNonNegative(450), account2Address),
                  List.of(account2KeyPair)));
      assertThat(tx4Result.getResultantAccountFungibleBalances()).isEqualTo(List.of());
    }
  }

  private LtsResultantAccountFungibleBalances findAccount(
      List<LtsResultantAccountFungibleBalances> resultantAccountFungibleBalances,
      ComponentAddress account) {
    var accountStr = account.encode(networkDefinition);
    for (LtsResultantAccountFungibleBalances resultantAccountFungibleBalance :
        resultantAccountFungibleBalances) {
      if (resultantAccountFungibleBalance.getAccountAddress().equals(accountStr)) {
        return resultantAccountFungibleBalance;
      }
    }
    return null;
  }

  private LtsResultantFungibleBalance balance(long amount, ResourceAddress resourceAddress) {
    return new LtsResultantFungibleBalance()
        .resultantBalance(Decimal.ofNonNegative(amount).toString())
        .resourceAddress(resourceAddress.encode(networkDefinition));
  }

  private LtsResultantAccountFungibleBalances account(
      ComponentAddress account, List<LtsResultantFungibleBalance> ltsResultantFungibleBalance) {
    return new LtsResultantAccountFungibleBalances()
        .resultantBalances(ltsResultantFungibleBalance)
        .accountAddress(account.encode(networkDefinition));
  }

  private LtsCommittedTransactionOutcome getSingleCommittedTransactionOutcome(
      CommittedResult committedResult) throws Exception {
    var outcomes =
        getLtsApi()
            .ltsStreamTransactionOutcomesPost(
                new LtsStreamTransactionOutcomesRequest()
                    .network(networkLogicalName)
                    .fromStateVersion(committedResult.stateVersion())
                    .limit(1))
            .getCommittedTransactionOutcomes();
    assertThat(outcomes.size()).isEqualTo(1);
    return outcomes.get(0);
  }

  @Test
  public void test_multiple_transactions_have_correct_outcomes() throws Exception {
    try (var test = buildRunningServerTest(new DatabaseFlags(true, true, false))) {
      test.suppressUnusedWarning();

      var faucetAddressStr = ScryptoConstants.FAUCET_ADDRESS.encode(networkDefinition);

      var account1KeyPair = ECKeyPair.generateNew();
      var account1Address = Address.virtualAccountAddress(account1KeyPair.getPublicKey());
      var account1AddressStr = account1Address.encode(networkDefinition);

      var account2KeyPair = ECKeyPair.generateNew();
      var account2Address = Address.virtualAccountAddress(account2KeyPair.getPublicKey());
      var account2AddressStr = account2Address.encode(networkDefinition);

      var account1FaucetClaim =
          submitAndWaitForSuccess(test, Manifest.depositFromFaucet(account1Address), List.of());
      var account2FaucetClaim =
          submitAndWaitForSuccess(test, Manifest.depositFromFaucet(account2Address), List.of());

      var account1SelfXrdTransferAmount = 1L;
      var account1SelfXrdTransfer =
          submitAndWaitForSuccess(
              test,
              Manifest.transferBetweenAccountsFeeFromSender(
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.ofNonNegative(account1SelfXrdTransferAmount),
                  account1Address),
              List.of(account1KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromAccount1Amount = 5;
      var account1ToAccount2XrdTransferWithFeeFromAccount1 =
          submitAndWaitForSuccess(
              test,
              Manifest.transferBetweenAccountsFeeFromSender(
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.ofNonNegative(account1ToAccount2XrdTransferWithFeeFromAccount1Amount),
                  account2Address),
              List.of(account1KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromAccount2Amount = 31;
      var account1ToAccount2XrdTransferWithFeeFromAccount2 =
          submitAndWaitForSuccess(
              test,
              Manifest.transferBetweenAccountsFeeFromReceiver(
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.ofNonNegative(account1ToAccount2XrdTransferWithFeeFromAccount2Amount),
                  account2Address),
              List.of(account1KeyPair, account2KeyPair));

      var account1ToAccount2XrdTransferWithFeeFromFaucetAmount = 6;
      var account1ToAccount2XrdTransferWithFeeFromFaucet =
          submitAndWaitForSuccess(
              test,
              Manifest.transferBetweenAccountsFeeFromFaucet(
                  account1Address,
                  ScryptoConstants.XRD_RESOURCE_ADDRESS,
                  Decimal.ofNonNegative(account1ToAccount2XrdTransferWithFeeFromFaucetAmount),
                  account2Address),
              List.of(account1KeyPair));

      validateAccountTransactions(
          account1Address,
          List.of(
              account1FaucetClaim.intentHash(),
              account1SelfXrdTransfer.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromAccount1.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromAccount2.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromFaucet.intentHash()));
      validateAccountTransactions(
          account2Address,
          List.of(
              account2FaucetClaim.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromAccount1.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromAccount2.intentHash(),
              account1ToAccount2XrdTransferWithFeeFromFaucet.intentHash()));

      var faucetFreeXrdAmount = 10000L;
      assertNonFeeXrdBalanceChange(
          account1FaucetClaim.stateVersion(), faucetAddressStr, -faucetFreeXrdAmount);
      assertNonFeeXrdBalanceChange(
          account1FaucetClaim.stateVersion(), account1AddressStr, faucetFreeXrdAmount);
      assertNoNonFeeXrdBalanceChange(account1FaucetClaim.stateVersion(), account2AddressStr);

      assertNonFeeXrdBalanceChange(
          account2FaucetClaim.stateVersion(), faucetAddressStr, -faucetFreeXrdAmount);
      assertNoNonFeeXrdBalanceChange(account2FaucetClaim.stateVersion(), account1AddressStr);
      assertNonFeeXrdBalanceChange(
          account2FaucetClaim.stateVersion(), account2AddressStr, faucetFreeXrdAmount);

      // In the self-transfer, there was no net balance transfer
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion(), faucetAddressStr);
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion(), account1AddressStr);
      assertNoNonFeeXrdBalanceChange(account1SelfXrdTransfer.stateVersion(), account2AddressStr);

      // Check
      assertNoNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion(), faucetAddressStr);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion(),
          account1AddressStr,
          -account1ToAccount2XrdTransferWithFeeFromAccount1Amount);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount1.stateVersion(),
          account2AddressStr,
          account1ToAccount2XrdTransferWithFeeFromAccount1Amount);

      assertNoNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion(), faucetAddressStr);

      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion(),
          account1AddressStr,
          -account1ToAccount2XrdTransferWithFeeFromAccount2Amount);

      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromAccount2.stateVersion(),
          account2AddressStr,
          account1ToAccount2XrdTransferWithFeeFromAccount2Amount);

      // Even though the faucet paid the fee, it didn't have any other balance transfers
      assertNoNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion(), faucetAddressStr);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion(),
          account1AddressStr,
          -account1ToAccount2XrdTransferWithFeeFromFaucetAmount);
      assertNonFeeXrdBalanceChange(
          account1ToAccount2XrdTransferWithFeeFromFaucet.stateVersion(),
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

  @SuppressWarnings("SameParameterValue")
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
                            item.getResourceAddress().equals(addressing.encode(resourceAddress)))
                    .findFirst());
  }

  private void validateAccountTransactions(
      ComponentAddress accountAddress, List<IntentHash> intentHashes) throws Exception {
    var accountOutcomesResponse =
        getLtsApi()
            .ltsStreamAccountTransactionOutcomesPost(
                new LtsStreamAccountTransactionOutcomesRequest()
                    .network(networkLogicalName)
                    .fromStateVersion(1L)
                    .limit(1000)
                    .accountAddress(addressing.encode(accountAddress)));
    var outcomes = accountOutcomesResponse.getCommittedTransactionOutcomes();
    assertThat(outcomes.size()).isEqualTo(intentHashes.size());
    for (var i = 0; i < outcomes.size(); i++) {
      var outcome = outcomes.get(i);
      if (outcome.getStatus() != LtsCommittedTransactionStatus.SUCCESS) {
        throw new RuntimeException("Status is not success");
      }
      var transactionIdentifiers = outcome.getUserTransactionIdentifiers();
      assertThat(transactionIdentifiers).isNotNull();
      assertThat(transactionIdentifiers.getIntentHash()).isEqualTo(intentHashes.get(i).hex());
    }
  }
}
