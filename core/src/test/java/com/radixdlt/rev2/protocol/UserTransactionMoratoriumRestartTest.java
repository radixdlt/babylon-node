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

package com.radixdlt.rev2.protocol;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.harness.predicates.EventPredicate.onlyConsensusEventsAndSelfLedgerUpdates;
import static com.radixdlt.harness.predicates.EventPredicate.onlyLocalMempoolAddEvents;
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverProtocolVersion;
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverStateVersion;
import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransactionSuccess;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import com.google.inject.AbstractModule;
import com.google.inject.Injector;
import com.google.inject.Key;
import com.google.inject.Module;
import com.google.inject.TypeLiteral;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTHighQCUpdate;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.deterministic.invariants.DeterministicMonitors;
import com.radixdlt.harness.deterministic.invariants.MessageMonitor;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolRejectedException;
import com.radixdlt.mempool.RustMempool;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.UserTransactionMoratorium;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.TransactionBuilder;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transactions.RawNotarizedTransaction;
import java.time.Duration;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

/** Tests restart with persisted votes, certificates and unequal ledger tips. */
public final class UserTransactionMoratoriumRestartTest {
  private static final String EAGLE_RAY = ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME;
  private static final int NUM_VALIDATORS = 4;
  private static final long ROUNDS_PER_EPOCH = 10;
  private static final long WARM_UP_EPOCH = 3;
  private static final int MAX_MESSAGES_PER_STEP = 200_000;
  private static final int MAX_LOCAL_EVENTS_PER_NODE = 500;
  private static final int MAX_VOTES_PER_ROUND = 20;
  private static final long MIN_ROUNDS_PER_EPOCH = 10;
  private static final long MAX_ROUNDS_PER_EPOCH = 100;
  private static final Duration EPOCH_TARGET_DURATION = Duration.ofMinutes(10);
  private static final Duration HALT_DURATION = Duration.ofDays(4);

  @Rule public final TemporaryFolder folder = new TemporaryFolder();

  private final List<Round> highQcRounds = new CopyOnWriteArrayList<>();

