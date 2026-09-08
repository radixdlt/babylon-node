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
import static com.radixdlt.harness.predicates.EventPredicate.onlyLedgerSyncEvents;
import static com.radixdlt.harness.predicates.EventPredicate.onlyLocalMempoolAddEvents;
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverProtocolVersion;
import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransactionSuccess;
import static com.radixdlt.harness.predicates.NodesPredicate.anyCommittedTransactionSuccess;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import com.google.inject.AbstractModule;
import com.google.inject.Injector;
import com.google.inject.Key;
import com.google.inject.Module;
import com.google.inject.TypeLiteral;
import com.google.inject.multibindings.OptionalBinder;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.api.CoreApiHelper;
import com.radixdlt.api.MeshApiHelper;
import com.radixdlt.api.core.generated.models.TransactionSubmitErrorResponse;
import com.radixdlt.api.core.generated.models.TransactionSubmitRejectedErrorDetails;
import com.radixdlt.api.core.generated.models.TransactionSubmitRequest;
import com.radixdlt.api.mesh.generated.api.ConstructionApi;
import com.radixdlt.api.mesh.generated.models.ConstructionSubmitRequest;
import com.radixdlt.consensus.ConsensusEvent;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.Vote;
import com.radixdlt.consensus.bft.BFTInsertUpdate;
import com.radixdlt.consensus.bft.NoVote;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.epoch.EpochProposalRejected;
import com.radixdlt.consensus.epoch.Epoched;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.liveness.ProposerElections;
import com.radixdlt.consensus.liveness.UserTransactionMoratoriumProvider;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.RemoteEventDispatcher;
import com.radixdlt.environment.deterministic.network.ControlledMessage;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.deterministic.invariants.MessageMonitor;
import com.radixdlt.harness.predicates.NodePredicate;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolRejectedException;
import com.radixdlt.mempool.RustMempool;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.UserTransactionMoratorium;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.rev2.TransactionBuilder;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.UInt64;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Predicate;
import java.util.stream.Stream;
import junitparams.JUnitParamsRunner;
import junitparams.Parameters;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;

@RunWith(JUnitParamsRunner.class)
public final class UserTransactionMoratoriumTest {
  private static final String EAGLE_RAY = ProtocolConfig.EAGLE_RAY_PROTOCOL_VERSION_NAME;
  private static final int NUM_VALIDATORS = 4;
  private static final long ROUNDS_PER_EPOCH = 10;
  private static final long MORATORIUM_FROM_EPOCH = 4;
  private static final long ENACTMENT_EPOCH = 5;
  private static final int MAX_MESSAGES_PER_STEP = 200_000;

  @Rule public final TemporaryFolder folder = new TemporaryFolder();

  private final List<EpochProposalRejected> rejectedProposals = new CopyOnWriteArrayList<>();

  @Test
  @Parameters({"4", "5"})
  public void future_user_proposal_waits_for_epoch_change_before_moratorium_check(
      long proposalEpoch) {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(proposalEpoch - 1), MAX_MESSAGES_PER_STEP);
      test.runUntilState(
          nodesMatch(List.of(1, 2, 3), NodePredicate.atOrOverEpoch(proposalEpoch)),
          MAX_MESSAGES_PER_STEP,
          message -> message.channelId().receiverIndex() != 0);
      final var transaction = TransactionBuilder.forTests().prepare().raw();
      test.restartNodeWithOverrideModule(1, oldBinaryEmulation(transaction));
      final Predicate<ControlledMessage> isFutureUserProposal =
          message ->
              message.channelId().receiverIndex() == 0
                  && message.message() instanceof Proposal proposal
                  && proposal.getEpoch() == proposalEpoch
                  && proposal.getVertex().getTransactions().contains(transaction);
      test.runUntilState(
          ignored -> test.getNetwork().allMessages().stream().anyMatch(isFutureUserProposal),
          MAX_MESSAGES_PER_STEP,
          message -> message.channelId().receiverIndex() != 0);
      final var queuedProposal =
          test.getNetwork().allMessages().stream()
              .filter(isFutureUserProposal)
              .findFirst()
              .orElseThrow();
      final var proposal = (Proposal) queuedProposal.message();
      final var vertexHash = proposal.getVertex().withId(test.getInstance(0, Hasher.class)).hash();
      final var metrics = test.getInstance(0, Metrics.class);
      final var queuedBefore = metrics.epochManager().enqueuedConsensusEvents().get();
      final var checkedBefore = metrics.bft().proposalsReceived().get();
      final var committedBefore = currentEpoch(test);
      final Predicate<ControlledMessage> isDecision =
          message ->
              message.channelId().senderIndex() == 0
                  && (message.message() instanceof Vote vote
                          && vote.getVoteData().getProposed().getVertexId().equals(vertexHash)
                      || message.message() instanceof EpochProposalRejected rejected
                          && rejected.epoch() == proposalEpoch
                          && rejected.proposalRejected().round().equals(proposal.getRound()));

