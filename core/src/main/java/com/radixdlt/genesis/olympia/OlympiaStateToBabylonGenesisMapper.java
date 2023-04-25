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
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisData2;
import com.radixdlt.genesis.GenesisDataChunk;
import com.radixdlt.genesis.GenesisResource;
import com.radixdlt.genesis.GenesisResourceAllocation;
import com.radixdlt.genesis.GenesisStakeAllocation;
import com.radixdlt.genesis.GenesisValidator;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.rev2.ComponentAddress2;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.ResourceAddress2;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.UInt32;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.math.BigInteger;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public final class OlympiaStateToBabylonGenesisMapper {
  private static final Logger log = LogManager.getLogger();

  private static final Decimal MAX_RESOURCE_SUPPLY_ON_BABYLON = Decimal.from(UInt256.TWO.pow(160));

  // all must be >= 1
  private static final int MAX_VALIDATORS_PER_TX = 100;
  private static final int MAX_STAKES_PER_TX = 200;
  private static final int MAX_RESOURCES_PER_TX = 100;
  private static final int MAX_RESOURCE_BALANCES_PER_TX = 200;
  private static final int MAX_XRD_BALANCES_PER_TX = 200;

  public static Result<GenesisData2, String> toGenesisData(OlympiaStateIR olympiaStateIR) {
    final var olympiaXrdResourceIndex = findXrdResourceIndex(olympiaStateIR);

    final ImmutableList.Builder<GenesisDataChunk.Validators> validatorsChunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<GenesisValidator> validatorsInCurrentChunkBuilder = ImmutableList.builder();
    int numValidatorsInCurrentChunk = 0;

    for (var i=0; i<olympiaStateIR.validators().size(); i++) {
      final var olympiaValidator = olympiaStateIR.validators().get(i);
      final var isLast = i == olympiaStateIR.validators().size() - 1;
      final var genesisValidator = convertValidator(olympiaStateIR.accounts(), olympiaValidator);
      validatorsInCurrentChunkBuilder.add(genesisValidator);
      numValidatorsInCurrentChunk += 1;
      if (numValidatorsInCurrentChunk >= MAX_VALIDATORS_PER_TX || isLast) {
        validatorsChunksBuilder.add(new GenesisDataChunk.Validators(validatorsInCurrentChunkBuilder.build()));
        validatorsInCurrentChunkBuilder = ImmutableList.builder();
        numValidatorsInCurrentChunk = 0;
      }
    }
    final var validatorsChunks = validatorsChunksBuilder.build();

    final var partitionedOlympiaBalances = olympiaStateIR.balances().stream()
            .collect(Collectors.partitioningBy(bal -> bal.resourceIndex() == olympiaXrdResourceIndex));
    final var olympiaXrdBalances = partitionedOlympiaBalances.getOrDefault(true, List.of());
    final var olympiaNonXrdBalances = partitionedOlympiaBalances.getOrDefault(false, List.of());

    final ImmutableList.Builder<GenesisDataChunk.XrdBalances> xrdBalancesChunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<Tuple2<ComponentAddress2, Decimal>> xrdBalancesInCurrentChunkBuilder = ImmutableList.builder();
    int numXrdBalancesInCurrentChunk = 0;

    for (var i=0; i<olympiaXrdBalances.size(); i++) {
      final var balance = olympiaXrdBalances.get(i);
      final var isLast = i == olympiaXrdBalances.size() - 1;
      final var account = olympiaStateIR.accounts().get(balance.accountIndex());
      xrdBalancesInCurrentChunkBuilder.add(Tuple2.of(
        ComponentAddress2.virtualEcdsaAccount(account.publicKeyBytes().asBytes()),
        Decimal.fromBigInt(balance.amount())));
      numXrdBalancesInCurrentChunk += 1;
      if (numXrdBalancesInCurrentChunk >= MAX_XRD_BALANCES_PER_TX || isLast) {
        xrdBalancesChunksBuilder.add(new GenesisDataChunk.XrdBalances(xrdBalancesInCurrentChunkBuilder.build()));
        xrdBalancesInCurrentChunkBuilder = ImmutableList.builder();
        numXrdBalancesInCurrentChunk = 0;
      }
    }
    final var xrdBalancesChunks = xrdBalancesChunksBuilder.build();

    // TODO: consider optimizing/merging with balance chunks preparation (and iterate the balances once)
    final BigInteger[] resourceTotalSuppliesOnOlympia = new BigInteger[olympiaStateIR.resources().size()];
    for (var balance: olympiaStateIR.balances()) {
      final var resIdx = balance.resourceIndex();
      if (resourceTotalSuppliesOnOlympia[resIdx] == null) {
        resourceTotalSuppliesOnOlympia[resIdx] = balance.amount();
      } else {
        resourceTotalSuppliesOnOlympia[resIdx] =
          resourceTotalSuppliesOnOlympia[resIdx].add(balance.amount());
      }
    }

    final Decimal[] resourceTotalSuppliesOnBabylon = new Decimal[olympiaStateIR.resources().size()];

    final ImmutableList.Builder<GenesisDataChunk.ResourceBalances> resourceBalancesChunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<Tuple2<ResourceAddress2, ImmutableList<GenesisResourceAllocation>>> resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
    LinkedHashMap<ComponentAddress2, Integer> accountsForCurrentResourceBalancesChunk = new LinkedHashMap<>();
    int numResourceBalancesInCurrentChunk = 0;

    final var olympiaNonXrdBalancesGrouped = olympiaNonXrdBalances.stream()
      .collect(Collectors.groupingBy(OlympiaStateIR.AccountBalance::resourceIndex))
            .entrySet().stream().toList();

    for (var i=0; i<olympiaNonXrdBalancesGrouped.size(); i++) {
      final var entry = olympiaNonXrdBalancesGrouped.get(i);
      final var resourceIndex = entry.getKey();
      final var balances = entry.getValue();
      final var resource = olympiaStateIR.resources().get(resourceIndex);

      final var totalSupplyOnOlympia =
        resourceTotalSuppliesOnOlympia[resourceIndex] == null
          ? BigInteger.ZERO
          : resourceTotalSuppliesOnOlympia[resourceIndex];

      // for current resource and current chunk
      ImmutableList.Builder<GenesisResourceAllocation> resourceAllocations = ImmutableList.builder();

      for (var j=0; j<balances.size(); j++) {
        final var balance = balances.get(j);
        final var olympiaAmount = balance.amount();
        final var babylonAmount =
          scaleResourceAmount(olympiaAmount, totalSupplyOnOlympia, MAX_RESOURCE_SUPPLY_ON_BABYLON.toBigInt());

        if (resourceTotalSuppliesOnBabylon[resourceIndex] == null) {
          resourceTotalSuppliesOnBabylon[resourceIndex] = babylonAmount;
        } else {
          resourceTotalSuppliesOnBabylon[resourceIndex] =
            resourceTotalSuppliesOnBabylon[resourceIndex].add(babylonAmount);
        }

        final var isLast =
          i == olympiaNonXrdBalancesGrouped.size() - 1 && j == balances.size() - 1;
        final var account = olympiaStateIR.accounts().get(balance.accountIndex());
        final var accountAddress = ComponentAddress2.virtualEcdsaAccount(account.publicKeyBytes().asBytes());

        // TODO: double check that this works as I expect it to work :)
        final var currAccountsSize = accountsForCurrentResourceBalancesChunk.size();
        final var accountIndex = accountsForCurrentResourceBalancesChunk.computeIfAbsent(
          accountAddress,
          unused -> currAccountsSize);
        resourceAllocations.add(new GenesisResourceAllocation(UInt32.fromNonNegativeInt(accountIndex), babylonAmount));
        numResourceBalancesInCurrentChunk += 1;
        if (numResourceBalancesInCurrentChunk >= MAX_RESOURCE_BALANCES_PER_TX || isLast) {
          resourceBalancesInCurrentChunkBuilder.add(Tuple2.of(
                  ResourceAddress2.globalFungible(resource.addr().getBytes()),
            resourceAllocations.build()
          ));
          resourceAllocations = ImmutableList.builder();

          final var accounts = accountsForCurrentResourceBalancesChunk.keySet().stream().collect(ImmutableList.toImmutableList());
          resourceBalancesChunksBuilder.add(new GenesisDataChunk.ResourceBalances(accounts, resourceBalancesInCurrentChunkBuilder.build()));
          accountsForCurrentResourceBalancesChunk = new LinkedHashMap<>();
          resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
          numResourceBalancesInCurrentChunk = 0;
        }
      }

      // IF the chunk hasn't been finalized, finalize the resource entry (but not the whole chunk)
      final var lastAllocations = resourceAllocations.build();
      if (!lastAllocations.isEmpty()) {
        resourceBalancesInCurrentChunkBuilder.add(Tuple2.of(
                ResourceAddress2.globalFungible(resource.addr().getBytes()),
                lastAllocations
        ));
      }
    }
    final var resourceBalancesChunks = resourceBalancesChunksBuilder.build();


    final ImmutableList.Builder<GenesisDataChunk.Resources> resourcesChunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<GenesisResource> resourcesInCurrentChunkBuilder = ImmutableList.builder();
    int numResourcesInCurrentChunk = 0;

    for (var i=0; i<olympiaStateIR.resources().size(); i++) {
      if (i == olympiaXrdResourceIndex) {
        // skip xrd
        continue;
      }
      final var olympiaResource = olympiaStateIR.resources().get(i);
      final var initialSupply = resourceTotalSuppliesOnBabylon[i] == null
        ? Decimal.zero()
        : resourceTotalSuppliesOnBabylon[i];
      final var isLast = i == olympiaStateIR.resources().size() - 1;
      final var genesisResource = convertResource(olympiaStateIR.accounts(), initialSupply, olympiaResource);
      resourcesInCurrentChunkBuilder.add(genesisResource);
      numResourcesInCurrentChunk += 1;
      if (numResourcesInCurrentChunk >= MAX_RESOURCES_PER_TX || isLast) {
        resourcesChunksBuilder.add(new GenesisDataChunk.Resources(resourcesInCurrentChunkBuilder.build()));
        resourcesInCurrentChunkBuilder = ImmutableList.builder();
        numResourcesInCurrentChunk = 0;
      }
    }
    final var resourceChunks = resourcesChunksBuilder.build();

    final ImmutableList.Builder<GenesisDataChunk.Stakes> stakesChunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<Tuple2<ECDSASecp256k1PublicKey, ImmutableList<GenesisStakeAllocation>>> stakesInCurrentChunkBuilder = ImmutableList.builder();
    LinkedHashMap<ComponentAddress2, Integer> accountsForCurrentStakesChunk = new LinkedHashMap<>();
    int numStakesInCurrentChunk = 0;

    final var olympiaStakesGrouped = olympiaStateIR.stakes().stream()
      .collect(Collectors.groupingBy(OlympiaStateIR.Stake::validatorIndex))
      .entrySet().stream().toList();

    for (var i=0; i<olympiaStakesGrouped.size(); i++) {
      final var entry = olympiaStakesGrouped.get(i);
      final var validatorIndex = entry.getKey();
      final var stakes = entry.getValue();
      final var validator = olympiaStateIR.validators().get(validatorIndex);

      final ECDSASecp256k1PublicKey validatorPublicKey;
      try {
        validatorPublicKey = ECDSASecp256k1PublicKey.fromBytes(validator.publicKeyBytes().asBytes());
      } catch (PublicKeyException e) {
        throw new RuntimeException(e);
      }

      // for current validator and current chunk
      ImmutableList.Builder<GenesisStakeAllocation> stakeAllocations = ImmutableList.builder();

      for (var j=0; j<stakes.size(); j++) {
        final var stake = stakes.get(j);
        // TODO: convert to XRD
        final var decimalXrdAmount = Decimal.from(stake.stakeUnitAmount());
        final var isLast =
                i == olympiaStakesGrouped.size() - 1 && j == stakes.size() - 1;
        final var account = olympiaStateIR.accounts().get(stake.accountIndex());
        final var accountAddress = ComponentAddress2.virtualEcdsaAccount(account.publicKeyBytes().asBytes());

        // TODO: double check that this works as I expect it to work :)
        final var currAccountsSize = accountsForCurrentStakesChunk.size();
        final var accountIndex = accountsForCurrentStakesChunk.computeIfAbsent(
                accountAddress,
                unused -> currAccountsSize);
        stakeAllocations.add(new GenesisStakeAllocation(UInt32.fromNonNegativeInt(accountIndex), decimalXrdAmount));
        numStakesInCurrentChunk += 1;
        if (numStakesInCurrentChunk >= MAX_STAKES_PER_TX || isLast) {
          stakesInCurrentChunkBuilder.add(Tuple2.of(
                  validatorPublicKey,
                  stakeAllocations.build()
          ));
          stakeAllocations = ImmutableList.builder();

          final var accounts = accountsForCurrentStakesChunk.keySet().stream().collect(ImmutableList.toImmutableList());
          stakesChunksBuilder.add(new GenesisDataChunk.Stakes(accounts, stakesInCurrentChunkBuilder.build()));
          accountsForCurrentStakesChunk = new LinkedHashMap<>();
          stakesInCurrentChunkBuilder = ImmutableList.builder();
          numStakesInCurrentChunk = 0;
        }
      }
      // IF the chunk hasn't been finalized, finalize the resource entry (but not the whole chunk)
      final var lastAllocations = stakeAllocations.build();
      if (!lastAllocations.isEmpty()) {
        stakesInCurrentChunkBuilder.add(Tuple2.of(
                validatorPublicKey,
                lastAllocations
        ));
      }
    }
    final var stakesChunks = stakesChunksBuilder.build();

    log.info("validators chunks {}", validatorsChunks.size());
    log.info("stakes chunks {}", stakesChunks.size());
    log.info("resources chunks {}", resourceChunks.size());
    log.info("resource bal chunks {}", resourceBalancesChunks.size());
    log.info("xrd bal chunks {}", xrdBalancesChunks.size());

    return Result.success(new GenesisData2(
      Stream.of(
        validatorsChunks.stream(),
        stakesChunks.stream(),
        resourceChunks.stream(),
        resourceBalancesChunks.stream(),
        xrdBalancesChunks.stream()
      )
      .flatMap(s -> s)
      .collect(ImmutableList.toImmutableList())
    ));
  }

  private static int findXrdResourceIndex(OlympiaStateIR olympiaStateIR) {
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
    BigInteger resourceMaxSupplyOnBabylon
  ) {
    if (resourceTotalSupplyOnOlympia.compareTo(resourceMaxSupplyOnBabylon) <= 0) {
      // No need to scale, guaranteed to fit
      return Decimal.fromBigInt(originalAmount);
    } else {
      // Scale it down, using integer div rounding
      final var scaledBigInt = resourceTotalSupplyOnOlympia
        .multiply(originalAmount)
        .divide(resourceMaxSupplyOnBabylon);
      return Decimal.fromBigInt(scaledBigInt);
    }
  }

  private static GenesisResource convertResource(
          ImmutableList<OlympiaStateIR.Account> accounts,
          Decimal initialSupply,
          OlympiaStateIR.Resource resource) {
    final var metadata =
        ImmutableList.of(
            Tuple2.of("symbol", resource.symbol()),
            Tuple2.of("name", resource.name()),
            Tuple2.of("description", resource.description()),
            Tuple2.of("url", resource.url()),
            Tuple2.of("icon_url", resource.iconUrl()));
    final var owner = resource.ownerAccountIndex()
            .map(idx -> ComponentAddress2.virtualEcdsaAccount(accounts.get(idx).publicKeyBytes().asBytes()));

    final var srcBytes = resource.addr().getBytes();
    var addrBytes = new byte[29];
    System.arraycopy(srcBytes, 0, addrBytes, 0, srcBytes.length);
    return new GenesisResource(
        addrBytes,
        initialSupply,
        metadata,
        Option.from(owner));
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

  private static GenesisValidator convertValidator(
    ImmutableList<OlympiaStateIR.Account> accounts,
    OlympiaStateIR.Validator olympiaValidator
  ) {
    final ECDSASecp256k1PublicKey publicKey;
    try {
      publicKey = ECDSASecp256k1PublicKey.fromBytes(olympiaValidator.publicKeyBytes().asBytes());
    } catch (PublicKeyException e) {
      // TODO: handle error?
      throw new RuntimeException(e);
    }
    final var metadata =
      ImmutableList.of(
        Tuple2.of("name", olympiaValidator.name()),
        Tuple2.of("url", olympiaValidator.url()));

    final var owner = accounts.get(olympiaValidator.ownerAccountIndex());
    return new GenesisValidator(
      publicKey,
      olympiaValidator.allowsDelegation(),
      olympiaValidator.isRegistered(),
      metadata,
      ComponentAddress2.virtualEcdsaAccount(owner.publicKeyBytes().asBytes()));
  }
}