  @Test
  public void split_in_progress_round_resolves_by_timeout_without_certifying_the_transaction() {
    try (var test = createTest()) {
      // Arrange
      final var halted = haltWithProposalDeliveredTo(test, List.of(1, 2));

      // Act
      restartAllWithMoratorium(test, halted.haltEpoch());
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);

      // Assert
      assertFalse(isCommittedOnAnyNode(test, halted.inFlight()));
      assertTrue(timeoutQuorumResolutions(test) > 0.0);
      assertEquals(NUM_VALIDATORS, test.numNodesLive());
    }
  }

  @Test
  public void replayed_votes_of_a_quorum_certify_the_in_flight_vertex_and_nothing_else_commits() {
    try (var test = createTest()) {
      // Arrange
      final var halted = haltWithProposalDeliveredTo(test, List.of(0, 1, 2));

      // Act
      restartAllWithMoratorium(test, halted.haltEpoch());
      final var submittedDuring = TransactionBuilder.forTests().prepare().raw();
      final var rejection =
          assertThrows(
              MempoolRejectedException.class,
              () -> test.getInstance(3, RustMempool.class).addTransaction(submittedDuring));
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var inFlightCommittedPerNode = commitStatusPerNode(test, halted.inFlight());
      final var submittedDuringCommitted = isCommittedOnAnyNode(test, submittedDuring);
      submit(test, submittedDuring);
      test.runUntilState(allCommittedTransactionSuccess(submittedDuring), MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(List.of(true, true, true, true), inFlightCommittedPerNode);
      assertTrue(
          rejection.getMessage(),
          rejection
              .getMessage()
              .contains("temporarily not accepted; retry from epoch " + (halted.haltEpoch() + 1)));
      assertFalse(submittedDuringCommitted);
    }
  }

  @Test
  public void validators_with_different_committed_tips_converge_after_restart() {
    try (var test = createTest()) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(WARM_UP_EPOCH), MAX_MESSAGES_PER_STEP);
      test.runUntilState(allAtOrOverStateVersion(stateVersion(test, 0) + 3), MAX_MESSAGES_PER_STEP);
      test.runUntilState(
          ignored -> test.getNetwork().allMessages().stream().anyMatch(this::isProposal),
          MAX_MESSAGES_PER_STEP,
          message -> !isProposal(message));
      test.runNext(message -> isProposal(message) && message.channelId().receiverIndex() == 1);
      drainLocalEvents(test, 1);
      final var aheadVersion = stateVersion(test, 1);
      final var behindVersion = stateVersion(test, 0);
      final var haltEpoch = highestEpoch(test);
      haltAll(test);

      // Act
      restartAllWithMoratorium(test, haltEpoch);
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);

      // Assert
      assertTrue(aheadVersion > behindVersion);
      assertTrue(allAtOrOverStateVersion(aheadVersion).test(test.getNodeInjectors()));
      assertEquals(NUM_VALIDATORS, test.numNodesLive());
    }
  }

  @Test
  public void node_refuses_to_start_when_the_enactment_epoch_has_already_passed() {
    try (var test = createTest()) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(WARM_UP_EPOCH), MAX_MESSAGES_PER_STEP);
      final var currentEpoch = currentEpoch(test, 0);
      final var staleConfig = withMoratorium(currentEpoch - 2, currentEpoch - 1);
      final var stateVersionBefore = stateVersion(test, 1);

      // Act
      final var failure =
          assertThrows(
              RuntimeException.class,
              () -> test.restartNodeWithOverrideModule(0, protocolConfigOverride(staleConfig)));
      test.runUntilState(
          nodesMatch(List.of(1, 2, 3), NodePredicate.atOrOverStateVersion(stateVersionBefore + 5)),
          MAX_MESSAGES_PER_STEP);

      // Assert
      assertTrue(
          messageChain(failure), messageChain(failure).contains("protocol misconfiguration"));
      assertEquals(NUM_VALIDATORS - 1, test.numNodesLive());
    }
  }

  /** Resumes the next leader's persisted QC without relying on vote replay. */
  @Test
  public void pre_halt_certified_vertex_in_persisted_stores_commits_after_restart() {
    try (var test = createTest()) {
      // Arrange
      final var halted = haltWithProposalDeliveredTo(test, List.of(0, 1, 2), true);
      final var certifiedBeforeHalt = highQcRounds.contains(halted.round());

      // Act
      restartAllWithMoratorium(test, halted.haltEpoch());
      final var submittedDuring = TransactionBuilder.forTests().prepare().raw();
      final var rejection =
          assertThrows(
              MempoolRejectedException.class,
              () -> test.getInstance(3, RustMempool.class).addTransaction(submittedDuring));
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var inFlightCommittedPerNode = commitStatusPerNode(test, halted.inFlight());

      // Assert
      assertTrue(certifiedBeforeHalt);
      assertEquals(List.of(true, true, true, true), inFlightCommittedPerNode);
      assertTrue(
          rejection.getMessage(),
          rejection
              .getMessage()
              .contains("temporarily not accepted; retry from epoch " + (halted.haltEpoch() + 1)));
    }
  }

  /**
   * The halt exceeds the epoch target duration, leaving the minimum round count as the remaining
   * constraint on epoch completion.
   */
  @Test
  public void
      restart_after_a_long_halt_accepts_proposals_and_ends_the_epoch_at_the_minimum_round() {
    try (var test =
        createTest(
            GenesisConsensusManagerConfig.Builder.testDefaults()
                .epochMinRoundCount(MIN_ROUNDS_PER_EPOCH)
                .epochMaxRoundCount(MAX_ROUNDS_PER_EPOCH)
                .epochTargetDurationMillis(EPOCH_TARGET_DURATION.toMillis()))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(WARM_UP_EPOCH), MAX_MESSAGES_PER_STEP);
      test.runUntilState(allAtOrOverStateVersion(stateVersion(test, 0) + 3), MAX_MESSAGES_PER_STEP);
      final var warmUpEpochChangeRound = epochChangeRound(test, 0);
      final var haltEpoch = highestEpoch(test);
      final var haltRound = highestRound(test);
      haltAll(test);
      test.advanceTime(HALT_DURATION);

      // Act
      restartAllWithMoratorium(test, haltEpoch);
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var enactingEpochChangeRound = epochChangeRound(test, 0);

      // Assert
      assertEquals(MAX_ROUNDS_PER_EPOCH, warmUpEpochChangeRound);
      assertTrue(haltRound < MIN_ROUNDS_PER_EPOCH);
      assertTrue(enactingEpochChangeRound >= MIN_ROUNDS_PER_EPOCH);
      assertTrue(enactingEpochChangeRound <= MIN_ROUNDS_PER_EPOCH + 5);
      assertEquals(0.0, rejectedProposalsForTimestamp(test), 0.0);
      assertEquals(NUM_VALIDATORS, test.numNodesLive());
    }
  }

  private record HaltedNetwork(RawNotarizedTransaction inFlight, Round round, long haltEpoch) {}

  /** Halts after the selected validators persist their votes, without delivering those votes. */
  private HaltedNetwork haltWithProposalDeliveredTo(DeterministicTest test, List<Integer> voters) {
    return haltWithProposalDeliveredTo(test, voters, false);
  }

  /**
   * Persists the selected validators' votes for a user proposal. With {@code deliverVotes},
   * delivers them so a quorum can form a QC before the halt. Drops queued messages when halting.
   */
  private HaltedNetwork haltWithProposalDeliveredTo(
      DeterministicTest test, List<Integer> voters, boolean deliverVotes) {
    test.startAllNodes();
    test.runUntilState(allAtOrOverEpoch(WARM_UP_EPOCH), MAX_MESSAGES_PER_STEP);
    final var inFlight = TransactionBuilder.forTests().prepare().raw();
    submit(test, inFlight);
    test.runUntilState(
        ignored ->
            test.getNetwork().allMessages().stream()
                .anyMatch(message -> carriesProposalOf(message, inFlight)),
        MAX_MESSAGES_PER_STEP,
        message -> !carriesProposalOf(message, inFlight));
    final var round =
        test.getNetwork().allMessages().stream()
            .filter(message -> carriesProposalOf(message, inFlight))
            .map(message -> ((Proposal) message.message()).getRound())
            .findFirst()
            .orElseThrow();
    for (final var voter : voters) {
      test.runNext(
          message ->
              carriesProposalOf(message, inFlight) && message.channelId().receiverIndex() == voter);
      drainLocalEvents(test, voter);
    }
    if (deliverVotes) {
      test.runUntilOutOfMessagesOfType(
          MAX_VOTES_PER_ROUND,
          message -> message.message() instanceof Vote vote && vote.getRound().equals(round));
      test.getNodeIndices().forEach(nodeIndex -> drainLocalEvents(test, nodeIndex));
    }
    final var haltEpoch = highestEpoch(test);
    haltAll(test);
    return new HaltedNetwork(inFlight, round, haltEpoch);
  }

  /**
   * Drains consensus events needed to persist votes. Excludes periodic triggers because they
   * reschedule themselves indefinitely.
   */
  private static void drainLocalEvents(DeterministicTest test, int nodeIndex) {
    test.runUntilOutOfMessagesOfType(
        MAX_LOCAL_EVENTS_PER_NODE,
        onlyConsensusEventsAndSelfLedgerUpdates()
            .and(message -> message.channelId().isLocal(nodeIndex)));
  }

  private static void haltAll(DeterministicTest test) {
    test.getNodeIndices().forEach(test::shutdownNode);
    test.getNetwork().dropAllMessages();
  }

  private static void restartAllWithMoratorium(DeterministicTest test, long haltEpoch) {
    final var upgradedBinary = protocolConfigOverride(withMoratorium(haltEpoch, haltEpoch + 1));
    test.getNodeIndices()
        .forEach(nodeIndex -> test.restartNodeWithOverrideModule(nodeIndex, upgradedBinary));
  }

  private static Module protocolConfigOverride(ProtocolConfig protocolConfig) {
    return new AbstractModule() {
      @Override
      protected void configure() {
        bind(ProtocolConfig.class).toInstance(protocolConfig);
      }
    };
  }

  private DeterministicTest createTest() {
    return createTest(
        GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(ROUNDS_PER_EPOCH));
  }

  private DeterministicTest createTest(GenesisConsensusManagerConfig.Builder consensusConfig) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true))
        .addMonitors(DeterministicMonitors.byzantineBehaviorNotDetected(), highQcRoundMonitor())
        .messageSelector(firstSelector())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                FunctionalRadixNodeModule.NodeStorageConfig.tempFolder(folder),
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.REAL,
                FunctionalRadixNodeModule.ConsensusConfig.testDefault(),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                    StateComputerConfig.rev2()
                        .withGenesis(
                            GenesisBuilder.createTestGenesisWithNumValidators(
                                NUM_VALIDATORS, Decimal.ONE, consensusConfig))
                        .withProtocolConfig(withoutMoratorium())
                        .withProposerConfig(
                            StateComputerConfig.REV2ProposerConfig.Mempool.defaults()),
                    SyncRelayConfig.of(5000, 10, 3000L))));
  }

  private Module highQcRoundMonitor() {
    return new AbstractModule() {
      @ProvidesIntoSet
      MessageMonitor recordHighQcRounds() {
        return (message, time) -> {
          if (message.message() instanceof BFTHighQCUpdate update) {
            highQcRounds.add(update.getHighQC().highestQC().getRound());
          }
        };
      }
    };
  }

  private static ProtocolConfig withMoratorium(long moratoriumFromEpoch, long enactmentEpoch) {
    return ProtocolConfig.enactAtEpochWithUserTransactionMoratorium(
        EAGLE_RAY, moratoriumFromEpoch, enactmentEpoch);
  }

  private static ProtocolConfig withoutMoratorium() {
    return ProtocolConfig.enactAtEpoch(EAGLE_RAY, 1000);
  }

  private static boolean carriesProposalOf(
      ControlledMessage message, RawNotarizedTransaction transaction) {
    return message.message() instanceof Proposal proposal
        && proposal.getVertex().getTransactions().stream()
            .anyMatch(carried -> Arrays.equals(carried.getPayload(), transaction.getPayload()));
  }

  private boolean isProposal(ControlledMessage message) {
    return message.message() instanceof Proposal;
  }

  private static void submit(DeterministicTest test, RawNotarizedTransaction transaction) {
    final var mempoolDispatcher =
        test.getInstance(0, Key.get(new TypeLiteral<EventDispatcher<MempoolAdd>>() {}));
    mempoolDispatcher.dispatch(new MempoolAdd(List.of(transaction)));
    test.runUntilOutOfMessagesOfType(100, onlyLocalMempoolAddEvents());
  }

  private static boolean isCommittedOnAnyNode(
      DeterministicTest test, RawNotarizedTransaction transaction) {
    return commitStatusPerNode(test, transaction).contains(true);
  }

  private static List<Boolean> commitStatusPerNode(
      DeterministicTest test, RawNotarizedTransaction transaction) {
    return test.getNodeInjectors().stream()
        .map(
            injector ->
                NodePredicate.committedUserTransaction(transaction, false, false).test(injector))
        .toList();
  }

  private static double timeoutQuorumResolutions(DeterministicTest test) {
    return test.getNodeInjectors().stream()
        .mapToDouble(
            injector ->
                injector
                    .getInstance(Metrics.class)
                    .bft()
                    .quorumResolutions()
                    .label(new Metrics.Bft.QuorumResolution(true))
                    .get())
        .sum();
  }

  private static long currentEpoch(DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, TransactionsAndProofReader.class)
        .getLatestProofBundle()
        .orElseThrow()
        .resultantEpoch();
  }

  private static long epochChangeRound(DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, TransactionsAndProofReader.class)
        .getLatestProofBundle()
        .orElseThrow()
        .latestProofWhichInitiatedAnEpochChange()
        .ledgerHeader()
        .round()
        .toLong();
  }

  private static long highestRound(DeterministicTest test) {
    return test.getNodeIndices().stream()
        .mapToLong(
            nodeIndex ->
                test.getInstance(nodeIndex, TransactionsAndProofReader.class)
                    .getLatestProofBundle()
                    .orElseThrow()
                    .resultantRound()
                    .number())
        .max()
        .orElseThrow();
  }

  private static double rejectedProposalsForTimestamp(DeterministicTest test) {
    return test.getNodeInjectors().stream()
        .flatMap(
            injector ->
                Stream.of(Metrics.RejectedConsensusEvent.TimestampIssue.values())
                    .map(
                        issue ->
                            injector
                                .getInstance(Metrics.class)
                                .bft()
                                .rejectedConsensusEvents()
                                .label(
                                    new Metrics.RejectedConsensusEvent(
                                        Metrics.RejectedConsensusEvent.Type.PROPOSAL, issue))
                                .get()))
        .mapToDouble(Double::doubleValue)
        .sum();
  }

  private static long highestEpoch(DeterministicTest test) {
    return test.getNodeIndices().stream()
        .mapToLong(nodeIndex -> currentEpoch(test, nodeIndex))
        .max()
        .orElseThrow();
  }

  private static long stateVersion(DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, TransactionsAndProofReader.class)
        .getLatestProofBundle()
        .orElseThrow()
        .resultantStateVersion();
  }

  private static Predicate<List<Injector>> nodesMatch(
      List<Integer> nodeIndices, Predicate<Injector> nodePredicate) {
    return injectors ->
        nodeIndices.stream().allMatch(index -> nodePredicate.test(injectors.get(index)));
  }

  private static String messageChain(Throwable throwable) {
    return Stream.iterate(throwable, Objects::nonNull, Throwable::getCause)
        .map(Throwable::getMessage)
        .filter(Objects::nonNull)
        .collect(Collectors.joining(" | "));
  }

  @SuppressWarnings("unused")
  private static Option<UserTransactionMoratorium> moratoriumOf(
      DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, RustStateComputer.class)
        .ensureUserTransactionsAllowed()
        .toOptionOfError();
  }
}
