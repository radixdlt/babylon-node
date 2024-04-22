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

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.SerializerConstants;
import com.radixdlt.serialization.SerializerDummy;
import com.radixdlt.serialization.SerializerId2;
import com.radixdlt.statecomputer.commit.LedgerProof;
import com.radixdlt.statecomputer.commit.LedgerProofOrigin;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Stream;

@SerializerId2("consensus.qc")
public final class QuorumCertificate {
  @JsonProperty(SerializerConstants.SERIALIZER_NAME)
  @DsonOutput(value = {Output.API, Output.WIRE, Output.PERSIST})
  SerializerDummy serializer = SerializerDummy.DUMMY;

  @JsonProperty("signatures")
  @DsonOutput(Output.ALL)
  private final TimestampedECDSASignatures signatures;

  @JsonProperty("vote_data")
  @DsonOutput(Output.ALL)
  private final VoteData voteData;

  @JsonCreator
  public QuorumCertificate(
      @JsonProperty(value = "vote_data", required = true) VoteData voteData,
      @JsonProperty(value = "signatures", required = true) TimestampedECDSASignatures signatures) {
    this.voteData = Objects.requireNonNull(voteData);
    this.signatures = Objects.requireNonNull(signatures);
  }

  /**
   * Create a mocked QC for genesis vertex
   *
   * @param genesisVertexWithHash the vertex to create a qc for
   * @return a mocked QC
   */
  public static QuorumCertificate createInitialEpochQC(
      VertexWithHash genesisVertexWithHash, LedgerHeader ledgerHeader) {
    final var vertex = genesisVertexWithHash.vertex();
    if (!vertex.getRound().isEpochInitial()) {
      throw new IllegalArgumentException(String.format("Vertex is not genesis: %s", vertex));
    }

    final var header = new BFTHeader(vertex.getRound(), genesisVertexWithHash.hash(), ledgerHeader);
    final var voteData = new VoteData(header, header, header);
    return new QuorumCertificate(voteData, new TimestampedECDSASignatures());
  }

  public Round getRound() {
    return voteData.getProposed().getRound();
  }

  public long getEpoch() {
    return this.voteData.getProposed().getLedgerHeader().getEpoch();
  }

  /**
   * @return The weighted timestamp of the signatures, in milliseconds since Unix Epoch.
   */
  public long getWeightedTimestampOfSignatures() {
    // If this is a genesis QC then its signatures are mocked, so just use the previous timestamp
    // NB - this does have the edge case of never increasing timestamps if configuration is
    // one round per epoch but this is likely good enough

    return getProposedHeader().getRound().isEpochInitial()
        ? getProposedHeader().getLedgerHeader().consensusParentRoundTimestamp()
        : getTimestampedSignatures().weightedTimestampMillis();
  }

  public BFTHeader getProposedHeader() {
    return voteData.getProposed();
  }

  public BFTHeader getParentHeader() {
    return voteData.getParent();
  }

  public Optional<BFTHeader> getCommittedHeader() {
    return voteData.getCommitted();
  }

  public Optional<ProcessedQcCommit> getProcessedCommit(Hasher hasher) {
    return voteData
        .getCommitted()
        .map(
            committed -> {
              if (signatures.count() == 0) {
                return new ProcessedQcCommit.OfInitialEpochQc(committed);
              } else {
                final var proofOrigin =
                    new LedgerProofOrigin.Consensus(
                        hasher.hashDsonEncoded(voteData), REv2ToConsensus.signatures(signatures));
                final var ledgerProof =
                    new LedgerProof(
                        REv2ToConsensus.ledgerHeader(committed.getLedgerHeader()), proofOrigin);
                return new ProcessedQcCommit.OfConensusQc(committed, ledgerProof, proofOrigin);
              }
            });
  }

  public VoteData getVoteData() {
    return voteData;
  }

  public TimestampedECDSASignatures getTimestampedSignatures() {
    return signatures;
  }

  public Stream<BFTValidatorId> getSigners() {
    return signatures.getSignatures().keySet().stream();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    QuorumCertificate that = (QuorumCertificate) o;
    return Objects.equals(signatures, that.signatures) && Objects.equals(voteData, that.voteData);
  }

  @Override
  public int hashCode() {
    return Objects.hash(signatures, voteData);
  }

  @Override
  public String toString() {
    return String.format(
        "QC{e=%s p=%s c=%s pv=%s num_signers=%s}",
        this.getEpoch(),
        this.getRound(),
        this.getCommittedHeader().map(h -> h.getRound().toString()).orElse(""),
        this.getProposedHeader().getVertexId(),
        this.signatures.count());
  }
}
