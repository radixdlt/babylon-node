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

import static com.radixdlt.utils.TypedMocks.rmock;
import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

import com.google.common.collect.ImmutableClassToInstanceMap;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.ScheduledEventDispatcher;
import com.radixdlt.ledger.*;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.PeerControl;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.p2p.PeersView.PeerInfo;
import com.radixdlt.p2p.capability.LedgerSyncCapability;
import com.radixdlt.sync.messages.local.SyncCheckReceiveStatusTimeout;
import com.radixdlt.sync.messages.local.SyncCheckTrigger;
import com.radixdlt.sync.messages.local.SyncLedgerUpdateTimeout;
import com.radixdlt.sync.messages.local.SyncRequestTimeout;
import com.radixdlt.sync.messages.remote.*;
import java.util.Arrays;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Stream;
import junitparams.JUnitParamsRunner;
import junitparams.Parameters;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;

@RunWith(JUnitParamsRunner.class)
public class LocalSyncServiceTest {
  private LocalSyncService localSyncService;
  private RemoteEventDispatcher<NodeId, StatusRequest> statusRequestDispatcher;
  private ScheduledEventDispatcher<SyncCheckReceiveStatusTimeout>
      syncCheckReceiveStatusTimeoutDispatcher;
  private RemoteEventDispatcher<NodeId, SyncRequest> syncRequestDispatcher;
  private ScheduledEventDispatcher<SyncRequestTimeout> syncRequestTimeoutDispatcher;
  private ScheduledEventDispatcher<SyncLedgerUpdateTimeout> syncLedgerUpdateTimeoutDispatcher;
  private SyncRelayConfig syncRelayConfig;
  private Metrics metrics;
  private PeersView peersView;
  private PeerControl peerControl;
  private SyncResponseHandler syncResponseHandler;

  @Before
  public void setUp() {
    this.statusRequestDispatcher = rmock(RemoteEventDispatcher.class);
    this.syncCheckReceiveStatusTimeoutDispatcher = rmock(ScheduledEventDispatcher.class);
    this.syncRequestDispatcher = rmock(RemoteEventDispatcher.class);
    this.syncRequestTimeoutDispatcher = rmock(ScheduledEventDispatcher.class);
    this.syncLedgerUpdateTimeoutDispatcher = rmock(ScheduledEventDispatcher.class);
    this.syncRelayConfig = SyncRelayConfig.of(1000L, 10, 10000L);
    this.metrics = new MetricsInitializer().initialize();
    this.peersView = mock(PeersView.class);
    this.peerControl = mock(PeerControl.class);
    this.syncResponseHandler = mock(SyncResponseHandler.class);
  }

  private void setupSyncServiceWithState(SyncState syncState) {
    this.localSyncService =
        new LocalSyncService(
            statusRequestDispatcher,
            syncCheckReceiveStatusTimeoutDispatcher,
            syncRequestDispatcher,
            syncRequestTimeoutDispatcher,
            syncLedgerUpdateTimeoutDispatcher,
            syncRelayConfig,
            metrics,
            peersView,
            peerControl,
            syncResponseHandler,
            syncState);
  }

  @Test
  public void when_sync_check_is_triggered_at_idle__then_should_ask_peers_for_their_statuses() {
    final var peer1 = createPeer();
    final var peer2 = createPeer();
    final var peer3 = createPeer();

    setupPeersView(peer1, peer2, peer3);

    final LedgerProof currentHeader = mock(LedgerProof.class);
    this.setupSyncServiceWithState(SyncState.IdleState.init(currentHeader));

    this.localSyncService.syncCheckTriggerEventProcessor().process(SyncCheckTrigger.create());

    verify(statusRequestDispatcher, times(1)).dispatch(eq(peer1), any());
    verify(statusRequestDispatcher, times(1)).dispatch(eq(peer2), any());
    verify(statusRequestDispatcher, times(1)).dispatch(eq(peer2), any());
  }

  @Test
  public void when_sync_check_is_triggered_at_non_idle__then_should_be_ignored() {
    final LedgerProof currentHeader = mock(LedgerProof.class);

    this.setupSyncServiceWithState(SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of()));
    this.localSyncService.syncCheckTriggerEventProcessor().process(SyncCheckTrigger.create());

