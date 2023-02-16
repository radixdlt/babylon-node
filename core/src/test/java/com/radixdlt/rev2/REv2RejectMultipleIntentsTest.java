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

package com.radixdlt.rev2;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.harness.invariants.Checkers.assertOneTransactionCommittedOutOf;
import static com.radixdlt.harness.predicates.EventPredicate.onlyConsensusEvents;
import static com.radixdlt.harness.predicates.NodesPredicate.*;
import static org.assertj.core.api.Assertions.assertThat;

import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.liveness.ProposalGenerator;
import com.radixdlt.consensus.vertexstore.ExecutedVertex;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.FunctionalRadixNodeModule.ConsensusConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.LedgerConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule.SafetyRecoveryConfig;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.modules.StateComputerConfig.REV2ProposerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt64;
import java.util.List;
import java.util.stream.IntStream;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class REv2RejectMultipleIntentsTest {
  private static final ECKeyPair NOTARY = PrivateKeys.ofNumeric(1);
  private static final NetworkDefinition NETWORK_DEFINITION = NetworkDefinition.INT_TEST_NET;

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(ProposalGenerator proposalGenerator) {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
        .messageSelector(firstSelector())
        .messageMutator(MessageMutator.dropTimeouts())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                false,
                SafetyRecoveryConfig.berkeleyStore(folder.getRoot().getAbsolutePath()),
                ConsensusConfig.of(1000),
                LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2(
                        Network.INTEGRATIONTESTNET.getId(),
                        TransactionBuilder.createGenesisWithNumValidators(
                            1, Decimal.of(1), UInt64.fromNonNegativeLong(10)),
                        REv2DatabaseConfig.rocksDB(folder.getRoot().getAbsolutePath()),
                        REV2ProposerConfig.transactionGenerator(proposalGenerator)))));
  }

  private static byte[] createValidIntentBytes(long nonce) {
    return REv2TestTransactions.constructValidIntentBytes(
        NETWORK_DEFINITION, 0, nonce, NOTARY.getPublicKey().toPublicKey());
  }

  private static RawNotarizedTransaction createValidTransactionWithSigs(
      byte[] intentBytes, int sigsCount) {
    var keys = IntStream.rangeClosed(1, sigsCount).mapToObj(PrivateKeys::ofNumeric).toList();
    return REv2TestTransactions.constructRawTransaction(intentBytes, NOTARY, keys);
  }

  private static class ControlledProposerGenerator implements ProposalGenerator {
    private List<RawNotarizedTransaction> nextTransactions = null;

    @Override
    public List<RawNotarizedTransaction> getTransactionsForProposal(
        Round round, List<ExecutedVertex> prepared) {
      if (nextTransactions == null) {
        return List.of();
      } else {
        var txns = nextTransactions;
        this.nextTransactions = null;
        return txns;
      }
    }
  }

  @Test
  public void duplicate_intents_are_not_committed() {
    var proposalGenerator = new ControlledProposerGenerator();

    try (var test = createTest(proposalGenerator)) {
      var fixedIntent1 = createValidIntentBytes(1);
      var fixedIntent2 = createValidIntentBytes(2);
      var fixedIntent3 = createValidIntentBytes(3);

      // Signatures aren't deterministic so create signed transactions up front
      var fixedIntent1Transactions =
          List.of(
              createValidTransactionWithSigs(fixedIntent1, 0),
              createValidTransactionWithSigs(fixedIntent1, 1),
              createValidTransactionWithSigs(fixedIntent1, 2),
              createValidTransactionWithSigs(fixedIntent1, 3),
              createValidTransactionWithSigs(fixedIntent1, 4),
              createValidTransactionWithSigs(fixedIntent1, 5));
      var fixedIntent2Transactions =
          List.of(
              createValidTransactionWithSigs(fixedIntent2, 0),
              createValidTransactionWithSigs(fixedIntent2, 3));
      var fixedIntent3Transactions = List.of(createValidTransactionWithSigs(fixedIntent3, 0));

      // Different payloads all with fixed intent 1 - only one of these should be committed
      var transactionsForFirstProposal = fixedIntent1Transactions.stream().limit(4).toList();

      // Mix of payloads with fixed intent 1, 2 and 3 - fixed intent 2 and 3 should be committed
      var transactionsForSecondProposal =
          List.of(
              fixedIntent2Transactions.get(1),
              fixedIntent1Transactions.get(0), // Exact payload repeat
              fixedIntent1Transactions.get(5),
              fixedIntent3Transactions.get(0),
              fixedIntent2Transactions.get(0));

      // Prepare - let's start the test
      test.startAllNodes();

      // Act: Submit proposal 1 transactions in a proposal and run consensus
      proposalGenerator.nextTransactions = transactionsForFirstProposal;
      test.runUntilState(ignored -> proposalGenerator.nextTransactions == null);
      test.runUntilState(
          allCommittedTransaction(transactionsForFirstProposal.get(0)), onlyConsensusEvents());

      // Assert: Check transaction and post submission state
      assertThat(proposalGenerator.nextTransactions).isNull();

      // Act 2: Submit proposal 2 transactions in a proposal and run consensus
      proposalGenerator.nextTransactions = transactionsForSecondProposal;
      test.runUntilState(ignored -> proposalGenerator.nextTransactions == null);
      test.runUntilState(
          allCommittedTransaction(transactionsForSecondProposal.get(0)), onlyConsensusEvents());

      // Assert: Check transaction and post submission state
      assertThat(proposalGenerator.nextTransactions).isNull();
      // Verify that only one transaction of some intent was committed
      assertOneTransactionCommittedOutOf(test.getNodeInjectors(), fixedIntent1Transactions);
      assertOneTransactionCommittedOutOf(test.getNodeInjectors(), fixedIntent2Transactions);
      assertOneTransactionCommittedOutOf(test.getNodeInjectors(), fixedIntent3Transactions);
    }
  }
}
