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

import com.google.common.collect.Lists;
import com.radixdlt.api.DeterministicEngineStateApiTestBase;
import com.radixdlt.api.engine_state.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.identifiers.Address;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import org.junit.Test;

public final class EntityInfoTest extends DeterministicEngineStateApiTestBase {

  @Test
  public void engine_state_api_returns_root_object_info() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // fetch ConsensusManager's info, since it has nice fields and collections to assert on
      final var entityInfo =
          getEntitiesApi()
              .entityInfoPost(
                  new EntityInfoRequest().entityAddress(wellKnownAddresses.getConsensusManager()))
              .getInfo();

      assertThat(entityInfo.getSystemType()).isEqualTo(SystemType.OBJECT);
      assertThat(entityInfo.getAncestry()).isNull();

      final var objectInfo = (ObjectEntityInfo) entityInfo;
      assertThat(objectInfo.getEntityType()).isEqualTo(EntityType.GLOBALCONSENSUSMANAGER);
      assertThat(objectInfo.getIsGlobal()).isEqualTo(true);
      assertThat(objectInfo.getIsInstantiated()).isEqualTo(true);

      final var fieldNames =
          Lists.transform(objectInfo.getMainModuleState().getFields(), ObjectFieldInfo::getName);
      assertThat(fieldNames)
          .isEqualTo(
              List.of(
                  "configuration",
                  "state",
                  "validator_rewards",
                  "current_validator_set",
                  "current_proposal_statistic",
                  "proposer_minute_timestamp",
                  "proposer_milli_timestamp"));

      final var collectionsNames =
          Lists.transform(
              objectInfo.getMainModuleState().getCollections(), ObjectCollectionInfo::getName);
      assertThat(collectionsNames).isEqualTo(List.of("registered_validator_by_stake"));

      final var attachedModuleIds =
          Lists.transform(
              objectInfo.getAttachedModules(),
              ObjectEntityInfoAllOfAttachedModules::getAttachedModuleId);
      assertThat(attachedModuleIds)
          .isEqualTo(List.of(AttachedModuleId.ROLEASSIGNMENT, AttachedModuleId.METADATA));

