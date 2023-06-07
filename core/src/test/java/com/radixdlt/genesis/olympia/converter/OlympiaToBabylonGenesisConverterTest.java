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

package com.radixdlt.genesis.olympia.converter;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.genesis.olympia.converter.OlympiaToBabylonGenesisConverterTestUtils.createXrdResource;
import static com.radixdlt.lang.Tuple.tuple;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.Blake2b256Hasher;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisDataChunk;
import com.radixdlt.genesis.GenesisValidator;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.identifiers.Address;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.lang.Tuple.Tuple2;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REv2StateReader;
import com.radixdlt.rev2.ResourceAddress;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.utils.UInt128;
import com.radixdlt.utils.UInt256;
import com.radixdlt.utils.UniqueListBuilder;
import java.math.BigInteger;
import java.security.Security;
import java.util.*;
import java.util.stream.Collectors;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

@RunWith(Parameterized.class)
public final class OlympiaToBabylonGenesisConverterTest {
  private static final Hasher HASHER = new Blake2b256Hasher(DefaultSerialization.getInstance());

  static {
    Security.insertProviderAt(new BouncyCastleProvider(), 1);
  }

  private static final OlympiaToBabylonConverterConfig CONVERTER_CONFIG =
      new OlympiaToBabylonConverterConfig(
          10,
          10,
          10,
          10,
          10,
          OlympiaToBabylonConverterConfig.DEFAULT.maxGenesisResourceUnscaledSupply());

  @Parameterized.Parameters
  public static Collection<Object[]> parameters() {
    return List.of(
        new Object[][] {
          {100, 100, 1000, 100, 100, 50},
          // Test XRD resource with edge index
          {10, 10, 10, 10, 1, 0},
          {10, 10, 10, 10, 1, 10}
        });
  }

  private final Random random = new Random(1234);
  private final int numValidators;
  private final int numStakesPerValidator;
  private final int numXrdBalances;
  private final int numResources;
  private final int numBalancesPerResource;
  private final int xrdResourceIndex;

  public OlympiaToBabylonGenesisConverterTest(
      int numValidators,
      int numStakesPerValidator,
      int numXrdBalances,
      int numResources,
      int numBalancesPerResource,
      int xrdResourceIndex) {
    this.numValidators = numValidators;
    this.numStakesPerValidator = numStakesPerValidator;
    this.numXrdBalances = numXrdBalances;
    this.numResources = numResources;
    this.numBalancesPerResource = numBalancesPerResource;
    this.xrdResourceIndex = xrdResourceIndex;
  }

