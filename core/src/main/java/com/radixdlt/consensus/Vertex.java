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
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.SerializerConstants;
import com.radixdlt.serialization.SerializerDummy;
import com.radixdlt.serialization.SerializerId2;
import com.radixdlt.transactions.Transaction;
import java.util.List;
import java.util.Objects;
import javax.annotation.concurrent.Immutable;

/**
 * A vertex representing a possible future committed round of transactions.
 *
 * <p>Note that this vertex class is a DTO and could have been sent from a peer, as such its content
 * is potentially untrusted.
 *
 * <p>This Vertex class should very rarely be used raw, and generally should be converted to a
 * VertexWithHash.
 *
 * <p>Note that this implementation of HotStuff, the quorum certificate always points the Vertex's
 * parent. In this node software, vertices are created on highQC.highestQC, so this is typically
 * qcToParent = highQC.qcToParent.
 */
@Immutable
@SerializerId2("consensus.vertex")
public final class Vertex {
  @JsonProperty(SerializerConstants.SERIALIZER_NAME)
  @DsonOutput(value = {Output.API, Output.WIRE, Output.PERSIST})
  SerializerDummy serializer = SerializerDummy.DUMMY;

  @JsonProperty("qc")
  @DsonOutput(Output.ALL)
  private final QuorumCertificate qcToParent;

  // This is serialized in getSerializerRound below
  private final Round round;

  @JsonProperty("txns")
  @DsonOutput(Output.ALL)
  private final List<byte[]> transactions;

  @JsonProperty("tout")
  @DsonOutput(Output.ALL)
  private final Boolean proposerTimedOut;

  // This is serialized in getProposerBytes below
  private final BFTNode proposer;

  private Vertex(
      QuorumCertificate qcToParent,
      Round round,
      List<byte[]> transactions,
      BFTNode proposer,
      Boolean proposerTimedOut) {
    this.qcToParent = requireNonNull(qcToParent);
    this.round = requireNonNull(round);

    if (proposerTimedOut != null && proposerTimedOut && !transactions.isEmpty()) {
      throw new IllegalArgumentException("Transactions must be empty if timeout");
    }

    if (transactions != null) {
      transactions.forEach(Objects::requireNonNull);
    }

    this.transactions = transactions;
    this.proposer = proposer;
    this.proposerTimedOut = proposerTimedOut;
  }

  @JsonCreator
  public static Vertex create(
      @JsonProperty(value = "qc", required = true) QuorumCertificate parentQC,
      @JsonProperty("round") long roundNumber,
      @JsonProperty("txns") List<byte[]> transactions,
      @JsonProperty("p") byte[] proposer,
      @JsonProperty("tout") Boolean proposerTimedOut)
      throws PublicKeyException {
    return new Vertex(
        parentQC,
        Round.of(roundNumber),
        transactions == null ? List.of() : transactions,
        proposer != null ? BFTNode.fromPublicKeyBytes(proposer) : null,
        proposerTimedOut);
  }

  public static Vertex createGenesis(LedgerHeader ledgerHeader) {
    BFTHeader header = BFTHeader.ofGenesisAncestor(ledgerHeader);
    final VoteData voteData = new VoteData(header, header, header);
    final QuorumCertificate parentQC =
        new QuorumCertificate(voteData, new TimestampedECDSASignatures());
    return new Vertex(parentQC, Round.genesis(), null, null, false);
  }

  public static Vertex createTimeout(QuorumCertificate parentQC, Round round, BFTNode proposer) {
    return new Vertex(parentQC, round, List.of(), proposer, true);
  }

  public static Vertex create(
      QuorumCertificate parentQC, Round round, List<Transaction> transactions, BFTNode proposer) {
    if (round.number() == 0) {
      throw new IllegalArgumentException("Only genesis can have round 0.");
    }

    var transactionBytes = transactions.stream().map(Transaction::getPayload).toList();

    return new Vertex(parentQC, round, transactionBytes, proposer, false);
  }

  @JsonProperty("p")
  @DsonOutput(Output.ALL)
  private byte[] getProposerBytes() {
    return proposer == null ? null : proposer.getKey().getCompressedBytes();
  }

  public VertexWithHash withId(Hasher hasher) {
    return VertexWithHash.from(this, hasher);
  }

  public BFTNode getProposer() {
    return proposer;
  }

  public boolean isTimeout() {
    return proposerTimedOut != null && proposerTimedOut;
  }

  public QuorumCertificate getQCToParent() {
    return qcToParent;
  }

  public Round getRound() {
    return round;
  }

  public List<Transaction> getTransactions() {
    return transactions == null
        ? List.of()
        : transactions.stream().map(Transaction::create).toList();
  }

  @JsonProperty("round")
  @DsonOutput(Output.ALL)
  private Long getSerializerRound() {
    return this.round == null ? null : this.round.number();
  }

  @Override
  public String toString() {
    return String.format("Vertex{round=%s, qc=%s, txns=%s}", round, qcToParent, getTransactions());
  }

  @Override
  public int hashCode() {
    return Objects.hash(qcToParent, proposer, round, transactions, proposerTimedOut);
  }

  @Override
  public boolean equals(Object o) {
    if (!(o instanceof Vertex)) {
      return false;
    }

    Vertex v = (Vertex) o;
    return Objects.equals(v.round, this.round)
        && Objects.equals(v.proposerTimedOut, this.proposerTimedOut)
        && Objects.equals(v.proposer, this.proposer)
        && Objects.equals(v.getTransactions(), this.getTransactions())
        && Objects.equals(v.qcToParent, this.qcToParent);
  }
}
