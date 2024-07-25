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

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.radixdlt.lang.Option;
import com.radixdlt.p2p.*;
import com.radixdlt.serialization.DeserializeException;
import java.net.URISyntaxException;
import java.time.Instant;
import java.util.List;
import java.util.Set;

public class RocksAddressBookStore implements AddressBookPersistence {
  private final RocksDbAddressBookStore addressBookStore;
  private final RocksDbHighPriorityPeersStore highPriorityPeersStore;

  public RocksAddressBookStore(
      RocksDbAddressBookStore addressBookStore,
      RocksDbHighPriorityPeersStore highPriorityPeersStore) {
    this.addressBookStore = addressBookStore;
    this.highPriorityPeersStore = highPriorityPeersStore;
  }

  public void open() {
    // no-op
  }

  public void close() {
    // no-op
  }

  public void reset() {
    addressBookStore.reset();
    highPriorityPeersStore.reset();
  }

  public boolean upsertEntry(AddressBookEntry entry) {
    return addressBookStore.upsertEntry(toDTO(entry));
  }

  public boolean removeEntry(NodeId nodeId) {
    return addressBookStore.removeEntry(toDTO(nodeId));
  }

  public ImmutableList<AddressBookEntry> getAllEntries() {
    return ImmutableList.copyOf(
        addressBookStore.getAllEntries().stream().map(RocksAddressBookStore::fromDTO).iterator());
  }

  public void storeHighPriorityPeers(List<NodeId> ids) {
    highPriorityPeersStore.storeHighPriorityPeers(
        ids.stream().map(RocksAddressBookStore::toDTO).collect(ImmutableList.toImmutableList()));
  }

  public List<NodeId> getHighPriorityPeers() {
    return highPriorityPeersStore.getHighPriorityPeers().stream()
        .map(RocksAddressBookStore::fromDTO)
        .collect(ImmutableList.toImmutableList());
  }

  private static AddressBookEntryDTO toDTO(AddressBookEntry entry) {
    return new AddressBookEntryDTO(
        new NodeIdDTO(entry.getNodeId().getPublicKey()),
        Option.from(entry.bannedUntil().map(Instant::toEpochMilli)),
        toDTO(entry.getKnownAddresses()));
  }

  private static AddressBookEntry fromDTO(AddressBookEntryDTO dto) {
    return new AddressBookEntry(
        NodeId.fromPublicKey(dto.nodeId().publicKey()),
        dto.bannedUntil().map(Instant::ofEpochMilli).toOptional(),
        fromDTO(dto.knownAddresses()));
  }

  private static NodeIdDTO toDTO(NodeId nodeId) {
    return new NodeIdDTO(nodeId.getPublicKey());
  }

  private static NodeId fromDTO(NodeIdDTO dto) {
    return NodeId.fromPublicKey(dto.publicKey());
  }

  private static PeerAddressEntryDTO toDTO(AddressBookEntry.PeerAddressEntry entry) {
    var connectionStatus =
        entry.getLatestConnectionStatus().map(RocksAddressBookStore::toConnectionStatus);

    var instant =
        entry
            .getMaybeFailedHandshake()
            .map(AddressBookEntry.PeerAddressEntry.FailedHandshake::retainUntil)
            .map(Instant::toEpochMilli);
    return new PeerAddressEntryDTO(
        entry.getUri().getSerializedValue(), Option.from(connectionStatus), Option.from(instant));
  }

  private static PeerAddressEntryDTO.ConnectionStatus toConnectionStatus(
      AddressBookEntry.PeerAddressEntry.LatestConnectionStatus status) {
    return switch (status) {
      case SUCCESS -> PeerAddressEntryDTO.ConnectionStatus.SUCCESS;
      case FAILURE -> PeerAddressEntryDTO.ConnectionStatus.FAILURE;
    };
  }

  private static AddressBookEntry.PeerAddressEntry fromDTO(PeerAddressEntryDTO dto) {
    return new AddressBookEntry.PeerAddressEntry(
        deserializeRadixNodeUri(dto),
        dto.latestConnectionStatus()
            .map(RocksAddressBookStore::toLatestConnectionStatus)
            .toOptional(),
        dto.maybeFailedHandshake()
            .map(Instant::ofEpochMilli)
            .map(AddressBookEntry.PeerAddressEntry.FailedHandshake::new)
            .toOptional());
  }

  private static AddressBookEntry.PeerAddressEntry.LatestConnectionStatus toLatestConnectionStatus(
      PeerAddressEntryDTO.ConnectionStatus status) {
    if (status instanceof PeerAddressEntryDTO.ConnectionStatus.Success) {
      return AddressBookEntry.PeerAddressEntry.LatestConnectionStatus.SUCCESS;
    }
    if (status instanceof PeerAddressEntryDTO.ConnectionStatus.Failure) {
      return AddressBookEntry.PeerAddressEntry.LatestConnectionStatus.FAILURE;
    }
    throw new IllegalStateException("Unknown connection status: " + status);
  }

  private static RadixNodeUri deserializeRadixNodeUri(PeerAddressEntryDTO dto) {
    RadixNodeUri nodeUri;
    try {
      nodeUri = RadixNodeUri.deserialize(dto.address());
    } catch (URISyntaxException | DeserializeException e) {
      throw new RuntimeException(e);
    }
    return nodeUri;
  }

  private static Set<PeerAddressEntryDTO> toDTO(Set<AddressBookEntry.PeerAddressEntry> input) {
    return input.stream().map(RocksAddressBookStore::toDTO).collect(ImmutableSet.toImmutableSet());
  }

  private static ImmutableSet<AddressBookEntry.PeerAddressEntry> fromDTO(
      Set<PeerAddressEntryDTO> input) {
    return input.stream()
        .map(RocksAddressBookStore::fromDTO)
        .collect(ImmutableSet.toImmutableSet());
  }
}
