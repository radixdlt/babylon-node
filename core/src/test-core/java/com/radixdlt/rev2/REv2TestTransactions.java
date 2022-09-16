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

import com.radixdlt.addressing.Addressing;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.PublicKey;
import com.radixdlt.crypto.SignatureWithPublicKey;
import com.radixdlt.networks.Network;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawTransaction;
import com.radixdlt.utils.PrivateKeys;
import java.util.List;

public final class REv2TestTransactions {
  // These have to come first for static ordering issues
  public static final ECKeyPair DEFAULT_NOTARY = generateKeyPair(1);

  public static final RawTransaction VALID_TXN_0 =
      constructTransaction(
          NetworkDefinition.INT_TEST_NET,
          String.format(
              """
        CALL_METHOD ComponentAddress("%s") "lock_fee" Decimal("10");
        CLEAR_AUTH_ZONE;
        """,
              Addressing.ofNetwork(Network.INTEGRATIONTESTNET)
                  .encodeSystemComponentAddress(ComponentAddress.SYSTEM_FAUCET_COMPONENT_ADDRESS)),
          0,
          List.of());
  public static final RawTransaction STATICALLY_VALID_BUT_REJECT_TXN_1 =
      constructTransaction(
          NetworkDefinition.INT_TEST_NET, "CLEAR_AUTH_ZONE;", 0, List.of(generateKeyPair(10)));
  public static final RawTransaction STATICALLY_VALID_BUT_REJECT_TXN_2 =
      constructTransaction(
          NetworkDefinition.INT_TEST_NET,
          "CLEAR_AUTH_ZONE; CLEAR_AUTH_ZONE;",
          0,
          List.of(generateKeyPair(21), generateKeyPair(22)));

  private static ECKeyPair generateKeyPair(int keySource) {
    return PrivateKeys.numeric(keySource).findFirst().orElseThrow();
  }

  public static byte[] constractValidIntentBytes(
      NetworkDefinition network, long nonce, PublicKey notary) {
    final var addressing = Addressing.ofNetwork(network);
    final var faucetAddress =
        addressing.encodeSystemComponentAddress(ComponentAddress.SYSTEM_FAUCET_COMPONENT_ADDRESS);

    var manifest =
        String.format(
            """
        CALL_METHOD ComponentAddress("%s") "lock_fee" Decimal("10");
        CLEAR_AUTH_ZONE;
    """,
            faucetAddress);
    var header = TransactionHeader.defaults(network, nonce, notary, false);
    return TransactionBuilder.createIntent(network, header, manifest, List.of());
  }

  public static String constructNewAccountManifest(NetworkDefinition networkDefinition) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress =
        addressing.encodeSystemComponentAddress(ComponentAddress.SYSTEM_FAUCET_COMPONENT_ADDRESS);
    final var xrdAddress = addressing.encodeResourceAddress(ResourceAddress.XRD_ADDRESS);
    final var accountPackageAddress =
        addressing.encodePackageAddress(PackageAddress.ACCOUNT_PACKAGE_ADDRESS);

    return String.format(
        """
                    CALL_METHOD ComponentAddress("%s") "lock_fee" Decimal("1000");
                    CALL_METHOD ComponentAddress("%s") "free_xrd";
                    TAKE_FROM_WORKTOP ResourceAddress("%s") Bucket("xrd");
                    CALL_FUNCTION PackageAddress("%s") "Account" "new_with_resource" Enum("AllowAll") Bucket("xrd");
                    """,
        faucetAddress, faucetAddress, xrdAddress, accountPackageAddress);
  }

  public static byte[] constructNewAccountIntent(
      NetworkDefinition networkDefinition, long nonce, PublicKey notary) {
    final var manifest = constructNewAccountManifest(networkDefinition);
    final var header = TransactionHeader.defaults(networkDefinition, nonce, notary, false);
    return TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());
  }

  public static RawTransaction constructNewAccountTransaction(
      NetworkDefinition networkDefinition, long nonce) {
    var manifest = constructNewAccountManifest(networkDefinition);
    var signatories = List.<ECKeyPair>of();

    return constructTransaction(
        networkDefinition, nonce, manifest, DEFAULT_NOTARY, false, signatories);
  }

  public static RawTransaction constructTransaction(
      NetworkDefinition networkDefinition,
      String manifest,
      long nonce,
      List<ECKeyPair> signatories) {
    return constructTransaction(
        networkDefinition, nonce, manifest, DEFAULT_NOTARY, false, signatories);
  }

  public static RawTransaction constructTransaction(
      NetworkDefinition networkDefinition,
      long nonce,
      String manifest,
      ECKeyPair notary,
      boolean notaryIsSignatory,
      List<ECKeyPair> signatories) {
    // Build intent
    final var header =
        TransactionHeader.defaults(
            networkDefinition, nonce, notary.getPublicKey().toPublicKey(), notaryIsSignatory);
    var intentBytes =
        TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());

    // Sign intent
    return constructTransaction(intentBytes, notary, signatories);
  }

  public static RawTransaction constructTransaction(
      NetworkDefinition networkDefinition,
      TransactionHeader header,
      String manifest,
      ECKeyPair notary,
      List<ECKeyPair> signatories) {
    // Build intent
    var intentBytes =
        TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());

    // Sign intent
    return constructTransaction(intentBytes, notary, signatories);
  }

  public static RawTransaction constructTransaction(
      byte[] intentBytes, ECKeyPair notary, List<ECKeyPair> signatories) {
    // Sign intent
    var hashedIntent = HashUtils.sha256Twice(intentBytes).asBytes();
    var intentSignatures =
        signatories.stream()
            .map(
                ecKeyPair ->
                    (SignatureWithPublicKey)
                        new SignatureWithPublicKey.EcdsaSecp256k1(ecKeyPair.sign(hashedIntent)))
            .toList();
    var signedIntentBytes =
        TransactionBuilder.createSignedIntentBytes(intentBytes, intentSignatures);

    // Notarize
    var hashedSignedIntent = HashUtils.sha256Twice(signedIntentBytes).asBytes();
    var notarySignature = notary.sign(hashedSignedIntent).toSignature();
    var notarizedBytes =
        TransactionBuilder.createNotarizedBytes(signedIntentBytes, notarySignature);

    return RawTransaction.create(notarizedBytes);
  }

  private REv2TestTransactions() {
    throw new IllegalStateException("Cannot instantiate.");
  }
}
