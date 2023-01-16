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

package com.radixdlt.messaging;

import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Scopes;
import com.google.inject.Singleton;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.sync.GetVerticesErrorResponse;
import com.radixdlt.consensus.sync.GetVerticesRequest;
import com.radixdlt.consensus.sync.GetVerticesResponse;
import com.radixdlt.environment.MessageTransportType;
import com.radixdlt.environment.rx.RemoteEvent;
import com.radixdlt.environment.rx.RxRemoteDispatcher;
import com.radixdlt.environment.rx.RxRemoteEnvironment;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.messaging.consensus.MessageCentralBFTNetwork;
import com.radixdlt.messaging.consensus.MessageCentralValidatorSync;
import com.radixdlt.messaging.core.GetVerticesRequestRateLimit;
import com.radixdlt.messaging.core.MessageCentralModule;
import com.radixdlt.messaging.ledgersync.MessageCentralLedgerSync;
import com.radixdlt.messaging.mempool.MessageCentralMempool;
import com.radixdlt.messaging.p2p.MessageCentralPeerDiscovery;
import com.radixdlt.messaging.p2p.MessageCentralPeerLiveness;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.discovery.GetPeers;
import com.radixdlt.p2p.discovery.PeersResponse;
import com.radixdlt.p2p.liveness.Ping;
import com.radixdlt.p2p.liveness.Pong;
import com.radixdlt.sync.messages.remote.LedgerStatusUpdate;
import com.radixdlt.sync.messages.remote.StatusRequest;
import com.radixdlt.sync.messages.remote.StatusResponse;
import com.radixdlt.sync.messages.remote.SyncRequest;
import com.radixdlt.sync.messages.remote.SyncResponse;
import com.radixdlt.utils.properties.RuntimeProperties;
import io.reactivex.rxjava3.core.Flowable;

public final class MessagingModule extends AbstractModule {

  private RuntimeProperties properties;

  public MessagingModule(RuntimeProperties properties) {
    this.properties = properties;
  }

