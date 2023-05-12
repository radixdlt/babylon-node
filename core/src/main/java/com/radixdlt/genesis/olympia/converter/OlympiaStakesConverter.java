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

import static com.radixdlt.lang.Tuple.tuple;

import com.google.common.collect.ImmutableList;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.genesis.GenesisDataChunk;
import com.radixdlt.genesis.GenesisStakeAllocation;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Tuple;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UniqueListBuilder;
import java.util.stream.Collectors;

public class OlympiaStakesConverter {
  public static ImmutableList<GenesisDataChunk.Stakes> prepareStakesChunks(
      OlympiaToBabylonConverterConfig config,
      ImmutableList<OlympiaStateIR.Account> accounts,
      ImmutableList<OlympiaStateIR.Validator> validators,
      ImmutableList<OlympiaStateIR.Stake> stakes) {
    final var olympiaStakesGrouped =
        stakes.stream()
            .collect(Collectors.groupingBy(OlympiaStateIR.Stake::validatorIndex))
            .entrySet()
            .stream()
            .toList();

    final ImmutableList.Builder<GenesisDataChunk.Stakes> chunksBuilder = ImmutableList.builder();
    ImmutableList.Builder<
            Tuple.Tuple2<ECDSASecp256k1PublicKey, ImmutableList<GenesisStakeAllocation>>>
        stakesInCurrentChunkBuilder = ImmutableList.builder();
    UniqueListBuilder<ComponentAddress> accountsForCurrentChunk = new UniqueListBuilder<>();
    int numStakesInCurrentChunk = 0;

    for (var i = 0; i < olympiaStakesGrouped.size(); i++) {
      final var entry = olympiaStakesGrouped.get(i);
      final var validatorIndex = entry.getKey();
      final var currStakes = entry.getValue();
      final var validator = validators.get(validatorIndex);

      if (validator.totalStakeUnits().isZero()) {
        // Shouldn't really happen, but better not to divide by 0 later on :)
        // We can just skip the validators without any stake
        continue;
      }

      final ECDSASecp256k1PublicKey validatorPublicKey;
      try {
        validatorPublicKey =
            ECDSASecp256k1PublicKey.fromBytes(validator.publicKeyBytes().asBytes());
      } catch (PublicKeyException e) {
        throw new OlympiaToBabylonGenesisConverterException(
            "Olympia validator public key is invalid", e);
      }

      ImmutableList.Builder<GenesisStakeAllocation> allocationsBuilder = ImmutableList.builder();

      for (var j = 0; j < currStakes.size(); j++) {
        final var stake = currStakes.get(j);
        // stake_xrd_value = total_xrd_staked * stake_unit_amount / total_stake_units
        final var xrdAmountBigInt =
            stake
                .stakeUnitAmount()
                .toBigInt()
                .multiply(validator.totalStakedXrd().toBigInt())
                .divide(validator.totalStakeUnits().toBigInt());
        final var decimalXrdAmount = Decimal.unsafeFromRawBigIntRepr(xrdAmountBigInt);

        final var isLast = i == olympiaStakesGrouped.size() - 1 && j == currStakes.size() - 1;
        final var account = accounts.get(stake.accountIndex());
        final var accountAddress =
            Address.virtualAccountAddress(account.publicKeyBytes().asBytes());

        final var accountIndex = accountsForCurrentChunk.insertIfMissingAndGetIndex(accountAddress);
        allocationsBuilder.add(
            new GenesisStakeAllocation(UInt32.fromNonNegativeInt(accountIndex), decimalXrdAmount));
        numStakesInCurrentChunk += 1;
        if (numStakesInCurrentChunk >= config.maxStakesPerChunk() || isLast) {
          stakesInCurrentChunkBuilder.add(tuple(validatorPublicKey, allocationsBuilder.build()));
          allocationsBuilder = ImmutableList.builder();

          chunksBuilder.add(
              new GenesisDataChunk.Stakes(
                  accountsForCurrentChunk.build(), stakesInCurrentChunkBuilder.build()));
          accountsForCurrentChunk = new UniqueListBuilder<>();
          stakesInCurrentChunkBuilder = ImmutableList.builder();
          numStakesInCurrentChunk = 0;
        }
      }
      final var lastAllocationsForCurrentValidator = allocationsBuilder.build();
      if (!lastAllocationsForCurrentValidator.isEmpty()) {
        stakesInCurrentChunkBuilder.add(
            tuple(validatorPublicKey, lastAllocationsForCurrentValidator));
      }
    }
    return chunksBuilder.build();
  }
}
