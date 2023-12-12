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

package com.radixdlt.sync;

import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.Inject;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.environment.EventProcessor;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.RemoteEventProcessor;
import com.radixdlt.ledger.*;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.rev2.LastProof;
import com.radixdlt.sync.messages.remote.*;
import java.util.Collections;
import java.util.Objects;
import java.util.stream.Collectors;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Service which serves remote sync requests. */
public final class RemoteSyncService {
  private static final Logger log = LogManager.getLogger();

  private final PeersView peersView;
  private final LocalSyncService localSyncService; // TODO: consider removing this dependency
  private final TransactionsAndProofReader committedReader;
  private final RemoteEventDispatcher<NodeId, StatusResponse> statusResponseDispatcher;
  private final RemoteEventDispatcher<NodeId, SyncResponse> syncResponseDispatcher;
  private final RemoteEventDispatcher<NodeId, LedgerStatusUpdate> statusUpdateDispatcher;
  private final SyncRelayConfig syncRelayConfig;
  private final Metrics metrics;
  private final RateLimiter ledgerStatusUpdateSendRateLimiter;

  private LedgerProof currentHeader;

  @Inject
  public RemoteSyncService(
      PeersView peersView,
      LocalSyncService localSyncService,
      TransactionsAndProofReader committedReader,
      RemoteEventDispatcher<NodeId, StatusResponse> statusResponseDispatcher,
      RemoteEventDispatcher<NodeId, SyncResponse> syncResponseDispatcher,
      RemoteEventDispatcher<NodeId, LedgerStatusUpdate> statusUpdateDispatcher,
      SyncRelayConfig syncRelayConfig,
      Metrics metrics,
      @LastProof LedgerProof initialHeader) {
    this.peersView = Objects.requireNonNull(peersView);
    this.localSyncService = Objects.requireNonNull(localSyncService);
    this.committedReader = Objects.requireNonNull(committedReader);
    this.syncRelayConfig = Objects.requireNonNull(syncRelayConfig);
    this.statusResponseDispatcher = Objects.requireNonNull(statusResponseDispatcher);
    this.syncResponseDispatcher = Objects.requireNonNull(syncResponseDispatcher);
    this.statusUpdateDispatcher = Objects.requireNonNull(statusUpdateDispatcher);
    this.metrics = metrics;
    this.ledgerStatusUpdateSendRateLimiter =
        RateLimiter.create(syncRelayConfig.maxLedgerUpdatesRate());

    this.currentHeader = initialHeader;
  }

  public RemoteEventProcessor<NodeId, SyncRequest> syncRequestEventProcessor() {
    return this::processSyncRequest;
  }

  private void processSyncRequest(NodeId sender, SyncRequest syncRequest) {
    final var remoteCurrentHeader = syncRequest.getHeader();
    final var ledgerExtension = getLedgerExtensionForSyncRequest(remoteCurrentHeader);

    if (ledgerExtension == null) {
      log.trace(
          "REMOTE_SYNC_REQUEST: Unable to serve sync request {} from sender {}.",
          remoteCurrentHeader,
          sender);
      return;
    }

    final var verifiable =
        new DtoLedgerExtension(
            ledgerExtension.getTransactions(),
            remoteCurrentHeader,
            ledgerExtension.getProof().toDto());

    log.trace(
        "REMOTE_SYNC_REQUEST: Sending response {} to request {} from {}",
        verifiable,
        remoteCurrentHeader,
        sender);

    metrics.sync().remoteRequestsReceived().inc();
    syncResponseDispatcher.dispatch(sender, SyncResponse.create(verifiable));
  }

  private LedgerExtension getLedgerExtensionForSyncRequest(DtoLedgerProof startHeader) {
    return committedReader.getTransactions(startHeader);
  }

  public RemoteEventProcessor<NodeId, StatusRequest> statusRequestEventProcessor() {
    return this::processStatusRequest;
  }

  private void processStatusRequest(NodeId sender, StatusRequest statusRequest) {
    statusResponseDispatcher.dispatch(sender, StatusResponse.create(this.currentHeader));
  }

  public EventProcessor<LedgerUpdate> ledgerUpdateEventProcessor() {
    return this::processLedgerUpdate;
  }

  private void processLedgerUpdate(LedgerUpdate ledgerUpdate) {
    final LedgerProof updatedHeader = ledgerUpdate.proof();
    if (updatedHeader.getStateVersion() > this.currentHeader.getStateVersion()) {
      this.currentHeader = updatedHeader;
      this.sendStatusUpdateToSomePeers(updatedHeader);
    }
  }

  private void sendStatusUpdateToSomePeers(LedgerProof header) {
    if (!(this.localSyncService.getSyncState() instanceof SyncState.IdleState)) {
      return; // not sending any updates if the node is syncing itself
    }

    final var statusUpdate = LedgerStatusUpdate.create(header);

    final var currentPeers = this.peersView.peers().collect(Collectors.toList());
    Collections.shuffle(currentPeers);

    currentPeers.stream()
        .limit(syncRelayConfig.ledgerStatusUpdateMaxPeersToNotify())
        .map(PeersView.PeerInfo::getNodeId)
        .forEach(
            peer -> {
              if (this.ledgerStatusUpdateSendRateLimiter.tryAcquire()) {
                statusUpdateDispatcher.dispatch(peer, statusUpdate);
              }
            });
  }
}
