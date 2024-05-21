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

package com.radixdlt.api.core;

import static com.radixdlt.harness.predicates.NodesPredicate.allAtOrOverEpoch;
import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransactionSuccess;
import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertEquals;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Iterables;
import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.EdDSAEd25519PublicKey;
import com.radixdlt.crypto.PublicKey;
import com.radixdlt.genesis.GenesisBuilder;
import com.radixdlt.genesis.GenesisConsensusManagerConfig;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.harness.deterministic.TransactionExecutor;
import com.radixdlt.message.CurveDecryptorSet;
import com.radixdlt.message.Decryptor;
import com.radixdlt.message.MessageContent;
import com.radixdlt.message.TransactionMessage;
import com.radixdlt.protocol.ProtocolConfig;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REv2TransactionsAndProofReader;
import com.radixdlt.rev2.TransactionBuilder;
import com.radixdlt.utils.Bytes;
import io.netty.handler.codec.http.HttpResponseStatus;
import java.util.List;
import org.junit.Test;

public class TransactionStreamTest extends DeterministicCoreApiTestBase {

  @Test
  public void test_core_api_can_submit_and_commit_transaction_after_running_all_scenarios()
      throws Exception {
    final var config =
        defaultConfig()
            .withGenesis(
                GenesisBuilder.createTestGenesisWithNumValidators(
                    1,
                    Decimal.ONE,
                    GenesisConsensusManagerConfig.Builder.testDefaults().epochExactRoundCount(100),
                    // This test checks that the transaction stream doesn't return errors when
                    // mapping
                    // genesis and the scenarios:
                    GenesisData.ALL_SCENARIOS));
    try (var test = buildRunningServerTest(config)) {
      test.suppressUnusedWarning();

      // Wait for all protocol updates:
      //test.runUntilState(allAtOrOverEpoch(protocolConfig.lastProtocolUpdateEnactmentEpoch()));

      var transaction = TransactionBuilder.forTests().prepare();

      // Submit transaction
      var response =
          getTransactionApi()
              .transactionSubmitPost(
                  new TransactionSubmitRequest()
                      .network(networkLogicalName)
                      .notarizedTransactionHex(transaction.hexPayloadBytes()));

      assertThat(response.getDuplicate()).isFalse();

      test.runUntilState(allCommittedTransactionSuccess(transaction.raw()), 100);

      var lastLedgerTransaction =
          (UserLedgerTransaction)
              readAllTransactionsFromStreamAndReturnLast().getLedgerTransaction();

      // In order for this assertion to pass, we must have downloaded all the scenario transactions
      // in the above first page of data.
      // This ensures that genesis and all scenarios must have data which can be mapped by the
      // Core API Transaction Stream
      assertThat(lastLedgerTransaction.getNotarizedTransaction().getPayloadHex())
          .isEqualTo(transaction.hexPayloadBytes());
    }
  }

