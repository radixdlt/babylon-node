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

import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransactionSuccess;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;

import com.google.common.collect.Streams;
import com.google.inject.AbstractModule;
import com.google.inject.Key;
import com.google.inject.Module;
import com.google.inject.TypeLiteral;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.api.CoreApiServer;
import com.radixdlt.api.CoreApiServerModule;
import com.radixdlt.api.core.generated.client.ApiClient;
import com.radixdlt.api.core.generated.models.CommittedTransaction;
import com.radixdlt.api.core.generated.models.DeletedSubstate;
import com.radixdlt.api.core.generated.models.FlashLedgerTransaction;
import com.radixdlt.api.core.generated.models.FlashSetSubstate;
import com.radixdlt.consensus.BFTConfiguration;
import com.radixdlt.environment.CoreApiServerFlags;
import com.radixdlt.environment.EventDispatcher;
import com.radixdlt.environment.StartProcessorOnRunner;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Option;
import com.radixdlt.mempool.MempoolAdd;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.Manifest;
import com.radixdlt.rev2.TransactionBuilder;
import com.radixdlt.state.RustStateReader;
import com.radixdlt.statecomputer.commit.NextEpoch;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.utils.FreePortFinder;
import com.radixdlt.utils.PrivateKeys;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Collectors;

public final class ProtocolUpdateTestUtils {
  public static long runUntilNextEpoch(DeterministicTest test) {
    final var store = test.getInstance(0, TransactionsAndProofReader.class);
    final var latestProof = store.getLatestProofBundle().orElseThrow();
    final var currentEpoch = latestProof.resultantEpoch();
    final var nextEpoch = currentEpoch + 1;
    test.runUntilState(allAtOrOverEpoch(nextEpoch));
    return nextEpoch;
  }

  public static void signalReadinessAndRunUntilCommit(
      DeterministicTest test, int validatorIdx, String readinessSignalName) {
    final var mempoolDispatcher =
        test.getInstance(0, Key.get(new TypeLiteral<EventDispatcher<MempoolAdd>>() {}));
    final var validatorKey = PrivateKeys.numeric(1).skip(validatorIdx).findFirst().orElseThrow();
    final var initialValidatorSet = test.getInstance(0, BFTConfiguration.class).getValidatorSet();
    final var ownerAddress = Address.virtualAccountAddress(validatorKey.getPublicKey());
    final var validatorAddress =
        initialValidatorSet.getValidators().stream()
            .filter(v -> v.getValidatorId().getKey().equals(validatorKey.getPublicKey()))
            .findFirst()
            .orElseThrow()
            .getValidatorId()
            .getValidatorAddress();
    final var signalReadinessTransaction =
        TransactionBuilder.forTests()
            .manifest(
                Manifest.validatorSignalProtocolUpdateReadiness(
                    validatorAddress, ownerAddress, readinessSignalName))
            .notary(validatorKey)
            .notaryIsSignatory(true)
            .prepare()
            .raw();
    mempoolDispatcher.dispatch(new MempoolAdd(List.of(signalReadinessTransaction)));
    test.runUntilState(allCommittedTransactionSuccess(signalReadinessTransaction));
    // Check that the state reader returns a correct value
    test.getNodeInjectors()
        .forEach(
            injector -> {
              final var valueFromStateReader =
                  injector
                      .getInstance(RustStateReader.class)
                      .getValidatorProtocolUpdateReadinessSignal(validatorAddress);
              assertEquals(readinessSignalName, valueFromStateReader.orElse(""));
            });
  }

  public static void verifyProtocolUpdateAtEpoch(
      DeterministicTest test, long epoch, String expectedProtocolVersion) {
    test.getNodeInjectors()
        .forEach(
            injector -> {
              final var store = injector.getInstance(REv2TransactionAndProofStore.class);
              final var proof = store.getEpochProof(epoch).orElseThrow();
              if (!proof
                  .ledgerHeader()
                  .nextProtocolVersion()
                  .equals(Option.some(expectedProtocolVersion))) {
                fail(
                    String.format(
                        "Expected protocol update to %s at epoch %s",
                        expectedProtocolVersion, epoch));
              }
            });
  }

