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

package com.radixdlt.monitoring;

import com.google.common.base.Preconditions;
import io.prometheus.client.Counter;
import io.prometheus.client.Gauge;
import io.prometheus.client.Summary;
import javax.annotation.Nullable;

/**
 * An entry point to metrics tracked by the Java part of the Node application.
 *
 * <p>The record hierarchy represents different sub-systems and services, and the leafs are:
 *
 * <ul>
 *   <li>{@link Counter}: a Prometheus-native up/down counter.
 *   <li>{@link Gauge}: a Prometheus-native indicator of an arbitrarily changing value.
 *   <li>{@link Summary}: a Prometheus-native [occurrence count + value sum] pair, by convention
 *       appropriate for representing a timer which tracks an average latency of an operation (note:
 *       we don't use Prometheus' "quantiles" support within the {@link Summary} yet).
 *   <li>{@link LabelledCounter}: our type-safe wrapper for a {@link Counter} with labels (i.e. to
 *       be used instead of Prometheus-native typo-prone {@link Counter#labels(String...)}.
 *   <li>{@link LabelledGauge}: our type-safe wrapper for a {@link Gauge} with labels (i.e. to be
 *       used instead of Prometheus-native typo-prone {@link Gauge#labels(String...)}.
 * </ul>
 *
 * Apart from that, this class also holds record definitions for the aforementioned type-safe labels
 * (see at the end).
 */
public record Metrics(
    Bft bft,
    Bdb bdb,
    Ledger ledger,
    LedgerSync sync,
    V1Mempool mempool,
    V1RadixEngine radixEngine,
    Messages messages,
    Networking networking,
    Crypto crypto,
    Misc misc) {

  public record Bft(
      Counter successfullyProcessedVotes,
      Counter successfullyProcessedProposals,
      Counter preconditionViolations,
      Counter proposalsReceivedFromNonLeaders,
      Counter duplicateProposalsReceived,
      Counter eventsReceived,
      Counter committedVertices,
      Counter noVotesSent,
      Counter voteQuorums,
      Counter timeoutQuorums,
      LabelledCounter<RejectedConsensusEvent> rejectedConsensusEvents,
      Pacemaker pacemaker,
      Sync sync,
      VertexStore vertexStore) {

    public record Pacemaker(
        Counter timeoutsSent,
        Gauge round,
        Counter proposedTransactions,
        Counter proposalsSent,
        Counter timedOutRounds,
        Counter proposalsWithSubstituteTimestamp) {}

    public record Sync(Counter requestsSent, Counter requestsReceived, Counter requestTimeouts) {}

    public record VertexStore(
        Gauge size, Counter forks, Counter rebuilds, Counter indirectParents) {}
  }

  public record Bdb(V1Ledger ledger, AddressBook addressBook, SafetyState safetyState) {

    public record V1Ledger(
        Counter commits,
        Summary transactionCreate,
        Summary read,
        Summary store,
        Summary lastCommittedRead,
        Summary lastVertexRead,
        Summary save,
        Summary interact,
        Counter bytesRead,
        Counter bytesWritten,
        Counter proofsAdded,
        Counter proofsRemoved,
        Counter headerBytesWritten) {}

    public record AddressBook(
        Summary interact, Counter bytesRead, Counter bytesWritten, Counter entriesDeleted) {}

    public record SafetyState(Summary commitState, Counter bytesRead, Counter bytesWritten) {}
  }

  public record Ledger(
      Gauge stateVersion, Counter syncTransactionsProcessed, Counter bftTransactionsProcessed) {}

  public record LedgerSync(
      Counter invalidResponsesReceived,
      Counter validResponsesReceived,
      Counter remoteRequestsReceived,
      Gauge currentStateVersion,
      Gauge targetStateVersion) {}

  public record V1Mempool(
      Gauge size, Counter relaysSent, Counter addSuccesses, Counter addFailures) {}

  public record V1RadixEngine(
      Counter invalidProposedTransactions, Counter userTransactions, Counter systemTransactions) {}

  public record Messages(Inbound inbound, Outbound outbound) {

    public record Inbound(
        Summary queueWait, Summary process, Counter received, Counter discarded) {}

    public record Outbound(Counter aborted, Gauge queued, Counter processed, Counter sent) {}
  }

  public record Networking(
      Counter messagesDropped,
      Counter bytesSent,
      Counter bytesReceived,
      LabelledGauge<ChannelProperties> activeChannels,
      Counter channelsInitialized) {}

  public record Crypto(Counter bytesHashed, Counter signaturesSigned, Counter signaturesVerified) {}

  public record Misc(
      Summary applicationStart,
      Counter epochManagerEnqueuedConsensusEvents,
      Counter vertexStoreSaved) {}

  public record RejectedConsensusEvent(
      Type type, Cause cause, @Nullable TimestampIssue timestampIssue) {

    public RejectedConsensusEvent(Type type, Cause cause) {
      this(type, cause, null);
      Preconditions.checkArgument(cause != Cause.TIMESTAMP);
    }

    public RejectedConsensusEvent(Type type, TimestampIssue timestampIssue) {
      this(type, Cause.TIMESTAMP, timestampIssue);
    }

    public enum Type {
      VOTE,
      PROPOSAL
    }

    public enum Cause {
      AUTHORS,
      SIGNATURES,
      QCS,
      TIMEOUT_SIGNATURES,
      TIMESTAMP
    }

    public enum TimestampIssue {
      TOO_OLD,
      TOO_YOUNG,
      NOT_MONOTONIC
    }
  }

  public record ChannelProperties(Direction direction) {

    public enum Direction {
      INBOUND,
      OUTBOUND;
    }
  }
}
