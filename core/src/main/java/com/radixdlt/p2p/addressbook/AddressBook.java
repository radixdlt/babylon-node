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

import static com.radixdlt.lang.Tuple.tuple;
import static com.radixdlt.lang.Unit.unit;
import static java.util.function.Predicate.not;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.inject.Inject;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.lang.Unit;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.P2PConfig;
import com.radixdlt.p2p.PeerEvent;
import com.radixdlt.p2p.PeerEvent.PeerBanned;
import com.radixdlt.p2p.RadixNodeUri;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry.LatestConnectionStatus;
import com.radixdlt.utils.InetUtils;
import com.radixdlt.utils.LRUCache;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.time.Duration;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Stream;

/** Manages known peers network addresses and their metadata. */
public final class AddressBook {

  /**
   * A stateful comparator for known peer addresses that uses both their latest connection status
   * (persistent) and a number of failed connection attempts (volatile). Failure counts are used to
   * cycle the addresses for retries. The entries are sorted in the following manner: successful ->
   * unknown (no connection attempted yet) -> failed (by num of failures)
   */
  private static final class PeerAddressEntryComparator implements Comparator<PeerAddressEntry> {
    private final Map<RadixNodeUri, Integer> failureCounts = new HashMap<>();

    private int toIntValue(PeerAddressEntry peerAddressEntry) {
      return peerAddressEntry
          .getLatestConnectionStatus()
          .map(
              latestConnectionStatus -> {
                if (latestConnectionStatus == LatestConnectionStatus.SUCCESS) {
                  return 1;
                } else {
                  return -(1 + failureCounts.getOrDefault(peerAddressEntry.getUri(), 0));
                }
              })
          .orElse(0);
    }

    @Override
    public int compare(PeerAddressEntry a, PeerAddressEntry b) {
      return Integer.compare(toIntValue(b), toIntValue(a));
    }

    void incFailures(RadixNodeUri uri) {
      synchronized (this.failureCounts) {
        final var curr = this.failureCounts.getOrDefault(uri, 0);
        this.failureCounts.put(uri, curr + 1);
      }
    }

    void resetFailures(RadixNodeUri uri) {
      synchronized (this.failureCounts) {
        this.failureCounts.remove(uri);
      }
    }
  }

  private final RadixNodeUri self;
  private final P2PConfig p2pConfig;
  private final EventDispatcher<PeerEvent> peerEventDispatcher;
  private final AddressBookPersistence persistence;
  private final Object lock = new Object();
  private final Map<NodeId, AddressBookEntry> knownPeers = new ConcurrentHashMap<>();
  private int numStoredAddresses;
  private final PeerAddressEntryComparator addressEntryComparator =
      new PeerAddressEntryComparator();

  /**
   * Stores nodes that are considered in some way important (validators, for example). Address book
   * makes sure to keep at least one address for each such peer (i.e. they receive special care
   * during address book clean up).
   */
  private final LRUCache<NodeId, Unit> highPriorityPeers;

  @Inject
  public AddressBook(
      @Self RadixNodeUri self,
      P2PConfig p2pConfig,
      EventDispatcher<PeerEvent> peerEventDispatcher,
      AddressBookPersistence persistence) {
    this.self = Objects.requireNonNull(self);
    this.p2pConfig = Objects.requireNonNull(p2pConfig);
    this.peerEventDispatcher = Objects.requireNonNull(peerEventDispatcher);
    this.persistence = Objects.requireNonNull(persistence);
    persistence.getAllEntries().forEach(e -> knownPeers.put(e.getNodeId(), e));
    this.numStoredAddresses =
        knownPeers.values().stream().mapToInt(e -> e.getKnownAddresses().size()).sum();

    // Reserving up to 1/10 of the address book for high priority peers
    this.highPriorityPeers = new LRUCache<>(p2pConfig.addressBookMaxSize() / 10);
    for (var nodeId : persistence.getHighPriorityPeers()) {
      this.highPriorityPeers.put(nodeId, unit());
    }
  }

