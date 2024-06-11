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

import com.radixdlt.api.DeterministicEngineStateApiTestBase;
import com.radixdlt.api.engine_state.generated.models.*;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public final class ObjectCollectionEntryTest extends DeterministicEngineStateApiTestBase {

  @Test
  public void engine_state_api_returns_object_kv_collection_entry() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // fetch the Blueprint of version 1.0.0 from some arbitrary package:
      final var value =
          (Map<?, ?>)
              getObjectsApi()
                  .objectCollectionEntryPost(
                      new ObjectCollectionEntryRequest()
                          .entityAddress(wellKnownAddresses.getIdentityPackage())
                          .collectionName("blueprint_version_definition")
                          .key(
                              new KeyValueStoreEntryKey()
                                  .key(
                                      new SborData()
                                          .programmaticJson(createBlueprintKeyJson("Identity")))
                                  .kind(ObjectCollectionKind.KEYVALUESTORE)))
                  .getContent()
                  .getProgrammaticJson();

      // assert that it is an SBOR struct of the expected type:
      assertThat(value.get("type_name")).isEqualTo("PackageBlueprintVersionDefinitionEntryPayload");
    }
  }

  @Test
  public void engine_state_api_returns_object_sorted_collection_entry() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // fetch a known entry from "validator by stake":
      final var value =
          (Map<?, ?>)
              getObjectsApi()
                  .objectCollectionEntryPost(
                      new ObjectCollectionEntryRequest()
                          .entityAddress(wellKnownAddresses.getConsensusManager())
                          .collectionName("registered_validator_by_stake")
                          .key(
                              new SortedIndexEntryKey()
                                  .sortPrefixHex("ffff")
                                  .key(
                                      new SborData()
                                          .programmaticJson(
                                              Map.of(
                                                  "type_name", "ComponentAddress",
                                                  "kind", "Reference",
                                                  "value",
                                                      "validator_test1s0a9c9kwjr3dmw79djvhalyz32sywumacv873yr7ms02v725kaygut")))
                                  .kind(ObjectCollectionKind.SORTEDINDEX)))
                  .getContent()
                  .getProgrammaticJson();

      // assert that it is an SBOR struct of the expected type:
      assertThat(value.get("type_name"))
          .isEqualTo("ConsensusManagerRegisteredValidatorByStakeEntryPayload");
    }
  }

  @Test
  public void engine_state_api_returns_not_found_for_unknown_key() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // try to fetch a blueprint of some made-up name:
      final var errorResponse =
          assertErrorResponse(
              () ->
                  getObjectsApi()
                      .objectCollectionEntryPost(
                          new ObjectCollectionEntryRequest()
                              .entityAddress(wellKnownAddresses.getIdentityPackage())
                              .collectionName("blueprint_version_definition")
                              .key(
                                  new KeyValueStoreEntryKey()
                                      .key(
                                          new SborData()
                                              .programmaticJson(createBlueprintKeyJson("MadeUp")))
                                      .kind(ObjectCollectionKind.KEYVALUESTORE))));

      // assert that it is "not found" type of error:
      assertThat(errorResponse.getDetails())
          .isEqualTo(
              new RequestedItemNotFoundDetails()
                  .itemType(RequestedItemType.ENTRYKEY)
                  .errorType(ErrorType.REQUESTEDITEMNOTFOUND));
      assertThat(errorResponse.getMessage()).contains("not found");
    }
  }

  @Test
  public void engine_state_api_accepts_and_outputs_sbors_as_raw_hex() throws Exception {
    // Note: this is NOT the only endpoint in the Engine State API which is supposed to support
    // SBORs in different formats, but it is a convenient place to test it (both input and output).

    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      final var sborData =
          getObjectsApi()
              .objectCollectionEntryPost(
                  new ObjectCollectionEntryRequest()
                      .entityAddress(wellKnownAddresses.getIdentityPackage())
                      .collectionName("blueprint_version_definition")
                      .key(
                          new KeyValueStoreEntryKey()
                              .key(
                                  new SborData()
                                      .rawHex(
                                          "5c21020c084964656e746974792103090100000009000000000900000000"))
                              .kind(ObjectCollectionKind.KEYVALUESTORE))
                      .sborFormatOptions(
                          new SborFormatOptions().rawHex(true).programmaticJson(false)))
              .getContent();

      assertThat(sborData.getRawHex()).isNotNull();
      assertThat(sborData.getProgrammaticJson()).isNull();
    }
  }

  private static Map<?, ?> createBlueprintKeyJson(String name) {
    return Map.of(
        "type_name",
        "BlueprintVersionKey",
        "kind",
        "Tuple",
        "fields",
        List.of(
            Map.of(
                "kind", "String",
                "field_name", "blueprint",
                "value", name),
            Map.of(
                "type_name",
                "BlueprintVersion",
                "kind",
                "Tuple",
                "field_name",
                "version",
                "fields",
                List.of(
                    Map.of(
                        "kind", "U32",
                        "field_name", "major",
                        "value", "1"),
                    Map.of(
                        "kind", "U32",
                        "field_name", "minor",
                        "value", "0"),
                    Map.of(
                        "kind", "U32",
                        "field_name", "patch",
                        "value", "0")))));
  }
}
