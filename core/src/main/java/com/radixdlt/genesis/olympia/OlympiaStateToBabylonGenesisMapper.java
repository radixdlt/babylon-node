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
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.genesis.*;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.Address;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.ResourceAddress;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.math.BigInteger;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Optional;
import java.util.function.BiFunction;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class OlympiaStateToBabylonGenesisMapper {
  private static final Logger log = LogManager.getLogger();

  // TODO(REP-73): Decimal.from(UInt256.TWO.pow(160))
  private static final Decimal MAX_GENESIS_RESOURCE_SUPPLY =
      Decimal.from(UInt256.from("1000000000000000000"));

  // all must be >= 1
  private static final int MAX_VALIDATORS_PER_TX = 100;
  private static final int MAX_STAKES_PER_TX = 400;
  private static final int MAX_RESOURCES_PER_TX = 100;
  private static final int MAX_RESOURCE_BALANCES_PER_TX = 500;
  private static final int MAX_XRD_BALANCES_PER_TX = 500;

  // Given a list of input elements I, creates chunks that are in a
  // form of List<E> wrapped in objects of type C (f.e. record C(ImmutableList<E> value)).
  // More specifically these are Validators, Resources and XrdBalances chunks.
  // The size limit parameter (maxPerChunk) applies to the length of a list in a chunk.
  private static <I, E, C> ImmutableList<C> createChunks(
      List<I> input,
      int maxPerChunk,
      BiFunction<Integer, I, Optional<E>> mkElement,
      Function<ImmutableList<E>, C> mkChunk) {
    final ImmutableList.Builder<C> chunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<E> elementsInCurrentChunkBuilder = ImmutableList.builder();
    int numElementsInCurrentChunk = 0;
    final var lastIndex = input.size() - 1;
    for (var i = 0; i < input.size(); i++) {
      final var next = input.get(i);
      final var isLast = i == lastIndex;
      final var maybeElement = mkElement.apply(i, next);
      if (maybeElement.isEmpty()) {
        continue;
      }
      final var element = maybeElement.get();
      elementsInCurrentChunkBuilder.add(element);
      numElementsInCurrentChunk += 1;
      if (numElementsInCurrentChunk >= maxPerChunk || isLast) {
        chunksBuilder.add(mkChunk.apply(elementsInCurrentChunkBuilder.build()));
        elementsInCurrentChunkBuilder = ImmutableList.builder();
        numElementsInCurrentChunk = 0;
      }
    }
    return chunksBuilder.build();
  }

  public static GenesisData toGenesisData(OlympiaStateIR olympiaStateIR) {
    final var validatorsChunks =
        createChunks(
            olympiaStateIR.validators(),
            MAX_VALIDATORS_PER_TX,
            (idx, olympiaValidator) ->
                Optional.of(convertValidator(olympiaStateIR.accounts(), olympiaValidator)),
            GenesisDataChunk.Validators::new);

    final var olympiaXrdResourceIndex = findXrdResourceIndex(olympiaStateIR);
    final var partitionedOlympiaBalances =
        olympiaStateIR.balances().stream()
            .collect(
                Collectors.partitioningBy(bal -> bal.resourceIndex() == olympiaXrdResourceIndex));
    final var olympiaXrdBalances = partitionedOlympiaBalances.getOrDefault(true, List.of());
    final var olympiaNonXrdBalances = partitionedOlympiaBalances.getOrDefault(false, List.of());

    final var xrdBalancesChunks =
        createChunks(
            olympiaXrdBalances,
            MAX_XRD_BALANCES_PER_TX,
            (idx, olympiaXrdBalance) -> {
              final var account = olympiaStateIR.accounts().get(olympiaXrdBalance.accountIndex());
              return Optional.of(
                  Tuple.Tuple2.of(
                      Address.virtualAccountAddress(account.publicKeyBytes().asBytes()),
                      Decimal.unsafeFromBigInt(olympiaXrdBalance.amount())));
            },
            GenesisDataChunk.XrdBalances::new);

    final BigInteger[] resourceTotalSuppliesOnOlympia =
        new BigInteger[olympiaStateIR.resources().size()];
    for (var balance : olympiaStateIR.balances()) {
      final var resIdx = balance.resourceIndex();
      if (resourceTotalSuppliesOnOlympia[resIdx] == null) {
        resourceTotalSuppliesOnOlympia[resIdx] = balance.amount();
      } else {
        resourceTotalSuppliesOnOlympia[resIdx] =
            resourceTotalSuppliesOnOlympia[resIdx].add(balance.amount());
      }
    }

    // We'll be updating this on the occasion of preparing the resource balances chunks below
    final Decimal[] resourceTotalSuppliesOnBabylon = new Decimal[olympiaStateIR.resources().size()];

    final ImmutableList.Builder<GenesisDataChunk.ResourceBalances> resourceBalancesChunksBuilder =
        ImmutableList.builder();
    ImmutableList.Builder<Tuple.Tuple2<ResourceAddress, ImmutableList<GenesisResourceAllocation>>>
        resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
    LinkedHashMap<ComponentAddress, Integer> accountsForCurrentResourceBalancesChunk =
        new LinkedHashMap<>();
    int numResourceBalancesInCurrentChunk = 0;

    final var olympiaNonXrdBalancesGrouped =
        olympiaNonXrdBalances.stream()
            .collect(Collectors.groupingBy(OlympiaStateIR.AccountBalance::resourceIndex))
            .entrySet()
            .stream()
            .toList();

    for (var i = 0; i < olympiaNonXrdBalancesGrouped.size(); i++) {
      final var entry = olympiaNonXrdBalancesGrouped.get(i);
      final var resourceIndex = entry.getKey();
      final var balances = entry.getValue();
      final var resource = olympiaStateIR.resources().get(resourceIndex);

      final var totalSupplyOnOlympia =
          resourceTotalSuppliesOnOlympia[resourceIndex] == null
              ? BigInteger.ZERO
              : resourceTotalSuppliesOnOlympia[resourceIndex];

      ImmutableList.Builder<GenesisResourceAllocation> currentAllocations = ImmutableList.builder();

      for (var j = 0; j < balances.size(); j++) {
        final var balance = balances.get(j);
        final var olympiaAmount = balance.amount();
        final var babylonAmount =
            scaleResourceAmount(
                olympiaAmount, totalSupplyOnOlympia, MAX_GENESIS_RESOURCE_SUPPLY.toBigInt());

        if (resourceTotalSuppliesOnBabylon[resourceIndex] == null) {
          resourceTotalSuppliesOnBabylon[resourceIndex] = babylonAmount;
        } else {
          resourceTotalSuppliesOnBabylon[resourceIndex] =
              resourceTotalSuppliesOnBabylon[resourceIndex].add(babylonAmount);
        }

        final var isLast = i == olympiaNonXrdBalancesGrouped.size() - 1 && j == balances.size() - 1;
        final var account = olympiaStateIR.accounts().get(balance.accountIndex());
        final var accountAddress =
            Address.virtualAccountAddress(account.publicKeyBytes().asBytes());

        final var currAccountsSize = accountsForCurrentResourceBalancesChunk.size();
        final var accountIndex =
            accountsForCurrentResourceBalancesChunk.computeIfAbsent(
                accountAddress, unused -> currAccountsSize);
        currentAllocations.add(
            new GenesisResourceAllocation(UInt32.fromNonNegativeInt(accountIndex), babylonAmount));
        numResourceBalancesInCurrentChunk += 1;
        if (numResourceBalancesInCurrentChunk >= MAX_RESOURCE_BALANCES_PER_TX || isLast) {
          resourceBalancesInCurrentChunkBuilder.add(
              Tuple.Tuple2.of(
                  ResourceAddress.globalFungible(resource.addr().getBytes()),
                  currentAllocations.build()));
          currentAllocations = ImmutableList.builder();

          final var accounts =
              accountsForCurrentResourceBalancesChunk.keySet().stream()
                  .collect(ImmutableList.toImmutableList());
          resourceBalancesChunksBuilder.add(
              new GenesisDataChunk.ResourceBalances(
                  accounts, resourceBalancesInCurrentChunkBuilder.build()));
          accountsForCurrentResourceBalancesChunk = new LinkedHashMap<>();
          resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
          numResourceBalancesInCurrentChunk = 0;
        }
      }

      final var remainingAllocations = currentAllocations.build();
      if (!remainingAllocations.isEmpty()) {
        resourceBalancesInCurrentChunkBuilder.add(
            Tuple.Tuple2.of(
                ResourceAddress.globalFungible(resource.addr().getBytes()), remainingAllocations));
      }
    }
    final var resourceBalancesChunks = resourceBalancesChunksBuilder.build();

    final var resourceChunks =
        createChunks(
            olympiaStateIR.resources(),
            MAX_RESOURCES_PER_TX,
            (idx, olympiaResource) -> {
              if (idx == olympiaXrdResourceIndex) {
                // skip xrd
                return Optional.empty();
              }
              final var initialSupply =
                  resourceTotalSuppliesOnBabylon[idx] == null
                      ? Decimal.ZERO
                      : resourceTotalSuppliesOnBabylon[idx];
              return Optional.of(
                  convertResource(olympiaStateIR.accounts(), initialSupply, olympiaResource));
            },
            GenesisDataChunk.Resources::new);

    final var stakesChunks = prepareStakesChunks(olympiaStateIR);

    return new GenesisData(
        Stream.of(
                validatorsChunks.stream(),
                stakesChunks.stream(),
                resourceChunks.stream(),
                resourceBalancesChunks.stream(),
                xrdBalancesChunks.stream())
            .flatMap(s -> s)
            .collect(ImmutableList.toImmutableList()),
        UInt64.fromNonNegativeLong(0L),
        UInt32.fromNonNegativeInt(10),
        UInt64.fromNonNegativeLong(100),
        UInt64.fromNonNegativeLong(10));
  }

  private static ImmutableList<GenesisDataChunk.Stakes> prepareStakesChunks(
      OlympiaStateIR olympiaStateIR) {
    final var olympiaStakesGrouped =
        olympiaStateIR.stakes().stream()
            .collect(Collectors.groupingBy(OlympiaStateIR.Stake::validatorIndex))
            .entrySet()
            .stream()
            .toList();

    final ImmutableList.Builder<GenesisDataChunk.Stakes> chunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<
            Tuple.Tuple2<ECDSASecp256k1PublicKey, ImmutableList<GenesisStakeAllocation>>>
        stakesInCurrentChunkBuilder = ImmutableList.builder();
    LinkedHashMap<ComponentAddress, Integer> accountsForCurrentChunk = new LinkedHashMap<>();
    int numStakesInCurrentChunk = 0;

    for (var i = 0; i < olympiaStakesGrouped.size(); i++) {
      final var entry = olympiaStakesGrouped.get(i);
      final var validatorIndex = entry.getKey();
      final var stakes = entry.getValue();
      final var validator = olympiaStateIR.validators().get(validatorIndex);

      final ECDSASecp256k1PublicKey validatorPublicKey;
      try {
        validatorPublicKey =
            ECDSASecp256k1PublicKey.fromBytes(validator.publicKeyBytes().asBytes());
      } catch (PublicKeyException e) {
        throw new RuntimeException(e);
      }

      ImmutableList.Builder<GenesisStakeAllocation> allocationsBuilder = ImmutableList.builder();

      for (var j = 0; j < stakes.size(); j++) {
        final var stake = stakes.get(j);
        // TODO: convert to XRD
        final var decimalXrdAmount = Decimal.from(stake.stakeUnitAmount());
        final var isLast = i == olympiaStakesGrouped.size() - 1 && j == stakes.size() - 1;
        final var account = olympiaStateIR.accounts().get(stake.accountIndex());
        final var accountAddress =
            Address.virtualAccountAddress(account.publicKeyBytes().asBytes());

        final var currAccountsSize = accountsForCurrentChunk.size();
        final var accountIndex =
            accountsForCurrentChunk.computeIfAbsent(accountAddress, unused -> currAccountsSize);
        allocationsBuilder.add(
            new GenesisStakeAllocation(UInt32.fromNonNegativeInt(accountIndex), decimalXrdAmount));
        numStakesInCurrentChunk += 1;
        if (numStakesInCurrentChunk >= MAX_STAKES_PER_TX || isLast) {
          stakesInCurrentChunkBuilder.add(
              Tuple.Tuple2.of(validatorPublicKey, allocationsBuilder.build()));
          allocationsBuilder = ImmutableList.builder();

          final var accounts =
              accountsForCurrentChunk.keySet().stream().collect(ImmutableList.toImmutableList());
          chunksBuilder.add(
              new GenesisDataChunk.Stakes(accounts, stakesInCurrentChunkBuilder.build()));
          accountsForCurrentChunk = new LinkedHashMap<>();
          stakesInCurrentChunkBuilder = ImmutableList.builder();
          numStakesInCurrentChunk = 0;
        }
      }
      final var remainingAllocations = allocationsBuilder.build();
      if (!remainingAllocations.isEmpty()) {
        stakesInCurrentChunkBuilder.add(Tuple.Tuple2.of(validatorPublicKey, remainingAllocations));
      }
    }
    return chunksBuilder.build();
  }

  public static int findXrdResourceIndex(OlympiaStateIR olympiaStateIR) {
    int olympiaXrdResourceIndex = -1;
    for (int i = 0; i < olympiaStateIR.resources().size(); i++) {
      final var resource = olympiaStateIR.resources().get(i);
      if (resource.addr().equals(REAddr.ofNativeToken())) {
        if (olympiaXrdResourceIndex > 0) {
          throw new RuntimeException("Duplicate native token found on the Olympia resource list!");
        }
        olympiaXrdResourceIndex = i;
      }
    }
    if (olympiaXrdResourceIndex < 0) {
      throw new RuntimeException("Native token was not found on the Olympia resource list!");
    } else {
      return olympiaXrdResourceIndex;
    }
  }

  private static Decimal scaleResourceAmount(
      BigInteger originalAmount,
      BigInteger resourceTotalSupplyOnOlympia,
      BigInteger resourceMaxSupplyOnBabylon) {
    if (resourceTotalSupplyOnOlympia.compareTo(resourceMaxSupplyOnBabylon) <= 0) {
      // No need to scale, guaranteed to fit
      return Decimal.unsafeFromBigInt(originalAmount);
    } else {
      // Scale it down, using integer div rounding
      final var scaledBigInt =
          resourceMaxSupplyOnBabylon.multiply(originalAmount).divide(resourceTotalSupplyOnOlympia);
      return Decimal.unsafeFromBigInt(scaledBigInt);
    }
  }

  private static GenesisResource convertResource(
      ImmutableList<OlympiaStateIR.Account> accounts,
      Decimal initialSupply,
      OlympiaStateIR.Resource resource) {
    final var metadata =
        ImmutableList.of(
            Tuple.Tuple2.of("symbol", resource.symbol()),
            Tuple.Tuple2.of("name", resource.name()),
            Tuple.Tuple2.of("description", resource.description()),
            Tuple.Tuple2.of("url", resource.url()),
            Tuple.Tuple2.of("icon_url", resource.iconUrl()));
    final var owner =
        resource
            .ownerAccountIndex()
            .map(
                idx -> Address.virtualAccountAddress(accounts.get(idx).publicKeyBytes().asBytes()));

    final var srcBytes = resource.addr().getBytes();
    var addrBytes = new byte[29];
    System.arraycopy(srcBytes, 0, addrBytes, 0, srcBytes.length);
    return new GenesisResource(addrBytes, initialSupply, metadata, Option.from(owner));
  }

  private static GenesisValidator convertValidator(
      ImmutableList<OlympiaStateIR.Account> accounts, OlympiaStateIR.Validator olympiaValidator) {
    final ECDSASecp256k1PublicKey publicKey;
    try {
      publicKey = ECDSASecp256k1PublicKey.fromBytes(olympiaValidator.publicKeyBytes().asBytes());
    } catch (PublicKeyException e) {
      throw new IllegalStateException("Olympia validator public key is invalid", e);
    }
    final var metadata =
        ImmutableList.of(
            Tuple.Tuple2.of("name", olympiaValidator.name()),
            Tuple.Tuple2.of("url", olympiaValidator.url()));

    final var owner = accounts.get(olympiaValidator.ownerAccountIndex());
    return new GenesisValidator(
        publicKey,
        olympiaValidator.allowsDelegation(),
        olympiaValidator.isRegistered(),
        metadata,
        Address.virtualAccountAddress(owner.publicKeyBytes().asBytes()));
  }
}
