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
import com.radixdlt.crypto.*;
import com.radixdlt.identifiers.Address;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.transactions.RawNotarizedTransaction;
import com.radixdlt.utils.PrivateKeys;
import java.util.List;

public final class REv2TestTransactions {
  // These have to come first for static ordering issues
  public static final ECKeyPair DEFAULT_NOTARY = generateKeyPair(1);

  private static ECKeyPair generateKeyPair(int keySource) {
    return PrivateKeys.numeric(keySource).findFirst().orElseThrow();
  }

  public static NotarizedTransactionBuilder validButRejectTransaction(long fromEpoch, long nonce) {

    final var header =
        TransactionHeader.defaults(
            NetworkDefinition.INT_TEST_NET,
            fromEpoch,
            100,
            nonce,
            DEFAULT_NOTARY.getPublicKey().toPublicKey(),
            false);
    // Note - rejects due to lack of fee payment
    final String rejectedManifest = "CLEAR_AUTH_ZONE;";
    final List<ECKeyPair> signatories = List.of(generateKeyPair(10));

    return new NotarizedTransactionBuilder(
        TransactionBuilder.createIntent(
            NetworkDefinition.INT_TEST_NET, header, rejectedManifest, List.of()),
        DEFAULT_NOTARY,
        signatories);
  }

  public static byte[] constructValidIntentBytes(
      NetworkDefinition network,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      PublicKey notary) {
    final var addressing = Addressing.ofNetwork(network);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);