      assertThat(objectInfo.getBlueprintReference())
          .isEqualTo(
              new BlueprintReference()
                  .packageAddress(wellKnownAddresses.getConsensusManagerPackage())
                  .blueprintVersion("1.0.0")
                  .blueprintName("ConsensusManager"));
      assertThat(objectInfo.getInstanceInfo().getOuterObjectAddress()).isNull();
    }
  }

  @Test
  public void engine_state_api_object_info_returns_fields_respecting_conditions() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // fetch info of 2 well-known objects instantiating the same blueprint with different features
      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();
      final var toFieldMap = Collectors.toMap(ObjectFieldInfo::getIndex, ObjectFieldInfo::getName);

      final var packageOwnerRes =
          (ObjectEntityInfo)
              getEntitiesApi()
                  .entityInfoPost(
                      new EntityInfoRequest()
                          .entityAddress(wellKnownAddresses.getPackageOwnerBadge()))
                  .getInfo();
      assertThat(packageOwnerRes.getMainModuleState().getFields().stream().collect(toFieldMap))
          .isEqualTo(Map.of(0, "id_type", 1, "mutable_fields"));

      final var validatorOwnerRes =
          (ObjectEntityInfo) // this one additionally tracks total supply
              getEntitiesApi()
                  .entityInfoPost(
                      new EntityInfoRequest()
                          .entityAddress(wellKnownAddresses.getValidatorOwnerBadge()))
                  .getInfo();
      assertThat(validatorOwnerRes.getMainModuleState().getFields().stream().collect(toFieldMap))
          .isEqualTo(Map.of(0, "id_type", 1, "mutable_fields", 2, "total_supply"));
    }
  }

  @Test
  public void engine_state_api_returns_kv_store_info() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // list entities and pick a random KV store
      final var kvStoreAddress =
          getEntitiesApi().entityIteratorPost(new EntityIteratorRequest()).getPage().stream()
              .filter(entity -> entity.getEntityType() == EntityType.INTERNALKEYVALUESTORE)
              .findFirst()
              .orElseThrow()
              .getEntityAddress();

      final var entityInfo =
          getEntitiesApi()
              .entityInfoPost(new EntityInfoRequest().entityAddress(kvStoreAddress))
              .getInfo();

      assertThat(entityInfo.getSystemType()).isEqualTo(SystemType.KEYVALUESTORE);
      assertThat(entityInfo.getAncestry()).isNotNull();

      assertThat(entityInfo).isInstanceOf(KeyValueStoreEntityInfo.class);
      // `KeyValueStoreEntityInfo` only has key/value type references, which are hard to assert on
    }
  }

  @Test
  public void engine_state_api_returns_uninstantiated_object_info() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var accountKeyPair = ECKeyPair.generateNew();
      final var accountAddress = Address.virtualAccountAddress(accountKeyPair.getPublicKey());

      final var entityInfo =
          getEntitiesApi()
              .entityInfoPost(
                  new EntityInfoRequest().entityAddress(accountAddress.encode(networkDefinition)))
              .getInfo();

      assertThat(entityInfo.getSystemType()).isEqualTo(SystemType.OBJECT);
      assertThat(entityInfo.getAncestry()).isNull();

      final var objectInfo = (ObjectEntityInfo) entityInfo;
      assertThat(objectInfo.getEntityType()).isEqualTo(EntityType.GLOBALVIRTUALSECP256K1ACCOUNT);
      assertThat(objectInfo.getIsGlobal()).isEqualTo(true);
      assertThat(objectInfo.getIsInstantiated()).isEqualTo(false);

      final var fieldNames =
          Lists.transform(objectInfo.getMainModuleState().getFields(), ObjectFieldInfo::getName);
      assertThat(fieldNames).isEqualTo(List.of("deposit_rule"));

      final var collectionsNames =
          Lists.transform(
              objectInfo.getMainModuleState().getCollections(), ObjectCollectionInfo::getName);
      assertThat(collectionsNames)
          .isEqualTo(List.of("resource_vault", "resource_preference", "authorized_depositor"));

      assertThat(objectInfo.getAttachedModules()).isEmpty();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();
      assertThat(objectInfo.getBlueprintReference())
          .isEqualTo(
              new BlueprintReference()
                  .packageAddress(wellKnownAddresses.getAccountPackage())
                  .blueprintVersion("1.0.0")
                  .blueprintName("Account"));
      assertThat(objectInfo.getInstanceInfo().getOuterObjectAddress()).isNull();
    }
  }

  @Test
  public void engine_state_api_entity_info_supports_history() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // The Entity info is effectively immutable, but we can still experience "historical state"
      // with this endpoint by querying an Entity before and after it was created.
      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();

      // The ConsensusManager was created at version 2, so let us ask about version 3...
      final var responseAtVersion3 =
          getEntitiesApi()
              .entityInfoPost(
                  new EntityInfoRequest()
                      .entityAddress(wellKnownAddresses.getConsensusManager())
                      .atLedgerState(
                          new VersionLedgerStateCoordinate()
                              .stateVersion(3L)
                              .type(LedgerStateCoordinateType.BYSTATEVERSION)));
      assertThat(responseAtVersion3.getInfo()).isNotNull();
      assertThat(responseAtVersion3.getAtLedgerState().getStateVersion()).isEqualTo(3L);

      // ... and at version 1, where it does not exist yet:
      final var errorResponseAtVersion1 =
          assertErrorResponse(
              () ->
                  getEntitiesApi()
                      .entityInfoPost(
                          new EntityInfoRequest()
                              .entityAddress(wellKnownAddresses.getConsensusManager())
                              .atLedgerState(
                                  new VersionLedgerStateCoordinate()
                                      .stateVersion(1L)
                                      .type(LedgerStateCoordinateType.BYSTATEVERSION))));
      assertThat((RequestedItemNotFoundDetails) errorResponseAtVersion1.getDetails())
          .isEqualTo(
              new RequestedItemNotFoundDetails()
                  .itemType(RequestedItemType.ENTITY)
                  .errorType(ErrorType.REQUESTEDITEMNOTFOUND));
    }
  }
}
