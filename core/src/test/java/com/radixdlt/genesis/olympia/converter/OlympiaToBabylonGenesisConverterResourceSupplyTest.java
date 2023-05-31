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

import static com.radixdlt.genesis.olympia.converter.OlympiaToBabylonGenesisConverterTestUtils.createXrdResource;
import static org.junit.Assert.assertEquals;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.genesis.GenesisDataChunk;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.REAddr;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.*;
import java.math.BigInteger;
import java.security.Security;
import java.util.*;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.junit.Test;

/** Tests resource supply conversion (including scaling down large supplies) */
public final class OlympiaToBabylonGenesisConverterResourceSupplyTest {
  static {
    Security.insertProviderAt(new BouncyCastleProvider(), 1);
  }

  private final Random random = new Random(1234);

  @Test
  public void test_resource_conversion_with_supply_scaling() {
    final var addrBytes = new byte[27];
    random.nextBytes(addrBytes);
    addrBytes[0] = REAddr.REAddrType.HASHED_KEY.byteValue();
    final var addr = REAddr.of(addrBytes);
    final var resource =
        new OlympiaStateIR.Resource(
            addr, UInt256.from(18), true, Optional.empty(), "LARGE", "", "", "", "");
    final var acc1 =
        new OlympiaStateIR.Account(
            HashCode.fromBytes(ECKeyPair.generateNew().getPublicKey().getCompressedBytes()));
    final var acc2 =
        new OlympiaStateIR.Account(
            HashCode.fromBytes(ECKeyPair.generateNew().getPublicKey().getCompressedBytes()));
    final var balances =
        ImmutableList.of(
            new OlympiaStateIR.AccountBalance(0, 1, BigInteger.valueOf(10000)),
            new OlympiaStateIR.AccountBalance(0, 1, BigInteger.valueOf(2001)));
    final var olympiaState =
        new OlympiaStateIR(
            ImmutableList.of(),
            ImmutableList.of(createXrdResource(), resource),
            ImmutableList.of(acc1, acc2),
            balances,
            ImmutableList.of(),
            1L,
            1L);

    final var config =
        new OlympiaToBabylonConverterConfig(
            10, 10, 10, 10, 10, Decimal.fromBigIntegerSubunits(BigInteger.valueOf(6000L)));
    final var converted = OlympiaStateToBabylonGenesisConverter.toGenesisData(olympiaState, config);

    // There are two balance entries: 10000 and 2001 (12k + 1 total), and the maximum supply is 6k.
    // The balances should be scaled by (6000/12001) and one of them rounded down.
    // The new total supply should then be 5999.
    final var resourcesChunk = converted.chunks().get(0);
    final var resourceBalancesChunk = (GenesisDataChunk.ResourceBalances) converted.chunks().get(1);

    assertEquals(
        Decimal.fromBigIntegerSubunits(BigInteger.valueOf(5999L)),
        ((GenesisDataChunk.Resources) resourcesChunk).value().get(0).initialSupply());

    assertEquals(
        Decimal.fromBigIntegerSubunits(BigInteger.valueOf(4999L)),
        resourceBalancesChunk.allocations().get(0).last().get(0).amount());

    assertEquals(
        Decimal.fromBigIntegerSubunits(BigInteger.valueOf(1000L)),
        resourceBalancesChunk.allocations().get(0).last().get(1).amount());
  }

