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

package com.radixdlt.genesis.olympia.converter.resource;

import static com.radixdlt.genesis.olympia.converter.GenesisDataChunkUtils.createChunks;
import static com.radixdlt.lang.Tuple.tuple;

import com.google.common.collect.ImmutableList;
import com.radixdlt.consensus.Blake2b256Hasher;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.genesis.GenesisDataChunk;
import com.radixdlt.genesis.GenesisResource;
import com.radixdlt.genesis.GenesisResourceAllocation;
import com.radixdlt.genesis.olympia.bech32.Bits;
import com.radixdlt.genesis.olympia.bech32.OlympiaBech32;
import com.radixdlt.genesis.olympia.converter.OlympiaToBabylonConverterConfig;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.Address;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.MetadataValue;
import com.radixdlt.rev2.ResourceAddress;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UniqueListBuilder;
import java.math.BigInteger;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public final class OlympiaNonXrdConverter {
  private static final Hasher HASHER = new Blake2b256Hasher(DefaultSerialization.getInstance());

  static Tuple2<
          ImmutableList<GenesisDataChunk.Resources>,
          ImmutableList<GenesisDataChunk.ResourceBalances>>
      prepareResourcesAndBalancesChunks(
          OlympiaToBabylonConverterConfig config,
          List<OlympiaStateIR.Account> accounts,
          List<OlympiaStateIR.Resource> resources,
          int olympiaXrdResourceIndex,
          List<OlympiaStateIR.AccountBalance> olympiaNonXrdBalances) {
    /*
    Non-xrd resource converter serves two purposes:
       1) it creates the genesis chunks containing the resources themselves
       2) it creates the chunks containing resource allocations

    There is a total-supply-limiting mechanism that adjusts the balances of large-supply Olympia resources.
    We use the following procedure to make it work:
       - we first create the *resource balances* chunks
       - as we iterate the Olympia balance entries, we check
         resourceTotalSuppliesOnOlympia to see whether the balance entry
         corresponds to a resource whose total supply is to be scaled down
       - we scale it (the individual balance entry) down if necessary
       - we add a new balance value to resourceTotalSuppliesOnBabylon
       - we can then use resourceTotalSuppliesOnBabylon to create
         the *resource* chunks with correct initial supplies (specifically when
         balance scaling has been applied)
    */

    final var resourceTotalSuppliesOnOlympia =
        olympiaTotalSupplies(resources, olympiaNonXrdBalances);

    final Decimal[] resourceTotalSuppliesOnBabylon = new Decimal[resources.size()];

    final var resourceBalancesChunksBuilder =
        ImmutableList.<GenesisDataChunk.ResourceBalances>builder();
    ImmutableList.Builder<Tuple2<ResourceAddress, ImmutableList<GenesisResourceAllocation>>>
        resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
    UniqueListBuilder<ComponentAddress> accountsForCurrentResourceBalancesChunk =
        new UniqueListBuilder<>();
    int numResourceBalancesInCurrentChunk = 0;

    final var olympiaNonXrdBalancesGroupedByResourceIdx =
        olympiaNonXrdBalances.stream()
            .collect(Collectors.groupingBy(OlympiaStateIR.AccountBalance::resourceIndex))
            .entrySet()
            .stream()
            .toList();

    for (var i = 0; i < olympiaNonXrdBalancesGroupedByResourceIdx.size(); i++) {
      final var entry = olympiaNonXrdBalancesGroupedByResourceIdx.get(i);
      final var resourceIndex = entry.getKey();
      final var balances = entry.getValue();
      final var resource = resources.get(resourceIndex);

      final var totalSupplyOnOlympia =
          resourceTotalSuppliesOnOlympia[resourceIndex] == null
              ? BigInteger.ZERO
              : resourceTotalSuppliesOnOlympia[resourceIndex];

      var currentAllocations = ImmutableList.<GenesisResourceAllocation>builder();

      for (var j = 0; j < balances.size(); j++) {
        final var balance = balances.get(j);
        final var olympiaAmount = balance.amount();
        final var babylonAmount =
            scaleResourceAmount(
                olympiaAmount,
                totalSupplyOnOlympia,
                config.maxGenesisResourceUnscaledSupply().toBigIntegerSubunits());

        if (resourceTotalSuppliesOnBabylon[resourceIndex] == null) {
          resourceTotalSuppliesOnBabylon[resourceIndex] = babylonAmount;
        } else {
          resourceTotalSuppliesOnBabylon[resourceIndex] =
              resourceTotalSuppliesOnBabylon[resourceIndex].add(babylonAmount);
        }

        final var isLast =
            i == olympiaNonXrdBalancesGroupedByResourceIdx.size() - 1 && j == balances.size() - 1;

        final var account = accounts.get(balance.accountIndex());
        final var accountAddress =
            Address.virtualAccountAddress(account.publicKeyBytes().asBytes());

        final var accountIndex =
            accountsForCurrentResourceBalancesChunk.insertIfMissingAndGetIndex(accountAddress);

        currentAllocations.add(
            new GenesisResourceAllocation(UInt32.fromNonNegativeInt(accountIndex), babylonAmount));

        numResourceBalancesInCurrentChunk += 1;
        if (numResourceBalancesInCurrentChunk >= config.maxNonXrdResourceBalancesPerChunk()
            || isLast) {
          // We're over the per-chunk entries limit, or this is the last entry
          // Build the chunk and add it to the list builder,
          // reset the counter and the accounts index for the next chunk
          resourceBalancesInCurrentChunkBuilder.add(
              tuple(olympiaToBabylonResourceAddress(resource.addr()), currentAllocations.build()));
          currentAllocations = ImmutableList.builder();

          resourceBalancesChunksBuilder.add(
              new GenesisDataChunk.ResourceBalances(
                  accountsForCurrentResourceBalancesChunk.build(),
                  resourceBalancesInCurrentChunkBuilder.build()));
          accountsForCurrentResourceBalancesChunk = new UniqueListBuilder<>();
          resourceBalancesInCurrentChunkBuilder = ImmutableList.builder();
          numResourceBalancesInCurrentChunk = 0;
        }
      }

      // We've consumed all entries under the current resource key
      // There might still be capacity for more entries (for different resources)
      // So we're going to add an entry for the current resource
      // and continue building the chunk
      final var lastAllocationsForCurrentResource = currentAllocations.build();
      if (!lastAllocationsForCurrentResource.isEmpty()) {
        resourceBalancesInCurrentChunkBuilder.add(
            tuple(
                olympiaToBabylonResourceAddress(resource.addr()),
                lastAllocationsForCurrentResource));
        // Note that we don't reset any chunk-scoped builders/counters
        // No need to reset currentAllocations here, we'll create a fresh instance
        // on next loop iter.
      }
    }

    final var resourceBalancesChunks = resourceBalancesChunksBuilder.build();

    final var resourcesChunks =
        prepareResourcesChunks(
            config, accounts, resources, olympiaXrdResourceIndex, resourceTotalSuppliesOnBabylon);

    return tuple(resourcesChunks, resourceBalancesChunks);
  }

  private static BigInteger[] olympiaTotalSupplies(
      List<OlympiaStateIR.Resource> resources,
      List<OlympiaStateIR.AccountBalance> olympiaNonXrdBalances) {
    // Just a note: resourceTotalSuppliesOnOlympia includes a spot
    // for XRD, but it's always null (because olympiaNonXrdBalances doesn't contain
    // and balances where resourceIndex == xrdIndex).
    // When we later use those values, we never use XRD's index.
    final BigInteger[] resourceTotalSuppliesOnOlympia = new BigInteger[resources.size()];
    for (var balance : olympiaNonXrdBalances) {
      final var resIdx = balance.resourceIndex();
      if (resourceTotalSuppliesOnOlympia[resIdx] == null) {
        resourceTotalSuppliesOnOlympia[resIdx] = balance.amount();
      } else {
        resourceTotalSuppliesOnOlympia[resIdx] =
            resourceTotalSuppliesOnOlympia[resIdx].add(balance.amount());
      }
    }
    return resourceTotalSuppliesOnOlympia;
  }

  private static Decimal scaleResourceAmount(
      BigInteger originalAmount,
      BigInteger resourceTotalSupplyOnOlympia,
      BigInteger resourceMaxSupplyOnBabylon) {
    if (resourceTotalSupplyOnOlympia.compareTo(resourceMaxSupplyOnBabylon) <= 0) {
      // No need to scale, guaranteed to fit
      return Decimal.fromBigIntegerSubunits(originalAmount);
    } else {
      // Scale it down, using integer div rounding
      final var scaledBigInt =
          resourceMaxSupplyOnBabylon.multiply(originalAmount).divide(resourceTotalSupplyOnOlympia);
      return Decimal.fromBigIntegerSubunits(scaledBigInt);
    }
  }

  private static ImmutableList<GenesisDataChunk.Resources> prepareResourcesChunks(
      OlympiaToBabylonConverterConfig config,
      List<OlympiaStateIR.Account> accounts,
      List<OlympiaStateIR.Resource> resources,
      int olympiaXrdResourceIndex,
      Decimal[] resourceTotalSuppliesOnBabylon) {
    return createChunks(
        resources,
        config.maxResourcesPerChunk(),
        (idx, olympiaResource) -> {
          if (idx == olympiaXrdResourceIndex) {
            // skip XRD
            return Optional.empty();
          }
          final var initialSupply =
              resourceTotalSuppliesOnBabylon[idx] == null
                  ? Decimal.ZERO
                  : resourceTotalSuppliesOnBabylon[idx];
          return Optional.of(convertResource(accounts, initialSupply, olympiaResource));
        },
        GenesisDataChunk.Resources::new);
  }

  private static GenesisResource convertResource(
      List<OlympiaStateIR.Account> accounts,
      Decimal initialSupply,
      OlympiaStateIR.Resource resource) {
    final var metadataBuilder = ImmutableList.<Tuple2<String, MetadataValue>>builder();
    metadataBuilder.addAll(
        List.of(
            tuple("symbol", new MetadataValue.String(resource.symbol())),
            tuple("name", new MetadataValue.String(resource.name())),
            tuple("description", new MetadataValue.String(resource.description()))));
    if (!resource.url().isBlank()) {
      metadataBuilder.add(tuple("info_url", new MetadataValue.Url(resource.url())));
    }
    if (!resource.iconUrl().isBlank()) {
      metadataBuilder.add(tuple("icon_url", new MetadataValue.Url(resource.iconUrl())));
    }

    final var owner =
        resource
            .ownerAccountIndex()
            .map(
                idx -> Address.virtualAccountAddress(accounts.get(idx).publicKeyBytes().asBytes()));

    final var address = olympiaToBabylonResourceAddress(resource.addr());
    return new GenesisResource(
        address, initialSupply, metadataBuilder.build(), Option.from(owner));
  }

  public static REAddr olympiaRriToReAddr(String rri) {
    final var decodedBech32 = OlympiaBech32.decode(rri);
    final var rriBytes =
        Bits.convertBits(decodedBech32.data, 0, decodedBech32.data.length, 5, 8, false);
    return REAddr.of(rriBytes);
  }

  public static ResourceAddress olympiaToBabylonResourceAddress(REAddr rri) {
    return Address.globalFungible(olympiaToBabylonResourceAddressBytes(rri));
  }

  public static byte[] olympiaToBabylonResourceAddressBytes(REAddr rri) {
    final var hash = HASHER.hashBytes(rri.getBytes());
    return Arrays.copyOfRange(
        hash.asBytes(), 0, ResourceAddress.BYTE_LENGTH - ResourceAddress.ENTITY_ID_LEN);
  }
}
