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

import static org.junit.Assert.assertEquals;

import com.radixdlt.api.CoreApiHelper;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.TransactionV2Builder;
import com.radixdlt.transactions.PreparedNotarizedTransaction;
import java.util.List;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class CuttlefishSubintentTests {
  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(CoreApiHelper coreApiHelper) {
    var test = DeterministicTest.rev2DefaultWithCoreApi(1, 100, folder, coreApiHelper);
    test.startAllNodes();
    return test;
  }

  @Test
  public void v2_submit_subintents_in_multiple_transactions() throws Exception {
    final var coreApiHelper = new CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(coreApiHelper)) {
      // Try one by one - both transactions contain subintent 5
      var transactionAOne =
          TransactionV2Builder.forTests().subintentDiscriminators(List.of(1000, 5)).prepare();
      coreApiHelper.submitAndWaitForSuccess(test, transactionAOne);
      var transactionATwo =
          TransactionV2Builder.forTests().subintentDiscriminators(List.of(5)).prepare();
      var rejection = coreApiHelper.submitExpectingRejection(transactionATwo);
      assertEquals(rejection.getIsIntentRejectionPermanent(), true);

      // Try two at the same time - both transactions contain subintent 8
      var transactionBOne =
          TransactionV2Builder.forTests().subintentDiscriminators(List.of(8)).prepare();
      coreApiHelper.submit(transactionBOne);
      var transactionBTwo =
          TransactionV2Builder.forTests().subintentDiscriminators(List.of(8)).prepare();
      coreApiHelper.submit(transactionBTwo);

      var firstResult =
          coreApiHelper.waitForFirstResult(
              test,
              transactionBOne.transactionIntentHash(),
              transactionBTwo.transactionIntentHash());
      firstResult.assertCommittedSuccess();
      PreparedNotarizedTransaction otherTransaction;
      if (firstResult.transactionIntentHash().equals(transactionBOne.transactionIntentHash())) {
        otherTransaction = transactionBTwo;
      } else {
        otherTransaction = transactionBOne;
      }
      // TODO:CUTTLEFISH
      // >> re-enable these lines when we fix immediate rejection of intents with matching subintent
      // hash
      // var statusB = coreApiHelper.ltsTransactionStatus(otherTransaction);
      // assertEquals(statusB.getIntentStatus(), LtsTransactionIntentStatus.PERMANENTREJECTION);
      var rejectionB = coreApiHelper.submitExpectingRejection(otherTransaction);
      assertEquals(rejectionB.getIsIntentRejectionPermanent(), false);
    }
  }
}