  public void addUncheckedPeers(Set<RadixNodeUri> peers) {
    synchronized (lock) {
      final var filteredUris =
          peers.stream()
              .filter(not(uri -> uri.getNodeId().equals(this.self.getNodeId())))
              .filter(this::sameNetworkHrp)
              .filter(this::isPeerIpAddressValid)
              .filter(
                  uri ->
                      Optional.ofNullable(knownPeers.get(uri.getNodeId()))
                          .filter(e -> e.hasAddress(uri))
                          .isEmpty())
              .collect(ImmutableList.toImmutableList());

      final var slotsAvailable = p2pConfig.addressBookMaxSize() - numStoredAddresses;
      if (slotsAvailable < filteredUris.size()) {
        final var additionalSlotsNeededToIngestAllNewUris = filteredUris.size() - slotsAvailable;
        this.removeLowestQualityAddresses(additionalSlotsNeededToIngestAllNewUris);
      }
      // No matter if we've been able to clean up any slots or not,
      // insert as many addresses as we can.

      // TODO: the response may contain some addresses for nodes on the `highPriorityPeers` list,
      // which we may not have in our address book and which we should prioritise over existing
      // addresses, even if they were used to establish a successful connection before.

      final var slotsAvailableAfterCleanup = p2pConfig.addressBookMaxSize() - numStoredAddresses;
      filteredUris.stream()
          .limit(Math.max(0, slotsAvailableAfterCleanup))
          .forEach(this::insertOrUpdateAddressBookEntryWithUri);
    }
  }

  private void insertOrUpdateAddressBookEntryWithUri(RadixNodeUri uri) {
    final var maybeExistingEntry = this.knownPeers.get(uri.getNodeId());
    final var newOrUpdatedEntry =
        maybeExistingEntry == null
            ? AddressBookEntry.create(uri)
            : maybeExistingEntry.cleanupExpiredFailedHandshakeUris().addUriIfNotExists(uri);

    if (!newOrUpdatedEntry.equals(maybeExistingEntry)) {
      upsertOrRemoveIfEmpty(newOrUpdatedEntry);
    }
  }

  private boolean sameNetworkHrp(RadixNodeUri uri) {
    return uri.getNetworkNodeHrp().equals(this.self.getNetworkNodeHrp());
  }

  public Optional<AddressBookEntry> findById(NodeId nodeId) {
    return Optional.ofNullable(this.knownPeers.get(nodeId));
  }

  public void reportHighPriorityPeer(NodeId nodeId) {
    final var currKeys = highPriorityPeers.keys();
    if (currKeys.size() == 0 || !currKeys.get(currKeys.size() - 1).equals(nodeId)) {
      highPriorityPeers.put(nodeId, unit());
      persistence.storeHighPriorityPeers(highPriorityPeers.keys());
    }
  }

  public ImmutableList<RadixNodeUri> bestKnownAddressesById(NodeId nodeId) {
    final Optional<AddressBookEntry> addressBookEntryOpt;
    synchronized (lock) {
      addressBookEntryOpt = Optional.ofNullable(this.knownPeers.get(nodeId));
    }
    return onlyValidUrisSorted(addressBookEntryOpt.stream())
        .collect(ImmutableList.toImmutableList());
  }

  private Stream<RadixNodeUri> onlyValidUrisSorted(Stream<AddressBookEntry> entries) {
    return entries
        .filter(not(AddressBookEntry::isBanned))
        .flatMap(e -> e.getKnownAddresses().stream())
        .filter(PeerAddressEntry::failedHandshakeIsEmptyOrExpired)
        .filter(addressBookEntry -> this.isPeerIpAddressValid(addressBookEntry.getUri()))
        .sorted(addressEntryComparator)
        .map(AddressBookEntry.PeerAddressEntry::getUri);
  }

  private boolean isPeerIpAddressValid(RadixNodeUri uri) {
    final InetAddress inetAddr;
    try {
      inetAddr = InetAddress.getByName(uri.getHost());
    } catch (UnknownHostException e) {
      return false;
    }

    // To filter out any local interface IP addresses (using the actual listen bind port)
    final var isLocalSelf =
        p2pConfig.listenPort() == uri.getPort() && InetUtils.isLocalAddress(inetAddr);

    // To filter out a public IP address (possibly running behind a NAT, hence using an advertised
    // "broadcast port")
    final var isPublicSelf =
        p2pConfig.broadcastPort() == uri.getPort() && self.getHost().equals(uri.getHost());

    return !isLocalSelf && !isPublicSelf;
  }

  public void addOrUpdatePeerWithSuccessfulConnection(RadixNodeUri radixNodeUri) {
    // We've managed to successfully connect (incl. a complete handshake)
    // to a peer using a specific URI.
    // If the same (host, port) pair has been stored for any other node ID,
    // we need to remove it (this can happen e.g. if a new key is used on the same server).
    this.cleanKnownPeersAddressesAfterSuccessfulConnection(radixNodeUri);
    this.addOrUpdatePeerWithLatestConnectionStatus(radixNodeUri, LatestConnectionStatus.SUCCESS);
    this.addressEntryComparator.resetFailures(radixNodeUri);
  }

