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
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import javax.annotation.Nullable;
import junitparams.JUnitParamsRunner;
import junitparams.Parameters;
import org.junit.Test;
import org.junit.runner.RunWith;

@RunWith(JUnitParamsRunner.class)
public final class EntityIteratorTest extends DeterministicEngineStateApiTestBase {

  private static final int SMALL_PAGE_SIZE = 7;

  @Test
  public void engine_state_api_entity_iterator_pages_through_all_entities() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // first list all entities with a single request
      final var allResponse = getEntitiesApi().entityIteratorPost(new EntityIteratorRequest());

      // high default limit of the endpoint should allow to get it all in one page in tests:
      assertThat(allResponse.getContinuationToken()).isNull();
      final var allEntities = allResponse.getPage();

      for (var e : allEntities) {
        var info =
            getEntitiesApi()
                .entityInfoPost(new EntityInfoRequest().entityAddress(e.getEntityAddress()))
                .getInfo();
        if (info instanceof ObjectEntityInfo obj) {
          obj.getAttachedModules().stream()
              .filter(f -> f.getAttachedModuleId() == AttachedModuleId.ROLEASSIGNMENT)
              .forEach(
                  m -> {
                    System.out.println(e.getEntityAddress() + ": " + m);
                  });
        }
      }

      // assert (heuristically) that "all" different kinds of entities are listed:
      final var allSystemTypes =
          allEntities.stream().map(ListedEntityItem::getSystemType).collect(Collectors.toSet());
      assertThat(allSystemTypes).containsAll(EnumSet.allOf(SystemType.class));
      final var allGlobalOrNonGlobal =
          allEntities.stream().map(ListedEntityItem::getIsGlobal).collect(Collectors.toSet());
      assertThat(allGlobalOrNonGlobal).containsAll(Set.of(true, false));

      // make sure that we should expect some continuation tokens when paging
      assertThat(allEntities.size()).isGreaterThan(SMALL_PAGE_SIZE);

      // now fetch all entities in small pages
      final List<ListedEntityItem> pagedEntities = new ArrayList<>();
      @Nullable String continuationToken = null;

      while (true) {
        final var smallResponse =
            getEntitiesApi()
                .entityIteratorPost(
                    new EntityIteratorRequest()
                        .continuationToken(continuationToken)
                        .maxPageSize(SMALL_PAGE_SIZE));
        final var smallEntities = smallResponse.getPage();
        pagedEntities.addAll(smallEntities);
        continuationToken = smallResponse.getContinuationToken();
        if (continuationToken == null) {
          assertThat(smallEntities.size()).isLessThanOrEqualTo(SMALL_PAGE_SIZE);
          break;
        } else {
          assertThat(smallEntities.size()).isEqualTo(SMALL_PAGE_SIZE);
        }
      }

      // the entities collected via paging should be the same as when returned in one response
      assertThat(pagedEntities).isEqualTo(allEntities);
    }
  }

  @Test
  @Parameters(method = "broadFilters")
  public void engine_state_api_entity_iterator_pages_through_filtered_entities(
      EntityIteratorFilter filter) throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var allResponse =
          getEntitiesApi().entityIteratorPost(new EntityIteratorRequest().filter(filter));

      assertThat(allResponse.getContinuationToken()).isNull();
      final var allEntities = allResponse.getPage();

      final List<ListedEntityItem> pagedEntities = new ArrayList<>();
      @Nullable String continuationToken = null;

      while (true) {
        final var smallResponse =
            getEntitiesApi()
                .entityIteratorPost(
                    new EntityIteratorRequest()
                        .filter(filter)
                        .continuationToken(continuationToken)
                        .maxPageSize(SMALL_PAGE_SIZE));
        final var smallEntities = smallResponse.getPage();
        pagedEntities.addAll(smallEntities);
        continuationToken = smallResponse.getContinuationToken();
        if (continuationToken == null) {
          assertThat(smallEntities.size()).isLessThanOrEqualTo(SMALL_PAGE_SIZE);
          break;
        } else {
          assertThat(smallEntities.size()).isEqualTo(SMALL_PAGE_SIZE);
        }
      }

      assertThat(pagedEntities).isEqualTo(allEntities);
    }
  }

  // Note: these parameters are meant to test that even with a filter applied, the paging infra can
  // correctly go through multiple pages of elements (without duplicates or skips). We only use some
  // broad filters (i.e. not excluding too many entities, so that pages of `SMALL_PAGE_SIZE` still
  // happen).
  // The actual filters' tests are separate below.
  public EntityIteratorFilter[] broadFilters() {
    return new EntityIteratorFilter[] {
      new SystemTypeFilter().systemType(SystemType.OBJECT),
      new EntityTypeFilter().entityType(EntityType.GLOBALPACKAGE),
    };
  }

  @Test
  public void engine_state_api_entity_iterator_sorts_by_creation_asc() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var allResponse = getEntitiesApi().entityIteratorPost(new EntityIteratorRequest());

      final var creationVersions =
          allResponse.getPage().stream()
              .mapToLong(ListedEntityItem::getCreatedAtStateVersion)
              .toArray();

      assertThat(creationVersions).isSorted();
      assertThat(creationVersions[0]).isLessThan(creationVersions[creationVersions.length - 1]);
    }
  }

  @Test
  public void engine_state_api_entity_iterator_filters_by_system_type() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var allResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(new SystemTypeFilter().systemType(SystemType.KEYVALUESTORE)));

      final var systemTypes =
          allResponse.getPage().stream()
              .map(ListedEntityItem::getSystemType)
              .collect(Collectors.toSet());

      assertThat(systemTypes).containsExactly(SystemType.KEYVALUESTORE);
    }
  }

  @Test
  public void engine_state_api_entity_iterator_filters_by_entity_type() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var allResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(
                          new EntityTypeFilter().entityType(EntityType.GLOBALNONFUNGIBLERESOURCE)));

      final var entityTypes =
          allResponse.getPage().stream()
              .map(ListedEntityItem::getEntityType)
              .collect(Collectors.toSet());

      assertThat(entityTypes).containsExactly(EntityType.GLOBALNONFUNGIBLERESOURCE);
    }
  }

  @Test
  public void engine_state_api_entity_iterator_filters_by_blueprint() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var wellKnownAddresses = getCoreApiHelper().getWellKnownAddresses();
      final var requestedBlueprint =
          new UnversionedBlueprintReference()
              .packageAddress(wellKnownAddresses.getResourcePackage())
              .blueprintName("FungibleResourceManager");

      final var allResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(new BlueprintFilter().blueprint(requestedBlueprint)));

      final var blueprints =
          allResponse.getPage().stream()
              .map(ListedEntityItem::getBlueprint)
              .collect(Collectors.toSet());

      assertThat(blueprints).containsExactly(requestedBlueprint);
    }
  }

  @Test
  public void engine_state_api_entity_iterator_requires_same_filter_across_pages()
      throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var firstPageResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(new EntityTypeFilter().entityType(EntityType.INTERNALKEYVALUESTORE))
                      .maxPageSize(1)); // we assume there is more than 1 KV-store

      final var errorResponse =
          assertErrorResponse(
              () ->
                  getEntitiesApi()
                      .entityIteratorPost(
                          new EntityIteratorRequest()
                              .filter(new EntityTypeFilter().entityType(EntityType.GLOBALPACKAGE))
                              .continuationToken(firstPageResponse.getContinuationToken())));

      assertThat(errorResponse.getMessage()).contains("filter");
      assertThat(errorResponse.getDetails()).isNull(); // it is not an application logic error
    }
  }
}