  @Test
  public void test_babylon_genesis_chunks_match_olympia_state() {
    final var stateAndSummary = prepareOlympiaState();
    final var olympiaState = stateAndSummary.first();
    final var stateSummary = stateAndSummary.last();
    final var converted =
        OlympiaStateToBabylonGenesisConverter.toGenesisData(olympiaState, CONVERTER_CONFIG);
    checkBabylonGenesisMatchesStateSummary(converted, stateSummary);

    // Create a deterministic test, just to check that the engine can successfully
    // ingest the generated genesis data.
    try (var test =
        DeterministicTest.builder()
            .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
            .messageSelector(firstSelector())
            .messageMutator(MessageMutator.dropTimeouts())
            .functionalNodeModule(
                new FunctionalRadixNodeModule(
                    FunctionalRadixNodeModule.NodeStorageConfig.none(),
                    false,
                    FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
                    FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                    FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                        StateComputerConfig.rev2(
                            Network.INTEGRATIONTESTNET.getId(),
                            converted,
                            REv2StateManagerModule.DatabaseType.IN_MEMORY,
                            StateComputerConfig.REV2ProposerConfig.mempool(
                                0, 0, 0, MempoolRelayConfig.of())))))) {
      test.startAllNodes();
      final var expectedBalanceEntry =
          stateSummary.xrdBalances.entrySet().stream().findFirst().orElseThrow();
      final var stateReader = test.getInstance(0, REv2StateReader.class);
      final var xrdAmount =
          stateReader.getComponentXrdAmount(
              Address.virtualAccountAddress(expectedBalanceEntry.getKey().asBytes()));
      assertEquals(Decimal.fromBigIntegerSubunits(expectedBalanceEntry.getValue()), xrdAmount);
    }
  }

  private Tuple2<OlympiaStateIR, OlympiaStateSummary> prepareOlympiaState() {

    final UniqueListBuilder<HashCode> accountsBuilder = new UniqueListBuilder<>();

    final var validatorsBuilder = ImmutableList.<OlympiaStateIR.Validator>builder();
    final var stakesBuilder = ImmutableList.<OlympiaStateIR.Stake>builder();
    final var validatorSummaryBuilder = ImmutableList.<ValidatorSummary>builder();
    for (int i = 0; i < numValidators; i++) {
      final var publicKey = ECKeyPair.generateNew().getPublicKey();
      final var totalStakedXrd = randomSmallU256();
      final var ownerIndex = randomAccount(accountsBuilder).last();
      BigInteger totalStakeUnitAmount = BigInteger.ZERO;
      final var stakesForSummary = new HashMap<HashCode, UInt256>();
      for (int j = 0; j < numStakesPerValidator; j++) {
        final var stakerAccAndIdx = randomAccount(accountsBuilder);
        final var stakeUnits = randomSmallU256();
        totalStakeUnitAmount = totalStakeUnitAmount.add(stakeUnits.toBigInt());
        stakesForSummary.put(stakerAccAndIdx.first(), stakeUnits);
        stakesBuilder.add(new OlympiaStateIR.Stake(stakerAccAndIdx.last(), i, stakeUnits));
      }

      final var acceptDelegatedStake = random.nextBoolean();
      final var isRegistered = random.nextBoolean();
      final var validator =
          new OlympiaStateIR.Validator(
              HashCode.fromBytes(publicKey.getCompressedBytes()),
              "Validator " + i,
              "Validator " + i + " URL",
              acceptDelegatedStake,
              isRegistered,
              totalStakedXrd,
              Decimal.fromBigIntegerSubunits(totalStakeUnitAmount).toUInt256(),
              1000,
              ownerIndex);
      validatorsBuilder.add(validator);

      final var validatorSummary =
          new ValidatorSummary(
              publicKey,
              acceptDelegatedStake,
              isRegistered,
              totalStakeUnitAmount,
              totalStakedXrd,
              stakesForSummary);
      validatorSummaryBuilder.add(validatorSummary);
    }

    final var xrdResource = createXrdResource();

    final var balancesForSummary = new HashMap<HashCode, BigInteger>();
    final var balancesBuilder = ImmutableList.<OlympiaStateIR.AccountBalance>builder();
    for (int i = 0; i < numXrdBalances; i++) {
      final var accAndIdx = randomAccount(accountsBuilder);
      final var amount = randomSmallU256().toBigInt();
      balancesBuilder.add(
          new OlympiaStateIR.AccountBalance(accAndIdx.last(), xrdResourceIndex, amount));
      balancesForSummary.put(accAndIdx.first(), amount);
    }

    final var resourcesBuilder = ImmutableList.<OlympiaStateIR.Resource>builder();

    final var resourcesSummary = ImmutableList.<ResourceSummary>builder();

    for (int i = 0; i < numResources; i++) {
      final var addrBytes = new byte[27];
      random.nextBytes(addrBytes);
      addrBytes[0] = REAddr.REAddrType.HASHED_KEY.byteValue();
      final var addr = REAddr.of(addrBytes);
      final var resource =
          new OlympiaStateIR.Resource(
              addr,
              UInt256.from(18),
              true,
              Optional.empty(),
              "RES" + i,
              "Resource " + i,
              "",
              "",
              "");

      if (i == xrdResourceIndex) {
        resourcesBuilder.add(xrdResource);
      }
      resourcesBuilder.add(resource);

      final var balances = new HashMap<HashCode, BigInteger>();
      final var unscaledBalances = new ArrayList<BigInteger>();
      for (int j = 0; j < numBalancesPerResource; j++) {
        var amount = randomU256().toBigInt();
        if (i == 0) {
          // Explicitly add at least one very large amount
          amount = amount.add(UInt256.MAX_VALUE.toBigInt()).add(UInt256.MAX_VALUE.toBigInt());
        }
        final var accAndIdx = randomAccount(accountsBuilder);
        balancesBuilder.add(
            new OlympiaStateIR.AccountBalance(
                accAndIdx.last(), i < xrdResourceIndex ? i : i + 1, amount));
        balances.put(accAndIdx.first(), amount);
        unscaledBalances.add(amount);
      }

      final var totalSupply = unscaledBalances.stream().reduce(BigInteger.ZERO, BigInteger::add);

      final var expectedScaledTotalSupply =
          unscaledBalances.stream()
              .map(
                  amount ->
                      scaleResourceAmount(
                          amount,
                          totalSupply,
                          CONVERTER_CONFIG
                              .maxGenesisResourceUnscaledSupply()
                              .toBigIntegerSubunits()))
              .reduce(Decimal.ZERO, Decimal::add);

      resourcesSummary.add(
          new ResourceSummary(
              HashCode.fromBytes(addr.getBytes()),
              totalSupply,
              expectedScaledTotalSupply,
              balances));
    }
    if (xrdResourceIndex == numResources) {
      resourcesBuilder.add(xrdResource);
    }

    final var olympiaState =
        new OlympiaStateIR(
            validatorsBuilder.build(),
            resourcesBuilder.build(),
            accountsBuilder.build().stream()
                .map(OlympiaStateIR.Account::new)
                .collect(ImmutableList.toImmutableList()),
            balancesBuilder.build(),
            stakesBuilder.build(),
            1L,
            1L);

    final var stateSummary =
        new OlympiaStateSummary(
            validatorSummaryBuilder.build(), balancesForSummary, resourcesSummary.build());

    return tuple(olympiaState, stateSummary);
  }

  private Tuple2<HashCode, Integer> randomAccount(UniqueListBuilder<HashCode> accountsBuilder) {
    final var publicKeyBytes = new byte[ECDSASecp256k1PublicKey.COMPRESSED_BYTES];
    random.nextBytes(publicKeyBytes);
    final var publicKey = HashCode.fromBytes(publicKeyBytes);
    final var accountIndex = accountsBuilder.insertIfMissingAndGetIndex(publicKey);
    return tuple(publicKey, accountIndex);
  }

  private UInt256 randomU256() {
    return UInt256.from(
        UInt128.from(random.nextLong(), random.nextLong()),
        UInt128.from(random.nextLong(), random.nextLong()));
  }

  private UInt256 randomSmallU256() {
    return UInt256.from(random.nextLong());
  }

  private void checkBabylonGenesisMatchesStateSummary(
      GenesisData genesisData, OlympiaStateSummary stateSummary) {
    final var chunksByType =
        genesisData.chunks().stream().collect(Collectors.groupingBy(GenesisDataChunk::getClass));

    checkValidatorsAndStakes(
        stateSummary,
        chunksByType.getOrDefault(GenesisDataChunk.Validators.class, List.of()).stream()
            .map(c -> (GenesisDataChunk.Validators) c)
            .toList(),
        chunksByType.getOrDefault(GenesisDataChunk.Stakes.class, List.of()).stream()
            .map(c -> (GenesisDataChunk.Stakes) c)
            .toList());

    checkXrdBalances(
        stateSummary,
        chunksByType.getOrDefault(GenesisDataChunk.XrdBalances.class, List.of()).stream()
            .map(c -> (GenesisDataChunk.XrdBalances) c)
            .toList());

    checkResources(
        stateSummary,
        chunksByType.getOrDefault(GenesisDataChunk.Resources.class, List.of()).stream()
            .map(c -> (GenesisDataChunk.Resources) c)
            .toList(),
        chunksByType.getOrDefault(GenesisDataChunk.ResourceBalances.class, List.of()).stream()
            .map(c -> (GenesisDataChunk.ResourceBalances) c)
            .toList());
  }

  private void checkValidatorsAndStakes(
      OlympiaStateSummary stateSummary,
      List<GenesisDataChunk.Validators> validatorsChunks,
      List<GenesisDataChunk.Stakes> stakesChunks) {

    verifyExpectedChunksCountAndNumElementsPerChunk(
        stateSummary.validators.size(), CONVERTER_CONFIG.maxValidatorsPerChunk(), validatorsChunks);

    final var allValidators =
        validatorsChunks.stream().flatMap(chunk -> chunk.value().stream()).toList();
    assertEquals(stateSummary.validators.size(), allValidators.size());

    final var allValidatorsByKey =
        allValidators.stream().collect(Collectors.toMap(GenesisValidator::key, v -> v));

    final var olympiaTotalStakes =
        stateSummary.validators.stream().mapToInt(v -> v.stakes.size()).sum();
    verifyExpectedChunksCountAndNumElementsPerChunk(
        olympiaTotalStakes, CONVERTER_CONFIG.maxStakesPerChunk(), stakesChunks);

    final var babylonTotalStakesAllocations =
        stakesChunks.stream()
            .mapToInt(s -> s.allocations().stream().mapToInt(allocs -> allocs.last().size()).sum())
            .sum();
    assertEquals(olympiaTotalStakes, babylonTotalStakesAllocations);

    final var babylonStakesByValidator =
        new HashMap<ECDSASecp256k1PublicKey, Map<ComponentAddress, Decimal>>();
    stakesChunks.forEach(
        stakes -> {
          stakes
              .allocations()
              .forEach(
                  t -> {
                    final var validatorKey = t.first();
                    final var validatorStakes =
                        babylonStakesByValidator.computeIfAbsent(
                            validatorKey, unused -> new HashMap<>());
                    t.last()
                        .forEach(
                            alloc -> {
                              final var acc = stakes.accounts().get(alloc.accountIndex().toInt());
                              validatorStakes.put(acc, alloc.xrdAmount());
                            });
                  });
        });

    stateSummary
        .validators()
        .forEach(
            summaryValidator -> {
              final var babylonValidator = allValidatorsByKey.get(summaryValidator.publicKey());
              final var babylonStakes = babylonStakesByValidator.get(summaryValidator.publicKey);
              assertEquals(
                  summaryValidator.acceptDelegatedStake(), babylonValidator.acceptDelegatedStake());
              assertEquals(summaryValidator.isRegistered(), babylonValidator.isRegistered());
              summaryValidator.stakes.forEach(
                  (stakerKey, stakeInStakeUnits) -> {
                    final var stakerAddress = Address.virtualAccountAddress(stakerKey.asBytes());
                    final var babylonStakerStake = babylonStakes.get(stakerAddress);
                    final var stakeInXrd =
                        stakeInStakeUnits
                            .toBigInt()
                            .multiply(summaryValidator.totalStakedXrd().toBigInt())
                            .divide(summaryValidator.totalStakeUnits);
                    assertEquals(Decimal.fromBigIntegerSubunits(stakeInXrd), babylonStakerStake);
                  });
            });
  }

  private void checkXrdBalances(
      OlympiaStateSummary stateSummary, List<GenesisDataChunk.XrdBalances> xrdBalances) {
    verifyExpectedChunksCountAndNumElementsPerChunk(
        stateSummary.xrdBalances().size(), CONVERTER_CONFIG.maxXrdBalancesPerChunk(), xrdBalances);

    final var babylonTotalXrdBalances = xrdBalances.stream().mapToInt(s -> s.value().size()).sum();
    assertEquals(stateSummary.xrdBalances().size(), babylonTotalXrdBalances);

    final var babylonXrdByAccount =
        xrdBalances.stream()
            .flatMap(c -> c.value().stream())
            .collect(Collectors.toMap(Tuple2::first, Tuple2::last));

    stateSummary.xrdBalances.forEach(
        (keyBytes, amount) -> {
          final var babylonAmount =
              babylonXrdByAccount.get(Address.virtualAccountAddress(keyBytes.asBytes()));
          assertEquals(Decimal.fromBigIntegerSubunits(amount), babylonAmount);
        });
  }

  private void checkResources(
      OlympiaStateSummary stateSummary,
      List<GenesisDataChunk.Resources> resources,
      List<GenesisDataChunk.ResourceBalances> resourceBalances) {

    verifyExpectedChunksCountAndNumElementsPerChunk(
        stateSummary.resources().size(), CONVERTER_CONFIG.maxResourcesPerChunk(), resources);

    final var numOlympiaResourceBalances =
        stateSummary.resources.stream().mapToInt(r -> r.balances.size()).sum();
    verifyExpectedChunksCountAndNumElementsPerChunk(
        numOlympiaResourceBalances,
        CONVERTER_CONFIG.maxNonXrdResourceBalancesPerChunk(),
        resourceBalances);

    assertEquals(
        stateSummary.resources.size(), resources.stream().mapToInt(r -> r.value().size()).sum());
    assertEquals(
        numOlympiaResourceBalances,
        resourceBalances.stream()
            .mapToInt(r -> r.allocations().stream().mapToInt(s -> s.last().size()).sum())
            .sum());

    final var babylonResourcesByAddr =
        resources.stream()
            .flatMap(r -> r.value().stream())
            .collect(
                Collectors.toMap(r -> HashCode.fromBytes(r.addressBytesWithoutEntityId()), r -> r));

    final var babylonAllocationsByResource =
        new HashMap<ResourceAddress, Map<ComponentAddress, Decimal>>();
    resourceBalances.forEach(
        resourcesChunk -> {
          resourcesChunk
              .allocations()
              .forEach(
                  t -> {
                    final var resourceAddr = t.first();
                    final var resourceAllocations =
                        babylonAllocationsByResource.computeIfAbsent(
                            resourceAddr, unused -> new HashMap<>());
                    t.last()
                        .forEach(
                            alloc -> {
                              final var acc =
                                  resourcesChunk.accounts().get(alloc.accountIndex().toInt());
                              resourceAllocations.put(acc, alloc.amount());
                            });
                  });
        });

    stateSummary.resources.forEach(
        summaryResource -> {
          final var babylonAddrBytes =
              olympiaToBabylonResourceAddressBytes(summaryResource.resourceAddrBytes.asBytes());
          final var babylonResource =
              babylonResourcesByAddr.get(HashCode.fromBytes(babylonAddrBytes));
          assertEquals(
              summaryResource.expectedScaledTotalSupply(), babylonResource.initialSupply());

          final var babylonBalances =
              babylonAllocationsByResource.get(
                  toGlobalFungibleAddr(summaryResource.resourceAddrBytes.asBytes()));
          summaryResource.balances.forEach(
              (accKey, amount) -> {
                final var babylonAmount =
                    babylonBalances.get(Address.virtualAccountAddress(accKey.asBytes()));
                assertEquals(
                    scaleResourceAmount(
                        amount,
                        summaryResource.totalSupply(),
                        CONVERTER_CONFIG.maxGenesisResourceUnscaledSupply().toBigIntegerSubunits()),
                    babylonAmount);
              });
        });
  }

  private static Decimal scaleResourceAmount(
      BigInteger originalAmount,
      BigInteger resourceTotalSupplyOnOlympia,
      BigInteger resourceMaxSupplyOnBabylon) {
    if (resourceTotalSupplyOnOlympia.compareTo(resourceMaxSupplyOnBabylon) <= 0) {
      return Decimal.fromBigIntegerSubunits(originalAmount);
    } else {
      final var scaledBigInt =
          resourceMaxSupplyOnBabylon.multiply(originalAmount).divide(resourceTotalSupplyOnOlympia);
      return Decimal.fromBigIntegerSubunits(scaledBigInt);
    }
  }

  private <T extends GenesisDataChunk> void verifyExpectedChunksCountAndNumElementsPerChunk(
      int totalNumElements, int maxElementsPerChunk, List<T> chunks) {
    final var expected = (int) Math.ceil((double) totalNumElements / maxElementsPerChunk);
    assertEquals(expected, chunks.size());
    int totalNumElementsInChunks = 0;
    for (var i = 0; i < chunks.size(); i++) {
      final var chunk = chunks.get(i);
      final int numElementsInChunk =
          switch (chunk) {
            case GenesisDataChunk.Validators validators -> validators.value().size();
            case GenesisDataChunk.Stakes stakes -> stakes.allocations().stream()
                .mapToInt(a -> a.last().size())
                .sum();
            case GenesisDataChunk.Resources resources -> resources.value().size();
            case GenesisDataChunk.ResourceBalances resourceBalances -> resourceBalances
                .allocations()
                .stream()
                .mapToInt(a -> a.last().size())
                .sum();
            case GenesisDataChunk.XrdBalances xrdBalances -> xrdBalances.value().size();
          };
      if (i == chunks.size() - 1) {
        // last chunk can have less elements
        assertTrue(numElementsInChunk <= maxElementsPerChunk);
      } else {
        // all other chunks have max elements
        assertEquals(maxElementsPerChunk, numElementsInChunk);
      }
      totalNumElementsInChunks += numElementsInChunk;
    }
    assertEquals(totalNumElements, totalNumElementsInChunks);
  }

  private static ResourceAddress toGlobalFungibleAddr(byte[] olympiaAddressBytes) {
    return Address.globalFungible(olympiaToBabylonResourceAddressBytes(olympiaAddressBytes));
  }

  private static byte[] olympiaToBabylonResourceAddressBytes(byte[] input) {
    final var hash = HASHER.hashBytes(input);
    return Arrays.copyOfRange(
        hash.asBytes(), 0, ResourceAddress.BYTE_LENGTH - ResourceAddress.ENTITY_ID_LEN);
  }

  private record OlympiaStateSummary(
      List<ValidatorSummary> validators,
      Map<HashCode, BigInteger> xrdBalances,
      List<ResourceSummary> resources) {}

  private record ValidatorSummary(
      ECDSASecp256k1PublicKey publicKey,
      boolean acceptDelegatedStake,
      boolean isRegistered,
      BigInteger totalStakeUnits,
      UInt256 totalStakedXrd,
      Map<HashCode, UInt256> stakes) {}

  private record ResourceSummary(
      HashCode resourceAddrBytes,
      BigInteger totalSupply,
      Decimal expectedScaledTotalSupply,
      Map<HashCode, BigInteger> balances) {}
}