  /**
   * This removes any addresses with a matching host and port, but stored for a different node ID.
   */
  private void cleanKnownPeersAddressesAfterSuccessfulConnection(RadixNodeUri uriToKeep) {
    synchronized (lock) {
      final var iter = knownPeers.entrySet().iterator();
      while (iter.hasNext()) {
        final var entry = iter.next();
        if (entry.getKey().equals(uriToKeep.getNodeId())) {
          // We're keeping the address that we've just used
          continue;
        }
        final var updatedEntry =
            entry
                .getValue()
                .removeAddressesThatMatchHostAndPort(uriToKeep.getHost(), uriToKeep.getPort());
        final var hasEntryBeenModified =
            entry.getValue().getKnownAddresses().size() != updatedEntry.getKnownAddresses().size();

        if (hasEntryBeenModified) {
          if (updatedEntry.isMeaningless()) {
            iter.remove();
            this.persistence.removeEntry(updatedEntry.getNodeId());
          } else {
            entry.setValue(updatedEntry);
            this.persistence.upsertEntry(updatedEntry);
          }
        }
      }
    }
  }

  public void addOrUpdatePeerWithFailedConnection(RadixNodeUri radixNodeUri) {
    this.addOrUpdatePeerWithLatestConnectionStatus(radixNodeUri, LatestConnectionStatus.FAILURE);
    this.addressEntryComparator.incFailures(radixNodeUri);
  }

  private void addOrUpdatePeerWithLatestConnectionStatus(
      RadixNodeUri radixNodeUri, LatestConnectionStatus latestConnectionStatus) {
    synchronized (lock) {
      final var maybeExistingEntry =
          Optional.ofNullable(this.knownPeers.get(radixNodeUri.getNodeId()));
      final var newOrUpdatedEntry =
          maybeExistingEntry
              .map(
                  e ->
                      e.cleanupExpiredFailedHandshakeUris()
                          .withLatestConnectionStatusForUri(radixNodeUri, latestConnectionStatus))
              .orElseGet(
                  () ->
                      AddressBookEntry.createWithLatestConnectionStatus(
                          radixNodeUri, latestConnectionStatus));
      /*
      This is a bit of a corner case. This should almost always be an address update, not insertion.
      We've just used a URI to establish a connection, so we must have had it in the address book.
      But in theory, it might have been removed in the meantime (f.e. due to a call to `addUncheckedPeers`),
      so we need to re-check the limits.
       */
      final var prevAddressesCount =
          maybeExistingEntry.map(e -> e.getKnownAddresses().size()).orElse(0);
      final var newAddressesCount = newOrUpdatedEntry.getKnownAddresses().size();
      final var numAddressesDiff = newAddressesCount - prevAddressesCount;
      // This will always be <= 1, but let's keep our abstractions right :)
      final var additionalFreeSlotsNeeded =
          (numStoredAddresses + numAddressesDiff) - p2pConfig.addressBookMaxSize();
      if (additionalFreeSlotsNeeded > 0) {
        this.removeLowestQualityAddresses(additionalFreeSlotsNeeded);
      }
      final var freeSlotsAfterCleanup = p2pConfig.addressBookMaxSize() - numStoredAddresses;
      if (freeSlotsAfterCleanup >= numAddressesDiff) {
        upsertOrRemoveIfEmpty(newOrUpdatedEntry);
      }
    }
  }

