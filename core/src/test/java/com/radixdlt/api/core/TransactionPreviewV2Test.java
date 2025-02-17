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

import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverStateVersion;
import static org.assertj.core.api.Assertions.assertThat;

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.environment.DatabaseConfig;
import com.radixdlt.environment.StateTreeGcConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.rev2.TransactionV2Builder;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.List;
import org.junit.Test;

public class TransactionPreviewV2Test extends DeterministicCoreApiTestBase {

  private DeterministicTest buildTest(boolean stateHistoryEnabled, long historyLength) {
    return buildRunningServerTest(
        defaultConfig()
            .withDatabaseConfig(new DatabaseConfig(true, false, stateHistoryEnabled, false))
            .withStateTreeGcConfig(
                new StateTreeGcConfig(
                    UInt32.fromNonNegativeInt(1), UInt64.fromNonNegativeLong(historyLength))));
  }

  @Test
  public void transaction_preview_v2_succeeds_with_minimal_options() throws Exception {
    try (var test = buildTest(false, 0)) {
      test.suppressUnusedWarning();

      test.runUntilState(allAtOrOverStateVersion(30));

      var currentStateVersion =
          getCoreApiHelper().getNetworkStatus().getCurrentStateIdentifier().getStateVersion();

      var request = createMinimalRequest();

      // Act
      var response = getCoreApiHelper().transactionApi().transactionPreviewV2Post(request);

      // Assert
      assertThat(response.getReceipt()).isNotNull();
      assertThat(response.getReceipt().getStatus()).isEqualTo(TransactionStatus.SUCCEEDED);
      assertThat(response.getRadixEngineToolkitReceipt()).isNull();
      assertThat(response.getLogs()).isNull();
      assertThat(response.getAtLedgerState().getStateVersion()).isEqualTo(currentStateVersion);
    }
  }

  @Test
  public void transaction_preview_v2_succeeds_with_all_overrides_on() throws Exception {
    try (var test = buildTest(true, 20L)) {
      test.suppressUnusedWarning();

      test.runUntilState(allAtOrOverStateVersion(110));

      var previewFlags =
          new PreviewFlags()
              .assumeAllSignatureProofs(true)
              .skipEpochCheck(true)
              .disableAuthChecks(true)
              .useFreeCredit(true);
      var options =
          new TransactionPreviewV2ResponseOptions()
              .coreApiReceipt(false) // Reverse normal
              .radixEngineToolkitReceipt(true)
              .logs(true);
      var request =
          createMinimalRequest()
              .atLedgerState(new VersionLedgerStateSelector().stateVersion(100L))
              .flags(previewFlags)
              .options(options);

      // Act
      var response = getCoreApiHelper().transactionApi().transactionPreviewV2Post(request);

      // Assert
      assertThat(response.getReceipt()).isNull();
      assertThat(response.getRadixEngineToolkitReceipt()).isNotNull();
      assertThat(response.getLogs()).isNotNull();
      assertThat(response.getAtLedgerState().getStateVersion()).isEqualTo(100);
    }
  }

  @Test
  public void transaction_preview_v2_of_invalid_fails_with_typed_error() throws Exception {
    try (var test = buildTest(false, 0)) {
      test.suppressUnusedWarning();

      test.runUntilState(allAtOrOverStateVersion(30));

      var request = createInvalidRequest();

      // Act / Assert
      var errorResponse =
          getCoreApiHelper()
              .assertErrorResponseOfType(
                  () -> {
                    getCoreApiHelper().transactionApi().transactionPreviewV2Post(request);
                  },
                  TransactionPreviewV2ErrorResponse.class);

      // Assert
      var invalidDetails = (InvalidTransactionPreviewV2ErrorDetails) errorResponse.getDetails();
      assertThat(invalidDetails).isNotNull();
      assertThat(invalidDetails.getValidationError().length()).isGreaterThan(10);
    }
  }

  TransactionPreviewV2Request createMinimalRequest() {
    var prepared =
        TransactionV2Builder.forTests()
            .subintentDiscriminators(List.of(5, 8, 1))
            .prepareUnsignedPreviewTransaction();
    var previewTransaction =
        new CompiledPreviewTransaction().previewTransactionHex(prepared.hexPayloadBytes());
    return new TransactionPreviewV2Request()
        .network(getCoreApiHelper().networkName())
        .previewTransaction(previewTransaction);
  }

  TransactionPreviewV2Request createInvalidRequest() {
    var prepared = TransactionV2Builder.forTests().prepareUnsignedPreviewTransaction();
    var invalidBytes = prepared.hexPayloadBytes() + "00";
    var previewTransaction = new CompiledPreviewTransaction().previewTransactionHex(invalidBytes);
    return new TransactionPreviewV2Request()
        .network(getCoreApiHelper().networkName())
        .previewTransaction(previewTransaction);
  }
}
