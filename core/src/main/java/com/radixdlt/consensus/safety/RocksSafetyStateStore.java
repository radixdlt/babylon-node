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

package com.radixdlt.consensus.safety;

import com.google.common.collect.ImmutableSet;
import com.google.common.hash.HashCode;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.lang.Option;
import com.radixdlt.safety.*;
import com.radixdlt.utils.UInt192;
import java.util.Optional;
import java.util.stream.Collectors;
import javax.inject.Inject;

public class RocksSafetyStateStore implements PersistentSafetyStateStore {
  private final RocksDbSafetyStore rocksDbSafetyStore;

  @Inject
  public RocksSafetyStateStore(RocksDbSafetyStore rocksDbSafetyStore) {
    this.rocksDbSafetyStore = rocksDbSafetyStore;
  }

  @Override
  public void commitState(SafetyState safetyState) {
    rocksDbSafetyStore.upsert(toDTO(safetyState));
  }

  @Override
  public void close() {
    // no-op
  }

  @Override
  public Optional<SafetyState> get() {
    return rocksDbSafetyStore.get().map(RocksSafetyStateStore::fromDTO).toOptional();
  }

  public boolean isMigrated() {
    return rocksDbSafetyStore.isMigrated();
  }

  public void markAsMigrated() {
    rocksDbSafetyStore.markAsMigrated();
  }

  // -------------------------------------------------------------------------------------
  // Conversion to DTO
  // -------------------------------------------------------------------------------------

  private static SafetyStateDTO toDTO(SafetyState safetyState) {
    var validatorId = toDTO(safetyState.getValidatorId());
    var round = toDTO(safetyState.getLockedRound());
    var vote = Option.from(safetyState.getLastVote().map(RocksSafetyStateStore::toDTO));
    return new SafetyStateDTO(validatorId, round, vote);
  }

  private static VoteDTO toDTO(Vote vote) {
    BFTValidatorIdDTO author = toDTO(vote.getAuthor());
    HighQCDTO highQC = toDTO(vote.highQC());
    VoteDataDTO voteData = toDTO(vote.getVoteData());
    long timestamp = vote.getTimestamp();
    ECDSASecp256k1Signature signature = vote.getSignature();
    Option<ECDSASecp256k1Signature> timeoutSignature = Option.from(vote.getTimeoutSignature());

    return new VoteDTO(author, highQC, voteData, timestamp, signature, timeoutSignature);
  }

  private static HighQCDTO toDTO(HighQC highQC) {
    return new HighQCDTO(
        toDTO(highQC.highestQC()),
        toDTO(highQC.highestCommittedQC()),
        Option.from(highQC.highestTC().map(RocksSafetyStateStore::toDTO)));
  }

  private static TimeoutCertificateDTO toDTO(TimeoutCertificate tc) {
    return new TimeoutCertificateDTO(
        tc.getEpoch(), toDTO(tc.getRound()), toDTO(tc.getTimestampedSignatures()));
  }

  private static QuorumCertificateDTO toDTO(QuorumCertificate quorumCertificate) {
    var signatures = toDTO(quorumCertificate.getTimestampedSignatures());
    var voteData = toDTO(quorumCertificate.getVoteData());

    return new QuorumCertificateDTO(signatures, voteData);
  }

  private static TimestampedECDSASignaturesDTO toDTO(
      TimestampedECDSASignatures timestampedSignatures) {
    return new TimestampedECDSASignaturesDTO(
        timestampedSignatures.getSignatures().entrySet().stream()
            .collect(Collectors.toMap(e -> toDTO(e.getKey()), e -> toDTO(e.getValue()))));
  }

  private static TimestampedECDSASignatureDTO toDTO(TimestampedECDSASignature signature) {
    return new TimestampedECDSASignatureDTO(signature.timestamp(), signature.signature());
  }

  private static VoteDataDTO toDTO(VoteData voteData) {
    return new VoteDataDTO(
        toDTO(voteData.getProposed()),
        toDTO(voteData.getParent()),
        Option.from(voteData.getCommitted().map(RocksSafetyStateStore::toDTO)));
  }

  private static BFTHeaderDTO toDTO(BFTHeader header) {
    return new BFTHeaderDTO(
        toDTO(header.getRound()), toDTO(header.getVertexId()), toDTO(header.getLedgerHeader()));
  }

  private static LedgerHeaderDTO toDTO(com.radixdlt.consensus.LedgerHeader ledgerHeader) {
    return new LedgerHeaderDTO(
        ledgerHeader.getEpoch(),
        toDTO(ledgerHeader.getRound()),
        ledgerHeader.getStateVersion(),
        toDTO(ledgerHeader.getHashes()),
        ledgerHeader.consensusParentRoundTimestamp(),
        ledgerHeader.proposerTimestamp(),
        ledgerHeader.getNextEpoch().map(RocksSafetyStateStore::toDTO),
        ledgerHeader.nextProtocolVersion());
  }

  private static com.radixdlt.statecomputer.commit.LedgerHashes toDTO(LedgerHashes hashes) {
    return new com.radixdlt.statecomputer.commit.LedgerHashes(
        hashes.getStateRoot(), hashes.getTransactionRoot(), hashes.getReceiptRoot());
  }

