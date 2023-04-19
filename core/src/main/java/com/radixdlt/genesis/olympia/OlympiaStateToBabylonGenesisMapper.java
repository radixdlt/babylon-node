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

package com.radixdlt.genesis.olympia;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.Address;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.UInt32;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public final class OlympiaStateToBabylonGenesisMapper {

  public static Result<GenesisData, String> toGenesisData(OlympiaStateIR olympiaStateIR) {
    /* In the Olympia state all resources are treated equally (including XRD).
    In Babylon genesis, we handle it separately, so we need to extract it.
    What we do is:
    1) remove XRD from the original resources list
    2) adjust resourceIndex in balances */
    int olympiaXrdResourceIndex = -1;
    final var nonXrdResourcesBuilder = ImmutableList.<GenesisData.GenesisResource>builder();
    for (int i = 0; i < olympiaStateIR.resources().size(); i++) {
      final var resource = olympiaStateIR.resources().get(i);
      if (resource.addr().equals(REAddr.ofNativeToken())) {
        if (olympiaXrdResourceIndex > 0) {
          return Result.error("Duplicate native token found on the Olympia resource list!");
        }
        olympiaXrdResourceIndex = i;
      } else {
        nonXrdResourcesBuilder.add(convertResource(resource));
      }
    }
    final var nonXrdResources = nonXrdResourcesBuilder.build();
    if (olympiaXrdResourceIndex < 0) {
      return Result.error("Native token was not found on the Olympia resource list!");
    }

    /* We're using a map here to make it easier to move the resources from validators'
    "default" account to their corresponding owner accounts (see below).
    Using a map breaks the ordering, so it's very important to sort it again,
    once we convert it to a list. */
    Map<HashCode, GenesisData.XrdBalance> xrdBalances = new HashMap<>();
    Map<HashCode, List<GenesisData.NonXrdResourceBalance>> nonXrdResourceBalances =
        new HashMap<>();

    for (var balance: olympiaStateIR.balances()){
      final var account = olympiaStateIR.accounts().get(balance.accountIndex());
      if (balance.resourceIndex() == olympiaXrdResourceIndex) {
        // Each (account, resource) pair is guaranteed to only appear once on the original list,
        // so this is safe (does not override previous values)
        xrdBalances.put(
                account.publicKeyBytes(),
                new GenesisData.XrdBalance(UInt32.fromNonNegativeInt(balance.accountIndex()), Decimal.from(balance.amount())));
      } else {
        // Since we've separated out XRD from the list, the indices need to be adjusted
        final var originalIndex = balance.resourceIndex();
        final var indexInNonXrdResourceList =
                originalIndex < olympiaXrdResourceIndex ? originalIndex : originalIndex - 1;
        final var currNonXrdBalances =
                nonXrdResourceBalances.computeIfAbsent(account.publicKeyBytes(), k -> new ArrayList<>());
        // Similarly, no need to merge here, we can just add to the list
        // because each (account, resource) pair is guaranteed to appear at most once
        currNonXrdBalances.add(
                new GenesisData.NonXrdResourceBalance(
                        UInt32.fromNonNegativeInt(indexInNonXrdResourceList), UInt32.fromNonNegativeInt(balance.accountIndex()), Decimal.from(balance.amount())));
      }
    }

    /* This moves any XRD associated with validator's "default" account
    (i.e. an account corresponding to the validator public key) to an owner account
    - if one was set, i.e. if it is different from the "default" account.
    Note that it only applies to XRD, not any other resources or stakes.
    Note: if they didn't have any other resources or stakes, this will leave
    a leftover account in the accounts list (i.e. an account for which
    there are no balance/stake entries), which is fine. */
    /*
    TODO: revisit this. do we really want this?
          should this include other resources and/or stakes?
     */
    for (int i = 0; i < olympiaStateIR.validators().size(); i++) {
      final var validator = olympiaStateIR.validators().get(i);
      final var ownerAccountKey =
          olympiaStateIR.accounts().get(validator.ownerAccountIndex()).publicKeyBytes();
      final var defaultAccountKey = validator.publicKeyBytes();
      if (!ownerAccountKey.equals(defaultAccountKey)) {
        final var xrdInDefaultAccount =
            Optional.ofNullable(xrdBalances.remove(defaultAccountKey))
                .map(GenesisData.XrdBalance::amount)
                .orElse(Decimal.zero());
        final var currXrdInOwnerAccount =
            Optional.ofNullable(xrdBalances.get(ownerAccountKey))
                .map(GenesisData.XrdBalance::amount)
                .orElse(Decimal.zero());
        final var newXrdInOwnerAccount = currXrdInOwnerAccount.add(xrdInDefaultAccount);
        if (!newXrdInOwnerAccount.equals(Decimal.zero())) {
          xrdBalances.put(
              ownerAccountKey,
              new GenesisData.XrdBalance(UInt32.fromNonNegativeInt(validator.ownerAccountIndex()), newXrdInOwnerAccount));
        }
      }
    }

    final var validators = convertValidators(olympiaStateIR);
    final var accounts = convertAccounts(olympiaStateIR);
    final var stakes = convertStakes(olympiaStateIR);

    final var nonXrdBalancesList =
        nonXrdResourceBalances.values().stream()
            .flatMap(Collection::stream)
            .sorted(
                Comparator.comparing(GenesisData.NonXrdResourceBalance::accountIndex)
                    .thenComparing(GenesisData.NonXrdResourceBalance::resourceIndex)
                    .thenComparing(GenesisData.NonXrdResourceBalance::amount))
            .collect(ImmutableList.toImmutableList());

    final var xrdBalancesList =
        xrdBalances.values().stream()
            .sorted(
                Comparator.comparing(GenesisData.XrdBalance::accountIndex)
                    .thenComparing(GenesisData.XrdBalance::amount))
            .collect(ImmutableList.toImmutableList());

    return Result.success(
        new GenesisData(
            validators, nonXrdResources, accounts, nonXrdBalancesList, xrdBalancesList, stakes));
  }

  private static GenesisData.GenesisResource convertResource(OlympiaStateIR.Resource resource) {
    final var metadata =
        ImmutableList.of(
            Tuple.Tuple2.of("symbol", resource.symbol()),
            Tuple.Tuple2.of("name", resource.name()),
            Tuple.Tuple2.of("description", resource.description()),
            Tuple.Tuple2.of("url", resource.url()),
            Tuple.Tuple2.of("icon_url", resource.iconUrl()));
    return new GenesisData.GenesisResource(
        resource.addr().getBytesWithoutTypePrefix(), metadata, Option.from(resource.ownerAccountIndex()).map(UInt32::fromNonNegativeInt));
  }

  private static ImmutableList<GenesisData.GenesisValidator> convertValidators(
      OlympiaStateIR olympiaStateIR) {
    return olympiaStateIR.validators().stream()
        .map(
            validator -> {
              final ECDSASecp256k1PublicKey publicKey;
              try {
                 publicKey = ECDSASecp256k1PublicKey.fromBytes(validator.publicKeyBytes().asBytes());
              } catch (PublicKeyException e) {
                // TODO: handle error?
                throw new RuntimeException(e);
              }
              final var metadata =
                  ImmutableList.of(
                      Tuple.Tuple2.of("name", validator.name()),
                      Tuple.Tuple2.of("url", validator.url()));
              return new GenesisData.GenesisValidator(
                  publicKey,
                  Address.virtualAccountAddress(publicKey),
                  validator.allowsDelegation(),
                  validator.isRegistered(),
                  metadata);
            })
        .collect(ImmutableList.toImmutableList());
  }

  private static ImmutableList<ComponentAddress> convertAccounts(OlympiaStateIR olympiaStateIR) {
    return olympiaStateIR.accounts().stream()
        .map(account -> Address.uncheckedVirtualAccountAddress(account.publicKeyBytes().asBytes()))
        .collect(ImmutableList.toImmutableList());
  }

  private static ImmutableList<GenesisData.Stake> convertStakes(OlympiaStateIR olympiaStateIR) {
    final var stakesBuilder = ImmutableList.<GenesisData.Stake>builder();
    olympiaStateIR
        .stakes()
        .forEach(
            stake -> {
              final var validator = olympiaStateIR.validators().get(stake.validatorIndex());
              // XRD won't overflow
              final var xrdAmount =
                  validator
                      .totalStakedXrd()
                      .multiply(stake.stakeUnitAmount())
                      .divide(validator.totalStakeUnits());
              stakesBuilder.add(
                  new GenesisData.Stake(
                          UInt32.fromNonNegativeInt(stake.validatorIndex()), UInt32.fromNonNegativeInt(stake.accountIndex()), Decimal.from(xrdAmount)));
            });
    return stakesBuilder.build();
  }
}
