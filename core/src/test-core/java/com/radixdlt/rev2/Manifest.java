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

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Functions;
import java.util.List;

/** Used for creating various manifests - designed to work well with the TransactionBuilder */
public class Manifest {
  public record Parameters(NetworkDefinition network) {
    public String faucetLockFeeLine() {
      return this.lockFeeLine(FAUCET);
    }

    public String lockFeeLine(ComponentAddress address) {
      return String.format(
          """
          CALL_METHOD Address("%s") "lock_fee" Decimal("1000");""", encode(address));
    }

    public String createProofOfValidatorOwnerBadge(
        ComponentAddress account, ComponentAddress validator) {
      return createProofOfOwnerBadge(
          account, ScryptoConstants.VALIDATOR_OWNER_TOKEN_RESOURCE_ADDRESS, validator);
    }

    public String createProofOfOwnerBadge(
        ComponentAddress account, ResourceAddress ownerBadgeResource, ComponentAddress owned) {
      return String.format(
          """
          CALL_METHOD Address("%s")
              "create_proof_of_non_fungibles"
              Address("%s")
              Array<NonFungibleLocalId>(
                  NonFungibleLocalId("%s"),
              )
          ;""",
          encode(account), encode(ownerBadgeResource), owned.ownerBadgeBytesLocalId());
    }

    public String encode(ComponentAddress address) {
      return address.encode(network);
    }

    public String encode(ResourceAddress address) {
      return address.encode(network);
    }

    @SuppressWarnings("unused")
    public String encode(PackageAddress address) {
      return address.encode(network);
    }
  }

  public static ComponentAddress FAUCET = ScryptoConstants.FAUCET_ADDRESS;
  public static ResourceAddress XRD = ScryptoConstants.XRD_RESOURCE_ADDRESS;

  public static Functions.Func1<Parameters, String> validButReject() {
    return (_params) -> "DROP_ALL_PROOFS;";
  }

  public static Functions.Func1<Parameters, String> valid() {
    return (params) ->
        String.format(
            """
            %s
            DROP_ALL_PROOFS;
            """,
            params.faucetLockFeeLine());
  }

  public static Functions.Func1<Parameters, String> newRandomAccount() {
    var address = Address.virtualAccountAddress(ECKeyPair.generateNew().getPublicKey());
    return depositFromFaucet(address);
  }

  public static Functions.Func1<Parameters, String> newAccountAllowAllOwner() {
    return (params) ->
        String.format(
            """
            %s
            CREATE_ACCOUNT_ADVANCED
                Enum<OwnerRole::Updatable>(
                    Enum<AccessRule::AllowAll>()
                )
            ;
            """,
            params.faucetLockFeeLine());
  }