    this.setupSyncServiceWithState(
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(), currentHeader));
    this.localSyncService.syncCheckTriggerEventProcessor().process(SyncCheckTrigger.create());

    verifyNoMoreInteractions(peersView);
    verifyNoMoreInteractions(statusRequestDispatcher);
  }

  @Test
  public void when_status_response_received_at_non_sync_check__then_should_be_ignored() {
    final LedgerProof currentHeader = mock(LedgerProof.class);
    final LedgerProof statusHeader = mock(LedgerProof.class);
    final NodeId sender = createPeer();

    this.setupSyncServiceWithState(
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(), currentHeader));
    this.localSyncService
        .statusResponseEventProcessor()
        .process(sender, StatusResponse.create(statusHeader));

    verifyNoMoreInteractions(peersView);
    verifyNoMoreInteractions(peerControl);
    verifyNoMoreInteractions(statusRequestDispatcher);
    verifyNoMoreInteractions(syncRequestDispatcher);
    verifyNoMoreInteractions(syncRequestTimeoutDispatcher);
  }

  @Test
  public void when_unexpected_status_response_received__then_should_be_ignored() {
    final LedgerProof currentHeader = mock(LedgerProof.class);
    final LedgerProof statusHeader = mock(LedgerProof.class);
    final NodeId expectedPeer = createPeer();
    final NodeId unexpectedPeer = createPeer();

    this.setupSyncServiceWithState(
        SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of(expectedPeer)));
    this.localSyncService
        .statusResponseEventProcessor()
        .process(unexpectedPeer, StatusResponse.create(statusHeader));

    verifyNoMoreInteractions(peersView);
    verifyNoMoreInteractions(peerControl);
    verifyNoMoreInteractions(statusRequestDispatcher);
    verifyNoMoreInteractions(syncRequestDispatcher);
    verifyNoMoreInteractions(syncRequestTimeoutDispatcher);
  }

  @Test
  public void when_duplicate_status_response_received__then_should_be_ignored() {
    final LedgerProof currentHeader = mock(LedgerProof.class);
    final LedgerProof statusHeader = mock(LedgerProof.class);
    final NodeId expectedPeer = createPeer();
    final NodeId alreadyReceivedPeer = createPeer();

    final var syncState =
        SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of(expectedPeer))
            .withStatusResponse(alreadyReceivedPeer, StatusResponse.create(statusHeader));

    this.setupSyncServiceWithState(syncState);
    this.localSyncService
        .statusResponseEventProcessor()
        .process(alreadyReceivedPeer, StatusResponse.create(statusHeader));

    verifyNoMoreInteractions(peersView);
    verifyNoMoreInteractions(peerControl);
    verifyNoMoreInteractions(statusRequestDispatcher);
    verifyNoMoreInteractions(syncRequestDispatcher);
    verifyNoMoreInteractions(syncRequestTimeoutDispatcher);
  }

  @Test
  public void when_all_status_responses_received__then_should_start_sync() {
    final LedgerProof currentHeader = createHeaderAtStateVersion(10L);
    final LedgerProof statusHeader1 = createHeaderAtStateVersion(2L);
    final LedgerProof statusHeader2 = createHeaderAtStateVersion(20L);
    final LedgerProof statusHeader3 = createHeaderAtStateVersion(15L);
    final NodeId waiting1 = createPeer();
    final NodeId waiting2 = createPeer();
    final NodeId waiting3 = createPeer();

    final var syncState =
        SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of(waiting1, waiting2, waiting3));
    this.setupSyncServiceWithState(syncState);

    setupPeersView(waiting2);

    this.localSyncService
        .statusResponseEventProcessor()
        .process(waiting1, StatusResponse.create(statusHeader1));
    this.localSyncService
        .statusResponseEventProcessor()
        .process(waiting2, StatusResponse.create(statusHeader2));
    this.localSyncService
        .statusResponseEventProcessor()
        .process(waiting3, StatusResponse.create(statusHeader3));

    verify(syncRequestDispatcher, times(1)).dispatch(eq(waiting2), any());
  }

  @Test
  public void when_status_timeout_with_no_responses__then_should_reschedule_another_check() {
    final LedgerProof currentHeader = createHeaderAtStateVersion(10L);
    final NodeId waiting1 = createPeer();
    setupPeersView(waiting1);

    final var syncState = SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of(waiting1));
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .syncCheckReceiveStatusTimeoutEventProcessor()
        .process(SyncCheckReceiveStatusTimeout.create());

    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  @Test
  public void when_status_timeout_with_at_least_one_response__then_should_start_sync() {
    final LedgerProof currentHeader = createHeaderAtStateVersion(10L);
    final LedgerProof statusHeader1 = createHeaderAtStateVersion(12L);
    final LedgerProof statusHeader2 = createHeaderAtStateVersion(20L);
    final var waiting1 = createPeer();
    final var waiting2 = createPeer();
    setupPeersView(waiting1, waiting2);

    final var syncState =
        SyncState.SyncCheckState.init(currentHeader, ImmutableSet.of(waiting1, waiting2));
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .statusResponseEventProcessor()
        .process(waiting1, StatusResponse.create(statusHeader1));

    this.localSyncService
        .syncCheckReceiveStatusTimeoutEventProcessor()
        .process(SyncCheckReceiveStatusTimeout.create());

    // even though statusHeader2 is more up to date, it should be ignored because was received
    // after a timeout event
    this.localSyncService
        .statusResponseEventProcessor()
        .process(waiting2, StatusResponse.create(statusHeader2));

    verify(syncRequestDispatcher, times(1)).dispatch(eq(waiting1), any());
  }

  @Test
  public void when_syncing_timeout__then_should_remove_candidate_and_retry_with_other_candidate() {
    final var currentHeader = createHeaderAtStateVersion(10L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    final var peer2 = createPeer();
    setupPeersView(peer1, peer2);

    final var requestId = 1L;
    final var originalCandidates = ImmutableList.of(peer1, peer2);
    final var syncState =
        SyncState.SyncingState.init(currentHeader, originalCandidates, targetHeader)
            .withPendingRequest(peer1, requestId);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .syncRequestTimeoutEventProcessor()
        .process(SyncRequestTimeout.create(peer1, requestId));

    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer2), any());
  }

  @Test
  public void when_syncing_timeout_for_different_peer_same_request_id__then_should_ignore() {
    final var currentHeader = createHeaderAtStateVersion(10L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    final var peer2 = createPeer();
    setupPeersView(peer1, peer2);

    final var requestId = 1L;
    final var originalCandidates = ImmutableList.of(peer1, peer2);
    final var syncState =
        SyncState.SyncingState.init(currentHeader, originalCandidates, targetHeader)
            .withPendingRequest(peer1, requestId);
    this.setupSyncServiceWithState(syncState);

    // waiting for response from peer1, but got a timeout for peer2
    this.localSyncService
        .syncRequestTimeoutEventProcessor()
        .process(SyncRequestTimeout.create(peer2, requestId));

    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  @Test
  public void when_syncing_timeout_for_same_peer_different_request_id__then_should_ignore() {
    final var currentHeader = createHeaderAtStateVersion(10L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = mock(NodeId.class);
    final var peer2 = mock(NodeId.class);
    when(peersView.peers()).thenAnswer(i -> Stream.of(peer1, peer2));

    final var originalCandidates = ImmutableList.of(peer1, peer2);
    final var syncState =
        SyncState.SyncingState.init(currentHeader, originalCandidates, targetHeader)
            .withPendingRequest(peer1, 2L);
    this.setupSyncServiceWithState(syncState);

    // waiting for response for request id 2, but got a timeout for 1
    this.localSyncService
        .syncRequestTimeoutEventProcessor()
        .process(SyncRequestTimeout.create(peer1, 1L));

    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  @Test
  public void when_received_a_valid_response__then_should_schedule_next() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    this.setupSyncServiceWithState(syncState);

    final var syncResponse = mock(SyncResponse.class);

    this.localSyncService.syncResponseEventProcessor().process(peer1, syncResponse);

    verify(syncResponseHandler, times(1)).handle(syncState, peer1, syncResponse);
    assertEquals(syncState.clearPendingRequest(), this.localSyncService.getSyncState());
    verify(peerControl, times(1)).reportHighPriorityPeer(peer1);
    verify(syncLedgerUpdateTimeoutDispatcher, times(1)).dispatch(any(), anyLong());
    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  @Parameters(method = "unsolicitedSyncResponseExceptions")
  @Test
  public void when_received_an_unsolicited_response__then_should_ignore(
      InvalidSyncResponseException unsolicitedSyncResponseException) {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    final var syncResponse = mock(SyncResponse.class);

    doThrow(unsolicitedSyncResponseException)
        .when(syncResponseHandler)
        .handle(syncState, peer1, syncResponse);

    this.setupSyncServiceWithState(syncState);

    this.localSyncService.syncResponseEventProcessor().process(peer1, syncResponse);

    verify(syncResponseHandler, times(1)).handle(syncState, peer1, syncResponse);
    assertEquals(syncState, this.localSyncService.getSyncState());
    verifyNoMoreInteractions(peerControl);
    verifyNoMoreInteractions(syncLedgerUpdateTimeoutDispatcher);
    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  public Object[] unsolicitedSyncResponseExceptions() {
    return new Object[] {
      new InvalidSyncResponseException.NoSyncRequestPending(),
      new InvalidSyncResponseException.SyncRequestSenderMismatch(),
      new InvalidSyncResponseException.LedgerExtensionStartMismatch(),
    };
  }

  @Parameters(method = "potentiallyMaliciousSyncResponseExceptions")
  @Test
  public void when_received_a_potentially_malicious_response__then_should_penalize_sender(
      InvalidSyncResponseException potentiallyMaliciousSyncResponseException) {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    final var syncResponse = mock(SyncResponse.class);

    doThrow(potentiallyMaliciousSyncResponseException)
        .when(syncResponseHandler)
        .handle(syncState, peer1, syncResponse);

    this.setupSyncServiceWithState(syncState);

    this.localSyncService.syncResponseEventProcessor().process(peer1, syncResponse);

    verify(syncResponseHandler, times(1)).handle(syncState, peer1, syncResponse);
    verify(peerControl, times(1)).banPeer(eq(peer1), any(), any());
  }

  public Object[] potentiallyMaliciousSyncResponseExceptions() {
    return new Object[] {
      new InvalidSyncResponseException.EmptySyncResponse(),
      new InvalidSyncResponseException.InconsistentTransactionCount(),
      new InvalidSyncResponseException.UnparseableTransaction(),
      new InvalidSyncResponseException.ComputedTransactionRootMismatch(),
      new InvalidSyncResponseException.NoQuorumInValidatorSet(),
      new InvalidSyncResponseException.ValidatorSignatureMismatch(),
    };
  }

  @Test
  public void
      when_received_ledger_update_and_fully_synced__then_should_wait_for_another_sync_trigger() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .ledgerUpdateEventProcessor()
        .process(ledgerUpdateAtStateVersion(targetHeader.getStateVersion()));

    verifyNoMoreInteractions(syncRequestDispatcher);
  }

  @Test
  public void when_ledger_update_timeout__then_should_continue_sync() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .syncLedgerUpdateTimeoutProcessor()
        .process(SyncLedgerUpdateTimeout.create(currentHeader.getStateVersion()));

    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer1), any());
  }

  @Test
  public void when_obsolete_ledger_update_timeout__then_should_ignore() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .syncLedgerUpdateTimeoutProcessor()
        .process(
            SyncLedgerUpdateTimeout.create(
                currentHeader.getStateVersion() - 1) // timeout event for a past state version
            );

    verifyNoInteractions(syncRequestDispatcher);
  }

  @Test
  public void when_remote_status_update_in_idle__then_should_start_sync() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);

    final var peer1 = createPeer();
    setupPeersView(peer1);

    final var syncState = SyncState.IdleState.init(currentHeader);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .ledgerStatusUpdateEventProcessor()
        .process(peer1, LedgerStatusUpdate.create(targetHeader));

    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer1), any());
  }

  @Test
  public void when_remote_status_update_in_syncing__then_should_update_target() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);
    final var newTargetHeader = createHeaderAtStateVersion(22L);

    final var peer1 = createPeer();
    final var peer2 = createPeer();
    setupPeersView(peer1, peer2);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .ledgerStatusUpdateEventProcessor()
        .process(peer2, LedgerStatusUpdate.create(newTargetHeader));

    assertEquals(
        newTargetHeader,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).getTargetHeader());
  }

  @Test
  public void when_remote_status_update_in_syncing_for_older_header__then_should_do_nothing() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);
    final var newTargetHeader = createHeaderAtStateVersion(20L);

    final var peer1 = createPeer();
    final var peer2 = createPeer();
    setupPeersView(peer1, peer2);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1), targetHeader)
            .withPendingRequest(peer1, 1L);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .ledgerStatusUpdateEventProcessor()
        .process(peer2, LedgerStatusUpdate.create(newTargetHeader));

    assertEquals(syncState, this.localSyncService.getSyncState());
  }

  @Test
  public void when_ledger_status_update__then_should_not_add_duplicate_candidate() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(21L);
    final var newTargetHeader = createHeaderAtStateVersion(22L);
    final var evenNewerTargetHeader = createHeaderAtStateVersion(23L);

    final var peer1 = mock(NodeId.class);
    final var peer2 = mock(NodeId.class);
    final var peer3 = mock(NodeId.class);
    setupPeersView(peer1, peer2, peer3);

    final var syncState =
        SyncState.SyncingState.init(currentHeader, ImmutableList.of(peer1, peer2), targetHeader)
            .withPendingRequest(peer1, 1L);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService
        .ledgerStatusUpdateEventProcessor()
        .process(peer3, LedgerStatusUpdate.create(newTargetHeader));

    // another, newer, ledger update from the same peer
    this.localSyncService
        .ledgerStatusUpdateEventProcessor()
        .process(peer3, LedgerStatusUpdate.create(evenNewerTargetHeader));

    assertEquals(
        peer3,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(0).get());
    assertEquals(
        peer1,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(1).get());
    assertEquals(
        peer2,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(2).get());
    assertEquals(
        peer3,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(3).get());
    assertEquals(
        peer1,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(4).get());
    assertEquals(
        peer2,
        ((SyncState.SyncingState) this.localSyncService.getSyncState()).peekNthCandidate(5).get());
  }

  @Test
  public void when_syncing__then_should_use_round_robin_peers() {
    final var currentHeader = createHeaderAtStateVersion(19L);
    final var targetHeader = createHeaderAtStateVersion(30L);

    final var peer1 = createPeer();
    final var peer2 = createPeer();
    final var peer3 = createPeer();
    setupPeersView(peer1, peer2, peer3);

    final var syncState =
        SyncState.SyncingState.init(
            currentHeader, ImmutableList.of(peer1, peer2, peer3), targetHeader);
    this.setupSyncServiceWithState(syncState);

    this.localSyncService.ledgerUpdateEventProcessor().process(ledgerUpdateAtStateVersion(20L));
    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer1), any());
    this.localSyncService.syncResponseEventProcessor().process(peer1, mock(SyncResponse.class));
    this.localSyncService.ledgerUpdateEventProcessor().process(ledgerUpdateAtStateVersion(21L));
    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer2), any());
    this.localSyncService.syncResponseEventProcessor().process(peer2, mock(SyncResponse.class));
    this.localSyncService.ledgerUpdateEventProcessor().process(ledgerUpdateAtStateVersion(22L));
    verify(syncRequestDispatcher, times(1)).dispatch(eq(peer3), any());
    this.localSyncService.syncResponseEventProcessor().process(peer3, mock(SyncResponse.class));
    this.localSyncService.ledgerUpdateEventProcessor().process(ledgerUpdateAtStateVersion(23L));
    verify(syncRequestDispatcher, times(2)).dispatch(eq(peer1), any());
  }

  private LedgerUpdate ledgerUpdateAtStateVersion(long stateVersion) {
    return new LedgerUpdate(
        LedgerExtension.create(ImmutableList.of(), createHeaderAtStateVersion(stateVersion)),
        ImmutableClassToInstanceMap.of());
  }

  private LedgerProof createHeaderAtStateVersion(long version) {
    final LedgerProof header = mock(LedgerProof.class);
    when(header.getStateVersion()).thenReturn(version);
    when(header.getProposerTimestamp()).thenReturn(version * 1000);
    return header;
  }

  private void setupPeersView(NodeId... nodes) {
    var peerChannelInfo =
        PeersView.PeerChannelInfo.create(
            Optional.empty(),
            "",
            0,
            true,
            Set.of(LedgerSyncCapability.Builder.asDefault().build().toRemotePeerCapability()));
    var channels = ImmutableList.of(peerChannelInfo);
    var peerInfoStream = Stream.of(nodes).map(it -> PeerInfo.create(it, channels));
    when(peersView.peers()).thenReturn(peerInfoStream);
    Arrays.stream(nodes).forEach(peer -> when(peersView.hasPeer(peer)).thenReturn(true));
  }

  private NodeId createPeer() {
    return NodeId.fromPublicKey(ECKeyPair.generateNew().getPublicKey());
  }
}