  private CommittedTransaction readAllTransactionsFromStreamAndReturnLast() throws Exception {
    var fromStateVersion = 1L;
    while (true) {
      var pageResponse =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .transactionFormatOptions(
                          new TransactionFormatOptions().rawLedgerTransaction(true))
                      .limit(1000)
                      .fromStateVersion(fromStateVersion));
      var lastItem = Iterables.getLast(pageResponse.getTransactions());
      if (pageResponse
          .getMaxLedgerStateVersion()
          .equals(lastItem.getResultantStateIdentifiers().getStateVersion())) {
        return lastItem;
      } else {
        fromStateVersion = lastItem.getResultantStateIdentifiers().getStateVersion() + 1;
      }
    }
  }

  @Test
  public void streamed_transactions_contain_their_message() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      // Prepare 3 different flavors of messages in transaction:
      var withoutMessage = TransactionBuilder.forTests().prepare();
      var withPlaintextMessage =
          TransactionBuilder.forTests()
              .message(
                  new TransactionMessage.Plaintext(
                      new com.radixdlt.message.PlaintextTransactionMessage(
                          "text/plain", new MessageContent.StringContent("hello transaction"))))
              .prepare();
      var withEncryptedMessage =
          TransactionBuilder.forTests()
              .message(
                  new TransactionMessage.Encrypted(
                      new com.radixdlt.message.EncryptedTransactionMessage(
                          bytes(47),
                          List.of(
                              new CurveDecryptorSet(
                                  new PublicKey.EddsaEd25519(
                                      EdDSAEd25519PublicKey.fromCompressedBytesUnchecked(
                                          bytes(32))),
                                  List.of(new Decryptor(bytes(8), bytes(24))))))))
              .prepare();

      // Commit them in this order:
      var createdTransactions = List.of(withoutMessage, withPlaintextMessage, withEncryptedMessage);
      for (var transaction : createdTransactions) {
        getTransactionApi()
            .transactionSubmitPost(
                new TransactionSubmitRequest()
                    .network(networkLogicalName)
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));
        test.runUntilState(allCommittedTransactionSuccess(transaction.raw()), 100);
      }

      // Retrieve them from the stream API:
      var streamedIntents =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .transactionFormatOptions(new TransactionFormatOptions().message(true))
                      .limit(1000)
                      .fromStateVersion(1L))
              .getTransactions()
              .stream()
              .map(CommittedTransaction::getLedgerTransaction)
              .filter(UserLedgerTransaction.class::isInstance)
              .map(UserLedgerTransaction.class::cast)
              .map(userTxn -> userTxn.getNotarizedTransaction().getSignedIntent().getIntent())
              .toList();

      // Assert on their messages:
      assertThat(streamedIntents.get(streamedIntents.size() - 3).getMessage()).isNull();
      assertThat(streamedIntents.get(streamedIntents.size() - 2).getMessage())
          .isEqualTo(
              new PlaintextTransactionMessage()
                  .mimeType("text/plain")
                  .content(
                      new StringPlaintextMessageContent()
                          .value("hello transaction")
                          .type(PlaintextMessageContentType.STRING))
                  .type(TransactionMessageType.PLAINTEXT));
      assertThat(streamedIntents.get(streamedIntents.size() - 1).getMessage())
          .isEqualTo(
              new EncryptedTransactionMessage()
                  .encryptedHex(Bytes.toHexString(bytes(47)))
                  .addCurveDecryptorSetsItem(
                      new EncryptedMessageCurveDecryptorSet()
                          .dhEphemeralPublicKey(
                              new EddsaEd25519PublicKey()
                                  .keyHex(Bytes.toHexString(bytes(32)))
                                  .keyType(PublicKeyType.EDDSAED25519))
                          .addDecryptorsItem(
                              new EncryptedMessageDecryptor()
                                  .publicKeyFingerprintHex(Bytes.toHexString(bytes(8)))
                                  .aesWrappedKeyHex(Bytes.toHexString(bytes(24)))))
                  .type(TransactionMessageType.ENCRYPTED));
    }
  }

  @Test
  public void requesting_state_version_out_of_bounds_returns_error() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      // Arrange: commit any transaction
      TransactionExecutor.executeTransaction(test, TransactionBuilder.forTests());

      // Act 1: request a totally valid state version; learn the max ledger state version as a side
      // effect
      final var maxLedgerStateVersion =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(1000)
                      .fromStateVersion(1L))
              .getMaxLedgerStateVersion();

      // Assert 1:
      assertThat(maxLedgerStateVersion).isGreaterThan(1L);

      // Act 2: request the last transaction
      final var lastTransactionResponse =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(1000)
                      .fromStateVersion(maxLedgerStateVersion));

      // Assert 2:
      assertThat(lastTransactionResponse.getMaxLedgerStateVersion())
          .isEqualTo(maxLedgerStateVersion);
      assertThat(lastTransactionResponse.getPreviousStateIdentifiers()).isNotNull();
      assertThat(lastTransactionResponse.getTransactions().size()).isEqualTo(1);

      // Act 3: request the maximum valid state version
      final var maximumValidStateVersionResponse =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(1000)
                      .fromStateVersion(maxLedgerStateVersion + 1));

      // Assert 3:
      assertThat(maximumValidStateVersionResponse.getMaxLedgerStateVersion())
          .isEqualTo(maxLedgerStateVersion);
      assertThat(maximumValidStateVersionResponse.getPreviousStateIdentifiers()).isNotNull();
      assertThat(maximumValidStateVersionResponse.getTransactions()).isEmpty();

      // Act 4: request the first out-of-bounds state version
      final var stateVersionOutOfBoundsResponse =
          assertErrorResponseOfType(
              () ->
                  getStreamApi()
                      .streamTransactionsPostWithHttpInfo(
                          new StreamTransactionsRequest()
                              .network(networkLogicalName)
                              .limit(1000)
                              .fromStateVersion(maxLedgerStateVersion + 2)),
              StreamTransactionsErrorResponse.class);

      // Assert 4:
      assertThat(stateVersionOutOfBoundsResponse.getCode())
          .isEqualTo(HttpResponseStatus.BAD_REQUEST.code());
      assertThat(stateVersionOutOfBoundsResponse.getDetails())
          .isEqualTo(
              new RequestedStateVersionOutOfBoundsErrorDetails()
                  .maxLedgerStateVersion(maxLedgerStateVersion)
                  .type(StreamTransactionsErrorDetailsType.REQUESTEDSTATEVERSIONOUTOFBOUNDS));
    }
  }

  @Test
  public void test_previous_state_identifiers_and_proofs() throws Exception {
    try (var test = buildRunningServerTest(defaultConfig())) {
      test.suppressUnusedWarning();

      var firstPartTransactions =
          List.of(
              TransactionBuilder.forTests().nonce(1).prepare(),
              TransactionBuilder.forTests().nonce(2).prepare(),
              TransactionBuilder.forTests().nonce(3).prepare());
      for (var transaction : firstPartTransactions) {
        getTransactionApi()
            .transactionSubmitPost(
                new TransactionSubmitRequest()
                    .network(networkLogicalName)
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));
        test.runUntilState(allCommittedTransactionSuccess(transaction.raw()), 100);
      }

      var firstPartResponse =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(100)
                      .fromStateVersion(1L));
      assertThat(firstPartResponse.getProofs()).isNull();

      var firstPartResponseWithProofs =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .includeProofs(true)
                      .limit(100)
                      .fromStateVersion(1L));
      assertThat(firstPartResponseWithProofs.getProofs()).isNotEmpty();

      var firstPartCommittedTransactions = firstPartResponse.getTransactions();

      assertThat(
              firstPartCommittedTransactions.stream()
                  .map(CommittedTransaction::getProposerTimestampMs))
          .isSorted();

      var proofQuery =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .includeProofs(true)
                      .limit(4)
                      .fromStateVersion(3L));
      assertThat(proofQuery.getProofs()).isNotEmpty();

      var lastCommittedTransactionIdentifiers =
          firstPartResponse
              .getTransactions()
              .get(firstPartResponse.getTransactions().size() - 1)
              .getResultantStateIdentifiers();

      var secondPartTransactions =
          List.of(
              TransactionBuilder.forTests().nonce(4).prepare(),
              TransactionBuilder.forTests().nonce(5).prepare(),
              TransactionBuilder.forTests().nonce(6).prepare());
      for (var transaction : secondPartTransactions) {
        getTransactionApi()
            .transactionSubmitPost(
                new TransactionSubmitRequest()
                    .network(networkLogicalName)
                    .notarizedTransactionHex(transaction.hexPayloadBytes()));
        test.runUntilState(allCommittedTransactionSuccess(transaction.raw()), 100);
      }

      var secondPartCommittedTransactions =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(100)
                      .fromStateVersion(lastCommittedTransactionIdentifiers.getStateVersion() + 1));

      assertThat(
              secondPartCommittedTransactions.getTransactions().stream()
                  .map(CommittedTransaction::getProposerTimestampMs))
          .isSorted();

      assertThat(secondPartCommittedTransactions.getPreviousStateIdentifiers())
          .isEqualTo(lastCommittedTransactionIdentifiers);
    }
  }

  @Test
  public void test_core_api_can_return_vm_boot_substate_in_protocol_update_receipt()
      throws Exception {
    final var config =
        defaultConfig()
            .withProtocolConfig(
                new ProtocolConfig(
                    ImmutableList.of(
                        new ProtocolUpdateTrigger(
                            ProtocolUpdateTrigger.ANEMONE,
                            ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(4L)))))
            .withGenesis(
                GenesisBuilder.createTestGenesisWithNumValidators(
                    1,
                    Decimal.ONE,
                    GenesisConsensusManagerConfig.Builder.testDefaults()
                        .epochExactRoundCount(100)));
    try (var test = buildRunningServerTest(config)) {
      test.runUntilState(allAtOrOverEpoch(4L));

      final var protocolUpdateStateVersion =
          test.getInstance(0, REv2TransactionsAndProofReader.class)
              .getLatestProofBundle()
              .orElseThrow()
              .closestProtocolUpdateInitProofOnOrBefore()
              .unwrap()
              .stateVersion();

      final var protocolUpdateTxns =
          getStreamApi()
              .streamTransactionsPost(
                  new StreamTransactionsRequest()
                      .network(networkLogicalName)
                      .limit(3) // We're inspecting the 3 Anemone txns
                      .fromStateVersion(protocolUpdateStateVersion + 1));

      // Just a quick sanity check that the expected number of created/updates substates was
      // returned

      // Consensus manager config update (1 updated)
      assertEquals(
          1,
          protocolUpdateTxns
              .getTransactions()
              .get(0)
              .getReceipt()
              .getStateUpdates()
              .getUpdatedSubstates()
              .size());

      // Seconds precision (1 updated)
      assertEquals(
          1,
          protocolUpdateTxns
              .getTransactions()
              .get(1)
              .getReceipt()
              .getStateUpdates()
              .getUpdatedSubstates()
              .size());

      // VM Boot (1 created)
      assertEquals(
          1,
          protocolUpdateTxns
              .getTransactions()
              .get(2)
              .getReceipt()
              .getStateUpdates()
              .getCreatedSubstates()
              .size());
    }
  }

  private static byte[] bytes(int length) {
    byte[] bytes = new byte[length];
    for (int i = 0; i < length; ++i) {
      bytes[i] = (byte) (length - i); // a descending sequence is easy to eyeball in the logs
    }
    return bytes;
  }
}