  @Override
  protected void configure() {
    // provides (for SharedMempool)
    bind(MessageCentralMempool.class).in(Scopes.SINGLETON);

    // Network BFT/Epoch Sync messages
    // TODO: make rate limits configurable
    bind(RateLimiter.class)
        .annotatedWith(GetVerticesRequestRateLimit.class)
        .toInstance(RateLimiter.create(50.0));
    bind(MessageCentralValidatorSync.class).in(Scopes.SINGLETON);

    // Network BFT messages
    bind(MessageCentralBFTNetwork.class).in(Scopes.SINGLETON);

    install(new MessageCentralModule(this.properties));
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> mempoolAddDispatcher(
      MessageCentralMempool messageCentralMempool) {
    return RxRemoteDispatcher.create(
        NodeId.class, MempoolAdd.class, messageCentralMempool.mempoolAddRemoteEventDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> proposalDispatcher(MessageCentralBFTNetwork bftNetwork) {
    return RxRemoteDispatcher.create(
        BFTNode.class, Proposal.class, bftNetwork.proposalDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> voteDispatcher(MessageCentralBFTNetwork bftNetwork) {
    return RxRemoteDispatcher.create(BFTNode.class, Vote.class, bftNetwork.voteDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> vertexRequestDispatcher(
      MessageCentralValidatorSync messageCentralValidatorSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class,
        GetVerticesRequest.class,
        messageCentralValidatorSync.verticesRequestDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> vertexResponseDispatcher(
      MessageCentralValidatorSync messageCentralValidatorSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class,
        GetVerticesResponse.class,
        messageCentralValidatorSync.verticesResponseDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> vertexErrorResponseDispatcher(
      MessageCentralValidatorSync messageCentralValidatorSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class,
        GetVerticesErrorResponse.class,
        messageCentralValidatorSync.verticesErrorResponseDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> syncRequestDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class, SyncRequest.class, messageCentralLedgerSync.syncRequestDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> syncResponseDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class, SyncResponse.class, messageCentralLedgerSync.syncResponseDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> statusRequestDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class, StatusRequest.class, messageCentralLedgerSync.statusRequestDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> statusResponseDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class, StatusResponse.class, messageCentralLedgerSync.statusResponseDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> pingDispatcher(
      MessageCentralPeerLiveness messageCentralPeerLiveness) {
    return RxRemoteDispatcher.create(
        NodeId.class, Ping.class, messageCentralPeerLiveness.pingDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> pongDispatcher(
      MessageCentralPeerLiveness messageCentralPeerLiveness) {
    return RxRemoteDispatcher.create(
        NodeId.class, Pong.class, messageCentralPeerLiveness.pongDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> getPeersDispatcher(
      MessageCentralPeerDiscovery messageCentralPeerDiscovery) {
    return RxRemoteDispatcher.create(
        NodeId.class, GetPeers.class, messageCentralPeerDiscovery.getPeersDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> peersResponseDispatcher(
      MessageCentralPeerDiscovery messageCentralPeerDiscovery) {
    return RxRemoteDispatcher.create(
        NodeId.class, PeersResponse.class, messageCentralPeerDiscovery.peersResponseDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> ledgerStatusUpdateDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        NodeId.class,
        LedgerStatusUpdate.class,
        messageCentralLedgerSync.ledgerStatusUpdateDispatcher());
  }

  @ProvidesIntoSet
  private RxRemoteDispatcher<?, ?> bftLedgerStatusUpdateDispatcher(
      MessageCentralLedgerSync messageCentralLedgerSync) {
    return RxRemoteDispatcher.create(
        BFTNode.class,
        LedgerStatusUpdate.class,
        (n, m) -> {
          var nodeId = NodeId.fromPublicKey(n.getKey());
          messageCentralLedgerSync.ledgerStatusUpdateDispatcher().dispatch(nodeId, m);
        });
  }

  // TODO: Clean this up
  @Provides
  @Singleton
  @SuppressWarnings("unchecked")
  RxRemoteEnvironment rxRemoteEnvironment(
      MessageCentralMempool messageCentralMempool,
      MessageCentralLedgerSync messageCentralLedgerSync,
      MessageCentralBFTNetwork messageCentralBFT,
      MessageCentralValidatorSync messageCentralBFTSync,
      MessageCentralPeerLiveness messageCentralPeerLiveness,
      MessageCentralPeerDiscovery messageCentralPeerDiscovery) {
    return new RxRemoteEnvironment() {
      @Override
      public <N, T> Flowable<RemoteEvent<N, T>> remoteEvents(
          MessageTransportType<N, T> messageTransportType) {
        if (messageTransportType.equals(MessageTransportType.create(BFTNode.class, Vote.class))) {
          return messageCentralBFT.remoteVotes().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(BFTNode.class, Proposal.class))) {
          return messageCentralBFT.remoteProposals().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(BFTNode.class, GetVerticesRequest.class))) {
          return messageCentralBFTSync.requests().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(BFTNode.class, GetVerticesResponse.class))) {
          return messageCentralBFTSync.responses().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(BFTNode.class, GetVerticesErrorResponse.class))) {
          return messageCentralBFTSync.errorResponses().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, MempoolAdd.class))) {
          return messageCentralMempool.mempoolComands().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, SyncRequest.class))) {
          return messageCentralLedgerSync.syncRequests().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, SyncResponse.class))) {
          return messageCentralLedgerSync.syncResponses().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, StatusRequest.class))) {
          return messageCentralLedgerSync.statusRequests().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, StatusResponse.class))) {
          return messageCentralLedgerSync.statusResponses().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, LedgerStatusUpdate.class))) {
          return messageCentralLedgerSync.ledgerStatusUpdates().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, Ping.class))) {
          return messageCentralPeerLiveness.pings().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, Pong.class))) {
          return messageCentralPeerLiveness.pongs().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, GetPeers.class))) {
          return messageCentralPeerDiscovery.getPeersEvents().map(m -> (RemoteEvent<N, T>) m);
        } else if (messageTransportType.equals(
            MessageTransportType.create(NodeId.class, PeersResponse.class))) {
          return messageCentralPeerDiscovery.peersResponses().map(m -> (RemoteEvent<N, T>) m);
        } else {
          throw new MessageTransportNotSupported(messageTransportType);
        }
      }
    };
  }
}
