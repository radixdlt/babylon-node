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

package com.radixdlt.consensus;

import static java.util.Objects.requireNonNull;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.common.annotations.VisibleForTesting;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.SerializerConstants;
import com.radixdlt.serialization.SerializerDummy;
import com.radixdlt.serialization.SerializerId2;
import java.util.Objects;
import java.util.Optional;
import javax.annotation.concurrent.Immutable;

/**
 * Ledger header which gets voted and agreed upon, as part of the BFT header.
 *
 * <p>This header is kept around alongside the committed transactions, inside the LedgerProof. The
 * rest of the BFT Header, for contrast, is reduced to only a hash (for size reasons, and for
 * "hiding" unimportant BFT information, so the ledger part of the node doesn't need to deserialize
 * BFT data).
 *
 * @see Vote for more information.
 */
@Immutable
@SerializerId2("consensus.ledger_header")
public final class LedgerHeader {
  @JsonProperty(SerializerConstants.SERIALIZER_NAME)
  @DsonOutput(value = {Output.API, Output.WIRE, Output.PERSIST})
  SerializerDummy serializer = SerializerDummy.DUMMY;

  @JsonProperty("epoch")
  @DsonOutput(Output.ALL)
  private final long epoch;

  private final Round round;

  @JsonProperty("accumulator_state")
  @DsonOutput(Output.ALL)
  private final AccumulatorState accumulatorState;

  @JsonProperty("state_hash")
  @DsonOutput(Output.ALL)
  private final HashCode stateHash;

  @JsonProperty("consensus_parent_round_timestamp_ms")
  @DsonOutput(Output.ALL)
  private final long consensusParentRoundTimestampMs;

  @JsonProperty("proposer_timestamp_ms")
  @DsonOutput(Output.ALL)
  private final long proposerTimestampMs;

  @JsonProperty("next_epoch")
  @DsonOutput(Output.ALL)
  private final NextEpoch nextEpoch;

  @JsonCreator
  @VisibleForTesting
  LedgerHeader(
      @JsonProperty("epoch") long epoch,
      @JsonProperty("round") long roundNumber,
      @JsonProperty(value = "accumulator_state", required = true) AccumulatorState accumulatorState,
      @JsonProperty(value = "state_hash", required = true) HashCode stateHash,
      @JsonProperty("consensus_parent_round_timestamp_ms") long consensusParentRoundTimestampMs,
      @JsonProperty("proposer_timestamp") long proposerTimestampMs,
      @JsonProperty("next_epoch") NextEpoch nextEpoch) {
    this(
        epoch,
        Round.of(roundNumber),
        accumulatorState,
        stateHash,
        consensusParentRoundTimestampMs,
        proposerTimestampMs,
        nextEpoch);
  }

  private LedgerHeader(
      long epoch,
      Round round,
      AccumulatorState accumulatorState,
      HashCode stateHash,
      long consensusParentRoundTimestampMs,
      long proposerTimestampMs,
      NextEpoch nextEpoch) {
    this.epoch = epoch;

    if (epoch < 0) {
      throw new IllegalArgumentException("Epoch can't be < 0");
    }

    this.round = round;
    this.accumulatorState = requireNonNull(accumulatorState);
    this.stateHash = requireNonNull(stateHash);
    this.nextEpoch = nextEpoch;
    this.consensusParentRoundTimestampMs = consensusParentRoundTimestampMs;
    this.proposerTimestampMs = proposerTimestampMs;
  }

  public static LedgerHeader genesis(
      AccumulatorState accumulatorState,
      HashCode stateHash,
      BFTValidatorSet validatorSet,
      long consensusParentRoundTimestamp,
      long proposerTimestamp) {
    return new LedgerHeader(
        0,
        Round.genesis(),
        accumulatorState,
        stateHash,
        consensusParentRoundTimestamp,
        proposerTimestamp,
        validatorSet == null ? null : NextEpoch.create(1, validatorSet.getValidators()));
  }

  public static LedgerHeader create(
      long epoch,
      Round round,
      AccumulatorState accumulatorState,
      HashCode stateHash,
      long consensusParentRoundTimestamp,
      long proposerTimestamp) {
    return new LedgerHeader(
        epoch,
        round,
        accumulatorState,
        stateHash,
        consensusParentRoundTimestamp,
        proposerTimestamp,
        null);
  }

  public static LedgerHeader create(
      long epoch,
      Round round,
      AccumulatorState accumulatorState,
      HashCode stateHash,
      long consensusParentRoundTimestamp,
      long proposerTimestamp,
      NextEpoch nextEpoch) {
    return new LedgerHeader(
        epoch,
        round,
        accumulatorState,
        stateHash,
        consensusParentRoundTimestamp,
        proposerTimestamp,
        nextEpoch);
  }

  public LedgerHeader updateRoundAndTimestamps(
      Round round, long consensusParentRoundTimestamp, long proposerTimestamp) {
    return new LedgerHeader(
        this.epoch,
        round,
        this.accumulatorState,
        this.stateHash,
        consensusParentRoundTimestamp,
        proposerTimestamp,
        this.nextEpoch);
  }

  @JsonProperty("round")
  @DsonOutput(Output.ALL)
  private long getSerializerForRoundNumber() {
    return round.number();
  }

  public Round getRound() {
    return round;
  }

  public Optional<NextEpoch> getNextEpoch() {
    return Optional.ofNullable(nextEpoch);
  }

  public AccumulatorState getAccumulatorState() {
    return accumulatorState;
  }

  public HashCode getStateHash() {
    return stateHash;
  }

  public long getEpoch() {
    return epoch;
  }

  public boolean isEndOfEpoch() {
    return nextEpoch != null;
  }

  public long consensusParentRoundTimestamp() {
    return this.consensusParentRoundTimestampMs;
  }

  public long proposerTimestamp() {
    return this.proposerTimestampMs;
  }

  @Override
  public int hashCode() {
    return Objects.hash(
        this.accumulatorState,
        this.stateHash,
        this.consensusParentRoundTimestampMs,
        this.proposerTimestampMs,
        this.epoch,
        this.round,
        this.nextEpoch);
  }

  @Override
  public boolean equals(Object o) {
    if (o == this) {
      return true;
    }

    return (o instanceof LedgerHeader other)
        && this.consensusParentRoundTimestampMs == other.consensusParentRoundTimestampMs
        && this.proposerTimestampMs == other.proposerTimestampMs
        && Objects.equals(this.accumulatorState, other.accumulatorState)
        && Objects.equals(this.stateHash, other.stateHash)
        && this.epoch == other.epoch
        && Objects.equals(this.round, other.round)
        && Objects.equals(this.nextEpoch, other.nextEpoch);
  }

  @Override
  public String toString() {
    return String.format(
        "%s{accumulator=%s stateHash=%s consensus_parent_round_timestamp=%s proposer_timestamp=%s"
            + " epoch=%s round=%s nextEpoch=%s}",
        getClass().getSimpleName(),
        this.accumulatorState,
        this.stateHash,
        this.consensusParentRoundTimestampMs,
        this.proposerTimestampMs,
        this.epoch,
        this.round,
        this.nextEpoch);
  }
}
