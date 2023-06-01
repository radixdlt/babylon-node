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

package com.radixdlt.api.core.regression;

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.TransactionReceiptRequest;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.identifiers.Address;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.rev2.ResourceAddress;
import java.util.List;
import org.junit.Ignore;
import org.junit.Test;

public class NonFungibleActionsTest extends DeterministicCoreApiTestBase {
  @Test
  @Ignore // TODO - Babylon RCnet-V2 - unignore once fix is merged
  public void test_can_mint_and_burn_in_same_transaction_against_previously_used_resource()
      throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      var accountKeyPair = ECKeyPair.generateNew();
      var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());
      var resourceAddress = createFreeMintBurnNonFungibleResource(test);

      // These particular manifests caused a panic in the engine at Ash / Birch

      // First - we create some data in the resource to ensure the data index is created
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(1), List.of()),
          List.of(accountKeyPair));
      // A mint+burn of a non-pristine resource currently panics
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(2), List.of(2)),
          List.of(accountKeyPair));
    }
  }

  @Test
  public void can_mint_non_fungible_id_which_previously_existed_transiently_inside_a_transaction()
      throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      var accountKeyPair = ECKeyPair.generateNew();
      var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());
      var resourceAddress = createFreeMintBurnNonFungibleResource(test);

      // Mint and burn id 1 inside a single transaction
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(1), List.of(1)),
          List.of(accountKeyPair));

      // We mint the id "1" again, this should be allowed, because it previously never existed
      // outside a transaction
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(1), List.of()),
          List.of(accountKeyPair));
    }
  }

  @Test
  @Ignore // TODO - Babylon RCnet-V2 - unignore once fix is merged
  public void
      cannot_mint_non_fungible_id_which_previously_existed_outside_of_a_transaction_and_then_got_burned()
          throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      var accountKeyPair = ECKeyPair.generateNew();
      var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());
      var resourceAddress = createFreeMintBurnNonFungibleResource(test);

      // Mint id 1
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(1), List.of()),
          List.of(accountKeyPair));

      // Burn id 1
      submitAndWaitForSuccess(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(), List.of(1)),
          List.of(accountKeyPair));

      // We mint the id "1" again, this should be a failure
      submitAndWaitForCommittedFailure(
          test,
          Manifest.mintNonFungiblesThenWithdrawAndBurnSome(
              resourceAddress, accountAddress, List.of(1), List.of()),
          List.of(accountKeyPair));
    }
  }

  private ResourceAddress createFreeMintBurnNonFungibleResource(DeterministicTest test)
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
}