  /**
   * Removes up to `maxAddressesToRemove` addresses of the lowest quality from the address book. The
   * interface doesn't specify an exact definition of address "quality", but caller can expect that:
   * 1. the removed addresses are, in some sense, worse than a default (i.e. freshly added address)
   * 2. nodes from the `highPriorityPeers` list get special treatment, i.e. if an address for any
   * such node is to be removed, there must be at least one different address for the same node ID
   * that will be kept.
   */
  private void removeLowestQualityAddresses(int maxAddressesToRemove) {
    /* The removal order is as follows:
    - first, any invalid URIs (just in case there are any)
    - then the addresses that have failed the handshake
    - then all banned peers (they come after the failed handshake
        because ban info is more important)
    - then the default comparator (based on connection attempts),
        reversed - from worst to best
     */
    final var worstAddressesComparator =
        Comparator.<Tuple2<AddressBookEntry, PeerAddressEntry>, Boolean>comparing(
                e -> !isPeerIpAddressValid(e.last().getUri()))
            .thenComparing(e -> e.last().failedHandshakeIsPresent())
            .thenComparing(e -> e.first().isBanned())
            .thenComparing((a, b) -> addressEntryComparator.reversed().compare(a.last(), b.last()));

    final var addressesToRemove =
        this.knownPeers.entrySet().stream()
            .filter(
                not(
                    e ->
                        // For now let's just keep all addresses of these nodes
                        this.highPriorityPeers.contains(e.getKey())))
            .flatMap(
                e -> e.getValue().getKnownAddresses().stream().map(a -> tuple(e.getValue(), a)))
            .filter(
                not(
                    t -> {
                      // Do not remove any addresses that haven't been tried yet
                      // (as they're considered the same quality as a default, unchecked address)
                      final var isUntried = t.last().getLatestConnectionStatus().isEmpty();
                      // Do not remove any addresses that have previously succeeded
                      // (as they're considered "better" than a default, unchecked address)
                      final var wasSuccess =
                          t.last().getLatestConnectionStatus().stream()
                              .anyMatch(l -> l.equals(LatestConnectionStatus.SUCCESS));
                      return isUntried || wasSuccess;
                    }))
            .sorted(worstAddressesComparator)
            .limit(Math.max(0, maxAddressesToRemove))
            .toList();

    for (var addressToRemove : addressesToRemove) {
      final var entry = addressToRemove.first();
      final var address = addressToRemove.last();
      final var updatedEntry = entry.removeAddressEntry(address);
      upsertOrRemoveIfEmpty(updatedEntry);
    }
  }

  private void upsertOrRemoveIfEmpty(AddressBookEntry entry) {
    final var maybePreviousEntry = Optional.ofNullable(this.knownPeers.get(entry.getNodeId()));
    final var previousAddressesCount =
        maybePreviousEntry.map(e -> e.getKnownAddresses().size()).orElse(0);
    final var addressesCountDiff = entry.getKnownAddresses().size() - previousAddressesCount;
    this.numStoredAddresses = this.numStoredAddresses + addressesCountDiff;
    if (entry.isMeaningless()) {
      this.knownPeers.remove(entry.getNodeId());
      this.persistence.removeEntry(entry.getNodeId());
    } else {
      this.knownPeers.put(entry.getNodeId(), entry);
      this.persistence.upsertEntry(entry);
    }
  }

  public Stream<RadixNodeUri> bestCandidatesToConnect() {
    return onlyValidUrisSorted(this.knownPeers.values().stream());
  }

  void banPeer(NodeId nodeId, Duration banDuration) {
    synchronized (lock) {
      final var banUntil = Instant.now().plus(banDuration);
      final var maybeExistingEntry = findById(nodeId);
      if (maybeExistingEntry.isPresent()) {
        final var existingEntry = maybeExistingEntry.get();
        final var alreadyBanned =
            existingEntry.bannedUntil().filter(bu -> bu.isAfter(banUntil)).isPresent();
        if (!alreadyBanned) {
          final var updatedEntry =
              existingEntry.cleanupExpiredFailedHandshakeUris().withBanUntil(banUntil);
          upsertOrRemoveIfEmpty(updatedEntry);
          this.peerEventDispatcher.dispatch(new PeerBanned(nodeId));
        }
      } else {
        final var newEntry = AddressBookEntry.createBanned(nodeId, banUntil);
        upsertOrRemoveIfEmpty(newEntry);
        this.peerEventDispatcher.dispatch(new PeerBanned(nodeId));
      }
    }
  }

  public ImmutableMap<NodeId, AddressBookEntry> knownPeers() {
    return ImmutableMap.copyOf(knownPeers);
  }

  public void reportFailedHandshake(RadixNodeUri uri) {
    synchronized (lock) {
      final var retainUntil =
          Instant.now().plus(p2pConfig.failedHandshakeAddressesRetentionDuration());
      final var maybeExistingEntry = this.knownPeers.get(uri.getNodeId());
      final var newOrUpdatedEntry =
          maybeExistingEntry == null
              ? AddressBookEntry.createWithFailedHandshake(uri, retainUntil)
                  .withLatestConnectionStatusForUri(uri, LatestConnectionStatus.FAILURE)
              : maybeExistingEntry
                  .cleanupExpiredFailedHandshakeUris()
                  .withFailedHandshakeUri(uri, retainUntil)
                  .withLatestConnectionStatusForUri(uri, LatestConnectionStatus.FAILURE);
      upsertOrRemoveIfEmpty(newOrUpdatedEntry);
    }
  }

  public void clear() {
    synchronized (lock) {
      this.persistence.close();
      this.persistence.reset();
      this.persistence.open();
      this.knownPeers.clear();
    }
  }
}
