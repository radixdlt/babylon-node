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

import com.google.common.hash.HashCode;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.networks.Network;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.Bytes;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt32;
import java.util.List;
import java.util.Random;
import java.util.stream.IntStream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.junit.Test;

public class REv2TransactionCreationTest {
  private static final Logger log = LogManager.getLogger();
  private static final ECKeyPair NOTARY = REv2TestTransactions.DEFAULT_NOTARY;

  public record TransactionInfo(RawNotarizedTransaction transaction, HashCode intentHash) {}

  @Test
  public void can_create_some_test_transactions() {
    // This test is mostly used to create signed transactions in various varieties for manually
    // testing the Core API
    var network = NetworkDefinition.from(Network.LOCALNET);
    var fromEpoch = 1; // Needs to be >= 1
    var baseNonce = new Random().nextLong() & Long.MAX_VALUE;

    // Used to create different intents
    var nonce1 = baseNonce + 1;
    var nonce2 = baseNonce + 2;
    // Varying signature counts are used to create different payloads

    log.info(
        String.format(
            "Generating for %s with validity from epoch %s with base nonce %s",
            network.logical_name(), fromEpoch, nonce1));
    var addressing = Addressing.ofNetwork(network);
    log.info(
        String.format(
            "XRD Address: %s",
            addressing.encodeResourceAddress(ScryptoConstants.XRD_RESOURCE_ADDRESS)));
    log.info("===================================");

    logTransaction(
        "Small valid transaction (Intent 1, Payload 1)",
        constructSmallValidTransaction(network, fromEpoch, nonce1, 0));
    logTransaction(
        "Small valid transaction (Intent 1, Payload 2)",
        constructSmallValidTransaction(network, fromEpoch, nonce1, 1));
    logTransaction(
        "Small valid transaction (Intent 1, Payload 3)",
        constructSmallValidTransaction(network, fromEpoch, nonce1, 2));
    logTransaction(
        "Small valid transaction (Intent 2, Payload 1)",
        constructSmallValidTransaction(network, fromEpoch, nonce2, 0));

    logTransaction(
        "New Account Transaction (Intent 1, Payload 1)",
        constructNewAccountTransactionJava(network, fromEpoch, nonce1, 0));
    logTransaction(
        "New Account Transaction (Intent 1, Payload 2)",
        constructNewAccountTransactionJava(network, fromEpoch, nonce1, 1));
    logTransaction(
        "New Account Transaction (Intent 2, Payload 1)",
        constructNewAccountTransactionJava(network, fromEpoch, nonce2, 1));

    logTransaction(
        "Statically-invalid Transaction (Intent 1, Payload 1)",
        constructStaticallyInvalidTransaction(network, fromEpoch, nonce1));

    logTransaction(
        "Execution-invalid Transaction (Intent 1, Payload 1)",
        constructExecutionInvalidTransaction(network, fromEpoch, nonce1, 0));
    logTransaction(
        "Execution-invalid Transaction (Intent 1, Payload 2)",
        constructExecutionInvalidTransaction(network, fromEpoch, nonce1, 1));
    logTransaction(
        "Execution-invalid Transaction (Intent 1, Payload 3)",
        constructExecutionInvalidTransaction(network, fromEpoch, nonce1, 2));
  }

  private static void logTransaction(String description, TransactionInfo transactionInfo) {
    log.info(description + ":");
    log.info("Intent Hash: " + Bytes.toHexString(transactionInfo.intentHash.asBytes()));
    log.info(
        "User Payload Hash: "
            + Bytes.toHexString(transactionInfo.transaction.getPayloadHash().asBytes()));
    log.info("Notarized Payload: " + Bytes.toHexString(transactionInfo.transaction.getPayload()));
    log.info("=============================");
  }

  public static TransactionInfo createTransaction(byte[] intentBytes, List<ECKeyPair> signatories) {
    final var intentHash = HashUtils.sha256Twice(intentBytes);
    final var transaction =
        REv2TestTransactions.constructRawTransaction(intentBytes, NOTARY, signatories);

    return new TransactionInfo(transaction, intentHash);
  }

  public static TransactionInfo constructSmallValidTransaction(
      NetworkDefinition networkDefinition, long fromEpoch, long nonce, int numSigs) {

    final var intentBytes =
        REv2TestTransactions.constructValidIntentBytes(
            networkDefinition, fromEpoch, nonce, NOTARY.getPublicKey().toPublicKey());

    return createTransaction(intentBytes, createSignatories(numSigs));
  }

  public static TransactionInfo constructNewAccountTransactionJava(
      NetworkDefinition networkDefinition, long fromEpoch, long nonce, int numSigs) {
    final var intentBytes =
        REv2TestTransactions.constructDepositFromFaucetIntent(
            networkDefinition, fromEpoch, nonce, NOTARY.getPublicKey().toPublicKey());
    return createTransaction(intentBytes, createSignatories(numSigs));
  }

  /*
   * By using too low a cost unit cap to cover the loan
   */
  public static TransactionInfo constructStaticallyInvalidTransaction(
      NetworkDefinition networkDefinition, long fromEpoch, long nonce) {

    final var intentBytes =
        REv2TestTransactions.constructValidIntentBytes(
            networkDefinition, fromEpoch, nonce, NOTARY.getPublicKey().toPublicKey());

    final var duplicateSignatories = List.of(PrivateKeys.ofNumeric(1), PrivateKeys.ofNumeric(1));

    return createTransaction(intentBytes, duplicateSignatories);
  }

  /*
   * By using too low a cost unit cap to cover the loan
   */
  public static TransactionInfo constructExecutionInvalidTransaction(
      NetworkDefinition networkDefinition, long fromEpoch, long nonce, int numSigs) {

    final var manifest = REv2TestTransactions.constructDepositFromFaucetManifest(networkDefinition);

    final var insufficientLimit = UInt32.fromNonNegativeInt(1000);

    final var header =
        TransactionHeader.defaults(
            networkDefinition,
            fromEpoch,
            5,
            nonce,
            NOTARY.getPublicKey().toPublicKey(),
            insufficientLimit,
            true);

    var intentBytes =
        TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());

    return createTransaction(intentBytes, createSignatories(numSigs));
  }

  public static List<ECKeyPair> createSignatories(int num) {
    return IntStream.rangeClosed(1, num).mapToObj(PrivateKeys::ofNumeric).toList();
  }
}