      // Act
      test.runNext(queuedProposal::equals);
      final var queuedAfter = metrics.epochManager().enqueuedConsensusEvents().get();
      final var checkedWhileBehind = metrics.bft().proposalsReceived().get();
      final var committedWhileQueued = currentEpoch(test);
      final var decidedWhileBehind = test.getNetwork().allMessages().stream().anyMatch(isDecision);
      test.runUntilState(
          ignored -> test.getNetwork().allMessages().stream().anyMatch(isDecision),
          MAX_MESSAGES_PER_STEP,
          message ->
              !(message.message() instanceof ConsensusEvent
                  || message.message() instanceof Epoched<?>));
      final var decision =
          test.getNetwork().allMessages().stream()
              .filter(isDecision)
              .findFirst()
              .orElseThrow()
              .message();
      final var committedAtDecision = currentEpoch(test);
      final var protocolAtDecision =
          test.getInstance(0, RustStateComputer.class).protocolState().currentProtocolVersion();
      test.runUntilState(allCommittedTransactionSuccess(transaction), MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(proposalEpoch - 1, committedBefore);
      assertEquals(committedBefore, committedWhileQueued);
      assertEquals(queuedBefore + 1, queuedAfter, 0.0);
      assertEquals(checkedBefore, checkedWhileBehind, 0.0);
      assertFalse(decidedWhileBehind);
      assertEquals(proposalEpoch, committedAtDecision);
      assertEquals(proposalEpoch < ENACTMENT_EPOCH, decision instanceof EpochProposalRejected);
      assertEquals(proposalEpoch >= ENACTMENT_EPOCH, decision instanceof Vote);
      assertEquals(proposalEpoch >= ENACTMENT_EPOCH, EAGLE_RAY.equals(protocolAtDecision));
    }
  }

  @Test
  @Parameters({"3", "4"})
  public void pending_vertex_vote_uses_vertex_epoch_after_committed_epoch_crosses_boundary(
      long vertexEpoch) {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(vertexEpoch), MAX_MESSAGES_PER_STEP);
      final var transaction = TransactionBuilder.forTests().prepare().raw();
      List.of(1, 2, 3)
          .forEach(
              index -> test.restartNodeWithOverrideModule(index, oldBinaryEmulation(transaction)));
      final var metrics = test.getInstance(0, Metrics.class);
      final Predicate<ControlledMessage> isUserInsert =
          message ->
              message.channelId().isLocal(0)
                  && message.message() instanceof BFTInsertUpdate update
                  && update.insertedVertex().vertex().getEpoch() == vertexEpoch
                  && update.insertedVertex().vertex().getTransactions().contains(transaction);
      final Predicate<ControlledMessage> isCurrentUserInsert =
          isUserInsert.and(
              message ->
                  ((BFTInsertUpdate) message.message()).insertedVertex().getRound().number()
                      == metrics.bft().pacemaker().round().get());
      test.runUntilState(
          ignored -> test.getNetwork().allMessages().stream().anyMatch(isCurrentUserInsert),
          MAX_MESSAGES_PER_STEP,
          message ->
              !isUserInsert.test(message)
                  && !(message.channelId().receiverIndex() == 0
                      && (message.message() instanceof Epoched<?>
                          || message.message() instanceof EpochProposalRejected)));
      final var pendingInsert =
          test.getNetwork().allMessages().stream()
              .filter(isCurrentUserInsert)
              .findFirst()
              .orElseThrow();
      final var vertex = ((BFTInsertUpdate) pendingInsert.message()).insertedVertex();
      final var committedBefore = currentEpoch(test);
      final var roundBefore = metrics.bft().pacemaker().round().get();
      final Predicate<ControlledMessage> isDecision =
          message ->
              message.channelId().senderIndex() == 0
                  && (message.message() instanceof Vote vote
                          && vote.getVoteData()
                              .getProposed()
                              .getVertexId()
                              .equals(vertex.getVertexHash())
                      || message.message() instanceof NoVote noVote
                          && noVote.vertex().hash().equals(vertex.getVertexHash()));

      // Act
      test.runUntilState(
          ignored -> currentEpoch(test) >= vertexEpoch + 1,
          MAX_MESSAGES_PER_STEP,
          message ->
              (message.channelId().receiverIndex() != 0
                      || onlyLedgerSyncEvents().test(message)
                          && !(message.message() instanceof LedgerUpdate))
                  && !(message.message() instanceof ConsensusEvent event
                      && event.getEpoch() > vertexEpoch));
      final var committedBeforeCallback = currentEpoch(test);
      final var roundBeforeCallback = metrics.bft().pacemaker().round().get();
      final var decidedBeforeCallback =
          test.getNetwork().allMessages().stream().anyMatch(isDecision);
      test.runNext(pendingInsert::equals);
      final var decisions =
          test.getNetwork().allMessages().stream()
              .filter(isDecision)
              .map(ControlledMessage::message)
              .toList();
      final var afterEnactment = TransactionBuilder.forTests().prepare().raw();
      test.runUntilState(allAtOrOverEpoch(ENACTMENT_EPOCH), MAX_MESSAGES_PER_STEP);
      submit(test, afterEnactment);
      test.runUntilState(allCommittedTransactionSuccess(afterEnactment), MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(vertexEpoch, committedBefore);
      assertEquals(vertexEpoch + 1, committedBeforeCallback);
      assertEquals(roundBefore, roundBeforeCallback, 0.0);
      assertFalse(decidedBeforeCallback);
      assertFalse(decisions.isEmpty());
      assertTrue(
          decisions.stream()
              .allMatch(
                  vertexEpoch < MORATORIUM_FROM_EPOCH
                      ? Vote.class::isInstance
                      : NoVote.class::isInstance));
    }
  }

  @Test
  public void consensus_checks_use_event_epoch_even_when_committed_epoch_is_across_a_boundary() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      final var stateComputer = test.getInstance(0, RustStateComputer.class);
      final var provider = test.getInstance(0, UserTransactionMoratoriumProvider.class);
      final var epochs = List.of(MORATORIUM_FROM_EPOCH - 1, MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH);
      final var committedEpochs = new ArrayList<Long>();
      final var allowed = Option.<UserTransactionMoratorium>none();
      final var refused =
          Option.some(
              new UserTransactionMoratorium(
                  UInt64.fromNonNegativeLong(MORATORIUM_FROM_EPOCH),
                  UInt64.fromNonNegativeLong(ENACTMENT_EPOCH)));

      // Act
      final var results =
          epochs.stream()
              .map(
                  committedEpoch -> {
                    test.runUntilState(allAtOrOverEpoch(committedEpoch), MAX_MESSAGES_PER_STEP);
                    committedEpochs.add(currentEpoch(test));
                    return List.of(
                        stateComputer.ensureUserTransactionsAllowed().toOptionOfError(),
                        provider
                            .ensureUserTransactionsAllowed(MORATORIUM_FROM_EPOCH - 1)
                            .toOptionOfError(),
                        provider
                            .ensureUserTransactionsAllowed(MORATORIUM_FROM_EPOCH)
                            .toOptionOfError(),
                        provider.ensureUserTransactionsAllowed(ENACTMENT_EPOCH).toOptionOfError());
                  })
              .toList();

      // Assert
      assertEquals(epochs, committedEpochs);
      assertEquals(
          List.of(
              List.of(allowed, allowed, refused, allowed),
              List.of(refused, allowed, refused, allowed),
              List.of(allowed, allowed, refused, allowed)),
          results);
    }
  }

  @Test
  public void submissions_are_rejected_and_nothing_commits_until_the_update_is_enacted() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      final var beforeMoratorium = TransactionBuilder.forTests().prepare().raw();
      submit(test, beforeMoratorium);
      test.runUntilState(allCommittedTransactionSuccess(beforeMoratorium), MAX_MESSAGES_PER_STEP);
      final var moratoriaBeforeStart = moratoria(test);
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      final var moratoriaDuring = moratoria(test);
      final var duringMoratorium = TransactionBuilder.forTests().prepare().raw();
      final var mempool = test.getInstance(0, RustMempool.class);

      // Act
      final var rejection =
          assertThrows(
              MempoolRejectedException.class, () -> mempool.addTransaction(duringMoratorium));
      submit(test, duringMoratorium);
      test.runUntilState(allAtOrOverEpoch(ENACTMENT_EPOCH), MAX_MESSAGES_PER_STEP);
      final var committedBeforeEnactment = isCommittedOnAnyNode(test, duringMoratorium);
      final var moratoriaAfterEnactment = moratoria(test);
      submit(test, duringMoratorium);
      test.runUntilState(allCommittedTransactionSuccess(duringMoratorium), MAX_MESSAGES_PER_STEP);
      final var epochOfCommitAfterEnactment = currentEpoch(test);

      // Assert
      assertEquals(noMoratoriumOnEveryNode(), moratoriaBeforeStart);
      assertEquals(moratoriumOnEveryNode(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH), moratoriaDuring);
      assertTrue(
          rejection.getMessage(),
          rejection
              .getMessage()
              .contains("temporarily not accepted; retry from epoch " + ENACTMENT_EPOCH));
      assertFalse(committedBeforeEnactment);
      assertTrue(allAtOrOverProtocolVersion(EAGLE_RAY).test(test.getNodeInjectors()));
      assertEquals(noMoratoriumOnEveryNode(), moratoriaAfterEnactment);
      assertEquals(ENACTMENT_EPOCH, epochOfCommitAfterEnactment);
    }
  }

  @Test
  public void proposals_carrying_user_transactions_are_rejected_during_the_moratorium() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      final var forcedTransaction = TransactionBuilder.forTests().prepare().raw();
      test.restartNodeWithOverrideModule(
          0,
          new AbstractModule() {
            @Override
            protected void configure() {
              bind(ProposalGenerator.class)
                  .toInstance((round, prepared) -> List.of(forcedTransaction));
            }
          });

      // Act
      test.runUntilState(allAtOrOverEpoch(ENACTMENT_EPOCH), MAX_MESSAGES_PER_STEP);
      final var committedBeforeEnactment = isCommittedOnAnyNode(test, forcedTransaction);
      final var rejectedDuringMoratorium =
          rejectedProposals.stream()
              .anyMatch(rejected -> rejected.epoch() == MORATORIUM_FROM_EPOCH);
      test.runUntilState(anyCommittedTransactionSuccess(forcedTransaction), MAX_MESSAGES_PER_STEP);

      // Assert
      assertFalse(committedBeforeEnactment);
      assertTrue(rejectedDuringMoratorium);
      assertTrue(allAtOrOverProtocolVersion(EAGLE_RAY).test(test.getNodeInjectors()));
    }
  }

  @Test
  public void coordinated_restart_into_a_moratorium_with_in_flight_transactions_converges() {
    try (var test = createTest(withoutMoratorium())) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(3), MAX_MESSAGES_PER_STEP);
      final var inFlight = TransactionBuilder.forTests().prepare().raw();
      submit(test, inFlight);
      test.runUntilMessage(
          timedMessage ->
              timedMessage.value().message() instanceof Proposal proposal
                  && proposal.getVertex().getTransactions().stream()
                      .anyMatch(
                          transaction ->
                              Arrays.equals(transaction.getPayload(), inFlight.getPayload())),
          true,
          MAX_MESSAGES_PER_STEP);
      test.runForCount(8);
      final var haltEpoch = currentEpoch(test);
      test.getNodeIndices().forEach(test::shutdownNode);
      test.getNetwork().dropMessages(message -> true);
      final var restartConfig = withMoratorium(haltEpoch, haltEpoch + 1);
      final var upgradedBinary =
          new AbstractModule() {
            @Override
            protected void configure() {
              bind(ProtocolConfig.class).toInstance(restartConfig);
            }
          };

      // Act
      test.getNodeIndices()
          .forEach(nodeIndex -> test.restartNodeWithOverrideModule(nodeIndex, upgradedBinary));
      final var moratoriaAfterRestart = moratoria(test);
      final var submittedDuring = TransactionBuilder.forTests().prepare().raw();
      final var rejection =
          assertThrows(
              MempoolRejectedException.class,
              () -> test.getInstance(0, RustMempool.class).addTransaction(submittedDuring));
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var inFlightCommittedPerNode =
          test.getNodeInjectors().stream()
              .map(
                  injector ->
                      NodePredicate.committedUserTransaction(inFlight, false, false).test(injector))
              .toList();
      final var submittedDuringCommitted = isCommittedOnAnyNode(test, submittedDuring);

      // Assert
      assertEquals(moratoriumOnEveryNode(haltEpoch, haltEpoch + 1), moratoriaAfterRestart);
      assertTrue(
          rejection.getMessage(),
          rejection
              .getMessage()
              .contains("temporarily not accepted; retry from epoch " + (haltEpoch + 1)));
      assertTrue(allAtOrOverEpoch(haltEpoch + 1).test(test.getNodeInjectors()));
      assertEquals(noMoratoriumOnEveryNode(), moratoria(test));
      assertEquals(1, inFlightCommittedPerNode.stream().distinct().count());
      assertFalse(submittedDuringCommitted);
    }
  }

  @Test
  public void validator_without_moratorium_cannot_get_user_transactions_certified() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      final var forcedTransaction = TransactionBuilder.forTests().prepare().raw();
      test.restartNodeWithOverrideModule(
          0,
          new AbstractModule() {
            @Override
            protected void configure() {
              // Emulate an old validator that still proposes and votes for user transactions.
              bind(ProposalGenerator.class)
                  .toInstance((round, prepared) -> List.of(forcedTransaction));
              OptionalBinder.newOptionalBinder(binder(), UserTransactionMoratoriumProvider.class)
                  .setBinding()
                  .toInstance(UserTransactionMoratoriumProvider.NONE);
            }
          });

      // Act
      test.runUntilState(allAtOrOverEpoch(ENACTMENT_EPOCH), MAX_MESSAGES_PER_STEP);
      final var committedBeforeEnactment = isCommittedOnAnyNode(test, forcedTransaction);
      final var rejectedDuringMoratorium =
          rejectedProposals.stream()
              .anyMatch(rejected -> rejected.epoch() == MORATORIUM_FROM_EPOCH);
      test.runUntilState(anyCommittedTransactionSuccess(forcedTransaction), MAX_MESSAGES_PER_STEP);

      // Assert
      assertFalse(committedBeforeEnactment);
      assertTrue(rejectedDuringMoratorium);
      assertTrue(allAtOrOverProtocolVersion(EAGLE_RAY).test(test.getNodeInjectors()));
    }
  }

  @Test
  public void late_joining_node_syncs_through_the_moratorium_and_restarted_validator_rejoins() {
    final var validatorIndices = List.of(0, 1, 2, 3);
    final var fullNodeIndex = NUM_VALIDATORS;
    final var nodes =
        Stream.concat(
                PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true).stream(),
                PhysicalNodeConfig.createBatchStream(false).skip(NUM_VALIDATORS).limit(1))
            .toList();
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH), nodes)) {
      // Arrange
      validatorIndices.forEach(test::startNode);
      test.runUntilState(
          nodesMatch(validatorIndices, NodePredicate.atOrOverProtocolVersion(EAGLE_RAY)),
          MAX_MESSAGES_PER_STEP);
      final var stateVersionAtEnactment = stateVersion(test, 0);

      // Act
      test.startNode(fullNodeIndex);
      test.runUntilState(
          nodesMatch(
              List.of(fullNodeIndex),
              NodePredicate.atOrOverProtocolVersion(EAGLE_RAY)
                  .and(NodePredicate.atOrOverStateVersion(stateVersionAtEnactment))),
          MAX_MESSAGES_PER_STEP);
      final var fullNodeMoratorium = moratoriumOf(test, fullNodeIndex);
      test.restartNode(1);
      final var restartedValidatorMoratorium = moratoriumOf(test, 1);
      final var stateVersionBeforeProgress = stateVersion(test, 0);
      test.runUntilState(
          nodesMatch(
              List.of(0, 1), NodePredicate.atOrOverStateVersion(stateVersionBeforeProgress + 5)),
          MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(Option.empty(), fullNodeMoratorium);
      assertEquals(Option.empty(), restartedValidatorMoratorium);
      assertEquals(NUM_VALIDATORS + 1, test.numNodesLive());
    }
  }

  @Test
  public void core_api_rejects_submission_during_moratorium_with_retry_epoch() {
    final var coreApiHelper = new CoreApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      test.restartNodeWithOverrideModule(0, coreApiHelper.module());
      final var transaction = TransactionBuilder.forTests().prepare();
      final var request =
          new TransactionSubmitRequest()
              .network(Network.INTEGRATIONTESTNET.getLogicalName())
              .notarizedTransactionHex(transaction.hexPayloadBytes());

      // Act
      final var errorResponse =
          coreApiHelper.assertErrorResponseOfType(
              () -> coreApiHelper.transactionApi().transactionSubmitPost(request),
              TransactionSubmitErrorResponse.class);
      final var details = (TransactionSubmitRejectedErrorDetails) errorResponse.getDetails();

      // Assert
      assertEquals(Integer.valueOf(400), errorResponse.getCode());
      assertEquals(Boolean.FALSE, details.getIsIntentRejectionPermanent());
      assertEquals(Boolean.FALSE, details.getIsPayloadRejectionPermanent());
      assertEquals(Long.valueOf(ENACTMENT_EPOCH), details.getRetryFromEpoch());
      assertTrue(
          details.getErrorMessage(),
          details
              .getErrorMessage()
              .contains("temporarily not accepted; retry from epoch " + ENACTMENT_EPOCH));
    }
  }

  @Test
  public void mesh_api_refusal_is_retryable_and_the_same_payload_commits_after_enactment()
      throws Exception {
    final var meshApiHelper = new MeshApiHelper(Network.INTEGRATIONTESTNET);
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      test.restartNodeWithOverrideModule(0, meshApiHelper.module());
      final var transaction = TransactionBuilder.forTests().prepare();
      final var api = new ConstructionApi(meshApiHelper.client());
      final var request =
          new ConstructionSubmitRequest()
              .networkIdentifier(meshApiHelper.networkIdentifier())
              .signedTransaction(transaction.hexPayloadBytes());

      // Act
      final var refusal =
          meshApiHelper.assertErrorResponseOfType(
              () -> api.constructionSubmit(request),
              com.radixdlt.api.mesh.generated.models.Error.class);
      final var mempoolCountDuringMoratorium = test.getInstance(0, RustMempool.class).getCount();
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var accepted = api.constructionSubmit(request);
      test.runUntilState(allCommittedTransactionSuccess(transaction.raw()), MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(Boolean.TRUE, refusal.getRetriable());
      assertTrue(
          refusal.getDetails().toString(),
          refusal
              .getDetails()
              .toString()
              .contains("temporarily not accepted; retry from epoch " + ENACTMENT_EPOCH));
      assertEquals(0, mempoolCountDuringMoratorium);
      assertFalse(accepted.getTransactionIdentifier().getHash().isEmpty());
      assertTrue(isCommittedOnAnyNode(test, transaction.raw()));
    }
  }

  /** The empty fallback still produces the round update needed to end the epoch. */
  @Test
  public void old_binary_leader_of_the_epoch_ending_round_cannot_delay_or_pollute_the_enactment() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      final var epochEndingRound = Round.of(ROUNDS_PER_EPOCH);
      final var oldLeader = leaderOf(test, MORATORIUM_FROM_EPOCH, epochEndingRound);
      final var upgradedNodes =
          test.getNodeIndices().stream().filter(nodeIndex -> nodeIndex != oldLeader).toList();
      final var forcedTransaction = TransactionBuilder.forTests().prepare().raw();
      test.restartNodeWithOverrideModule(oldLeader, oldBinaryEmulation(forcedTransaction));

      // Act
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var committedByEnactment = isCommittedOnAnyNode(test, forcedTransaction);
      final var enactingEpochChangeRound = epochChangeRound(test, upgradedNodes.get(0));
      final var rejectedInEpochEndingRound =
          rejectedProposals.stream()
              .anyMatch(
                  rejected ->
                      rejected.epoch() == MORATORIUM_FROM_EPOCH
                          && rejected.proposalRejected().round().equals(epochEndingRound));
      test.runUntilState(anyCommittedTransactionSuccess(forcedTransaction), MAX_MESSAGES_PER_STEP);

      // Assert
      assertFalse(committedByEnactment);
      assertTrue(rejectedProposals.toString(), rejectedInEpochEndingRound);
      assertEquals(epochEndingRound.number(), enactingEpochChangeRound);
    }
  }

  @Test
  public void relayed_user_transactions_are_discarded_during_the_moratorium_and_accepted_after() {
    try (var test = createTest(withMoratorium(MORATORIUM_FROM_EPOCH, ENACTMENT_EPOCH))) {
      // Arrange
      test.startAllNodes();
      test.runUntilState(allAtOrOverEpoch(MORATORIUM_FROM_EPOCH), MAX_MESSAGES_PER_STEP);
      final var gossiped = TransactionBuilder.forTests().prepare().raw();

      // Act
      relayFromTo(test, 0, 1, gossiped);
      final var heldByReceiver = test.getInstance(1, RustMempool.class).getCount();
      test.runUntilState(allAtOrOverProtocolVersion(EAGLE_RAY), MAX_MESSAGES_PER_STEP);
      final var committedByEnactment = isCommittedOnAnyNode(test, gossiped);
      final var heldByAnyNodeAtEnactment =
          test.getNodeIndices().stream()
              .mapToInt(nodeIndex -> test.getInstance(nodeIndex, RustMempool.class).getCount())
              .sum();
      relayFromTo(test, 0, 1, gossiped);
      test.runUntilState(allCommittedTransactionSuccess(gossiped), MAX_MESSAGES_PER_STEP);

      // Assert
      assertEquals(0, heldByReceiver);
      assertFalse(committedByEnactment);
      assertEquals(0, heldByAnyNodeAtEnactment);
    }
  }

  private DeterministicTest createTest(ProtocolConfig protocolConfig) {
    return createTest(protocolConfig, PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true));
  }

  private DeterministicTest createTest(
      ProtocolConfig protocolConfig, List<PhysicalNodeConfig> nodes) {
    return DeterministicTest.builder()
        .addPhysicalNodes(nodes)
        .addMonitors(rejectedProposalMonitor())
        .messageSelector(firstSelector())
        .functionalNodeModule(functionalNodeModule(protocolConfig));
  }

  private FunctionalRadixNodeModule functionalNodeModule(ProtocolConfig protocolConfig) {
    return new FunctionalRadixNodeModule(
        FunctionalRadixNodeModule.NodeStorageConfig.tempFolder(folder),
        true,
        FunctionalRadixNodeModule.SafetyRecoveryConfig.REAL,
        FunctionalRadixNodeModule.ConsensusConfig.testDefault(),
        FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
            StateComputerConfig.rev2()
                .withGenesis(
                    GenesisBuilder.createTestGenesisWithNumValidators(
                        NUM_VALIDATORS,
                        Decimal.ONE,
                        GenesisConsensusManagerConfig.Builder.testWithRoundsPerEpoch(
                            ROUNDS_PER_EPOCH)))
                .withProtocolConfig(protocolConfig)
                .withProposerConfig(StateComputerConfig.REV2ProposerConfig.Mempool.defaults()),
            SyncRelayConfig.of(5000, 10, 3000L)));
  }

  private Module rejectedProposalMonitor() {
    return new AbstractModule() {
      @ProvidesIntoSet
      MessageMonitor recordRejectedProposals() {
        return (message, time) -> {
          if (message.message() instanceof EpochProposalRejected rejected) {
            rejectedProposals.add(rejected);
          }
        };
      }
    };
  }

  /** Keeps proposing and voting for user transactions, as an old binary would. */
  private static Module oldBinaryEmulation(RawNotarizedTransaction proposedTransaction) {
    return new AbstractModule() {
      @Override
      protected void configure() {
        bind(ProposalGenerator.class).toInstance((round, prepared) -> List.of(proposedTransaction));
        OptionalBinder.newOptionalBinder(binder(), UserTransactionMoratoriumProvider.class)
            .setBinding()
            .toInstance(UserTransactionMoratoriumProvider.NONE);
      }
    };
  }

  /** Returns the node index of the round's leader. */
  private static int leaderOf(DeterministicTest test, long epoch, Round round) {
    final var epochProof =
        test.getInstance(0, TransactionsAndProofReader.class)
            .getLatestProofBundle()
            .orElseThrow()
            .latestProofWhichInitiatedAnEpochChange();
    final var nextEpoch = epochProof.ledgerHeader().nextEpoch().orElseThrow();
    if (nextEpoch.epoch().toLong() != epoch) {
      throw new IllegalStateException("The nodes are not at the start of epoch " + epoch);
    }
    final var validatorSet = REv2ToConsensus.validatorSet(nextEpoch.validators());
    final var leaderKey =
        ProposerElections.defaultRotation(epoch, validatorSet).getProposer(round).getKey();
    return test.getNodeIndices().stream()
        .filter(nodeIndex -> selfKey(test, nodeIndex).equals(leaderKey))
        .findFirst()
        .orElseThrow();
  }

  private static ECDSASecp256k1PublicKey selfKey(DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, Key.get(ECDSASecp256k1PublicKey.class, Self.class));
  }

  private static void relayFromTo(
      DeterministicTest test, int fromNode, int toNode, RawNotarizedTransaction transaction) {
    final var receiver = test.getInstance(toNode, Key.get(NodeId.class, Self.class));
    test.getInstance(
            fromNode, Key.get(new TypeLiteral<RemoteEventDispatcher<NodeId, MempoolAdd>>() {}))
        .dispatch(receiver, new MempoolAdd(List.of(transaction)));
    test.runUntilOutOfMessagesOfType(
        100, message -> message.message() instanceof MempoolAdd && !message.channelId().isLocal());
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

  private static Predicate<List<Injector>> nodesMatch(
      List<Integer> nodeIndices, Predicate<Injector> nodePredicate) {
    return injectors ->
        nodeIndices.stream().allMatch(index -> nodePredicate.test(injectors.get(index)));
  }

  private static long stateVersion(DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, TransactionsAndProofReader.class)
        .getLatestProofBundle()
        .orElseThrow()
        .resultantStateVersion();
  }

  private static Option<UserTransactionMoratorium> moratoriumOf(
      DeterministicTest test, int nodeIndex) {
    return test.getInstance(nodeIndex, RustStateComputer.class)
        .ensureUserTransactionsAllowed()
        .toOptionOfError();
  }

  private static ProtocolConfig withMoratorium(long moratoriumFromEpoch, long enactmentEpoch) {
    return ProtocolConfig.enactAtEpochWithUserTransactionMoratorium(
        EAGLE_RAY, moratoriumFromEpoch, enactmentEpoch);
  }

  private static ProtocolConfig withoutMoratorium() {
    return ProtocolConfig.enactAtEpoch(EAGLE_RAY, 1000);
  }

  private static void submit(DeterministicTest test, RawNotarizedTransaction transaction) {
    final var mempoolDispatcher =
        test.getInstance(0, Key.get(new TypeLiteral<EventDispatcher<MempoolAdd>>() {}));
    mempoolDispatcher.dispatch(new MempoolAdd(List.of(transaction)));
    test.runUntilOutOfMessagesOfType(100, onlyLocalMempoolAddEvents());
  }

  private static boolean isCommittedOnAnyNode(
      DeterministicTest test, RawNotarizedTransaction transaction) {
    return test.getNodeInjectors().stream()
        .anyMatch(NodePredicate.committedUserTransaction(transaction, false, false));
  }

  private static long currentEpoch(DeterministicTest test) {
    return test.getInstance(0, TransactionsAndProofReader.class)
        .getLatestProofBundle()
        .orElseThrow()
        .resultantEpoch();
  }

  private static List<Option<UserTransactionMoratorium>> moratoria(DeterministicTest test) {
    return test.getNodeInjectors().stream()
        .map(
            injector ->
                injector
                    .getInstance(RustStateComputer.class)
                    .ensureUserTransactionsAllowed()
                    .toOptionOfError())
        .toList();
  }

  private static List<Option<UserTransactionMoratorium>> moratoriumOnEveryNode(
      long fromEpoch, long enactmentEpoch) {
    final var moratorium =
        new UserTransactionMoratorium(
            UInt64.fromNonNegativeLong(fromEpoch), UInt64.fromNonNegativeLong(enactmentEpoch));
    return java.util.Collections.nCopies(NUM_VALIDATORS, Option.some(moratorium));
  }

  private static List<Option<UserTransactionMoratorium>> noMoratoriumOnEveryNode() {
    return java.util.Collections.nCopies(NUM_VALIDATORS, Option.empty());
  }
}
