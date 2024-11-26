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

package com.radixdlt.p2p;

import static com.radixdlt.lang.Option.none;
import static com.radixdlt.lang.Option.some;
import static org.junit.Assert.*;

import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.helper.NodeRustEnvironmentBuilder;
import com.radixdlt.lang.Option;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.p2p.PeerAddressEntryDTO.ConnectionStatus;
import java.util.Random;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RocksDbAddressBookStoreTest {
  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private static final Random RANDOM = new Random();

  @Test
  public void test_address_book_can_be_marked_as_migrated() throws Exception {
    try (var nodeRustEnvironment =
        NodeRustEnvironmentBuilder.createNodeRustEnvironment(folder.newFolder().getPath())) {
      var addressBookStore =
          RocksDbAddressBookStore.create(
              new MetricsInitializer().initialize(), nodeRustEnvironment);

      assertFalse(addressBookStore.isMigrated());

      addressBookStore.markAsMigrated();

      assertTrue(addressBookStore.isMigrated());
    }
  }

  @Test
  public void test_address_book_entries_can_be_saved_and_restored() throws Exception {
    try (var nodeRustEnvironment =
        NodeRustEnvironmentBuilder.createNodeRustEnvironment(folder.newFolder().getPath())) {
      var addressBookStore =
          RocksDbAddressBookStore.create(
              new MetricsInitializer().initialize(), nodeRustEnvironment);

      // New store is empty
      var empty = addressBookStore.getAllEntries();
      assertTrue(empty.isEmpty());

      // Ensure keys are repeatable to make test deterministic
      var entry1 = newAddressBookEntry(1);

      addressBookStore.upsertEntry(entry1);

      // Store now contains one entry
      var allEntries = addressBookStore.getAllEntries();
      assertEquals(1L, allEntries.size());
      assertEquals(entry1, allEntries.get(0));

      // Ensure keys are repeatable to make test deterministic
      var entry2 = newAddressBookEntry(2);

      // Add another entry
      addressBookStore.upsertEntry(entry2);

      allEntries = addressBookStore.getAllEntries();
      assertEquals(2L, allEntries.size());
      assertEquals(entry1, allEntries.get(0));
      assertEquals(entry2, allEntries.get(1));
    }
  }

  @Test
  public void test_address_book_entry_can_be_added_and_removed() throws Exception {
    try (var nodeRustEnvironment =
        NodeRustEnvironmentBuilder.createNodeRustEnvironment(folder.newFolder().getPath())) {
      var addressBookStore =
          RocksDbAddressBookStore.create(
              new MetricsInitializer().initialize(), nodeRustEnvironment);

      // New store is empty
      var empty = addressBookStore.getAllEntries();
      assertTrue(empty.isEmpty());

      // Ensure keys are repeatable to make test deterministic
      var entry1 = newAddressBookEntry(1);
      var entry2 = newAddressBookEntry(2);

      addressBookStore.upsertEntry(entry1);
      addressBookStore.upsertEntry(entry2);

      // Check that entries were added
      var allEntries = addressBookStore.getAllEntries();
      assertEquals(2L, allEntries.size());
      assertEquals(entry1, allEntries.get(0));
      assertEquals(entry2, allEntries.get(1));

      // Remove entry1
      var removed = addressBookStore.removeEntry(entry1.nodeId());
      assertTrue(removed);

      // Check that entry1 was removed
      allEntries = addressBookStore.getAllEntries();
      assertEquals(1L, allEntries.size());
      assertEquals(entry2, allEntries.get(0));
    }
  }

  @Test
  public void test_address_book_can_be_reset() throws Exception {
    try (var nodeRustEnvironment =
        NodeRustEnvironmentBuilder.createNodeRustEnvironment(folder.newFolder().getPath())) {
      var addressBookStore =
          RocksDbAddressBookStore.create(
              new MetricsInitializer().initialize(), nodeRustEnvironment);

      // New store is empty
      var empty = addressBookStore.getAllEntries();
      assertTrue(empty.isEmpty());

      // Ensure keys are repeatable to make test deterministic
      var entry1 = newAddressBookEntry(1);
      var entry2 = newAddressBookEntry(2);

      addressBookStore.upsertEntry(entry1);
      addressBookStore.upsertEntry(entry2);

      // Check that entries were added
      var allEntries = addressBookStore.getAllEntries();
      assertEquals(2L, allEntries.size());
      assertEquals(entry1, allEntries.get(0));
      assertEquals(entry2, allEntries.get(1));

      // Reset store
      addressBookStore.reset();

      // Check that entry1 was removed
      empty = addressBookStore.getAllEntries();
      assertTrue(empty.isEmpty());
    }
  }

  private static AddressBookEntryDTO newAddressBookEntry(int id) {
    return new AddressBookEntryDTO(
        new NodeIdDTO(ECKeyPair.fromSeed(new byte[] {(byte) id}).getPublicKey()),
        RANDOM.nextBoolean() ? some(RANDOM.nextLong()) : none(),
        peerAddresses());
  }

  private static Set<PeerAddressEntryDTO> peerAddresses() {
    return IntStream.range(0, RANDOM.nextInt(3))
        .mapToObj(__ -> newPeerAddressEntry())
        .collect(Collectors.toSet());
  }

  private static PeerAddressEntryDTO newPeerAddressEntry() {
    var pubKey = ECKeyPair.generateNew().getPublicKey().getBytes();
    return new PeerAddressEntryDTO(pubKey, connectionStatus(), Option.some(RANDOM.nextLong()));
  }

  private static Option<ConnectionStatus> connectionStatus() {
    return RANDOM.nextBoolean()
        ? Option.some(RANDOM.nextBoolean() ? ConnectionStatus.SUCCESS : ConnectionStatus.FAILURE)
        : Option.none();
  }
}
