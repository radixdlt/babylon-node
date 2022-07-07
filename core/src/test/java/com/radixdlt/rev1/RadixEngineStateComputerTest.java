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

package com.radixdlt.rev1;

import static com.radixdlt.substate.TxAction.*;
import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import com.google.common.collect.ImmutableList;
import com.google.inject.AbstractModule;
import com.google.inject.Guice;
import com.google.inject.Inject;
import com.google.inject.Module;
import com.google.inject.TypeLiteral;
import com.radixdlt.application.system.state.RoundData;
import com.radixdlt.application.tokens.Amount;
import com.radixdlt.consensus.BFTHeader;
import com.radixdlt.consensus.LedgerHeader;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.Sha256Hasher;
import com.radixdlt.consensus.TimestampedECDSASignatures;
import com.radixdlt.consensus.Vertex;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.constraintmachine.PermissionLevel;
import com.radixdlt.constraintmachine.REEvent;
import com.radixdlt.constraintmachine.exceptions.ConstraintMachineException;
import com.radixdlt.constraintmachine.exceptions.InvalidPermissionException;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.engine.RadixEngine;
import com.radixdlt.engine.RadixEngineException;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.ledger.ByzantineQuorumException;
import com.radixdlt.ledger.CommittedTransactionsWithProof;
import com.radixdlt.ledger.LedgerAccumulator;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.ledger.NoOpCommittedReader;
import com.radixdlt.ledger.SimpleLedgerAccumulatorAndVerifier;
import com.radixdlt.ledger.StateComputerLedger.StateComputerResult;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolAddSuccess;
import com.radixdlt.mempool.MempoolConfig;
import com.radixdlt.mempool.MempoolRelayTrigger;
import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.rev1.checkpoint.Genesis;
import com.radixdlt.rev1.checkpoint.MockedGenesisModule;
import com.radixdlt.rev1.forks.CurrentForkView;
import com.radixdlt.rev1.forks.ForksEpochStore;
import com.radixdlt.rev1.forks.ForksModule;
import com.radixdlt.rev1.forks.MainnetForksModule;
import com.radixdlt.rev1.forks.NoOpForksEpochStore;
import com.radixdlt.rev1.forks.RadixEngineForksLatestOnlyModule;
import com.radixdlt.rev1.modules.RadixEngineModule;
import com.radixdlt.rev1.modules.RadixEngineStateComputerModule;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.store.EngineStore;
import com.radixdlt.store.InMemoryEngineStore;
import com.radixdlt.substate.*;
import com.radixdlt.sync.CommittedReader;
import com.radixdlt.transactions.Transaction;
import com.radixdlt.utils.RandomHasher;
import com.radixdlt.utils.TypedMocks;
import com.radixdlt.utils.UInt256;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.assertj.core.api.Condition;
import org.junit.Before;
import org.junit.Ignore;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public class RadixEngineStateComputerTest {
  @Rule public TemporaryFolder folder = new TemporaryFolder();

  @Inject @Genesis private CommittedTransactionsWithProof genesisTxns;

  @Inject private RadixEngine<LedgerAndBFTProof> radixEngine;

  @Inject private RadixEngineStateComputer sut;

  @Inject private CurrentForkView currentForkView;

  @Inject private ProposerElection proposerElection;

  @Inject private EventDispatcher<MempoolAddSuccess> mempoolAddSuccessEventDispatcher;

  private Serialization serialization = DefaultSerialization.getInstance();
  private InMemoryEngineStore<LedgerAndBFTProof> engineStore;
  private ImmutableList<ECKeyPair> registeredNodes =
      ImmutableList.of(ECKeyPair.generateNew(), ECKeyPair.generateNew());
  private ECKeyPair unregisteredNode = ECKeyPair.generateNew();

  private static final Hasher hasher = new Sha256Hasher(DefaultSerialization.getInstance());

  private Module getExternalModule() {
    return new AbstractModule() {
      @Override
      public void configure() {
        var validatorSet =
            BFTValidatorSet.from(
                registeredNodes.stream()
                    .map(ECKeyPair::getPublicKey)
                    .map(BFTNode::create)
                    .map(n -> BFTValidator.from(n, UInt256.ONE)));

        bind(ProposerElection.class).toInstance(new WeightedRotatingLeaders(validatorSet));
        bind(Serialization.class).toInstance(serialization);
        bind(Hasher.class).toInstance(new Sha256Hasher(DefaultSerialization.getInstance()));
        bind(new TypeLiteral<EngineStore<LedgerAndBFTProof>>() {}).toInstance(engineStore);
        bind(PersistentVertexStore.class).toInstance(mock(PersistentVertexStore.class));

        install(MempoolConfig.asModule(10, 10));
        install(new MainnetForksModule());
        install(new RadixEngineForksLatestOnlyModule());
        install(new ForksModule());

        // HACK
        bind(CommittedReader.class).toInstance(new NoOpCommittedReader());
        bind(ForksEpochStore.class).toInstance(new NoOpForksEpochStore());

        bind(LedgerAccumulator.class).to(SimpleLedgerAccumulatorAndVerifier.class);

        bind(new TypeLiteral<EventDispatcher<MempoolAddSuccess>>() {})
            .toInstance(TypedMocks.rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<InvalidProposedTransaction>>() {})
            .toInstance(TypedMocks.rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<REOutput>>() {})
            .toInstance(TypedMocks.rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<MempoolRelayTrigger>>() {})
            .toInstance(TypedMocks.rmock(EventDispatcher.class));
        bind(new TypeLiteral<EventDispatcher<LedgerUpdate>>() {})
            .toInstance(TypedMocks.rmock(EventDispatcher.class));

        bind(SystemCounters.class).to(SystemCountersImpl.class);
      }
    };
  }

  private void setupGenesis() throws RadixEngineException {
    var branch = radixEngine.transientBranch();
    var result = branch.execute(genesisTxns.getTransactions(), PermissionLevel.SYSTEM);
    var genesisValidatorSet =
        result.getProcessedTxns().get(0).getEvents().stream()
            .filter(REEvent.NextValidatorSetEvent.class::isInstance)
            .map(REEvent.NextValidatorSetEvent.class::cast)
            .findFirst()
            .map(
                e ->
                    BFTValidatorSet.from(
                        e.nextValidators().stream()
                            .map(
                                v ->
                                    BFTValidator.from(
                                        BFTNode.create(v.validatorKey()), v.amount()))))
            .orElseThrow(() -> new IllegalStateException("No validator set in genesis."));
    radixEngine.deleteBranches();

    var genesisLedgerHeader =
        LedgerProof.genesis(
            new AccumulatorState(
                0, hasher.hashDsonEncoded(genesisTxns.getTransactions().get(0).getId())),
            genesisValidatorSet,
            0);
    if (!genesisLedgerHeader.isEndOfEpoch()) {
      throw new IllegalStateException("Genesis must be end of epoch");
    }
    radixEngine.execute(
        genesisTxns.getTransactions(),
        LedgerAndBFTProof.create(genesisLedgerHeader),
        PermissionLevel.SYSTEM);
  }

  @Before
  public void setup() throws RadixEngineException {
    this.engineStore = new InMemoryEngineStore<>();
    Guice.createInjector(
            new RadixEngineStateComputerModule(),
            new RadixEngineModule(),
            new MockedGenesisModule(
                registeredNodes.stream().map(ECKeyPair::getPublicKey).collect(Collectors.toSet()),
                Amount.ofTokens(1000),
                Amount.ofTokens(100)),
            getExternalModule())
        .injectMembers(this);
    setupGenesis();
  }

  private Transaction systemUpdateTransaction(long nextRound, long nextEpoch)
      throws TxBuilderException {
    TxBuilder builder;
    if (nextEpoch >= 2) {
      var request =
          TxnConstructionRequest.create()
              .action(
                  new NextRound(
                      10, true, 0, v -> proposerElection.getProposer(Round.of(v)).getKey()))
              .action(new NextEpoch(0));
      builder = radixEngine.construct(request);
    } else {
      builder =
          radixEngine.construct(
              new NextRound(nextRound, false, 0, i -> registeredNodes.get(0).getPublicKey()));
    }

    return builder.buildWithoutSignature();
  }

  private Transaction registerTransaction(ECKeyPair keyPair) throws TxBuilderException {
    return radixEngine
        .construct(new RegisterValidator(keyPair.getPublicKey()))
        .signAndBuild(keyPair::sign);
  }

  @Test
  @Ignore("Ignore for now given need for more refactoring to get this test to work")
  public void executing_non_epoch_max_round_should_return_no_validator_set() {
    // Arrange
    var v = Vertex.create(mock(QuorumCertificate.class), Round.of(9), List.of(), BFTNode.random());
    var vertex = v.withId(RandomHasher.INSTANCE);

    // Action
    var result = sut.prepare(List.of(), vertex, 0);

    // Assert
    assertThat(result.getSuccessfullyExecutedTransactions()).hasSize(1);
    assertThat(result.getFailedTransactions()).isEmpty();
    assertThat(result.getNextValidatorSet()).isEmpty();
  }

  @Test
  public void executing_epoch_max_round_should_return_next_validator_set() {
    // Arrange
    var qc = mock(QuorumCertificate.class);
    var parentHeader = mock(BFTHeader.class);
    when(parentHeader.getRound()).thenReturn(Round.of(0));
    when(qc.getProposed()).thenReturn(parentHeader);
    var unverified = Vertex.create(qc, Round.of(11), List.of(), BFTNode.random());
    var vertex = unverified.withId(RandomHasher.INSTANCE);

    // Act
    StateComputerResult result = sut.prepare(List.of(), vertex, 0);

    // Assert
    assertThat(result.getSuccessfullyExecutedTransactions()).hasSize(1);
    assertThat(result.getFailedTransactions()).isEmpty();
    assertThat(result.getNextValidatorSet())
        .hasValueSatisfying(
            set ->
                assertThat(set.getValidators())
                    .isNotEmpty()
                    .allMatch(
                        v ->
                            v.getNode().getKey().equals(unregisteredNode.getPublicKey())
                                || registeredNodes.stream()
                                    .anyMatch(k -> k.getPublicKey().equals(v.getNode().getKey()))));
  }

  @Test
  public void executing_epoch_max_round_with_register_should_not_return_new_next_validator_set()
      throws Exception {
    // Arrange
    ECKeyPair keyPair = ECKeyPair.generateNew();
    var txn = registerTransaction(keyPair);
    BFTNode node = BFTNode.create(keyPair.getPublicKey());
    var qc = mock(QuorumCertificate.class);
    var parentHeader = mock(BFTHeader.class);
    when(parentHeader.getRound()).thenReturn(Round.of(0));
    when(qc.getProposed()).thenReturn(parentHeader);
    var vertex =
        Vertex.create(qc, Round.of(11), List.of(txn), BFTNode.random())
            .withId(RandomHasher.INSTANCE);

    // Act
    StateComputerResult result = sut.prepare(List.of(), vertex, 0);

    // Assert
    assertThat(result.getSuccessfullyExecutedTransactions())
        .hasSize(1); // since max round, transaction is not executed
    assertThat(result.getNextValidatorSet())
        .hasValueSatisfying(
            s -> {
              assertThat(s.getValidators()).hasSize(2);
              assertThat(s.getValidators()).extracting(BFTValidator::getNode).doesNotContain(node);
            });
  }

  @Test
  public void preparing_system_update_from_vertex_should_fail() throws TxBuilderException {
    // Arrange
    var txn =
        radixEngine
            .construct(
                new NextRound(1, false, 0, i -> proposerElection.getProposer(Round.of(i)).getKey()))
            .buildWithoutSignature();
    var illegalTxn =
        TxLowLevelBuilder.newBuilder(
                currentForkView.currentForkConfig().engineRules().serialization())
            .down(SubstateId.ofSubstate(txn.getId(), 1))
            .up(new RoundData(2, 0))
            .end()
            .build();
    var v =
        Vertex.create(
            mock(QuorumCertificate.class),
            Round.of(1),
            List.of(illegalTxn),
            proposerElection.getProposer(Round.of(1)));
    var vertex = v.withId(RandomHasher.INSTANCE);

    // Act
    var result = sut.prepare(ImmutableList.of(), vertex, 0);

    // Assert
    assertThat(result.getSuccessfullyExecutedTransactions()).hasSize(1);
    assertThat(result.getFailedTransactions())
        .hasValueSatisfying(
            new Condition<>(
                e -> {
                  var ex = (RadixEngineException) e;
                  var cmException = (ConstraintMachineException) ex.getCause();
                  return cmException.getCause() instanceof InvalidPermissionException;
                },
                "Is invalid_execution_permission error"));
  }

  // TODO: should catch this and log it somewhere as proof of byzantine quorum
  @Test
  // Note that checking upper bound round for epoch now requires additional
  // state not easily obtained where checked in the RadixEngine
  @Ignore("FIXME: Reinstate when upper bound on epoch round is in place.")
  public void committing_epoch_max_rounds_should_fail() throws TxBuilderException {
    // Arrange
    var cmd0 = systemUpdateTransaction(10, 1);
    var ledgerProof =
        new LedgerProof(
            HashUtils.random256(),
            LedgerHeader.create(0, Round.of(11), new AccumulatorState(3, HashUtils.zero256()), 0),
            new TimestampedECDSASignatures());
    var transactionsWithProof =
        CommittedTransactionsWithProof.create(ImmutableList.of(cmd0), ledgerProof);

    // Act
    // Assert
    assertThatThrownBy(() -> sut.commit(transactionsWithProof, null))
        .isInstanceOf(ByzantineQuorumException.class);
  }

  // TODO: should catch this and log it somewhere as proof of byzantine quorum
  @Test
  public void committing_epoch_change_with_additional_cmds_should_fail() throws Exception {
    // Arrange
    var keyPair = ECKeyPair.generateNew();
    var cmd0 = systemUpdateTransaction(0, 2);
    var cmd1 = registerTransaction(keyPair);
    var ledgerProof =
        new LedgerProof(
            HashUtils.random256(),
            LedgerHeader.create(0, Round.of(9), new AccumulatorState(3, HashUtils.zero256()), 0),
            new TimestampedECDSASignatures());
    var transactionsWithProof =
        CommittedTransactionsWithProof.create(ImmutableList.of(cmd0, cmd1), ledgerProof);

    // Act
    // Assert
    assertThatThrownBy(() -> sut.commit(transactionsWithProof, null))
        .isInstanceOf(ByzantineQuorumException.class);
  }

  // TODO: should catch this and log it somewhere as proof of byzantine quorum
  @Test
  public void committing_epoch_change_with_different_validator_signed_should_fail()
      throws Exception {
    // Arrange
    var cmd1 = systemUpdateTransaction(0, 2);
    var ledgerProof =
        new LedgerProof(
            HashUtils.random256(),
            LedgerHeader.create(
                0,
                Round.of(9),
                new AccumulatorState(3, HashUtils.zero256()),
                0,
                BFTValidatorSet.from(Stream.of(BFTValidator.from(BFTNode.random(), UInt256.ONE)))),
            new TimestampedECDSASignatures());
    var transactionsWithProof =
        CommittedTransactionsWithProof.create(ImmutableList.of(cmd1), ledgerProof);

    // Act
    // Assert
    assertThatThrownBy(() -> sut.commit(transactionsWithProof, null))
        .isInstanceOf(ByzantineQuorumException.class);
  }

  // TODO: should catch this and log it somewhere as proof of byzantine quorum
  @Test
  public void committing_epoch_change_when_there_shouldnt_be_one__should_fail()
      throws TxBuilderException {
    // Arrange
    var cmd0 = systemUpdateTransaction(1, 1);
    var ledgerProof =
        new LedgerProof(
            HashUtils.random256(),
            LedgerHeader.create(
                0,
                Round.of(9),
                new AccumulatorState(3, HashUtils.zero256()),
                0,
                BFTValidatorSet.from(Stream.of(BFTValidator.from(BFTNode.random(), UInt256.ONE)))),
            new TimestampedECDSASignatures());
    var transactionsWithProof =
        CommittedTransactionsWithProof.create(ImmutableList.of(cmd0), ledgerProof);

    // Act
    // Assert
    assertThatThrownBy(() -> sut.commit(transactionsWithProof, null))
        .isInstanceOf(ByzantineQuorumException.class);
  }

  @Test
  public void add_to_mempool__should_forward_the_origin_to_the_event() throws TxBuilderException {
    // Arrange
    final var origin = BFTNode.random();
    var txn = registerTransaction(ECKeyPair.generateNew());

    // Act
    sut.addToMempool(MempoolAdd.create(txn), origin);

    // Assert
    verify(mempoolAddSuccessEventDispatcher)
        .dispatch(
            argThat(ev -> ev.getOrigin().orElseThrow().equals(origin) && ev.getTxn().equals(txn)));
  }
}