  private static NextEpochDTO toDTO(NextEpoch epoch) {
    return new NextEpochDTO(
        epoch.getEpoch(),
        epoch.getValidators().stream()
            .map(RocksSafetyStateStore::toDTO)
            .collect(ImmutableSet.toImmutableSet()));
  }

  private static BFTValidatorDTO toDTO(BFTValidator bftValidator) {
    return new BFTValidatorDTO(
        bftValidator.getPower().toBigEndianBytes(), toDTO(bftValidator.getValidatorId()));
  }

  private static VertexIdDTO toDTO(HashCode vertexId) {
    return new VertexIdDTO(vertexId.asBytes());
  }

  private static BFTValidatorIdDTO toDTO(BFTValidatorId validatorId) {
    return new BFTValidatorIdDTO(validatorId.getKey(), validatorId.getValidatorAddress());
  }

  private static RoundDTO toDTO(Round round) {
    return new RoundDTO(round.number());
  }

  // -------------------------------------------------------------------------------------
  // Conversion to domain object
  // -------------------------------------------------------------------------------------

  private static SafetyState fromDTO(SafetyStateDTO dto) {
    return SafetyState.create(
        fromDTO(dto.validatorId()),
        fromDTO(dto.round()),
        dto.lastVote().map(RocksSafetyStateStore::fromDTO).toOptional());
  }

  private static BFTValidatorId fromDTO(BFTValidatorIdDTO dto) {
    return BFTValidatorId.create(dto.validatorAddress(), dto.key());
  }

  private static Round fromDTO(RoundDTO dto) {
    return Round.of(dto.round());
  }

  private static Vote fromDTO(VoteDTO dto) {
    return new Vote(
        fromDTO(dto.author()),
        fromDTO(dto.voteData()),
        dto.timestamp(),
        dto.signature(),
        fromDTO(dto.highQC()),
        dto.timeoutSignature().toOptional());
  }

  private static VoteData fromDTO(VoteDataDTO dto) {
    return new VoteData(
        fromDTO(dto.proposed()),
        fromDTO(dto.parent()),
        dto.committed().map(RocksSafetyStateStore::fromDTO).toNullable());
  }

  private static BFTHeader fromDTO(BFTHeaderDTO dto) {
    return new BFTHeader(
        fromDTO(dto.round()), fromDTO(dto.vertexId()), fromDTO(dto.ledgerHeader()));
  }

  private static HashCode fromDTO(VertexIdDTO dto) {
    return HashCode.fromBytes(dto.idBytes());
  }

  private static LedgerHeader fromDTO(LedgerHeaderDTO dto) {
    return new LedgerHeader(
        dto.epoch(),
        fromDTO(dto.round()),
        dto.stateVersion(),
        fromDTO(dto.hashes()),
        dto.consensusParentRoundTimestampMs(),
        dto.proposerTimestampMs(),
        dto.nextEpoch().map(RocksSafetyStateStore::fromDTO).toNullable(),
        dto.nextProtocolVersion().toNullable());
  }

  private static LedgerHashes fromDTO(com.radixdlt.statecomputer.commit.LedgerHashes hashes) {
    return LedgerHashes.create(hashes.stateRoot(), hashes.transactionRoot(), hashes.receiptRoot());
  }

  private static NextEpoch fromDTO(NextEpochDTO dto) {
    return NextEpoch.create(
        dto.epoch(),
        dto.validators().stream()
            .map(RocksSafetyStateStore::fromDTO)
            .collect(ImmutableSet.toImmutableSet()));
  }

  private static BFTValidator fromDTO(BFTValidatorDTO dto) {
    return BFTValidator.from(fromDTO(dto.validatorId()), UInt192.fromBigEndianBytes(dto.power()));
  }

  private static HighQC fromDTO(HighQCDTO dto) {
    return HighQC.from(
        fromDTO(dto.highestQC()),
        fromDTO(dto.highestCommittedQC()),
        dto.highestTC().map(RocksSafetyStateStore::fromDTO).toOptional());
  }

  private static QuorumCertificate fromDTO(QuorumCertificateDTO dto) {
    return new QuorumCertificate(fromDTO(dto.voteData()), fromDTO(dto.signatures()));
  }

  private static TimestampedECDSASignatures fromDTO(TimestampedECDSASignaturesDTO dto) {
    return new TimestampedECDSASignatures(
        dto.nodeToTimestampedSignature().entrySet().stream()
            .collect(Collectors.toMap(e -> fromDTO(e.getKey()), e -> fromDTO(e.getValue()))));
  }

  private static TimestampedECDSASignature fromDTO(TimestampedECDSASignatureDTO dto) {
    return TimestampedECDSASignature.from(dto.timestamp(), dto.signature());
  }

  private static TimeoutCertificate fromDTO(TimeoutCertificateDTO dto) {
    return new TimeoutCertificate(dto.epoch(), fromDTO(dto.round()), fromDTO(dto.signatures()));
  }
}
