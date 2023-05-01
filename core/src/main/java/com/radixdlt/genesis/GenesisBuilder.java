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

package com.radixdlt.genesis;

import com.google.common.collect.ImmutableList;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.lang.Tuple;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.Map;
import java.util.stream.IntStream;

public final class GenesisBuilder {
  public static GenesisData createGenesisWithSingleValidator(
      ECDSASecp256k1PublicKey validator,
      Decimal initialStake,
      UInt64 roundsPerEpoch,
      UInt64 numUnstakeEpochs) {
    final var validatorsAndStakesChunks =
        prepareValidatorsAndStakesChunks(
            ImmutableList.of(Tuple.Tuple2.of(validator, initialStake)));
    return new GenesisData(
        ImmutableList.of(validatorsAndStakesChunks.first(), validatorsAndStakesChunks.last()),
        UInt64.fromNonNegativeLong(0),
        UInt32.fromNonNegativeInt(100),
        roundsPerEpoch,
        numUnstakeEpochs);
  }

  public static GenesisData createGenesisWithNumValidators(
      int numValidators, Decimal initialStake, UInt64 roundsPerEpoch) {
    return createGenesisWithNumValidatorsAndXrdBalances(
        numValidators, initialStake, Map.of(), roundsPerEpoch);
  }

  public static GenesisData createGenesisWithNumValidatorsAndXrdBalances(
      int numValidators,
      Decimal initialStake,
      Map<ECDSASecp256k1PublicKey, Decimal> xrdBalances,
      UInt64 roundsPerEpoch) {
    final var chunksBuilder = ImmutableList.<GenesisDataChunk>builder();

    if (!xrdBalances.isEmpty()) {
      chunksBuilder.add(prepareXrdBalancesChunk(xrdBalances));
    }

    final var validatorsAndStakesChunks =
        prepareValidatorsAndStakesChunks(
            PrivateKeys.numeric(1)
                .map(keyPair -> Tuple.Tuple2.of(keyPair.getPublicKey(), initialStake))
                .limit(numValidators)
                .collect(ImmutableList.toImmutableList()));

    chunksBuilder.add(validatorsAndStakesChunks.first());
    chunksBuilder.add(validatorsAndStakesChunks.last());

    return new GenesisData(
        chunksBuilder.build(),
        UInt64.fromNonNegativeLong(0),
        UInt32.fromNonNegativeInt(100),
        roundsPerEpoch,
        UInt64.fromNonNegativeLong(10));
  }

  public static GenesisData createGenesisWithValidatorsAndXrdBalances(
      ImmutableList<ECDSASecp256k1PublicKey> validators,
      Decimal initialStake,
      ComponentAddress stakerAddress,
      Map<ECDSASecp256k1PublicKey, Decimal> xrdBalances,
      UInt64 roundsPerEpoch) {
    final var chunksBuilder = ImmutableList.<GenesisDataChunk>builder();

    if (!xrdBalances.isEmpty()) {
      chunksBuilder.add(prepareXrdBalancesChunk(xrdBalances));
    }

    final var validatorsAndStakesChunks =
        prepareValidatorsAndStakesChunks(
            validators.stream()
                .map(v -> Tuple.Tuple2.of(v, initialStake))
                .collect(ImmutableList.toImmutableList()),
            stakerAddress);

    chunksBuilder.add(validatorsAndStakesChunks.first());
    chunksBuilder.add(validatorsAndStakesChunks.last());

    return new GenesisData(
        chunksBuilder.build(),
        UInt64.fromNonNegativeLong(0),
        UInt32.fromNonNegativeInt(100),
        roundsPerEpoch,
        UInt64.fromNonNegativeLong(10));
  }

  private static GenesisDataChunk.XrdBalances prepareXrdBalancesChunk(
      Map<ECDSASecp256k1PublicKey, Decimal> xrdBalances) {
    return new GenesisDataChunk.XrdBalances(
        xrdBalances.entrySet().stream()
            .map(
                e -> {
                  final var accountAddress =
                      ComponentAddress.virtualAccountFromPublicKey(e.getKey());
                  return Tuple.Tuple2.of(accountAddress, e.getValue());
                })
            .collect(ImmutableList.toImmutableList()));
  }

  // Allocates stakes to validator's default component address (same pub key)
  private static Tuple.Tuple2<GenesisDataChunk.Validators, GenesisDataChunk.Stakes>
      prepareValidatorsAndStakesChunks(
          ImmutableList<Tuple.Tuple2<ECDSASecp256k1PublicKey, Decimal>> validatorsAndStake) {
    final var validators =
        validatorsAndStake.stream()
            .map(tuple -> GenesisValidator.defaultFromPubKey(tuple.first()))
            .collect(ImmutableList.toImmutableList());
    final var validatorsChunk = new GenesisDataChunk.Validators(validators);

    final var stakeOwners =
        validators.stream().map(GenesisValidator::owner).collect(ImmutableList.toImmutableList());

    final var stakeAllocations =
        IntStream.range(0, validators.size())
            .mapToObj(
                idx -> {
                  final var validator = validators.get(idx);
                  final var stake = validatorsAndStake.get(idx).last();
                  return Tuple.Tuple2.of(
                      validator.key(),
                      ImmutableList.of(
                          new GenesisStakeAllocation(UInt32.fromNonNegativeInt(idx), stake)));
                })
            .collect(ImmutableList.toImmutableList());
    final var stakeChunk = new GenesisDataChunk.Stakes(stakeOwners, stakeAllocations);

    return Tuple.Tuple2.of(validatorsChunk, stakeChunk);
  }

  // Allocates all stakes to a specified staker account
  private static Tuple.Tuple2<GenesisDataChunk.Validators, GenesisDataChunk.Stakes>
      prepareValidatorsAndStakesChunks(
          ImmutableList<Tuple.Tuple2<ECDSASecp256k1PublicKey, Decimal>> validatorsAndStake,
          ComponentAddress staker) {
    final var validators =
        validatorsAndStake.stream()
            .map(tuple -> GenesisValidator.defaultFromPubKey(tuple.first()))
            .collect(ImmutableList.toImmutableList());
    final var validatorsChunk = new GenesisDataChunk.Validators(validators);

    final var stakeAllocations =
        IntStream.range(0, validators.size())
            .mapToObj(
                idx -> {
                  final var validator = validators.get(idx);
                  final var stake = validatorsAndStake.get(idx).last();
                  return Tuple.Tuple2.of(
                      validator.key(),
                      ImmutableList.of(
                          new GenesisStakeAllocation(UInt32.fromNonNegativeInt(0), stake)));
                })
            .collect(ImmutableList.toImmutableList());
    final var stakeChunk = new GenesisDataChunk.Stakes(ImmutableList.of(staker), stakeAllocations);
    return Tuple.Tuple2.of(validatorsChunk, stakeChunk);
  }
}
