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

package com.radixdlt.integration.steady_state.deterministic.rev2;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.harness.deterministic.invariants.DeterministicMonitors.*;

import com.google.inject.Key;
import com.google.inject.TypeLiteral;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.NodesReader;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.harness.invariants.Checkers;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.*;
import com.radixdlt.statemanager.REv2DatabaseConfig;
import com.radixdlt.sync.SyncRelayConfig;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt64;
import java.util.HashMap;
import java.util.Random;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class RandomValidatorsTest {
  private static final int NUM_VALIDATORS = 30;
  private static final RawLedgerTransaction GENESIS =
      TransactionBuilder.createGenesisWithNumValidators(
          NUM_VALIDATORS / 2, Decimal.of(1000), UInt64.fromNonNegativeLong(10));

  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest() {
    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(NUM_VALIDATORS, true))
        .messageSelector(firstSelector())
        .addMonitors(
            byzantineBehaviorNotDetected(), consensusLiveness(3000), ledgerTransactionSafety())
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.berkeleyStore(
                    folder.getRoot().getAbsolutePath()),
                FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerWithSyncRelay(
                    StateComputerConfig.rev2(
                        Network.INTEGRATIONTESTNET.getId(),
                        GENESIS,
                        REv2DatabaseConfig.rocksDB(folder.getRoot().getAbsolutePath()),
                        StateComputerConfig.REV2ProposerConfig.mempool(
                            10, 10 * 1024 * 1024, 100, MempoolRelayConfig.of(5, 5))),
                    SyncRelayConfig.of(5000, 10, 3000L))));
  }

  @Test
  public void normal_run_should_not_cause_unexpected_errors() {
    try (var test = createTest()) {
      test.startAllNodes();

      var random = new Random(12345);

      var creating_validators = new HashMap<Integer, RawNotarizedTransaction>();
      var validators = new HashMap<Integer, ComponentAddress>();
      var genesis = NodesReader.getCommittedLedgerTransaction(test.getNodeInjectors(), GENESIS);
      var componentAddresses =
          genesis.newComponentAddresses().stream()
              .filter(componentAddress -> componentAddress.value()[0] == 5)
              .toList();
      for (int i = 0; i < NUM_VALIDATORS / 2; i++) {
        validators.put(i, componentAddresses.get(i));
      }

      // Run
      for (int i = 0; i < 100; i++) {
        test.runForCount(1000);

        var mempoolDispatcher =
            test.getInstance(
                random.nextInt(0, NUM_VALIDATORS),
                Key.get(new TypeLiteral<EventDispatcher<MempoolAdd>>() {}));

        var randomValidatorIndex = random.nextInt(0, NUM_VALIDATORS);
        var componentAddress = validators.get(randomValidatorIndex);
        if (componentAddress == null) {
          var inflightTransaction = creating_validators.get(randomValidatorIndex);
          if (inflightTransaction == null) {
            var txn =
                REv2TestTransactions.constructCreateValidatorTransaction(
                    NetworkDefinition.INT_TEST_NET,
                    0,
                    random.nextInt(1000000),
                    PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            creating_validators.put(randomValidatorIndex, txn);
            mempoolDispatcher.dispatch(MempoolAdd.create(txn));
          } else {
            var maybeExecuted =
                NodesReader.tryGetCommittedUserTransaction(
                    test.getNodeInjectors().get(randomValidatorIndex), inflightTransaction);
            maybeExecuted.ifPresent(
                executedTransaction -> {
                  var validatorAddress = executedTransaction.newComponentAddresses().get(0);
                  test.restartNodeWithConfig(
                      randomValidatorIndex,
                      PhysicalNodeConfig.create(
                          PrivateKeys.ofNumeric(randomValidatorIndex + 1).getPublicKey(),
                          validatorAddress));
                  validators.put(randomValidatorIndex, validatorAddress);
                  creating_validators.remove(randomValidatorIndex);
                });
          }
        } else {
          final RawNotarizedTransaction txn;
          switch (random.nextInt(0, 5)) {
            case 0 -> {
              txn =
                  REv2TestTransactions.constructRegisterValidatorTransaction(
                      NetworkDefinition.INT_TEST_NET,
                      0,
                      random.nextInt(1000000),
                      componentAddress,
                      PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            }
            case 1 -> {
              txn =
                  REv2TestTransactions.constructUnregisterValidatorTransaction(
                      NetworkDefinition.INT_TEST_NET,
                      0,
                      random.nextInt(1000000),
                      componentAddress,
                      PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            }
            case 2 -> {
              txn =
                  REv2TestTransactions.constructStakeValidatorTransaction(
                      NetworkDefinition.INT_TEST_NET,
                      0,
                      random.nextInt(1000000),
                      componentAddress,
                      PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            }
            case 3 -> {
              var stateReader = test.getInstance(randomValidatorIndex, REv2StateReader.class);
              var validatorInfo = stateReader.getValidatorInfo(componentAddress);
              txn =
                  REv2TestTransactions.constructUnstakeValidatorTransaction(
                      NetworkDefinition.INT_TEST_NET,
                      0,
                      random.nextInt(1000000),
                      validatorInfo.lpTokenAddress(),
                      componentAddress,
                      PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            }
            default -> {
              var stateReader = test.getInstance(randomValidatorIndex, REv2StateReader.class);
              var validatorInfo = stateReader.getValidatorInfo(componentAddress);
              txn =
                  REv2TestTransactions.constructClaimXrdTransaction(
                      NetworkDefinition.INT_TEST_NET,
                      0,
                      random.nextInt(1000000),
                      componentAddress,
                      validatorInfo.unstakeResource(),
                      PrivateKeys.ofNumeric(randomValidatorIndex + 1));
            }
          }
          mempoolDispatcher.dispatch(MempoolAdd.create(txn));
        }
      }

      // Post-run assertions
      Checkers.assertNodesSyncedToVersionAtleast(test.getNodeInjectors(), 20);
      Checkers.assertNoInvalidSyncResponses(test.getNodeInjectors());
    }
  }
}
