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

import static org.assertj.core.api.Assertions.assertThat;

import com.google.common.collect.Iterables;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.rev2.Manifest;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;
import org.junit.Test;

public final class BrowseKeyValueStoreIteratorTest extends DeterministicCoreApiTestBase {

  private static final int FAUCET_TRANSACTION_COUNT = 10;

  private static final int SMALL_PAGE_SIZE = 3;

  @Test
  public void browse_api_kv_store_iterator_pages_through_all_entries() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // execute a couple of dummy transactions (just to create multiple entries in the Faucet's
      // KV-Store):
      for (int i = 0; i < FAUCET_TRANSACTION_COUNT; ++i) {
        submitAndWaitForSuccess(test, Manifest.newRandomAccount(), List.of());
      }

      // fetch all keys from that KV-Store
      final var request =
          new BrowseKeyValueStoreIteratorRequest()
              .entityAddress(getTransactionsKeyValueStoreAddress())
              .network(networkLogicalName);
      final var allKeys = getBrowseApi().browseKvStoreIteratorPost(request).getPage();
      assertThat(allKeys.size()).isEqualTo(FAUCET_TRANSACTION_COUNT);

      // now fetch all the same keys, but in small pages
      final List<KeyValueStoreMapKey> pagedKeys = new ArrayList<>();
      @Nullable String continuationToken = null;

      while (true) {
        final var smallResponse =
            getBrowseApi()
                .browseKvStoreIteratorPost(
                    request.continuationToken(continuationToken).maxPageSize(SMALL_PAGE_SIZE));
        final var smallPageKeys = smallResponse.getPage();
        pagedKeys.addAll(smallPageKeys);
        continuationToken = smallResponse.getContinuationToken();
        if (continuationToken == null) {
          assertThat(smallPageKeys.size()).isLessThanOrEqualTo(SMALL_PAGE_SIZE);
          break;
        } else {
          assertThat(smallPageKeys.size()).isEqualTo(SMALL_PAGE_SIZE);
        }
      }

      // the keys collected via paging should be the same as when returned in one response
      assertThat(pagedKeys).isEqualTo(allKeys);
    }
  }

  @Test
  public void browse_api_entity_iterator_returns_actual_kv_store_keys() throws Exception {
    try (var test = buildRunningServerTest()) {
      test.suppressUnusedWarning();

      // execute a dummy transaction to create a known entry in the Faucet's KV-Store:
      final var intentHash =
          submitAndWaitForSuccess(test, Manifest.newRandomAccount(), List.of()).intentHash();

      // iterate over the only key in that KV-Store
      final var allKeys =
          getBrowseApi()
              .browseKvStoreIteratorPost(
                  new BrowseKeyValueStoreIteratorRequest()
                      .entityAddress(getTransactionsKeyValueStoreAddress())
                      .network(networkLogicalName))
              .getPage();
      final var theOnlyKey = Iterables.getOnlyElement(allKeys).getProgrammaticJson();

      // assert it is the executed transaction's hash, as SBOR in programmatic JSON format:
      assertThat(theOnlyKey)
          .isEqualTo(
              Map.of(
                  "type_name", "Hash",
                  "kind", "Bytes",
                  "element_kind", "U8",
                  "hex", intentHash.hex()));
    }
  }

  private String getTransactionsKeyValueStoreAddress() throws ApiException {
    final var wellKnownAddresses =
        getStatusApi().statusNetworkConfigurationPost().getWellKnownAddresses();
    final var faucetState =
        (Map<?, ?>)
            getBrowseApi()
                .browseObjectFieldPost(
                    new BrowseObjectFieldRequest()
                        .network(networkLogicalName)
                        .entityAddress(wellKnownAddresses.getFaucet())
                        .fieldIndex(0))
                .getContent()
                .getProgrammaticJson();
    final var faucetFields = (List<?>) faucetState.get("fields");
    return faucetFields.stream()
        .map(field -> (Map<?, ?>) field)
        .filter(field -> field.get("field_name").equals("transactions"))
        .findAny()
        .map(field -> (String) field.get("value"))
        .orElseThrow();
  }
}