  public static void verifyNoProtocolUpdateAtEpoch(DeterministicTest test, long epoch) {
    test.getNodeInjectors()
        .forEach(
            injector -> {
              final var store = injector.getInstance(REv2TransactionAndProofStore.class);
              final var proof = store.getEpochProof(epoch).orElseThrow();
              if (proof.ledgerHeader().nextProtocolVersion().isPresent()) {
                fail(String.format("Unexpected protocol update at epoch %s", epoch));
              }
            });
  }

  public static void verifyCurrentEpochReadiness(
      DeterministicTest test, Function<Map<String, Decimal>, Boolean> verifyFn) {
    final var store = test.getInstance(0, REv2TransactionAndProofStore.class);
    final var latestProofHeader = store.getLatestProof().orElseThrow().ledgerHeader();
    final var currentEpoch =
        latestProofHeader.nextEpoch().map(NextEpoch::epoch).or(latestProofHeader.epoch());
    verifyReadinessAtEpoch(test, currentEpoch.toLong(), verifyFn);
  }

  public static void verifyReadinessAtEpoch(
      DeterministicTest test, long epoch, Function<Map<String, Decimal>, Boolean> verifyFn) {
    test.getNodeInjectors()
        .forEach(
            injector -> {
              final var readiness =
                  injector
                      .getInstance(REv2TransactionAndProofStore.class)
                      .getSignificantProtocolUpdateReadinessForEpoch(epoch)
                      .orElseThrow();
              if (!verifyFn.apply(readiness)) {
                fail(
                    String.format(
                        "Protocol update readiness at epoch %s (= %s) does not match expectations",
                        epoch, readiness));
              }
            });
  }

  public static void verifyFlashTransactionReceipts(
      List<CommittedTransaction> committedFlashTransactions) {
    final var flashStateUpdates =
        committedFlashTransactions.stream()
            .map(txn -> (FlashLedgerTransaction) txn.getLedgerTransaction())
            .map(FlashLedgerTransaction::getFlashedStateUpdates)
            .toList();
    final var receiptStateUpdates =
        committedFlashTransactions.stream().map(txn -> txn.getReceipt().getStateUpdates()).toList();
    Streams.forEachPair(
        flashStateUpdates.stream(),
        receiptStateUpdates.stream(),
        (fromFlash, fromReceipt) -> {
          // all deleted partitions specified by flash were really deleted:
          assertEquals(fromFlash.getDeletedPartitions(), fromReceipt.getDeletedPartitions());

          // substate values set by flash transactions end up as the receipt's created + updated:
          final var setFromFlash =
              fromFlash.getSetSubstates().stream()
                  .collect(
                      Collectors.toMap(
                          FlashSetSubstate::getSubstateId, FlashSetSubstate::getValue));
          final var setFromReceipt =
              Streams.concat(
                      fromReceipt.getCreatedSubstates().stream()
                          .map(create -> Map.entry(create.getSubstateId(), create.getValue())),
                      fromReceipt.getUpdatedSubstates().stream()
                          .map(update -> Map.entry(update.getSubstateId(), update.getNewValue())))
                  .collect(Collectors.toMap(Map.Entry::getKey, Map.Entry::getValue));
          assertEquals(setFromFlash, setFromReceipt);

          // and the same for deletes:
          final var deletedFromReceipt =
              fromReceipt.getDeletedSubstates().stream()
                  .map(DeletedSubstate::getSubstateId)
                  .toList();
          assertEquals(fromFlash.getDeletedSubstates(), deletedFromReceipt);
        });
  }

  public static class CoreApiHelper {

    private final int coreApiPort;

    public CoreApiHelper() {
      this.coreApiPort = FreePortFinder.findFreeLocalPort();
    }

    public Module module() {
      return new AbstractModule() {
        @Override
        protected void configure() {
          install(new CoreApiServerModule("127.0.0.1", coreApiPort, new CoreApiServerFlags(true)));
        }

        @ProvidesIntoSet
        private StartProcessorOnRunner startCoreApi(CoreApiServer coreApiServer) {
          // This is a slightly hacky way to run something on node start-up in a Deterministic test.
          // Stop is called by the AutoClosable binding in CoreApiServerModule
          return new StartProcessorOnRunner("coreApi", coreApiServer::start);
        }
      };
    }

    public ApiClient client() {
      final var apiClient = new ApiClient();
      apiClient.updateBaseUri("http://127.0.0.1:" + coreApiPort + "/core");
      return apiClient;
    }
  }
}
