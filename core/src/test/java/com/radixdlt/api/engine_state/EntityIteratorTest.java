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

import com.google.common.collect.Multimaps;
import com.google.common.collect.Ordering;
import com.radixdlt.api.DeterministicEngineStateApiTestBase;
import com.radixdlt.api.engine_state.generated.models.*;
import com.radixdlt.environment.StateTreeGcConfig;
import com.radixdlt.harness.predicates.NodesPredicate;
import com.radixdlt.testutil.TestStateReader;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.*;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import javax.annotation.Nullable;
import junitparams.JUnitParamsRunner;
import junitparams.Parameters;
import org.awaitility.Awaitility;
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
  public void engine_state_api_entity_iterator_requires_same_params_across_pages()
      throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      final var sameFilter = new EntityTypeFilter().entityType(EntityType.INTERNALKEYVALUESTORE);
      final var sameLedgerStateSelector =
          new VersionLedgerStateSelector()
              .stateVersion(5L)
              .type(LedgerStateSelectorType.BYSTATEVERSION);

      final var firstPageResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(sameFilter)
                      .atLedgerState(sameLedgerStateSelector)
                      .maxPageSize(1)); // we assume there is more than 1 KV-store

      final var whenDifferingFilter =
          assertErrorResponse(
              () ->
                  getEntitiesApi()
                      .entityIteratorPost(
                          new EntityIteratorRequest()
                              .filter(new EntityTypeFilter().entityType(EntityType.GLOBALPACKAGE))
                              .atLedgerState(sameLedgerStateSelector)
                              .continuationToken(firstPageResponse.getContinuationToken())));
      assertThat(whenDifferingFilter.getMessage()).contains("filter");
      assertThat(whenDifferingFilter.getDetails()).isNull(); // it is not an application logic error

      final var whenDifferingLedgerStateSelector =
          assertErrorResponse(
              () ->
                  getEntitiesApi()
                      .entityIteratorPost(
                          new EntityIteratorRequest()
                              .filter(sameFilter)
                              .atLedgerState(
                                  new VersionLedgerStateSelector()
                                      .stateVersion(4L)
                                      .type(LedgerStateSelectorType.BYSTATEVERSION))
                              .continuationToken(firstPageResponse.getContinuationToken())));
      assertThat(whenDifferingLedgerStateSelector.getMessage()).contains("filter");
      assertThat(whenDifferingLedgerStateSelector.getDetails())
          .isNull(); // it is not an application logic error
    }
  }

  // Note: we want to test "historical state" working with and without filters.
  public EntityIteratorFilter[] noneOrBroadFilters() {
    return Stream.concat(Stream.of((EntityIteratorFilter) null), Arrays.stream(broadFilters()))
        .toArray(EntityIteratorFilter[]::new);
  }

  @Test
  @Parameters(method = "noneOrBroadFilters")
  public void engine_state_api_entity_iterator_supports_history(
      @Nullable EntityIteratorFilter filter) throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // Arrange: collect all entities (as of now) and index them by creation version:
      final var currentVersionResponse =
          getEntitiesApi().entityIteratorPost(new EntityIteratorRequest().filter(filter));
      // sanity check: make sure we don't have to deal with pages in this scenario:
      assertThat(currentVersionResponse.getContinuationToken()).isNull();

      final var entitiesByCreationVersion =
          Multimaps.index(
              currentVersionResponse.getPage(), ListedEntityItem::getCreatedAtStateVersion);
      // find all the non-latest entities (called "older" from this point on):
      final var latestEntitiesCreatedAtVersion =
          Ordering.natural().max(entitiesByCreationVersion.keySet());
      final var olderEntities =
          entitiesByCreationVersion.asMap().entrySet().stream()
              .filter(entry -> entry.getKey() < latestEntitiesCreatedAtVersion)
              .flatMap(entry -> entry.getValue().stream())
              .toList();
      // sanity check: make sure that there are >0 older entities
      assertThat(olderEntities).isNotEmpty();

      // Act: list all entities at a state version just before the creation of the latest ones:
      final var olderVersionResponse =
          getEntitiesApi()
              .entityIteratorPost(
                  new EntityIteratorRequest()
                      .filter(filter)
                      .atLedgerState(
                          new VersionLedgerStateSelector()
                              .stateVersion(latestEntitiesCreatedAtVersion - 1)
                              .type(LedgerStateSelectorType.BYSTATEVERSION)));

      // Assert: we clearly know the older entities:
      assertThat(olderVersionResponse.getPage()).hasSameElementsAs(olderEntities);
      assertThat(olderVersionResponse.getAtLedgerState().getStateVersion())
          .isEqualTo(latestEntitiesCreatedAtVersion - 1);
    }
  }

  // Note: the claim in the test's name below is technically false - in fact, the historical Entity
  // iteration is powered by the Node's internal index of "Entities created at state versions", and
  // does not touch the JMT at all.
  // However, for consistency with other "historical" endpoints of the API, we explicitly validate
  // this aspect, so it's also worth testing.
  @Test
  public void engine_state_api_entity_iterator_at_state_version_requires_history_feature()
      throws Exception {
    final var tooShortHistory =
        new StateTreeGcConfig(UInt32.fromNonNegativeInt(1), UInt64.fromNonNegativeLong(10));
    try (var test = buildRunningServerTest(tooShortHistory)) {
      test.suppressUnusedWarning();

      // Reach a known state version:
      test.runUntilState(NodesPredicate.anyAtOrOverStateVersion(37));

      // Wait for the async GC to catch up its target:
      Awaitility.await()
          .until(
              test.getInstance(0, TestStateReader.class)::getLeastStaleStateTreeVersion,
              Predicate.isEqual(27L));

      // Try to list Entities at "too old" state version:
      final var errorResponse =
          assertErrorResponse(
              () ->
                  getEntitiesApi()
                      .entityIteratorPost(
                          new EntityIteratorRequest()
                              .atLedgerState(
                                  new VersionLedgerStateSelector()
                                      .stateVersion(26L)
                                      .type(LedgerStateSelectorType.BYSTATEVERSION))));

      // Assert that the error informs about it:
      assertThat(errorResponse.getMessage())
          .containsIgnoringCase("state version past the earliest available");
      assertThat(errorResponse.getDetails())
          .isEqualTo(
              new StateVersionInTooDistantPastDetails()
                  .earliestAvailableStateVersion(27L)
                  .errorType(ErrorType.STATEVERSIONINTOODISTANTPAST));
    }
  }
}