    var manifest =
        String.format(
            """
        CALL_METHOD Address("%s") "lock_fee" Decimal("100");
        CLEAR_AUTH_ZONE;
    """,
            faucetAddress);
    var header = TransactionHeader.defaults(network, fromEpoch, 100, nonce, notary, false);
    return TransactionBuilder.createIntent(network, header, manifest, List.of());
  }

  public static String constructDepositFromFaucetToRandomManifest(
      NetworkDefinition networkDefinition, ComponentAddress faucet) {
    return constructDepositFromFaucetManifest(
        networkDefinition,
        faucet,
        Address.virtualAccountAddress(ECKeyPair.generateNew().getPublicKey()));
  }

  public static String constructNewAccountManifest(
      NetworkDefinition networkDefinition, ComponentAddress faucet) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    return String.format(
        """
                    CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                    CREATE_ACCOUNT_ADVANCED
                      Tuple(                                    # Access Rules Config Struct
                          Map<Tuple, Enum>(),                   # Method auth Field
                          Map<String, Enum>(),                  # Grouped Auth Field
                          Enum(                                 # Default Auth Field
                              "AccessRuleEntry::AccessRule",
                              Enum("AccessRule::AllowAll")
                          ),
                          Map<Tuple, Enum>(),                   # Method Auth Mutability Field
                          Map<String, Enum>(),                  # Group Auth Mutability Field
                          Enum(                                 # Default Auth Mutability Field
                              "AccessRuleEntry::AccessRule",
                              Enum("AccessRule::DenyAll")
                          )
                      );
                    """,
        faucetAddress);
  }

  public static String constructDepositFromFaucetManifest(
      NetworkDefinition networkDefinition, ComponentAddress faucet, ComponentAddress to) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    return String.format(
        """
                    CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                    CALL_METHOD Address("%s") "free";
                    CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
                    """,
        faucetAddress, faucetAddress, addressing.encodeAccountAddress(to));
  }

  public static String constructDepositFromAccountManifest(
      NetworkDefinition networkDefinition, ComponentAddress from) {
    // NOTE: A test relies on this only being able to be performed once per account
    // So we transfer 900 XRD (which is the majority of the account start amount
    // of 1000, the size of the free XRD bucket)
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var fromAddress = addressing.encodeAccountAddress(from);
    final var xrdAddress = addressing.encodeResourceAddress(ScryptoConstants.XRD_RESOURCE_ADDRESS);
    final var accountAddress =
        addressing.encodeAccountAddress(
            Address.virtualAccountAddress(ECKeyPair.generateNew().getPublicKey()));
    return String.format(
        """
                        CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                        CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("9900");
                        CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
                        """,
        fromAddress, fromAddress, xrdAddress, accountAddress);
  }

  public static String constructCreateValidatorManifest(
      NetworkDefinition networkDefinition, ComponentAddress faucet, ECDSASecp256k1PublicKey key) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);

    // Re-using the validator key for holding its own token
    final var ownerTokenHolder =
        addressing.encodeAccountAddress(Address.virtualAccountAddress(key));

    return String.format(
        """
                            CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                            CREATE_VALIDATOR Bytes("%s");
                            CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
                            """,
        faucetAddress, key.toHex(), ownerTokenHolder);
  }

  public static String constructRegisterValidatorManifest(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      ComponentAddress validatorAddress,
      ComponentAddress ownerTokenHolder) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    final var componentAddress = addressing.encodeValidatorAddress(validatorAddress);
    final var validatorOwnerTokenAddress =
        addressing.encodeResourceAddress(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS);
    final var ownerTokenHolderAddress = addressing.encodeAccountAddress(ownerTokenHolder);
    return String.format(
        """
                        CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                        CALL_METHOD Address("%s") "create_proof" Address("%s");
                        CALL_METHOD Address("%s") "register";
                        """,
        faucetAddress, ownerTokenHolderAddress, validatorOwnerTokenAddress, componentAddress);
  }

  public static String constructUnregisterValidatorManifest(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      ComponentAddress validatorAddress,
      ComponentAddress ownerTokenHolder) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    final var componentAddress = addressing.encodeValidatorAddress(validatorAddress);
    final var validatorOwnerTokenAddress =
        addressing.encodeResourceAddress(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS);
    final var ownerTokenHolderAddress = addressing.encodeAccountAddress(ownerTokenHolder);

    return String.format(
        """
                            CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                            CALL_METHOD Address("%s") "create_proof" Address("%s");
                            CALL_METHOD Address("%s") "unregister";
                            """,
        faucetAddress, ownerTokenHolderAddress, validatorOwnerTokenAddress, componentAddress);
  }

  public static String constructStakeValidatorManifest(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      ComponentAddress validatorAddress,
      ComponentAddress ownerTokenHolder) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    final var xrdAddress = addressing.encodeResourceAddress(ScryptoConstants.XRD_RESOURCE_ADDRESS);
    final var validatorHrpAddress = addressing.encodeValidatorAddress(validatorAddress);
    final var toAccount = Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    final var toAccountAddress = addressing.encodeAccountAddress(toAccount);
    final var validatorOwnerTokenAddress =
        addressing.encodeResourceAddress(ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS);
    final var ownerTokenHolderAddress = addressing.encodeAccountAddress(ownerTokenHolder);

    return String.format(
        """
                                CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                                CALL_METHOD Address("%s") "create_proof" Address("%s");
                                CALL_METHOD Address("%s") "free";
                                TAKE_FROM_WORKTOP Address("%s") Bucket("xrd");
                                CALL_METHOD Address("%s") "stake" Bucket("xrd");
                                CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
                                """,
        faucetAddress,
        ownerTokenHolderAddress,
        validatorOwnerTokenAddress,
        faucetAddress,
        xrdAddress,
        validatorHrpAddress,
        toAccountAddress);
  }

  public static String constructUnstakeValidatorManifest(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      ResourceAddress lpTokenAddress,
      ComponentAddress validatorAddress) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    final var validatorHrpAddress = addressing.encodeValidatorAddress(validatorAddress);
    final var account = Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    final var accountAddress = addressing.encodeAccountAddress(account);
    final var lpAddress = addressing.encodeResourceAddress(lpTokenAddress);

    return String.format(
        """
                                CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                                CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("1");
                                TAKE_FROM_WORKTOP Address("%s") Bucket("lp_token");
                                CALL_METHOD Address("%s") "unstake" Bucket("lp_token");
                                CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
                                """,
        faucetAddress, accountAddress, lpAddress, lpAddress, validatorHrpAddress, accountAddress);
  }

  public static String constructClaimXrdManifest(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      ComponentAddress validatorAddress,
      ResourceAddress unstakeResource) {
    final var addressing = Addressing.ofNetwork(networkDefinition);
    final var faucetAddress = addressing.encodeNormalComponentAddress(faucet);
    final var unstakeResourceAddress = addressing.encodeResourceAddress(unstakeResource);
    final var xrdAddress = addressing.encodeResourceAddress(ScryptoConstants.XRD_RESOURCE_ADDRESS);
    final var validatorHrpAddress = addressing.encodeValidatorAddress(validatorAddress);
    final var account = Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    final var accountAddress = addressing.encodeAccountAddress(account);

    return String.format(
        """
                                    CALL_METHOD Address("%s") "lock_fee" Decimal("100");
                                    CALL_METHOD Address("%s") "withdraw" Address("%s");
                                    TAKE_FROM_WORKTOP Address("%s") Bucket("unstake");
                                    CALL_METHOD Address("%s") "claim_xrd" Bucket("unstake");
                                    TAKE_FROM_WORKTOP Address("%s") Bucket("xrd");
                                    CALL_METHOD Address("%s") "deposit" Bucket("xrd");
                                    """,
        faucetAddress,
        accountAddress,
        unstakeResourceAddress,
        unstakeResourceAddress,
        validatorHrpAddress,
        xrdAddress,
        accountAddress);
  }

  public static byte[] constructDepositFromFaucetIntent(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      PublicKey notary) {
    final var manifest = constructDepositFromFaucetToRandomManifest(networkDefinition, faucet);
    final var header =
        TransactionHeader.defaults(networkDefinition, fromEpoch, 100, nonce, notary, false);
    return TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());
  }

  public static byte[] constructLargeValidTransactionIntent(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      PublicKey notary,
      int blobsSize) {
    final var manifest = constructDepositFromFaucetToRandomManifest(networkDefinition, faucet);
    final var header =
        TransactionHeader.defaults(networkDefinition, fromEpoch, 100, nonce, notary, false);
    final var blobs = List.of(new byte[blobsSize]);
    return TransactionBuilder.createIntent(networkDefinition, header, manifest, blobs);
  }

  public static RawNotarizedTransaction constructValidRawTransaction(
      ComponentAddress faucet, long fromEpoch, long nonce) {
    var intentBytes =
        constructValidIntentBytes(
            NetworkDefinition.INT_TEST_NET,
            faucet,
            fromEpoch,
            nonce,
            DEFAULT_NOTARY.getPublicKey().toPublicKey());
    return REv2TestTransactions.constructRawTransaction(intentBytes, DEFAULT_NOTARY, List.of());
  }

  public static NotarizedTransactionBuilder constructValidTransaction(
      ComponentAddress faucet, long fromEpoch, long nonce) {
    var intentBytes =
        constructValidIntentBytes(
            NetworkDefinition.INT_TEST_NET,
            faucet,
            fromEpoch,
            nonce,
            DEFAULT_NOTARY.getPublicKey().toPublicKey());
    return new NotarizedTransactionBuilder(intentBytes, DEFAULT_NOTARY, List.of());
  }

  public static RawNotarizedTransaction constructCreateValidatorTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ECKeyPair keyPair) {
    final var manifest =
        constructCreateValidatorManifest(networkDefinition, faucet, keyPair.getPublicKey());
    final var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructRegisterValidatorTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ComponentAddress validatorAddress,
      ECKeyPair keyPair) {
    // Assuming that the validator key was re-used to hold its own token
    final var ownerTokenHolder = Address.virtualAccountAddress(keyPair.getPublicKey());
    var manifest =
        constructRegisterValidatorManifest(
            networkDefinition, faucet, validatorAddress, ownerTokenHolder);
    var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructUnregisterValidatorTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ComponentAddress validatorAddress,
      ECKeyPair keyPair) {
    // Assuming that the validator key was re-used to hold its own token
    final var ownerTokenHolder = Address.virtualAccountAddress(keyPair.getPublicKey());
    var manifest =
        constructUnregisterValidatorManifest(
            networkDefinition, faucet, validatorAddress, ownerTokenHolder);
    var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructStakeValidatorTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ComponentAddress validatorAddress,
      ECKeyPair keyPair) {
    // Assuming that the validator key was re-used to hold its own token
    final var ownerTokenHolder = Address.virtualAccountAddress(keyPair.getPublicKey());
    var manifest =
        constructStakeValidatorManifest(
            networkDefinition, faucet, validatorAddress, ownerTokenHolder);
    var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructUnstakeValidatorTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ResourceAddress lpTokenAddress,
      ComponentAddress validatorAddress,
      ECKeyPair keyPair) {
    var manifest =
        constructUnstakeValidatorManifest(
            networkDefinition, faucet, lpTokenAddress, validatorAddress);
    var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructClaimXrdTransaction(
      NetworkDefinition networkDefinition,
      ComponentAddress faucet,
      long fromEpoch,
      long nonce,
      ComponentAddress validatorAddress,
      ResourceAddress unstakeAddress,
      ECKeyPair keyPair) {
    var manifest =
        constructClaimXrdManifest(networkDefinition, faucet, validatorAddress, unstakeAddress);
    var signatories = List.of(keyPair);
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, keyPair, false, signatories);
  }

  public static RawNotarizedTransaction constructDepositFromFaucetTransaction(
      NetworkDefinition networkDefinition, ComponentAddress faucet, long fromEpoch, long nonce) {
    var manifest = constructDepositFromFaucetToRandomManifest(networkDefinition, faucet);
    var signatories = List.<ECKeyPair>of();

    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, DEFAULT_NOTARY, false, signatories);
  }

  public static RawNotarizedTransaction constructRawTransaction(
      NetworkDefinition networkDefinition,
      String manifest,
      long fromEpoch,
      long nonce,
      List<ECKeyPair> signatories) {
    return constructRawTransaction(
        networkDefinition, fromEpoch, nonce, manifest, DEFAULT_NOTARY, false, signatories);
  }

  public static RawNotarizedTransaction constructRawTransaction(
      NetworkDefinition networkDefinition,
      long fromEpoch,
      long nonce,
      String manifest,
      ECKeyPair notary,
      boolean notaryIsSignatory,
      List<ECKeyPair> signatories) {
    // Build intent
    final var header =
        TransactionHeader.defaults(
            networkDefinition,
            fromEpoch,
            100,
            nonce,
            notary.getPublicKey().toPublicKey(),
            notaryIsSignatory);
    var intentBytes =
        TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());

    // Sign intent
    return constructRawTransaction(intentBytes, notary, signatories);
  }

  public static RawNotarizedTransaction constructRawTransaction(
      NetworkDefinition networkDefinition,
      TransactionHeader header,
      String manifest,
      ECKeyPair notary,
      List<ECKeyPair> signatories) {
    // Build intent
    var intentBytes =
        TransactionBuilder.createIntent(networkDefinition, header, manifest, List.of());

    // Sign intent
    return constructRawTransaction(intentBytes, notary, signatories);
  }

  public static RawNotarizedTransaction constructRawTransaction(
      byte[] intentBytes, ECKeyPair notary, List<ECKeyPair> signatories) {

    return new NotarizedTransactionBuilder(intentBytes, notary, signatories)
        .constructRawTransaction();
  }

  private REv2TestTransactions() {
    throw new IllegalStateException("Cannot instantiate.");
  }

  public record NotarizedTransactionBuilder(
      byte[] intentBytes, ECKeyPair notary, List<ECKeyPair> signatories) {

    public HashCode hashedIntent() {
      return HashUtils.blake2b256(this.intentBytes);
    }

    public RawNotarizedTransaction constructRawTransaction() {
      var intentSignatures =
          this.signatories().stream()
              .map(
                  ecKeyPair ->
                      (SignatureWithPublicKey)
                          new SignatureWithPublicKey.EcdsaSecp256k1(
                              ecKeyPair.sign(this.hashedIntent().asBytes())))
              .toList();
      var signedIntentBytes =
          TransactionBuilder.createSignedIntentBytes(this.intentBytes(), intentSignatures);

      // Notarize
      var hashedSignedIntent = HashUtils.blake2b256(signedIntentBytes).asBytes();
      var notarySignature = this.notary().sign(hashedSignedIntent).toSignature();
      var notarizedBytes =
          TransactionBuilder.createNotarizedBytes(signedIntentBytes, notarySignature);
      return RawNotarizedTransaction.create(notarizedBytes);
    }
  }
}