  public static Functions.Func1<Parameters, String> transferBetweenAccountsFeeFromSender(
      ComponentAddress fromAccount,
      ResourceAddress resource,
      Decimal amount,
      ComponentAddress toAccount) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("%s");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.lockFeeLine(fromAccount),
            params.encode(fromAccount),
            params.encode(resource),
            amount,
            params.encode(toAccount));
  }

  public static Functions.Func1<Parameters, String> transferBetweenAccountsFeeFromReceiver(
      ComponentAddress fromAccount,
      ResourceAddress resource,
      Decimal amount,
      ComponentAddress toAccount) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("%s");
            CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
            """,
            params.lockFeeLine(toAccount),
            params.encode(fromAccount),
            params.encode(resource),
            amount,
            params.encode(toAccount));
  }

  public static Functions.Func1<Parameters, String> transferBetweenAccountsFeeFromFaucet(
      ComponentAddress fromAccount,
      ResourceAddress resource,
      Decimal amount,
      ComponentAddress toAccount) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("%s");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(),
            params.encode(fromAccount),
            params.encode(resource),
            amount,
            params.encode(toAccount));
  }

  public static Functions.Func1<Parameters, String> depositFromFaucet(ComponentAddress toAccount) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "free";
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(), params.encode(FAUCET), params.encode(toAccount));
  }

  public static Functions.Func1<Parameters, String> drainAccount(ComponentAddress account) {
    // NOTE: A test relies on this only being able to be performed once per account
    // So we transfer slightly less than the free XRD amount
    final var accountToDrainTo =
        Address.virtualAccountAddress(ECKeyPair.generateNew().getPublicKey());
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("%s");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.lockFeeLine(account),
            params.encode(account),
            params.encode(XRD),
            ScryptoConstants.FREE_AMOUNT_FROM_FAUCET.subtract(Decimal.of(1000)),
            params.encode(accountToDrainTo));
  }

  public static Functions.Func1<Parameters, String> createValidator(
      ECDSASecp256k1PublicKey key, ComponentAddress ownerAccount) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "free";
            TAKE_FROM_WORKTOP Address("%s") Decimal("1000") Bucket("validator_creation_fee");
            CREATE_VALIDATOR Bytes("%s") Decimal("0") Bucket("validator_creation_fee");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(),
            params.encode(FAUCET),
            params.encode(XRD),
            key.toHex(),
            params.encode(ownerAccount));
  }

  public static Functions.Func1<Parameters, String> registerValidator(
      ComponentAddress validatorAddress, ComponentAddress ownerAccount) {
    return (params) ->
        String.format(
            """
            %s
            %s
            CALL_METHOD Address("%s") "update_accept_delegated_stake" true;
            CALL_METHOD Address("%s") "register";
            """,
            params.faucetLockFeeLine(),
            params.createProofOfValidatorOwnerBadge(ownerAccount, validatorAddress),
            params.encode(validatorAddress),
            params.encode(validatorAddress));
  }

  public static Functions.Func1<Parameters, String> unregisterValidator(
      ComponentAddress validatorAddress, ComponentAddress ownerAccount) {
    return (params) ->
        String.format(
            """
            %s
            %s
            CALL_METHOD Address("%s") "unregister";
            """,
            params.faucetLockFeeLine(),
            params.createProofOfValidatorOwnerBadge(ownerAccount, validatorAddress),
            params.encode(validatorAddress));
  }

  public static Functions.Func1<Parameters, String> stakeValidatorAsNormalUser(
      ComponentAddress stakingAccount, ComponentAddress validatorAddress) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "free";
            TAKE_ALL_FROM_WORKTOP Address("%s") Bucket("xrd");
            CALL_METHOD Address("%s") "stake" Bucket("xrd");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(),
            params.encode(FAUCET),
            params.encode(XRD),
            params.encode(validatorAddress),
            params.encode(stakingAccount));
  }

  public static Functions.Func1<Parameters, String> stakeValidatorAsOwner(
      ComponentAddress stakingAccount,
      ComponentAddress validatorAddress,
      ComponentAddress ownerAccount) {
    return (params) ->
        String.format(
            """
            %s
            %s
            CALL_METHOD Address("%s") "free";
            TAKE_ALL_FROM_WORKTOP Address("%s") Bucket("xrd");
            CALL_METHOD Address("%s") "stake_as_owner" Bucket("xrd");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(),
            params.createProofOfValidatorOwnerBadge(ownerAccount, validatorAddress),
            params.encode(FAUCET),
            params.encode(XRD),
            params.encode(validatorAddress),
            params.encode(stakingAccount));
  }

  public static Functions.Func1<Parameters, String> unstakeValidator(
      ComponentAddress stakingAccount,
      ComponentAddress validatorAddress,
      ResourceAddress stakeUnitAddress) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s") Decimal("1");
            TAKE_ALL_FROM_WORKTOP Address("%s") Bucket("stake_units");
            CALL_METHOD Address("%s") "unstake" Bucket("stake_units");
            CALL_METHOD Address("%s") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
            """,
            params.faucetLockFeeLine(),
            params.encode(stakingAccount),
            params.encode(stakeUnitAddress),
            params.encode(stakeUnitAddress),
            params.encode(validatorAddress),
            params.encode(stakingAccount));
  }

  public static Functions.Func1<Parameters, String> claimXrdFromClaimResource(
      ComponentAddress stakingAccount,
      ComponentAddress validatorAddress,
      ResourceAddress claimResourceAddress) {
    return (params) ->
        String.format(
            """
            %s
            CALL_METHOD Address("%s") "withdraw" Address("%s");
            TAKE_ALL_FROM_WORKTOP Address("%s") Bucket("unstake");
            CALL_METHOD Address("%s") "claim_xrd" Bucket("unstake");
            TAKE_ALL_FROM_WORKTOP Address("%s") Bucket("xrd");
            CALL_METHOD Address("%s") "deposit" Bucket("xrd");
            """,
            params.faucetLockFeeLine(),
            params.encode(stakingAccount),
            params.encode(claimResourceAddress),
            params.encode(claimResourceAddress),
            params.encode(validatorAddress),
            params.encode(XRD),
            params.encode(stakingAccount));
  }

  public static Functions.Func1<Parameters, String> createAllowAllNonFungibleResource() {
    return (params) ->
        String.format(
            """
            %s
            CREATE_NON_FUNGIBLE_RESOURCE
                Enum<OwnerRole::None>()
                Enum<NonFungibleIdType::Integer>()
                false                            # Track Supply
                Tuple(                           # The NonFungibleDataSchema
                    Tuple(                       # The SBOR schema for the non-fungible data (here: empty schema)
                        Array<Enum>(),
                        Array<Tuple>(),
                        Array<Enum>()
                    ),
                    Enum<0u8>(66u8),             # The local type index in the schema (here: well-known - Unit tuple)
                    Array<String>()              # Mutable field names
                )
                Tuple(
                    Some(Tuple(                  # Mintable
                        Some(Enum<AccessRule::AllowAll>()),
                        Some(Enum<AccessRule::DenyAll>()),
                    )),
                    Some(Tuple(                  # Burnable
                        Some(Enum<AccessRule::AllowAll>()),
                        Some(Enum<AccessRule::DenyAll>()),
                    )),
                    None,                        # Freezable
                    None,                        # Recallable
                    None,                        # Restrict Withdraw
                    None,                        # Restrict Deposit
                    None,                        # Update Non Fungible Data
                )
                Tuple(                           # Metadata
                    Map<String, Tuple>(),
                    Map<String, Enum>()
                )
                None                             # Initial supply
            ;
            """,
            params.faucetLockFeeLine());
  }

  public static Functions.Func1<Parameters, String> mintNonFungiblesThenWithdrawAndBurnSome(
      ResourceAddress allowAllNfResource,
      ComponentAddress account,
      List<Integer> idsToMint,
      List<Integer> idsToWithdrawAndBurn) {
    var mintDataMap =
        String.join(
            ",",
            idsToMint.stream()
                .map(
                    id ->
                        String.format(
                            """
            NonFungibleLocalId("#%s#") => Tuple(Tuple())
            """,
                            id))
                .toList());
    var idsToBurn =
        String.join(
            ",",
            idsToWithdrawAndBurn.stream()
                .map(
                    id ->
                        String.format(
                            """
            NonFungibleLocalId("#%s#")
            """, id))
                .toList());
    return (params) ->
        String.format(
            """
            %s
            MINT_NON_FUNGIBLE
                Address("%s")
                Map<NonFungibleLocalId, Tuple>(%s);
            CALL_METHOD Address("%s") "deposit_batch" Expression("ENTIRE_WORKTOP");
            CALL_METHOD Address("%s") "withdraw_non_fungibles" Address("%s") Array<NonFungibleLocalId>(%s);
            TAKE_NON_FUNGIBLES_FROM_WORKTOP Address("%s") Array<NonFungibleLocalId>(%s) Bucket("burn");
            BURN_RESOURCE Bucket("burn");
            """,
            params.faucetLockFeeLine(),
            params.encode(allowAllNfResource),
            mintDataMap,
            params.encode(account),
            params.encode(account),
            params.encode(allowAllNfResource),
            idsToBurn,
            params.encode(allowAllNfResource),
            idsToBurn);
  }
}
