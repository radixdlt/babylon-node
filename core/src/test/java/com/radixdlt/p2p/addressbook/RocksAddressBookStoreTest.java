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

package com.radixdlt.p2p.addressbook;

import static org.junit.Assert.*;

import com.google.common.collect.ImmutableSet;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.p2p.*;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry.FailedHandshake;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry.LatestConnectionStatus;
import com.radixdlt.rev2.NodeRustEnvironmentHelper;
import java.io.IOException;
import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.function.Consumer;
import java.util.stream.IntStream;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RocksAddressBookStoreTest {
  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private static final Random RANDOM = new Random();

  @Test
  public void test_address_book_entries_can_be_saved_and_restored() {
    runTest(
        rocksAddressBookStore -> {
          // New store is empty
          var empty = rocksAddressBookStore.getAllEntries();
          assertTrue(empty.isEmpty());

          // One entry can be stored and restored
          var entry1 = newAddressBookEntry(1);
          assertTrue(rocksAddressBookStore.upsertEntry(entry1));

          var allEntries = rocksAddressBookStore.getAllEntries();
          assertEquals(1L, allEntries.size());
          assertEquals(entry1, allEntries.get(0));

          // Another entry can be stored and restored
          var entry2 = newAddressBookEntry(2);
          assertTrue(rocksAddressBookStore.upsertEntry(entry2));

          allEntries = rocksAddressBookStore.getAllEntries();
          assertEquals(2L, allEntries.size());
          assertEquals(entry1, allEntries.get(0));
          assertEquals(entry2, allEntries.get(1));

          // An entry can be deleted
          assertTrue(rocksAddressBookStore.removeEntry(entry1.getNodeId()));

          allEntries = rocksAddressBookStore.getAllEntries();
          assertEquals(1L, allEntries.size());
          assertEquals(entry2, allEntries.get(0));

          // Address book can be reset
          rocksAddressBookStore.reset();

          empty = rocksAddressBookStore.getAllEntries();
          assertTrue(empty.isEmpty());
        });
  }

  @Test
  public void test_high_priority_peers_can_be_saved_and_restored() {
    runTest(
        rocksAddressBookStore -> {
          var peers = List.of(newNodeId(11), newNodeId(12), newNodeId(13));

          // New store is empty
          var empty = rocksAddressBookStore.getHighPriorityPeers();
          assertTrue(empty.isEmpty());

          // Peers can be stored and restored
          rocksAddressBookStore.storeHighPriorityPeers(peers);

          var allPeers = rocksAddressBookStore.getHighPriorityPeers();

          assertEquals(3L, allPeers.size());
          assertEquals(peers, allPeers);
        });
  }

  private void runTest(Consumer<RocksAddressBookStore> test) {
    try (var environment =
        NodeRustEnvironmentHelper.createNodeRustEnvironment(folder.newFolder().getPath())) {
      var addressBook = RocksDbAddressBookStore.create(newMetrics(), environment);
      var peersStore = RocksDbHighPriorityPeersStore.create(newMetrics(), environment);
      try (var underTest = new RocksAddressBookStore(addressBook, peersStore)) {
        test.accept(underTest);
      }
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  private static Metrics newMetrics() {
    return new MetricsInitializer().initialize();
  }

  public static AddressBookEntry newAddressBookEntry(int id) {
    var pubKey = newPubKey((byte) id);
    var bannedUntil = newBannedUntil();

    return new AddressBookEntry(
        NodeId.fromPublicKey(pubKey), bannedUntil, newKnownAddresses(pubKey));
  }

  public static NodeId newNodeId(int id) {
    return NodeId.fromPublicKey(newPubKey((byte) id));
  }

  private static ECDSASecp256k1PublicKey newPubKey(byte id) {
    return ECKeyPair.fromSeed(new byte[] {id}).getPublicKey();
  }

  private static ImmutableSet<PeerAddressEntry> newKnownAddresses(ECDSASecp256k1PublicKey pubKey) {
    return IntStream.range(0, RANDOM.nextInt(3))
        .mapToObj(__ -> newPeerAddressEntry(pubKey))
        .collect(ImmutableSet.toImmutableSet());
  }

  private static PeerAddressEntry newPeerAddressEntry(ECDSASecp256k1PublicKey pubKey) {
    var failedHandshake = newFailedHandshake();
    var latestConnectionStatus = newLatestConnectionStatus();
    var uri = RadixNodeUri.fromPubKeyAndAddress(1, pubKey, "127.0.0.1", 30000);

    return new PeerAddressEntry(uri, latestConnectionStatus, failedHandshake);
  }

  private static Optional<LatestConnectionStatus> newLatestConnectionStatus() {
    return RANDOM.nextBoolean()
        ? Optional.of(
            RANDOM.nextBoolean() ? LatestConnectionStatus.FAILURE : LatestConnectionStatus.SUCCESS)
        : Optional.empty();
  }

  private static Optional<Instant> newBannedUntil() {
    return RANDOM.nextBoolean()
        ? Optional.of(Instant.ofEpochMilli(Math.abs(RANDOM.nextLong())))
        : Optional.empty();
  }

  private static Optional<FailedHandshake> newFailedHandshake() {
    return RANDOM.nextBoolean()
        ? Optional.of(new FailedHandshake(Instant.ofEpochMilli(Math.abs(RANDOM.nextLong()))))
        : Optional.empty();
  }
}
