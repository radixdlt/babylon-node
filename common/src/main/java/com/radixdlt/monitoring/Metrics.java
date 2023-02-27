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
import com.google.common.base.Predicates;
import com.google.common.collect.MoreCollectors;
import io.prometheus.client.*;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.function.DoubleSupplier;
import javax.annotation.Nullable;

/**
 * An entry point to metrics tracked by the Java part of the Node application.
 *
 * <p>We use the official <a href="https://github.com/prometheus/client_java">Prometheus client</a>,
 * directly referencing its measurement primitives and our light wrappers (i.e. we don't hide
 * Prometheus' presence from our business logic).
 *
 * <p>The record hierarchy represents different sub-systems and services, and the leafs are:
 *
 * <ul>
 *   <li>{@link Counter}: a Prometheus-native up/down counter. Its name should be a plural noun
 *       describing the things being counted (e.g. "parsingErrors").
 *   <li>{@link Gauge}: a Prometheus-native indicator of an arbitrarily changing value. Its name
 *       should be a noun describing the value - may be singular (e.g. "versionNumber") or plural
 *       (e.g. "activeClients").
 *   <li>{@link GetterGauge}: our getter-based counterpart of the {@link Gauge}, for which a direct
 *       "sample provider" needs to be {@link GetterGauge#initialize(DoubleSupplier) initialized}.
 *       <b>Note:</b> such approach goes against Prometheus' conventions, and we only use it in
 *       legacy cases (which could not be easily migrated to regular "{@link Gauge} update" style).
 *   <li>{@link Timer}: our duration-specific wrapper for a {@link Summary} without quantiles. It
 *       effectively represents a pair of [occurrence count, cumulative elapsed seconds],
 *       appropriate for tracking an average latency of an operation. Its name should be a verb
 *       describing the performed operation (e.g. "saveState").
 *   <li>{@link LabelledCounter}: our type-safe wrapper for a {@link Counter} with labels (i.e. to
 *       be used instead of Prometheus-native typo-prone {@link Counter#labels(String...)}.
 *   <li>{@link LabelledGauge}: our type-safe wrapper for a {@link Gauge} with labels (i.e. to be
 *       used instead of Prometheus-native typo-prone {@link Gauge#labels(String...)}.
 *   <li>{@link LabelledTimer}: type-safe labels support for our {@link Timer}.
 *   <li>{@link TypedInfo}: our type-safe wrapper for a special information-only metric, to be
 *       always used instead of the raw Prometheus-native {@link Info}.
 * </ul>
 *
 * <p>Any leaf may be annotated as {@link NotExposed}, if we wish to keep it around, but exclude it
 * from the exposition endpoint - please see the annotation's javadoc for motivation.
 *
 * <p>The type-safe labels mentioned above are implemented using {@link Record}s - we {@link
 * NameRenderer#labelNames(Class) render the record component's name as label name} (converting it
 * {@link NameRenderer#render(String) to Prometheus' format} on the way), and we use the plain
 * {@link Object#toString()} for the label's value.
 *
 * <p>For a complete example, suppose that we have a metric field definition:
 *
 * <pre>
 *   public record Metrics(...) {
 *
 *     public record Crypto(
 *        LabelledCounter<SignDetails> signErrors
 *     ) {}
 *
 *     ...
 *
 *     public record SignDetails(String signedBy, SignType type) {}
 *   }
 * </pre>
 *
 * If we bump it like:
 *
 * <pre>
 *   Metrics.crypto().signErrors().label(new SignDetails("Verifier 1", SignType.SHA_256)).inc(3);
 * </pre>
 *
 * Then the exposition endpoint will return:
 *
 * <pre>
 *   rn_crypto_sign_errors{signed_by="Verifier 1",type="SHA_256",} 3.0
 * </pre>
 *
 * (the "rn_" is a prefix identifying our application, just for convenience)
 *
 * <p>The concrete record label definitions for our metrics are defined at the end of this class.
 */
