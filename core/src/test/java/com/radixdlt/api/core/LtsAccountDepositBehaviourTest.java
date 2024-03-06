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
import com.radixdlt.environment.DatabaseConfig;
import com.radixdlt.harness.deterministic.TransactionExecutor;
import com.radixdlt.identifiers.Address;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.rev2.ScryptoConstants;
import com.radixdlt.rev2.TransactionBuilder;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public final class LtsAccountDepositBehaviourTest extends DeterministicCoreApiTestBase {

  @Test
  public void account_with_default_config_allows_all_deposits() throws Exception {
    try (final var test = buildRunningServerTest(new DatabaseConfig(true, true))) {
      test.suppressUnusedWarning();

      // Arrange: pretty empty state
      final var accountKeyPair = ECKeyPair.generateNew();
      final var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());

      // Act: ask about 2 resources (XRD and not-XRD) from a non-existent virtual account
      final var result =
          getLtsApi()
              .ltsStateAccountDepositBehaviourPost(
                  new LtsStateAccountDepositBehaviourRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress))
                      .resourceAddresses(
                          List.of(
                              addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                              addressing.encode(
                                  ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS))));

      // Assert:
      assertThat(result.getIsBadgeAuthorizedDepositor()).isNull(); // was not requested
      assertThat(result.getDefaultDepositRule()).isEqualTo(DefaultDepositRule.ACCEPT);
      assertThat(result.getResourceSpecificBehaviours())
          .isEqualTo(
              Map.of(
                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(false)
                      .isXrd(true)
                      .allowsTryDeposit(true),
                  addressing.encode(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(false)
                      .isXrd(false)
                      .allowsTryDeposit(true)));

      // Follow-up: deposit some actual XRD into that account
      submitAndWaitForSuccess(test, Manifest.depositFromFaucet(accountAddress), List.of());

      // Act: this time also pass some dummy badge to be checked
      final var differentResult =
          getLtsApi()
              .ltsStateAccountDepositBehaviourPost(
                  new LtsStateAccountDepositBehaviourRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress))
                      .badge(
                          new ResourcePresentedBadge()
                              .resourceAddress(
                                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS)))
                      .resourceAddresses(
                          List.of(addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS))));

      // Assert: a slight change in the response to the same request (XRD vault now exists)
      assertThat(differentResult.getIsBadgeAuthorizedDepositor())
          .isFalse(); // badge was given, but no AD list exists
      assertThat(differentResult.getResourceSpecificBehaviours())
          .isEqualTo(
              Map.of(
                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(true)
                      .isXrd(true)
                      .allowsTryDeposit(true)));
    }
  }

  @Test
  public void account_with_reject_default_rule_disallows_all_deposits() throws Exception {
    try (final var test = buildRunningServerTest(new DatabaseConfig(true, true))) {
      test.suppressUnusedWarning();

      // Arrange: create account and set its default deposit rule to `Reject`
      var accountAddress =
          TransactionExecutor.executeTransaction(
                  test,
                  TransactionBuilder.forTests()
                      .manifest(
                          Manifest.newAccountAllowAllOwner()) // we are only testing `try_deposit_*`
                  )
              .newComponentAddresses()
              .get(0);
      TransactionExecutor.executeTransaction(
          test,
          TransactionBuilder.forTests()
              .manifest(Manifest.setDefaultDepositRule(accountAddress, "Reject")));

      // Act: ask about 2 resources (XRD and not-XRD)
      final var result =
          getLtsApi()
              .ltsStateAccountDepositBehaviourPost(
                  new LtsStateAccountDepositBehaviourRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress))
                      .resourceAddresses(
                          List.of(
                              addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                              addressing.encode(
                                  ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS))));

      // Assert:
      assertThat(result.getIsBadgeAuthorizedDepositor()).isNull(); // was not requested
      assertThat(result.getDefaultDepositRule()).isEqualTo(DefaultDepositRule.REJECT); // configured
      assertThat(result.getResourceSpecificBehaviours())
          .isEqualTo(
              Map.of(
                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(false)
                      .isXrd(true)
                      .allowsTryDeposit(false), // due to the default rule
                  addressing.encode(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(false)
                      .isXrd(false)
                      .allowsTryDeposit(false))); // due to the default rule
    }
  }

  @Test
  public void configured_resource_preference_and_depositor_badge_is_returned() throws Exception {
    try (final var test = buildRunningServerTest(new DatabaseConfig(true, true))) {
      test.suppressUnusedWarning();

      // Arrange: create account with some resource preference and some authorized depositor badge
      var accountAddress =
          TransactionExecutor.executeTransaction(
                  test,
                  TransactionBuilder.forTests()
                      .manifest(
                          Manifest.newAccountAllowAllOwner()) // we are only testing `try_deposit_*`
                  )
              .newComponentAddresses()
              .get(0);
      var depositorBadgeResource = // for easier test, we pick the "fungible" badge
          TransactionExecutor.executeTransaction(
                  test,
                  TransactionBuilder.forTests()
                      .manifest(Manifest.createDummyFungibleResource("FungibleDepositorBadge")))
              .newResourceAddresses()
              .get(0);
      TransactionExecutor.executeTransaction(
          test,
          TransactionBuilder.forTests()
              .manifest(
                  Manifest.setResourcePreference(
                      accountAddress, ScryptoConstants.XRD_RESOURCE_ADDRESS, "Disallowed")));
      TransactionExecutor.executeTransaction(
          test,
          TransactionBuilder.forTests()
              .manifest(Manifest.addAuthorizedDepositor(accountAddress, depositorBadgeResource)));

      // Act: ask about 2 resources (XRD and not-XRD), providing the right badge
      final var result =
          getLtsApi()
              .ltsStateAccountDepositBehaviourPost(
                  new LtsStateAccountDepositBehaviourRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress))
                      .badge(
                          new ResourcePresentedBadge()
                              .resourceAddress(addressing.encode(depositorBadgeResource)))
                      .resourceAddresses(
                          List.of(
                              addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                              addressing.encode(
                                  ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS))));

      // Assert:
      assertThat(result.getIsBadgeAuthorizedDepositor()).isTrue();
      assertThat(result.getDefaultDepositRule()).isEqualTo(DefaultDepositRule.ACCEPT);
      assertThat(result.getResourceSpecificBehaviours())
          .isEqualTo(
              Map.of(
                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(ResourcePreference.DISALLOWED) // configured
                      .vaultExists(false)
                      .isXrd(true)
                      .allowsTryDeposit(true), // allowed anyway due to AD badge
                  addressing.encode(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS),
                  new ResourceSpecificDepositBehaviour()
                      .resourcePreference(null)
                      .vaultExists(false)
                      .isXrd(false)
                      .allowsTryDeposit(true)));

      // Follow-up: present a badge that is not on the AD list; also, ask about 0 resources
      final var differentResult =
          getLtsApi()
              .ltsStateAccountDepositBehaviourPost(
                  new LtsStateAccountDepositBehaviourRequest()
                      .network(networkLogicalName)
                      .accountAddress(addressing.encode(accountAddress))
                      .badge(
                          new ResourcePresentedBadge()
                              .resourceAddress(
                                  addressing.encode(ScryptoConstants.XRD_RESOURCE_ADDRESS))));

      // Assert:
      assertThat(differentResult.getIsBadgeAuthorizedDepositor()).isFalse();
      assertThat(differentResult.getDefaultDepositRule()).isEqualTo(DefaultDepositRule.ACCEPT);
      assertThat(differentResult.getResourceSpecificBehaviours()).isNull();
    }
  }
}
