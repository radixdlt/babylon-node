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

import com.radixdlt.rev2.Decimal;
import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import java.util.Objects;

public record GenesisConsensusManagerConfig(
    UInt32 maxValidators,
    UInt64 epochMinRoundCount,
    UInt64 epochMaxRoundCount,
    UInt64 epochTargetDurationMillis,
    UInt64 numUnstakeEpochs,
    Decimal totalEmissionXrdPerEpoch,
    Decimal minValidatorReliability,
    UInt64 numOwnerStakeUnitsUnlockEpochs,
    UInt64 numFeeIncreaseDelayEpochs,
    Decimal validatorCreationXrdCost) {

  public GenesisConsensusManagerConfig {
    Objects.requireNonNull(maxValidators);
    Objects.requireNonNull(epochMinRoundCount);
    Objects.requireNonNull(epochMaxRoundCount);
    Objects.requireNonNull(epochTargetDurationMillis);
    Objects.requireNonNull(numUnstakeEpochs);
    Objects.requireNonNull(totalEmissionXrdPerEpoch);
    Objects.requireNonNull(minValidatorReliability);
    Objects.requireNonNull(numOwnerStakeUnitsUnlockEpochs);
    Objects.requireNonNull(numFeeIncreaseDelayEpochs);
    Objects.requireNonNull(validatorCreationXrdCost);
  }

  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        GenesisConsensusManagerConfig.class,
        codecs -> StructCodec.fromRecordComponents(GenesisConsensusManagerConfig.class, codecs));
  }

  public static GenesisConsensusManagerConfig testingDefaultEmpty() {
    return Builder.testDefaults().build();
  }

  public static class Builder {
    UInt32 maxValidators;
    UInt64 epochMinRoundCount;
    UInt64 epochMaxRoundCount;
    UInt64 epochTargetDurationMillis;
    UInt64 numUnstakeEpochs;
    Decimal totalEmissionXrdPerEpoch;
    Decimal minValidatorReliability;
    UInt64 numOwnerStakeUnitsUnlockEpochs;
    UInt64 numFeeIncreaseDelayEpochs;
    Decimal validatorCreationXrdCost;

    private Builder() {}

    public static Builder empty() {
      return new Builder();
    }

    public static long SECONDS_PER_DAY = 24L * 60L * 60L;
    public static long SECONDS_PER_YEAR = 365L * SECONDS_PER_DAY;

    public static Builder productionDefaults() {
      long targetEpochLengthSeconds = 5L * 60L;
      // NB based on Olympia, we estimate about 1800 rounds per epoch
      // Choose numbers we're unlikely to hit here outside of
      var minRounds = 500;
      var maxRounds = 3000;

      var approxEpochsPerDay = SECONDS_PER_DAY / targetEpochLengthSeconds;
      var approxEpochsPerYear = SECONDS_PER_YEAR / targetEpochLengthSeconds;

      var targetEmissionsPerYear = 300L * 1000L * 1000L;
      var totalXrdEmissionPerEpoch = Decimal.fraction(targetEmissionsPerYear, approxEpochsPerYear);

      // Epochs are shorter than Olympia, hence we have less to work with.
      // So we require 100% reliability in these short epochs to get emissions.
      var minReliabilityForEmissions = Decimal.fraction(100, 100);

      var numOwnerStakeUnitsUnlockEpochs = 4 * 7 * approxEpochsPerDay;
      var numFeeIncreaseDelayEpochs = 2 * 7 * approxEpochsPerDay;
      var unstakeEpochs = 7 * approxEpochsPerDay;

      return new Builder()
          .maxValidators(100)
          .epochMinRoundCount(minRounds)
          .epochMaxRoundCount(maxRounds)
          .epochTargetDurationMillis(targetEpochLengthSeconds * 1000)
          .numUnstakeEpochs(unstakeEpochs)
          .totalEmissionXrdPerEpoch(totalXrdEmissionPerEpoch)
          .minValidatorReliability(minReliabilityForEmissions)
          .numOwnerStakeUnitsUnlockEpochs(numOwnerStakeUnitsUnlockEpochs)
          .numFeeIncreaseDelayEpochs(numFeeIncreaseDelayEpochs)
          .validatorCreationXrdCost(Decimal.of(1000));
    }

    public static Builder testEnvironmentDefaults() {
      return productionDefaults()
          .numUnstakeEpochs(1)
          .numOwnerStakeUnitsUnlockEpochs(1)
          .numFeeIncreaseDelayEpochs(1)
          .validatorCreationXrdCost(Decimal.of(1));
    }

    public static Builder testInfiniteEpochs() {
      return Builder.testDefaults().epochExactRoundCount(Long.MAX_VALUE);
    }

    public static Builder testWithRoundsPerEpoch(long roundsPerEpoch) {
      return Builder.testDefaults().epochExactRoundCount(roundsPerEpoch);
    }

    public static Builder testDefaults() {
      return new Builder()
          .maxValidators(100)
          .epochExactRoundCount(100)
          .epochTargetDurationMillis(100)
          .numUnstakeEpochs(10)
          .totalEmissionXrdPerEpoch(Decimal.of(100))
          .minValidatorReliability(Decimal.fraction(8, 10))
          .numOwnerStakeUnitsUnlockEpochs(100)
          .numFeeIncreaseDelayEpochs(100)
          .validatorCreationXrdCost(Decimal.of(1));
    }

    public GenesisConsensusManagerConfig build() {
      // Note - if any of these haven't been set, we get a null-ref exception
      // in the GenesisConsensusManagerConfig constructor
      return new GenesisConsensusManagerConfig(
          maxValidators,
          epochMinRoundCount,
          epochMaxRoundCount,
          epochTargetDurationMillis,
          numUnstakeEpochs,
          totalEmissionXrdPerEpoch,
          minValidatorReliability,
          numOwnerStakeUnitsUnlockEpochs,
          numFeeIncreaseDelayEpochs,
          validatorCreationXrdCost);
    }

    public Builder maxValidators(int value) {
      maxValidators = UInt32.fromNonNegativeInt(value);
      return this;
    }

    public Builder epochExactRoundCount(long value) {
      epochMinRoundCount = UInt64.fromNonNegativeLong(value);
      epochMaxRoundCount = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder epochMinRoundCount(long value) {
      epochMinRoundCount = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder epochMaxRoundCount(long value) {
      epochMaxRoundCount = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder epochTargetDurationMillis(long value) {
      epochTargetDurationMillis = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder numUnstakeEpochs(long value) {
      numUnstakeEpochs = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder totalEmissionXrdPerEpoch(Decimal value) {
      totalEmissionXrdPerEpoch = value;
      return this;
    }

    public Builder minValidatorReliability(Decimal value) {
      minValidatorReliability = value;
      return this;
    }

    public Builder numOwnerStakeUnitsUnlockEpochs(long value) {
      numOwnerStakeUnitsUnlockEpochs = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder numFeeIncreaseDelayEpochs(long value) {
      numFeeIncreaseDelayEpochs = UInt64.fromNonNegativeLong(value);
      return this;
    }

    public Builder validatorCreationXrdCost(Decimal value) {
      validatorCreationXrdCost = value;
      return this;
    }
  }
}