public record Metrics(
    Bft bft,
    BerkeleyDb berkeleyDb,
    Ledger ledger,
    LedgerSync sync,
    Mempool mempool,
    V1Mempool v1Mempool,
    V1RadixEngine v1RadixEngine,
    Messages messages,
    Networking networking,
    Crypto crypto,
    EpochManager epochManager,
    StateManager stateManager,
    Misc misc) {

  public record Bft(
      Counter successfullyProcessedVotes,
      LabelledCounter<IgnoredVote> ignoredVotes,
      Counter successfullyProcessedProposals,
      Counter preconditionViolations,
      Counter duplicateProposalsReceived,
      Counter eventsReceived,
      Counter committedVertices,
      Counter noVotesSent,
      Counter voteQuorums,
      Counter timeoutQuorums,
      Counter prolongedRoundTimeouts,
      Counter obsoleteEventsIgnored,
      LabelledCounter<RoundChange> roundChanges,
      Timer consensusEventsQueueWait,
      LabelledCounter<RejectedConsensusEvent> rejectedConsensusEvents,
      GetterGauge validatorCount,
      GetterGauge inValidatorSet,
      Pacemaker pacemaker,
      Sync sync,
      VertexStore vertexStore,
      Summary leaderMaxProposalPayloadSize,
      Summary leaderNumTransactionsIncludedInProposal,
      Summary leaderTransactionBytesIncludedInProposal,
      Summary leaderTransactionBytesIncludedInProposalAndPreviousVertices) {

    public record IgnoredVote(VoteIgnoreReason reason) {}

    public enum VoteIgnoreReason {
      QUORUM_ALREADY_REACHED,
      UNEXPECTED_VOTE
    }

    public record Pacemaker(
        Counter timeoutsSent,
        Gauge round,
        Counter proposedTransactions,
        Counter proposalsSent,
        Counter timedOutRounds,
        Counter proposalsWithSubstituteTimestamp,
        Timer roundDuration) {}

    public record Sync(Counter requestsSent, Counter requestsReceived, Counter requestTimeouts) {}

    public record VertexStore(
        Gauge size, Counter forks, Counter rebuilds, Counter indirectParents) {}
  }

  public record BerkeleyDb(V1Ledger v1Ledger, AddressBook addressBook, SafetyState safetyState) {

    public record V1Ledger(
        Counter commits,
        Timer transactionCreate,
        Timer read,
        Timer store,
        Timer lastCommittedRead,
        Timer lastVertexRead,
        Timer save,
        Timer interact,
        Counter bytesRead,
        Counter bytesWritten,
        Counter proofsAdded,
        Counter proofsRemoved,
        Counter headerBytesWritten) {}

    public record AddressBook(
        Timer interact, Counter bytesRead, Counter bytesWritten, Counter entriesDeleted) {}

    public record SafetyState(Timer commitState, Counter bytesRead, Counter bytesWritten) {}
  }

  public record Ledger(
      @NotExposed Gauge stateVersion,
      Counter syncTransactionsProcessed,
      Counter bftTransactionsProcessed,
      Timer commit,
      Timer prepare) {}

  public record LedgerSync(
      Counter invalidResponsesReceived,
      Counter validResponsesReceived,
      Counter remoteRequestsReceived,
      Gauge currentStateVersion,
      Gauge targetStateVersion) {}

  public record Mempool(Timer addTransaction) {}

  public record V1Mempool(
      Gauge size, Counter relaysSent, Counter addSuccesses, Counter addFailures) {}

  public record V1RadixEngine(
      Counter invalidProposedTransactions, Counter userTransactions, Counter systemTransactions) {}

  public record Messages(Inbound inbound, Outbound outbound) {

    public record Inbound(Timer queueWait, Timer process, Counter received, Counter discarded) {}

    public record Outbound(Counter aborted, Gauge queued, Counter processed, Counter sent) {}
  }

  public record Networking(
      Counter messagesDropped,
      Counter bytesSent,
      Counter bytesReceived,
      LabelledGauge<ChannelProperties> activeChannels,
      Counter channelsInitialized) {}

  public record Crypto(Counter bytesHashed, Counter signaturesSigned, Counter signaturesVerified) {}

  public record EpochManager(
      GetterGauge currentEpoch, GetterGauge currentRound, Counter enqueuedConsensusEvents) {}

  public record StateManager(LabelledTimer<MethodId> nativeCall) {}

  public record Misc(
      TypedInfo<Config> config,
      Timer applicationStart,
      Counter vertexStoreSaved,
      GetterGauge peerCount) {}

  public record MethodId(String cls, String method) {

    /**
     * Creates a method ID while ensuring that only one such native method exists within the class.
     */
    public MethodId(Class<?> cls, String methodName) {
      this(
          cls.getSimpleName(),
          Arrays.stream(cls.getDeclaredMethods())
              .filter(m -> Modifier.isNative(m.getModifiers()))
              .map(Method::getName)
              .filter(Predicates.equalTo(methodName))
              .collect(MoreCollectors.onlyElement()));
    }
  }

  public record RoundChange(
      String leaderKey,
      String leaderComponentAddress,
      HighQcSource highQcSource,
      CertificateType certificateType) {
    public enum HighQcSource {
      CREATED_ON_RECEIVED_NON_TIMEOUT_VOTE,
      CREATED_ON_RECEIVED_TIMEOUT_VOTE,
      RECEIVED_ALONG_WITH_PROPOSAL,
      RECEIVED_ALONG_WITH_VOTE,
      RECEIVED_IN_BFT_SYNC_VERTICES_ERROR_RESPONSE
    }

    public enum CertificateType {
      QC_ON_REGULAR_VERTEX,
      QC_ON_FALLBACK_VERTEX,
      TC
    }
  }

  public record RejectedConsensusEvent(
      Type type, Invalidity invalidity, @Nullable TimestampIssue timestampIssue) {

    public RejectedConsensusEvent(Type type, Invalidity invalidity) {
      this(type, invalidity, null);
      Preconditions.checkArgument(invalidity != Invalidity.TIMESTAMP);
    }

    public RejectedConsensusEvent(Type type, TimestampIssue timestampIssue) {
      this(type, Invalidity.TIMESTAMP, timestampIssue);
    }

    public enum Type {
      VOTE,
      PROPOSAL
    }

    public enum Invalidity {
      AUTHOR,
      SIGNATURE,
      ATTACHED_QC,
      TIMEOUT_SIGNATURE,
      TIMESTAMP
    }

    public enum TimestampIssue {
      TOO_FAR_PAST,
      TOO_FAR_FUTURE,
      NOT_MONOTONIC
    }
  }

  public record ChannelProperties(Direction direction) {

    public enum Direction {
      INBOUND,
      OUTBOUND;
    }
  }

  public record Config(String branchAndCommit, String key) {}
}
