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
import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.ImmutableList;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.environment.LedgerProofsGcConfig;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import org.junit.Test;

public class ProofStreamTest extends DeterministicCoreApiTestBase {
  @Test
  public void test_proof_stream() throws Exception {
    final var config =
        defaultConfig()
            .withProtocolConfig(
                new ProtocolConfig(
                    ImmutableList.of(
                        new ProtocolUpdateTrigger(
                            ProtocolUpdateTrigger.ANEMONE,
                            ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(3L)))))
            // This effectively disables the proofs' GC, so that we can have exact asserts:
            .withLedgerProofsGcConfig(new LedgerProofsGcConfig(UInt32.MAX_VALUE, UInt64.MAX_VALUE))
            .withGenesis(
                GenesisBuilder.createTestGenesisWithNumValidators(
                    1,
                    Decimal.ONE,
                    GenesisConsensusManagerConfig.Builder.testDefaults()
                        .epochExactRoundCount(100)));
    try (var test = buildRunningServerTest(config)) {
      test.runUntilState(allAtOrOverEpoch(5L));

      // ==================
      // Test the ANY filter
      // ===================
      var page1Size = 8;
      var responseAnyFilterPage1 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(page1Size)
                      .filter(new StreamProofsFilterAny()));

      var page = responseAnyFilterPage1.getPage();
      assertThat(page.size()).isEqualTo(page1Size);
      assertThat(page.get(0).getOrigin()).isInstanceOf(GenesisLedgerProofOrigin.class);
      assertThat(page.get(0).getLedgerHeader().getStateVersion()).isEqualTo(1);
      assertThat(page.get(page1Size - 1).getOrigin())
          .isInstanceOf(ConsensusLedgerProofOrigin.class);
      assertThat(page.get(page1Size - 1).getLedgerHeader().getStateVersion()).isEqualTo(page1Size);

      var page2Size = 5;
      var responseAnyFilterPage2 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(page2Size)
                      .filter(new StreamProofsFilterAny())
                      .continuationToken(responseAnyFilterPage1.getContinuationToken()));

      page = responseAnyFilterPage2.getPage();
      assertThat(page.size()).isEqualTo(page2Size);
      assertThat(page.get(0).getLedgerHeader().getStateVersion()).isEqualTo(page1Size + 1);

      var responseAnyFilterOffset7 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(new StreamProofsFilterAny().fromStateVersion(7L)))
              .continuationToken(responseAnyFilterPage1.getContinuationToken());

      page = responseAnyFilterOffset7.getPage();
      assertThat(page.size()).isEqualTo(1);
      assertThat(page.get(0).getLedgerHeader().getStateVersion()).isEqualTo(7);

      // ==================
      // Test NO filter
      // ===================
      var responseNoFilterPage1 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest().network(networkLogicalName).maxPageSize(page1Size));

      assertThat(responseNoFilterPage1)
          .usingRecursiveComparison()
          .isEqualTo(responseAnyFilterPage1);

      // ===================
      // Test the NewEpochs filter
      // ===================
      var responseNewEpochFilterPage1 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(2)
                      .filter(new StreamProofsFilterNewEpochs()));

      page = responseNewEpochFilterPage1.getPage();
      assertThat(page.size()).isEqualTo(2);
      assertThat(page.get(0).getLedgerHeader().getEpoch()).isEqualTo(1);
      assertThat(page.get(0).getLedgerHeader().getNextEpoch()).isNotNull();
      assertThat(page.get(0).getLedgerHeader().getNextEpoch().getEpoch()).isEqualTo(2);
      assertThat(page.get(1).getLedgerHeader().getNextEpoch()).isNotNull();
      assertThat(page.get(1).getLedgerHeader().getNextEpoch().getEpoch()).isEqualTo(3);

      var responseNewEpochFilterPage2 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(new StreamProofsFilterNewEpochs())
                      .continuationToken(responseNewEpochFilterPage1.getContinuationToken()));

      page = responseNewEpochFilterPage2.getPage();
      assertThat(page.size()).isEqualTo(1);
      assertThat(page.get(0).getLedgerHeader().getNextEpoch()).isNotNull();
      assertThat(page.get(0).getLedgerHeader().getNextEpoch().getEpoch()).isEqualTo(4);

      var responseNewEpochFilterPageOffset =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(new StreamProofsFilterNewEpochs().fromEpoch(3L)));

      page = responseNewEpochFilterPageOffset.getPage();
      assertThat(page.size()).isEqualTo(1);
      assertThat(page.get(0).getLedgerHeader().getNextEpoch().getEpoch()).isEqualTo(3);

      // ===================
      // Test the ProtocolUpdateInitializations filter
      // ===================
      var responseUpdateInitializationsFilterPage1 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(new StreamProofsFilterProtocolUpdateInitializations()));

      page = responseUpdateInitializationsFilterPage1.getPage();
      assertThat(page.size()).isEqualTo(1);
      assertThat(page.get(0).getLedgerHeader().getNextProtocolVersion())
          .isEqualTo(ProtocolUpdateTrigger.ANEMONE);
      assertThat(page.get(0).getLedgerHeader().getNextEpoch()).isNotNull();
      assertThat(page.get(0).getLedgerHeader().getNextEpoch().getEpoch()).isEqualTo(3);
      assertThat(page.get(0).getLedgerHeader().getEpoch()).isEqualTo(2);
      assertThat(responseUpdateInitializationsFilterPage1.getContinuationToken()).isNull();
      var anenomeTriggerStateVersion = page.get(0).getLedgerHeader().getStateVersion();

      // ===================
      // Test the ProtocolUpdateExecution filter
      // ===================
      var responseUpdateExecutionFilterPage1 =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(
                          new StreamProofsFilterProtocolUpdateExecution()
                              .protocolVersion("anemone")));

      page = responseUpdateExecutionFilterPage1.getPage();
      assertThat(page.size()).isEqualTo(1);
      // Before consensus starts in this epoch, the consensus manager is set to epoch N + 1, round 0
      // The proofs should agree
      assertThat(page.get(0).getLedgerHeader().getEpoch()).isEqualTo(3);
      assertThat(page.get(0).getLedgerHeader().getRound()).isEqualTo(0);
      // Anenome consists of 1 transaction batch of length 4
      assertThat(page.get(0).getLedgerHeader().getStateVersion())
          .isEqualTo(anenomeTriggerStateVersion + 4);
      assertThat(page.get(0).getOrigin()).isInstanceOf(ProtocolUpdateLedgerProofOrigin.class);
      // There's only 1 transaction in anemone
      // ...and we're only applying anenome, so there are no further updates
      assertThat(responseUpdateExecutionFilterPage1.getContinuationToken()).isNull();

      var responseUpdateExecutionFilterPage1Filtered =
          getStreamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .maxPageSize(1)
                      .filter(
                          new StreamProofsFilterProtocolUpdateExecution()
                              .protocolVersion(ProtocolUpdateTrigger.ANEMONE)));

      assertThat(responseUpdateExecutionFilterPage1)
          .usingRecursiveComparison()
          .isEqualTo(responseUpdateExecutionFilterPage1Filtered);
    }
  }
}
