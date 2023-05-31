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

import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Maps;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.statecomputer.commit.ActiveValidatorInfo;
import com.radixdlt.statecomputer.commit.TimestampedValidatorSignature;
import com.radixdlt.utils.UInt64;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

public final class REv2ToConsensus {
  private REv2ToConsensus() {
    throw new IllegalStateException("Cannot instantiate.");
  }

  public static BFTValidator validator(ActiveValidatorInfo validator) {
    return BFTValidator.from(
        BFTValidatorId.create(validator.address(), validator.key()),
        validator.stake().toUInt256());
  }

  public static ActiveValidatorInfo validator(BFTValidator validator) {
    BFTValidatorId id = validator.getValidatorId();
    var validatorAddress = id.getValidatorAddress();
    if (validatorAddress.isEmpty()) {
      throw new IllegalStateException("Active validator must have a validator address");
    }
    return new ActiveValidatorInfo(
            validatorAddress.get(), id.getKey(), Decimal.from(validator.getPower()));
  }

  public static BFTValidatorSet validatorSet(Set<ActiveValidatorInfo> validators) {
    return BFTValidatorSet.from(validators.stream().map(REv2ToConsensus::validator));
  }

  public static NextEpoch nextEpoch(com.radixdlt.statecomputer.commit.NextEpoch nextEpoch) {
    var validators =
        nextEpoch.validators().stream()
            .map(REv2ToConsensus::validator)
            .collect(ImmutableSet.toImmutableSet());
    return NextEpoch.create(nextEpoch.epoch().toNonNegativeLong().unwrap(), validators);
  }

  public static com.radixdlt.statecomputer.commit.NextEpoch nextEpoch(NextEpoch nextEpoch) {
    ImmutableSet<ActiveValidatorInfo> validators =
        nextEpoch.getValidators().stream()
            .map(REv2ToConsensus::validator)
            .collect(ImmutableSet.toImmutableSet());
    return new com.radixdlt.statecomputer.commit.NextEpoch(
        validators, UInt64.fromNonNegativeLong(nextEpoch.getEpoch()));
  }

  public static LedgerHashes ledgerHashes(
      com.radixdlt.statecomputer.commit.LedgerHashes ledgerHashes) {
    return LedgerHashes.create(
        ledgerHashes.stateRoot(), ledgerHashes.transactionRoot(), ledgerHashes.receiptRoot());
  }

  public static com.radixdlt.statecomputer.commit.LedgerHashes ledgerHashes(
      LedgerHashes ledgerHashes) {
    return new com.radixdlt.statecomputer.commit.LedgerHashes(
        ledgerHashes.getStateRoot(),
        ledgerHashes.getTransactionRoot(),
        ledgerHashes.getReceiptRoot());
  }

  public static LedgerProof ledgerProof(com.radixdlt.statecomputer.commit.LedgerProof ledgerProof) {
    return new LedgerProof(
        ledgerProof.opaque(),
        REv2ToConsensus.ledgerHeader(ledgerProof.ledgerHeader()),
        new TimestampedECDSASignatures(
            ledgerProof.signatures().stream()
                .map(REv2ToConsensus::timestampedValidatorSignature)
                .collect(Collectors.toMap(Map.Entry::getKey, Map.Entry::getValue))));
  }

  public static com.radixdlt.statecomputer.commit.LedgerProof ledgerProof(LedgerProof ledgerProof) {
    return new com.radixdlt.statecomputer.commit.LedgerProof(
        ledgerProof.getOpaque(),
        REv2ToConsensus.ledgerHeader(ledgerProof.getHeader()),
        ledgerProof.getSignatures().getSignatures().entrySet().stream()
            .map(e -> REv2ToConsensus.timestampedValidatorSignature(e.getKey(), e.getValue()))
            .toList());
  }

  public static Map.Entry<BFTValidatorId, TimestampedECDSASignature> timestampedValidatorSignature(
      TimestampedValidatorSignature timestampedSignature) {
    return Maps.immutableEntry(
        BFTValidatorId.create(
            timestampedSignature.validatorAddress().or((ComponentAddress) null),
            timestampedSignature.key()),
        TimestampedECDSASignature.from(
            timestampedSignature.timestampMs(), timestampedSignature.signature()));
  }

  public static TimestampedValidatorSignature timestampedValidatorSignature(
      BFTValidatorId id, TimestampedECDSASignature timestampedSignature) {
    return new TimestampedValidatorSignature(
        id.getKey(),
        Option.from(id.getValidatorAddress()),
        timestampedSignature.timestamp(),
        timestampedSignature.signature());
  }

  public static LedgerHeader ledgerHeader(
      com.radixdlt.statecomputer.commit.LedgerHeader ledgerHeader) {
    return LedgerHeader.create(
        ledgerHeader.epoch().toNonNegativeLong().unwrap(),
        Round.of(ledgerHeader.round().toNonNegativeLong().unwrap()),
        REv2ToConsensus.accumulatorState(ledgerHeader.accumulatorState()),
        REv2ToConsensus.ledgerHashes(ledgerHeader.hashes()),
        ledgerHeader.consensusParentRoundTimestampMs(),
        ledgerHeader.proposerTimestampMs(),
        ledgerHeader.nextEpoch().map(REv2ToConsensus::nextEpoch).or((NextEpoch) null));
  }

  public static com.radixdlt.statecomputer.commit.LedgerHeader ledgerHeader(
      LedgerHeader ledgerHeader) {
    return new com.radixdlt.statecomputer.commit.LedgerHeader(
        UInt64.fromNonNegativeLong(ledgerHeader.getEpoch()),
        UInt64.fromNonNegativeLong(ledgerHeader.getRound().number()),
        REv2ToConsensus.accumulatorState(ledgerHeader.getAccumulatorState()),
        REv2ToConsensus.ledgerHashes(ledgerHeader.getHashes()),
        ledgerHeader.consensusParentRoundTimestamp(),
        ledgerHeader.proposerTimestamp(),
        Option.from(ledgerHeader.getNextEpoch().map(REv2ToConsensus::nextEpoch)));
  }

  public static AccumulatorState accumulatorState(
      com.radixdlt.statecomputer.commit.AccumulatorState accumulatorState) {
    return new AccumulatorState(
        accumulatorState.stateVersion().toNonNegativeLong().unwrap(),
        accumulatorState.accumulatorHash());
  }

  public static com.radixdlt.statecomputer.commit.AccumulatorState accumulatorState(
      AccumulatorState accumulatorState) {
    return new com.radixdlt.statecomputer.commit.AccumulatorState(
        UInt64.fromNonNegativeLong(accumulatorState.getStateVersion()),
        accumulatorState.getAccumulatorHash());
  }
}
