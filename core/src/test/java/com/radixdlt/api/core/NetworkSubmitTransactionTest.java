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

import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransactionSuccess;
import static org.assertj.core.api.Assertions.*;

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.rev2.TransactionBuilder;
import org.junit.Test;

public class NetworkSubmitTransactionTest extends DeterministicCoreApiTestBase {
  @Test
  public void test_core_api_can_submit_and_commit_transaction() throws Exception {
    try (var test = buildRunningServerTest()) {

      var transaction = TransactionBuilder.forTests().prepare();

      // Submit transaction
      var response =
          getTransactionApi()
              .transactionSubmitPost(
                  new TransactionSubmitRequest()
                      .network(networkLogicalName)
                      .notarizedTransactionHex(transaction.hexPayloadBytes()));

      assertThat(response.getDuplicate()).isFalse();

      // Check that it's in mempool
      var statusResponse1 =
          getTransactionApi()
              .transactionStatusPost(
                  new TransactionStatusRequest()
                      .network(networkLogicalName)
                      .intentHash(transaction.hexIntentHash()));

      assertThat(statusResponse1.getIntentStatus()).isEqualTo(TransactionIntentStatus.INMEMPOOL);

      // Now we run consensus
      test.runUntilState(allCommittedTransactionSuccess(transaction.toRaw()), 1000);

      // Check the status response again
      var statusResponse2 =
          getTransactionApi()
              .transactionStatusPost(
                  new TransactionStatusRequest()
                      .network(networkLogicalName)
                      .intentHash(transaction.hexIntentHash()));

      assertThat(statusResponse2.getIntentStatus())
          .isEqualTo(TransactionIntentStatus.COMMITTEDSUCCESS);
    }
  }

  @Test
  @SuppressWarnings("try")
  public void test_valid_but_rejected_transaction_should_be_rejected() throws Exception {
    try (var ignored = buildRunningServerTest()) {
      var transaction = TransactionBuilder.forTests().manifest(Manifest.validButReject()).prepare();

      var response =
          assertErrorResponseOfType(
              () ->
                  getTransactionApi()
                      .transactionSubmitPost(
                          new TransactionSubmitRequest()
                              .network(networkLogicalName)
                              .notarizedTransactionHex(transaction.hexPayloadBytes())),
              TransactionSubmitErrorResponse.class);

      var details = response.getDetails();
      assertThat(details).isNotNull();

      assertThat(details).isInstanceOfAny(TransactionSubmitRejectedErrorDetails.class);
      var rejectedDetails = (TransactionSubmitRejectedErrorDetails) details;

      assertThat(response.getCode()).isEqualTo(400);
      assertThat(rejectedDetails).isNotNull();
      assertThat(rejectedDetails.getIsPayloadRejectionPermanent()).isFalse();
      assertThat(rejectedDetails.getIsIntentRejectionPermanent()).isFalse();
      assertThat(rejectedDetails.getIsRejectedBecauseIntentAlreadyCommitted()).isFalse();
      assertThat(rejectedDetails.getIsFresh()).isTrue();
      assertThat(rejectedDetails.getErrorMessage())
          .isEqualTo(
              "ErrorBeforeFeeLoanRepaid(ModuleError(CostingError(FeeReserveError(LoanRepaymentFailed))))");
    }
  }

  @Test
  public void
      test_valid_but_future_epoch_transaction_should_be_rejected_but_resubmittable_immediately_when_epoch_reached()
          throws Exception {
    try (var test = buildRunningServerTest(100)) {
      var transaction = TransactionBuilder.forTests().fromEpoch(2).prepare();

      var response =
          assertErrorResponseOfType(
              () ->
                  getTransactionApi()
                      .transactionSubmitPost(
                          new TransactionSubmitRequest()
                              .network(networkLogicalName)
                              .notarizedTransactionHex(transaction.hexPayloadBytes())),
              TransactionSubmitErrorResponse.class);

      var details = response.getDetails();
      assertThat(details).isNotNull();

      assertThat(details).isInstanceOfAny(TransactionSubmitRejectedErrorDetails.class);
      var rejectedDetails = (TransactionSubmitRejectedErrorDetails) details;

      assertThat(response.getCode()).isEqualTo(400);
      assertThat(rejectedDetails).isNotNull();
      assertThat(rejectedDetails.getIsPayloadRejectionPermanent()).isFalse();
      assertThat(rejectedDetails.getIsIntentRejectionPermanent()).isFalse();
      assertThat(rejectedDetails.getIsRejectedBecauseIntentAlreadyCommitted()).isFalse();
      assertThat(rejectedDetails.getIsFresh()).isTrue();
      assertThat(rejectedDetails.getRetryFromTimestamp()).isNull();
      assertThat(rejectedDetails.getRetryFromEpoch()).isEqualTo(2);
      assertThat(rejectedDetails.getErrorMessage())
          .isEqualTo("TransactionEpochNotYetValid { valid_from: 2, current_epoch: 0 }");

      // Now we run consensus up to epoch 2
      test.runUntilState(allAtOrOverEpoch(2), 10000);

      // And we resubmit, as the Gateway would - this time it should be submittable
      var response2 =
          getTransactionApi()
              .transactionSubmitPost(
                  new TransactionSubmitRequest()
                      .network(networkLogicalName)
                      .notarizedTransactionHex(transaction.hexPayloadBytes()));
      assertThat(response2.getDuplicate()).isFalse();

      // And get committed...
      test.runUntilState(allCommittedTransactionSuccess(transaction.toRaw()), 1000);

      // Check the status response again to check it's been marked as committed
      var statusResponse2 =
          getTransactionApi()
              .transactionStatusPost(
                  new TransactionStatusRequest()
                      .network(networkLogicalName)
                      .intentHash(transaction.hexIntentHash()));

      assertThat(statusResponse2.getIntentStatus())
          .isEqualTo(TransactionIntentStatus.COMMITTEDSUCCESS);
    }
  }
}
