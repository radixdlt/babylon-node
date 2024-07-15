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

import com.google.common.collect.ImmutableList;
import com.radixdlt.api.DeterministicEngineStateApiTestBase;
import com.radixdlt.api.engine_state.generated.client.ApiException;
import com.radixdlt.api.engine_state.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.identifiers.Address;
import com.radixdlt.rev2.Decimal;
import org.junit.Test;

public final class ObjectRoyaltyTest extends DeterministicEngineStateApiTestBase {

  @Test
  public void engine_state_api_returns_object_royalty() throws Exception {
    final var config =
        defaultConfig()
            .withGenesis(
                GenesisBuilder.createTestGenesisWithNumValidators(
                    1,
                    Decimal.ONE,
                    GenesisConsensusManagerConfig.Builder.testDefaults()
                        .epochExactRoundCount(1000000),
                    ImmutableList.of("royalties")));
    try (var test = buildRunningServerTest(config)) {
      test.suppressUnusedWarning();

      // Find all the component instances created by the `royalties` Scenario:
      final var royaltyObjectAddresses =
          getExtraApi()
              .extraEntitySearchPost(
                  new ExtraEntitySearchRequest()
                      .filter(new EntityTypeFilter().entityType(EntityType.GLOBALGENERICCOMPONENT)))
              .getPage()
              .stream()
              .filter(entity -> entity.getSystemType() == SystemType.OBJECT)
              .filter(object -> object.getBlueprint().getBlueprintName().equals("RoyaltiesBp"))
              .map(object -> object.getEntityAddress())
              .toList();
      assertThat(royaltyObjectAddresses).hasSize(3); // (sanity check) there should be 3

      // Collect their method royalties from the endpoint under test:
      final var methodRoyalties =
          royaltyObjectAddresses.stream()
              .flatMap(
                  address -> {
                    try {
                      return getAttachedModulesApi()
                          .objectAttachedModulesRoyaltyPost(
                              new ObjectRoyaltyRequest().entityAddress(address))
                          .getMethodRoyalties()
                          .stream();
                    } catch (ApiException e) {
                      throw new RuntimeException(e);
                    }
                  })
              .toList();

      // Assert all 9 combinations (i.e. `{Free, XRD, USD}^{Package, Component}`) of known amounts:
      assertThat(methodRoyalties)
          .containsExactlyInAnyOrder(
              new ObjectMethodRoyalty()
                  .name("method_with_no_package_royalty")
                  .packageRoyaltyAmount(null)
                  .componentRoyaltyAmount(null),
              new ObjectMethodRoyalty()
                  .name("method_with_no_package_royalty")
                  .packageRoyaltyAmount(null)
                  .componentRoyaltyAmount(xrd(17)),
              new ObjectMethodRoyalty()
                  .name("method_with_no_package_royalty")
                  .packageRoyaltyAmount(null)
                  .componentRoyaltyAmount(usd(2)),
              new ObjectMethodRoyalty()
                  .name("method_with_xrd_package_royalty")
                  .packageRoyaltyAmount(xrd(31))
                  .componentRoyaltyAmount(null),
              new ObjectMethodRoyalty()
                  .name("method_with_xrd_package_royalty")
                  .packageRoyaltyAmount(xrd(31))
                  .componentRoyaltyAmount(xrd(18)),
              new ObjectMethodRoyalty()
                  .name("method_with_xrd_package_royalty")
                  .packageRoyaltyAmount(xrd(31))
                  .componentRoyaltyAmount(usd(3)),
              new ObjectMethodRoyalty()
                  .name("method_with_usd_package_royalty")
                  .packageRoyaltyAmount(usd(1))
                  .componentRoyaltyAmount(null),
              new ObjectMethodRoyalty()
                  .name("method_with_usd_package_royalty")
                  .packageRoyaltyAmount(usd(1))
                  .componentRoyaltyAmount(xrd(19)),
              new ObjectMethodRoyalty()
                  .name("method_with_usd_package_royalty")
                  .packageRoyaltyAmount(usd(1))
                  .componentRoyaltyAmount(usd(4)));
    }
  }

  @Test
  public void engine_state_api_returns_object_royalty_of_uninstantiated_entity() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      var uninstantiatedIdentityAddress =
          Address.virtualIdentityAddress(ECKeyPair.generateNew().getPublicKey());
      var methodRoyalties =
          getAttachedModulesApi()
              .objectAttachedModulesRoyaltyPost(
                  new ObjectRoyaltyRequest()
                      .entityAddress(uninstantiatedIdentityAddress.encode(networkDefinition)))
              .getMethodRoyalties();

      assertThat(methodRoyalties)
          .containsExactly(
              new ObjectMethodRoyalty()
                  .name("securify")
                  .packageRoyaltyAmount(null)
                  .componentRoyaltyAmount(null));
    }
  }

  private static RoyaltyAmount xrd(int xrd) {
    return new RoyaltyAmount().amount(String.valueOf(xrd)).unit(RoyaltyAmount.UnitEnum.XRD);
  }

  private static RoyaltyAmount usd(int usd) {
    return new RoyaltyAmount().amount(String.valueOf(usd)).unit(RoyaltyAmount.UnitEnum.USD);
  }
}
