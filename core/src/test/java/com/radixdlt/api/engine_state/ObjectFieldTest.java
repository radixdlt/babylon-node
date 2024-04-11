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

package com.radixdlt.api.engine_state;

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.ContiguousSet;
import com.radixdlt.api.DeterministicEngineStateApiTestBase;
import com.radixdlt.api.core.generated.models.LedgerHeader;
import com.radixdlt.api.core.generated.models.LedgerProof;
import com.radixdlt.api.core.generated.models.StreamProofsFilterAny;
import com.radixdlt.api.core.generated.models.StreamProofsRequest;
import com.radixdlt.api.engine_state.generated.models.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochRound;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.rev2.Manifest;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.function.Function;
import java.util.stream.Collectors;
import org.junit.Test;

public final class ObjectFieldTest extends DeterministicEngineStateApiTestBase {

  @Test
  public void engine_state_api_returns_object_field() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      final var response =
          getObjectsApi()
              .objectFieldPost(
                  new ObjectFieldRequest()
                      .entityAddress(wellKnownAddresses.getConsensusManager())
                      // the module is Main by default
                      .fieldName("proposer_minute_timestamp"));

      assertThat(response.getContent().getProgrammaticJson())
          .isEqualTo(
              Map.of(
                  "kind", "Enum",
                  "type_name", "ConsensusManagerProposerMinuteTimestampFieldPayload",
                  "variant_id", "0",
                  "variant_name", "V1",
                  "fields",
                      List.of(
                          Map.of(
                              "kind", "I32",
                              "type_name", "ProposerMinuteTimestampSubstate",
                              "value", "0"))));
    }
  }

  @Test
  public void engine_state_api_returns_transient_object_field_default_value() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // Locate literally any fungible vault:
      final var vault =
          getEntitiesApi().entityIteratorPost(new EntityIteratorRequest()).getPage().stream()
              .filter(entity -> entity.getEntityType() == EntityType.INTERNALFUNGIBLEVAULT)
              .findFirst()
              .orElseThrow();

      // Fetch the expected transient `locked_balance` value from the vault's blueprint:
      final var vaultBlueprint = vault.getBlueprint();
      final var transientDefaultValue =
          getTypesApi()
              .blueprintInfoPost(
                  new BlueprintInfoRequest()
                      .packageAddress(vaultBlueprint.getPackageAddress())
                      .blueprintName(vaultBlueprint.getBlueprintName()))
              .getInfo()
              .getFields()
              .stream()
              .filter(field -> field.getName().equals("locked_balance"))
              .findAny()
              .orElseThrow()
              .getTransience()
              .getDefaultValue()
              .getProgrammaticJson();

      // And fetch the same from the vault instance:
      final var value =
          getObjectsApi()
              .objectFieldPost(
                  new ObjectFieldRequest()
                      .entityAddress(vault.getEntityAddress())
                      .fieldName("locked_balance"))
              .getContent()
              .getProgrammaticJson();
      assertThat(value).isEqualTo(transientDefaultValue);
    }
  }

  @Test
  public void engine_state_api_object_field_supports_history() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // The easiest way to observe history is to look at the Consensus Manager's state field:
      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();
      final var baseRequest =
          new ObjectFieldRequest()
              .entityAddress(wellKnownAddresses.getConsensusManager())
              .fieldName("state");

      // Progress to a known version and capture Epoch and Round:
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(23));
      final var epochRoundAtCurrentVersion =
          parseEpochRound(getObjectsApi().objectFieldPost(baseRequest).getContent());

      // Assert on a slightly-older historical Epoch + Round:
      final var epochRoundAtVersion19 =
          parseEpochRound(
              getObjectsApi()
                  .objectFieldPost(
                      baseRequest.atLedgerState(
                          new VersionLedgerStateSelector()
                              .stateVersion(19L)
                              .type(LedgerStateSelectorType.BYSTATEVERSION)))
                  .getContent());
      assertThat(epochRoundAtVersion19).isLessThan(epochRoundAtCurrentVersion);

      // Assert on even older historical state:
      final var epochRoundAtVersion10 =
          parseEpochRound(
              getObjectsApi()
                  .objectFieldPost(
                      baseRequest.atLedgerState(
                          new VersionLedgerStateSelector()
                              .stateVersion(10L)
                              .type(LedgerStateSelectorType.BYSTATEVERSION)))
                  .getContent());
      assertThat(epochRoundAtVersion10).isLessThan(epochRoundAtVersion19);
    }
  }

  @Test
  public void engine_state_api_returns_accurate_historical_state_summary() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // We will query for some arbitrary field (that we know exists since very early versions):
      final var baseRequest =
          new ObjectFieldRequest()
              .entityAddress(getCoreApiHelper().getWellKnownAddresses().getConsensusManager())
              .fieldName("state");

      // Ensure we have some non-trivial history:
      final var historicalVersions = ContiguousSet.closed(2L, 20L);
      getCoreApiHelper().submitAndWaitForSuccess(test, Manifest.valid(), List.of());
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(historicalVersions.last()));

      // Capture actually-existing ledger proofs:
      final var versionsToActualHeaders =
          getCoreApiHelper()
              .streamApi()
              .streamProofsPost(
                  new StreamProofsRequest()
                      .network(networkLogicalName)
                      .filter(
                          new StreamProofsFilterAny().fromStateVersion(historicalVersions.first())))
              .getPage()
              .stream()
              .map(LedgerProof::getLedgerHeader)
              .collect(
                  Collectors.toMap(
                      LedgerHeader::getStateVersion,
                      Function.identity(),
                      (left, right) -> left,
                      TreeMap::new));

      // (sanity check only) There are some proofs, and also some state versions in-between proofs:
      assertThat(versionsToActualHeaders).isNotEmpty();
      assertThat(versionsToActualHeaders.size()).isLessThan(historicalVersions.size());

      // We can assert something about responses for every historical state version:
      for (Long historicalVersion : historicalVersions) {
        final var historicalResponse =
            getObjectsApi()
                .objectFieldPost(
                    baseRequest.atLedgerState(
                        new VersionLedgerStateSelector()
                            .stateVersion(historicalVersion)
                            .type(LedgerStateSelectorType.BYSTATEVERSION)));

        // Assert that the returned state version is exactly the requested one:
        final var historicalLedgerState = historicalResponse.getAtLedgerState();
        assertThat(historicalLedgerState.getStateVersion()).isEqualTo(historicalVersion);

        // Assert on those items of the summary which must come from the proving header:
        final var atOrNextHeader =
            versionsToActualHeaders.ceilingEntry(historicalVersion).getValue();
        final var historicalHeaderSummary = historicalLedgerState.getHeaderSummary();
        assertThat(historicalHeaderSummary.getEpochRound().getEpoch())
            .isEqualTo(atOrNextHeader.getEpoch());
        assertThat(historicalHeaderSummary.getEpochRound().getRound())
            .isEqualTo(atOrNextHeader.getRound());
        assertThat(historicalHeaderSummary.getProposerTimestamp().getUnixTimestampMs())
            .isEqualTo(atOrNextHeader.getProposerTimestampMs());

        // Assert on the returned ledger hashes...
        final var historicalLedgerHashes = historicalHeaderSummary.getLedgerHashes();
        final var atOrNextHeaderHashes = atOrNextHeader.getHashes();
        if (atOrNextHeader.getStateVersion().equals(historicalVersion)) {
          // ... if we happened to query exactly at the "proof point", all these hashes must match:
          assertThat(historicalLedgerHashes.getStateTreeHash())
              .isEqualTo(atOrNextHeaderHashes.getStateTreeHash());
          assertThat(historicalLedgerHashes.getTransactionTreeHash())
              .isEqualTo(atOrNextHeaderHashes.getTransactionTreeHash());
          assertThat(historicalLedgerHashes.getReceiptTreeHash())
              .isEqualTo(atOrNextHeaderHashes.getReceiptTreeHash());
        } else {
          // ... if we happened to query at a version in-between proofs, none of these hashes can
          // match:
          assertThat(historicalLedgerHashes.getStateTreeHash())
              .isNotEqualTo(atOrNextHeaderHashes.getStateTreeHash());
          assertThat(historicalLedgerHashes.getTransactionTreeHash())
              .isNotEqualTo(atOrNextHeaderHashes.getTransactionTreeHash());
          assertThat(historicalLedgerHashes.getReceiptTreeHash())
              .isNotEqualTo(atOrNextHeaderHashes.getReceiptTreeHash());
        }
      }
    }
  }

  private static EpochRound parseEpochRound(SborData stateField) {
    final var wrapper = (Map<String, Object>) stateField.getProgrammaticJson();
    final var wrapperFields = (List<Map<String, Object>>) wrapper.get("fields");
    final var value = wrapperFields.get(0);
    final var valueFields = (List<Map<String, Object>>) value.get("fields");
    final var fieldMap =
        valueFields.stream()
            .collect(
                Collectors.toMap(
                    field -> (String) field.get("field_name"),
                    field -> String.valueOf(field.get("value"))));
    return EpochRound.of(
        Long.parseLong(fieldMap.get("epoch")), Round.of(Long.parseLong(fieldMap.get("round"))));
  }
}
