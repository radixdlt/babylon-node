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

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.junit.Assert.*;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.google.common.collect.ImmutableSet;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.P2PConfig;
import com.radixdlt.p2p.RadixNodeUri;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.store.berkeley.BerkeleyDatabaseEnvironment;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.IOException;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.apache.commons.cli.ParseException;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class AddressBookTest {

  private final Random random = new Random(12345);

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  BerkeleyAddressBookStore addressBookStore;

  @Before
  public void setup() throws IOException {
    final var dbEnv = new BerkeleyDatabaseEnvironment(folder.newFolder().getAbsolutePath(), 100000);
    this.addressBookStore =
        new BerkeleyAddressBookStore(
            DefaultSerialization.getInstance(), dbEnv, new MetricsInitializer().initialize());
  }

  @Test
  public void address_book_should_filter_out_peers_with_different_network_hrp() {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "1.1.1.1", 30000);
    final var invalidPeer =
        RadixNodeUri.fromPubKeyAndAddress(
            2, ECKeyPair.generateNew().getPublicKey(), "2.2.2.2", 30000);
    final var sut =
        new AddressBook(self, defaultConfig(), rmock(EventDispatcher.class), addressBookStore);
    sut.addUncheckedPeers(Set.of(invalidPeer));
    assertTrue(sut.knownPeers().isEmpty());
  }

  @Test
  public void address_book_should_sort_entries_by_latest_connection_status() {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "127.0.0.10", 30303);
    final var peerKey = ECKeyPair.generateNew().getPublicKey();
    final var peerId = NodeId.fromPublicKey(peerKey);
    final var addr1 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.1", 30303);
    final var addr2 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.2", 30303);
    final var addr3 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.3", 30303);
    final var addr4 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.4", 30303);

    final var sut =
        new AddressBook(self, defaultConfig(), rmock(EventDispatcher.class), addressBookStore);

    sut.addUncheckedPeers(ImmutableSet.of(addr1, addr2, addr3, addr4));

    sut.addOrUpdatePeerWithSuccessfulConnection(addr1);
    final var bestAddr = sut.bestKnownAddressesById(peerId).get(0);
    assertEquals(addr1, bestAddr);

    sut.addOrUpdatePeerWithSuccessfulConnection(addr2);
    final var bestAddr2 = sut.bestKnownAddressesById(peerId).get(0);
    assertTrue(bestAddr2 == addr1 || bestAddr2 == addr2);

    sut.addOrUpdatePeerWithFailedConnection(addr1);
    final var bestAddr3 = sut.bestKnownAddressesById(peerId).get(0);
    assertEquals(addr2, bestAddr3);

    sut.addOrUpdatePeerWithFailedConnection(addr2);
    final var bestAddr4 = sut.bestKnownAddressesById(peerId).get(0);
    assertTrue(bestAddr4 == addr3 || bestAddr4 == addr4);

    sut.addOrUpdatePeerWithSuccessfulConnection(addr4);
    final var bestAddr5 = sut.bestKnownAddressesById(peerId).get(0);
    assertEquals(addr4, bestAddr5);
  }

  @Test
  public void address_book_should_cycle_failed_connection_uris() {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "127.0.0.10", 30303);
    final var peerKey = ECKeyPair.generateNew().getPublicKey();
    final var peerId = NodeId.fromPublicKey(peerKey);
    final var addr1 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.1", 30303);
    final var addr2 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.2", 30303);
    final var addr3 = RadixNodeUri.fromPubKeyAndAddress(1, peerKey, "127.0.0.3", 30303);

    final var sut =
        new AddressBook(self, defaultConfig(), rmock(EventDispatcher.class), addressBookStore);

    sut.addUncheckedPeers(ImmutableSet.of(addr1, addr2, addr3));

    var prevPrevBestAddr = sut.bestKnownAddressesById(peerId).get(0);
    sut.addOrUpdatePeerWithFailedConnection(prevPrevBestAddr);
    var prevBestAddr = sut.bestKnownAddressesById(peerId).get(0);
    for (int i = 0; i < 50; i++) {
      sut.addOrUpdatePeerWithFailedConnection(prevBestAddr);
      final var currBestAddr = sut.bestKnownAddressesById(peerId).get(0);
      assertNotEquals(prevBestAddr, currBestAddr);
      assertNotEquals(prevPrevBestAddr, prevBestAddr);
      prevPrevBestAddr = prevBestAddr;
      prevBestAddr = currBestAddr;
    }
  }

  @Test
  public void unchecked_localhost_uri_should_not_be_added() {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "192.168.50.50", 30303);
    final var localAddrSamePort =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "127.0.0.1", 30303);
    final var publicAddrSamePort =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), self.getHost(), 30303);
    final var localAddrDifferentPort =
        RadixNodeUri.fromPubKeyAndAddress(
            1, ECKeyPair.generateNew().getPublicKey(), "127.0.0.1", 30304);

    final var sut =
        new AddressBook(
            self, p2pConfig(30303, 30303, 100), rmock(EventDispatcher.class), addressBookStore);

    // Self addresses with the same port shouldn't be added
    sut.addUncheckedPeers(ImmutableSet.of(localAddrSamePort, publicAddrSamePort));
    assertTrue(addressBookStore.getAllEntries().isEmpty());

    // Self address with a different port should be added
    sut.addUncheckedPeers(ImmutableSet.of(localAddrDifferentPort));
    assertEquals(1, addressBookStore.getAllEntries().size());
  }

  @Test
  public void address_book_should_respect_its_size_limits() throws ParseException {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            Network.INTEGRATIONTESTNET.getId(),
            ECKeyPair.generateNew().getPublicKey(),
            "127.0.0.1",
            9000);

    final var sut =
        new AddressBook(
            self,
            P2PConfig.fromRuntimeProperties(
                RuntimeProperties.defaultWithOverrides(
                    Map.of("network.p2p.address_book_max_size", "5"))),
            rmock(EventDispatcher.class),
            addressBookStore);

    sut.addUncheckedPeers(
        Stream.generate(this::randomNodeUri).limit(5).collect(Collectors.toSet()));

    final var knownPeers1 = sut.knownPeers().values();
    assertEquals(knownPeers1.size(), 5);

    sut.addUncheckedPeers(Set.of(randomNodeUri()));
    final var knownPeers2 = sut.knownPeers().values();
    assertEquals(knownPeers1, knownPeers2);

    final var failedConnectionPeer = knownPeers2.stream().findFirst().orElseThrow();

    // Now let's simulate a connection failure on one of the nodes
    sut.addOrUpdatePeerWithFailedConnection(
        failedConnectionPeer.getKnownAddresses().stream().findFirst().orElseThrow().getUri());

    // We should be able to take the slot of the failed address
    final var newUri = randomNodeUri();
    sut.addUncheckedPeers(Set.of(newUri));
    final var knownPeers3 = sut.knownPeers().values();
    assertTrue(knownPeers3.stream().anyMatch(e -> e.hasAddress(newUri)));
    assertTrue(knownPeers3.stream().noneMatch(e -> e.equals(failedConnectionPeer)));
  }

  @Test
  public void address_book_should_not_remove_high_priority_peers() throws ParseException {
    final var self =
        RadixNodeUri.fromPubKeyAndAddress(
            Network.INTEGRATIONTESTNET.getId(),
            ECKeyPair.generateNew().getPublicKey(),
            "127.0.0.1",
            9000);

    final var sut =
        new AddressBook(
            self,
            P2PConfig.fromRuntimeProperties(
                RuntimeProperties.defaultWithOverrides(
                    Map.of("network.p2p.address_book_max_size", "100"))),
            rmock(EventDispatcher.class),
            addressBookStore);

    sut.addUncheckedPeers(
        Stream.generate(this::randomNodeUri).limit(100).collect(Collectors.toSet()));

    final var knownPeers1 = sut.knownPeers().values();
    assertEquals(knownPeers1.size(), 100);

    // Let's now simulate a failed connection on all peers
    for (var e : sut.knownPeers().entrySet()) {
      sut.addOrUpdatePeerWithFailedConnection(
          e.getValue().getKnownAddresses().stream().findFirst().orElseThrow().getUri());
    }

    final var knownPeers2 = sut.knownPeers().keySet().stream().toList();
    final var requestedNode = knownPeers2.get(5);
    // And explicitly request an address of some node
    sut.bestKnownAddressesById(requestedNode);

    // Now we try to insert 100 new addresses.
    // All (failed) addresses but the one that was requested will be replaced.
    sut.addUncheckedPeers(
        Stream.generate(this::randomNodeUri).limit(100).collect(Collectors.toSet()));

    final var knownPeers3 = sut.knownPeers().keySet();
    assertTrue(knownPeers3.contains(requestedNode));
    assertEquals(knownPeers3.size(), 100);
  }

  private RadixNodeUri randomNodeUri() {
    return RadixNodeUri.fromPubKeyAndAddress(
        Network.INTEGRATIONTESTNET.getId(),
        ECKeyPair.generateNew().getPublicKey(),
        "127.0." + random.nextInt(127) + "." + random.nextInt(127),
        random.nextInt(30000));
  }

  private P2PConfig defaultConfig() {
    return p2pConfig(30000, 30000, 1000);
  }

  private P2PConfig p2pConfig(int broadcastPort, int listenPort, int maxSize) {
    final var config = mock(P2PConfig.class);
    when(config.broadcastPort()).thenReturn(broadcastPort);
    when(config.listenPort()).thenReturn(listenPort);
    when(config.addressBookMaxSize()).thenReturn(maxSize);
    return config;
  }
}
