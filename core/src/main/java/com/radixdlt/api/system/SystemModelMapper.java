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

package com.radixdlt.api.system;

import com.google.inject.Inject;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.system.generated.models.*;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.p2p.P2PConfig;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.p2p.RadixNodeUri;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry;
import com.radixdlt.p2p.addressbook.AddressBookEntry.PeerAddressEntry.LatestConnectionStatus;
import com.radixdlt.p2p.capability.AppVersionCapability;
import com.radixdlt.p2p.capability.RemotePeerCapability;
import com.radixdlt.sync.SyncRelayConfig;
import java.math.BigDecimal;
import java.time.Instant;
import java.util.Map;

public final class SystemModelMapper {
  private final Addressing addressing;

  @Inject
  SystemModelMapper(Addressing addressing) {
    this.addressing = addressing;
  }

  public NetworkingConfiguration networkingConfiguration(
      ECDSASecp256k1PublicKey self, P2PConfig config) {
    return new NetworkingConfiguration()
        .defaultPort(config.defaultPort())
        .discoveryInterval(config.discoveryInterval())
        .listenAddress(config.listenAddress())
        .listenPort(config.listenPort())
        .broadcastPort(config.broadcastPort())
        .peerConnectionTimeout(config.peerConnectionTimeout())
        .maxInboundChannels(config.maxInboundChannels())
        .maxOutboundChannels(config.maxOutboundChannels())
        .channelBufferSize(config.channelBufferSize())
        .peerLivenessCheckInterval(config.peerLivenessCheckInterval())
        .pingTimeout(config.pingTimeout())
        .seedNodes(config.seedNodes())
        .nodeAddress(addressing.encodeNodeAddress(self));
  }

  public SyncConfiguration syncConfiguration(SyncRelayConfig syncRelayConfig) {
    return new SyncConfiguration()
        .syncCheckInterval(syncRelayConfig.syncCheckInterval())
        .syncCheckMaxPeers(syncRelayConfig.syncCheckMaxPeers())
        .requestTimeout(syncRelayConfig.requestTimeout())
        .ledgerStatusUpdateMaxPeersToNotify(syncRelayConfig.ledgerStatusUpdateMaxPeersToNotify())
        .maxLedgerUpdatesRate(BigDecimal.valueOf(syncRelayConfig.maxLedgerUpdatesRate()));
  }

  public Peer peer(PeersView.PeerInfo peerInfo) {
    final var peerId = addressing.encodeNodeAddress(peerInfo.getNodeId().getPublicKey());
    final var peer = new Peer().peerId(peerId).keyHex(peerInfo.getNodeId().getPublicKey().toHex());

    peerInfo
        .getChannels()
        .forEach(
            channel -> {
              final var maybeVersionCapability =
                  channel.getCapabilities().stream()
                      .filter(c -> c.getName().equals(AppVersionCapability.NAME))
                      .findFirst()
                      .map(RemotePeerCapability::getConfiguration)
                      .orElse(Map.of());
              final var versionString =
                  maybeVersionCapability.getOrDefault(
                      AppVersionCapability.CONFIG_VERSION, "unknown (likely pre-1.0.4)");
              final var versionCommit =
                  maybeVersionCapability.getOrDefault(
                      AppVersionCapability.CONFIG_COMMIT, "unknown");

              final var peerChannel =
                  new PeerChannel()
                      .type(
                          channel.isOutbound() ? PeerChannel.TypeEnum.OUT : PeerChannel.TypeEnum.IN)
                      .localPort(channel.getPort())
                      .ip(channel.getHost())
                      .applicationVersion(
                          new PeerApplicationVersion()
                              .version(versionString)
                              .commit(versionCommit));
              channel.getUri().map(RadixNodeUri::toString).ifPresent(peerChannel::uri);
              peer.addChannelsItem(peerChannel);
            });
    return peer;
  }

  public Address address(PeerAddressEntry entry) {
    return new Address()
        .uri(entry.getUri().toString())
        .failedHandshake(entry.failedHandshakeIsPresent())
        .lastConnectionStatus(
            Address.LastConnectionStatusEnum.fromValue(
                entry
                    .getLatestConnectionStatus()
                    .map(LatestConnectionStatus::toString)
                    .orElse("UNKNOWN")));
  }

  public AddressBookEntry addressBookEntry(com.radixdlt.p2p.addressbook.AddressBookEntry entry) {
    var addressBookEntry =
        new AddressBookEntry()
            .peerId(addressing.encodeNodeAddress(entry.getNodeId().getPublicKey()))
            .banned(entry.isBanned());
    entry.bannedUntil().map(Instant::toEpochMilli).ifPresent(addressBookEntry::bannedUntil);
    entry.getKnownAddresses().stream()
        .map(this::address)
        .forEach(addressBookEntry::addKnownAddressesItem);

    return addressBookEntry;
  }
}