  @Test
  public void test_large_supply_exceeding_u256() {
    final var addrBytes = new byte[27];
    random.nextBytes(addrBytes);
    addrBytes[0] = REAddr.REAddrType.HASHED_KEY.byteValue();
    final var addr = REAddr.of(addrBytes);
    final var resource =
        new OlympiaStateIR.Resource(
            addr, UInt256.from(18), true, Optional.empty(), "LARGE", "", "", "", "");
    final var acc1 =
        new OlympiaStateIR.Account(
            HashCode.fromBytes(ECKeyPair.generateNew().getPublicKey().getCompressedBytes()));
    final var balances =
        ImmutableList.of(
            new OlympiaStateIR.AccountBalance(
                0, 1, UInt256.MAX_VALUE.toBigInt().multiply(BigInteger.TWO)));
    final var olympiaState =
        new OlympiaStateIR(
            ImmutableList.of(),
            ImmutableList.of(createXrdResource(), resource),
            ImmutableList.of(acc1),
            balances,
            ImmutableList.of(),
            1L,
            1L);

    final var config =
        new OlympiaToBabylonConverterConfig(10, 10, 10, 10, 10, Decimal.from(UInt256.MAX_VALUE));
    final var converted = OlympiaStateToBabylonGenesisConverter.toGenesisData(olympiaState, config);

    final var resourcesChunk = converted.chunks().get(0);
    final var resourceBalancesChunk = (GenesisDataChunk.ResourceBalances) converted.chunks().get(1);

    assertEquals(
        Decimal.from(UInt256.MAX_VALUE),
        ((GenesisDataChunk.Resources) resourcesChunk).value().get(0).initialSupply());

    assertEquals(
        Decimal.from(UInt256.MAX_VALUE),
        resourceBalancesChunk.allocations().get(0).last().get(0).amount());
  }

  @Test
  public void test_conversion_with_160_bit_babylon_max_supply() {
    final var maxSupply = BigInteger.TWO.pow(160);
    final var addrBytes = new byte[27];
    random.nextBytes(addrBytes);
    addrBytes[0] = REAddr.REAddrType.HASHED_KEY.byteValue();
    final var addr = REAddr.of(addrBytes);
    final var resource =
        new OlympiaStateIR.Resource(
            addr, UInt256.from(18), true, Optional.empty(), "LARGE", "", "", "", "");
    final var acc1 =
        new OlympiaStateIR.Account(
            HashCode.fromBytes(ECKeyPair.generateNew().getPublicKey().getCompressedBytes()));
    final var acc2 =
        new OlympiaStateIR.Account(
            HashCode.fromBytes(ECKeyPair.generateNew().getPublicKey().getCompressedBytes()));
    final var balances =
        ImmutableList.of(
            new OlympiaStateIR.AccountBalance(
                0, 1, maxSupply.multiply(BigInteger.TWO)), // twice the max supply
            new OlympiaStateIR.AccountBalance(
                1, 1, maxSupply.divide(BigInteger.TWO))); // half the max supply
    final var olympiaState =
        new OlympiaStateIR(
            ImmutableList.of(),
            ImmutableList.of(createXrdResource(), resource),
            ImmutableList.of(acc1, acc2),
            balances,
            ImmutableList.of(),
            1L,
            1L);

    final var config =
        new OlympiaToBabylonConverterConfig(
            10, 10, 10, 10, 10, Decimal.fromBigIntegerSubunits(maxSupply));
    final var converted = OlympiaStateToBabylonGenesisConverter.toGenesisData(olympiaState, config);

    final var resourcesChunk = converted.chunks().get(0);
    final var resourceBalancesChunk = (GenesisDataChunk.ResourceBalances) converted.chunks().get(1);

    // Max supply is 1461501637330902918203684832716283019655932542976 (2^160)
    // but we end up with one unit less due to rounding errors.
    assertEquals(
        Decimal.fromBigIntegerSubunits(
            new BigInteger("1461501637330902918203684832716283019655932542975")),
        ((GenesisDataChunk.Resources) resourcesChunk).value().get(0).initialSupply());

    // One account receives 4/5 of total supply (rounded down), and the other one 1/5
    assertEquals(
        Decimal.fromBigIntegerSubunits(
            new BigInteger("1169201309864722334562947866173026415724746034380")),
        resourceBalancesChunk.allocations().get(0).last().get(0).amount());

    assertEquals(
        Decimal.fromBigIntegerSubunits(
            new BigInteger("292300327466180583640736966543256603931186508595")),
        resourceBalancesChunk.allocations().get(0).last().get(1).amount());
  }
}
