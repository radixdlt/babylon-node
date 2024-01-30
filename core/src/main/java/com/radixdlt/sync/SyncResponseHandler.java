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

import com.google.inject.Inject;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.ledger.*;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.statecomputer.commit.InvalidCommitRequestError;
import com.radixdlt.statecomputer.commit.LedgerProof;
import com.radixdlt.sync.SyncState.SyncingState;
import com.radixdlt.sync.messages.remote.*;
import com.radixdlt.sync.validation.RemoteSyncResponseSignaturesVerifier;
import com.radixdlt.sync.validation.RemoteSyncResponseValidatorSetVerifier;

/** An implementation delegate of {@link LocalSyncService}, for handling the sync responses. */
public final class SyncResponseHandler {

  private final RemoteSyncResponseValidatorSetVerifier validatorSetVerifier;
  private final RemoteSyncResponseSignaturesVerifier signaturesVerifier;
  private final EventDispatcher<LedgerExtension> syncedLedgerExtensionDispatcher;

  @Inject
  public SyncResponseHandler(
      RemoteSyncResponseValidatorSetVerifier validatorSetVerifier,
      RemoteSyncResponseSignaturesVerifier signaturesVerifier,
      EventDispatcher<LedgerExtension> syncedLedgerExtensionDispatcher) {
    this.validatorSetVerifier = validatorSetVerifier;
    this.signaturesVerifier = signaturesVerifier;
    this.syncedLedgerExtensionDispatcher = syncedLedgerExtensionDispatcher;
  }

  /**
   * Commits the transactions from the given sync response to the ledger. Throws a {@link
   * InvalidSyncResponseException} in case of any validation failure (including current syncing
   * state, ledger state, validator signatures, etc. - see the sealed exception subclasses for
   * details).
   */
  public void handle(SyncingState currentState, NodeId sender, SyncResponse syncResponse) {
    final var dto = syncResponse.getLedgerExtension();
    final var start =
        LedgerSyncDtoConversions.syncDtoToConsensusOriginatedLedgerProof(dto.getStart());
    final var end = LedgerSyncDtoConversions.syncDtoToConsensusOriginatedLedgerProof(dto.getEnd());
    final var ledgerExtension = LedgerExtension.create(dto.getTransactions(), end);
    this.checkMatchesPendingRequest(currentState, sender, start);
    this.checkValidTransactionCount(start, ledgerExtension);
    this.checkConsensusProof(end);
    this.commit(ledgerExtension);
  }

  private void checkMatchesPendingRequest(
      SyncingState currentState, NodeId sender, LedgerProof ledgerExtensionStart) {
    final var optPendingRequestPeer =
        currentState.getPendingRequest().map(SyncState.PendingRequest::getPeer);
    if (optPendingRequestPeer.isEmpty()) {
      throw new InvalidSyncResponseException.NoSyncRequestPending();
    }

    final var pendingRequestPeer = optPendingRequestPeer.get();
    if (!pendingRequestPeer.equals(sender)) {
      throw new InvalidSyncResponseException.SyncRequestSenderMismatch();
    }

    final var currentHeader = currentState.getLatestProof();
    if (ledgerExtensionStart.stateVersion() != currentHeader.stateVersion()) {
      throw new InvalidSyncResponseException.LedgerExtensionStartMismatch();
    }
  }

  private void checkValidTransactionCount(LedgerProof start, LedgerExtension ledgerExtension) {
    final var transactionCount = ledgerExtension.transactions().size();
    if (transactionCount == 0) {
      throw new InvalidSyncResponseException.EmptySyncResponse();
    }

    final var expectedEndStateVersion = start.stateVersion() + transactionCount;
    if (expectedEndStateVersion != ledgerExtension.proof().stateVersion()) {
      throw new InvalidSyncResponseException.InconsistentTransactionCount();
    }
  }

  private void checkConsensusProof(LedgerProof ledgerExtensionEnd) {

    if (!this.validatorSetVerifier.verifyValidatorSet(ledgerExtensionEnd)) {
      throw new InvalidSyncResponseException.NoQuorumInValidatorSet();
    }

    if (!this.signaturesVerifier.verifyResponseSignatures(ledgerExtensionEnd)) {
      throw new InvalidSyncResponseException.ValidatorSignatureMismatch();
    }
  }

  private void commit(LedgerExtension ledgerExtension) {
    try {
      this.syncedLedgerExtensionDispatcher.dispatch(ledgerExtension);
    } catch (InvalidCommitRequestException exception) {
      switch (exception.getError()) {
        case InvalidCommitRequestError.TransactionParsingFailed ignored -> {
          throw new InvalidSyncResponseException.UnparseableTransaction();
        }

        case InvalidCommitRequestError.TransactionRootMismatch ignored -> {
          throw new InvalidSyncResponseException.ComputedTransactionRootMismatch();
        }
      }
    }
  }
}
