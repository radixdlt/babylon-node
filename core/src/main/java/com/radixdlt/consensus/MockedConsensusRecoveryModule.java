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

import com.google.common.hash.HashCode;
import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.TypeLiteral;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.liveness.ProposerElection;
import com.radixdlt.consensus.liveness.WeightedRotatingLeaders;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.store.LastEpochProof;
import com.radixdlt.utils.UInt256;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.function.Function;
import java.util.function.LongFunction;
import java.util.stream.IntStream;
import java.util.stream.Stream;

/** Starting configuration for simulation/deterministic steady state tests. */
public class MockedConsensusRecoveryModule extends AbstractModule {

  private final Builder builder;

  private MockedConsensusRecoveryModule(Builder builder) {
    this.builder = builder;
  }

  @Override
  protected void configure() {
    super.configure();
    if (this.builder.withEpoch) {
      bind(new TypeLiteral<Function<Long, BFTValidatorSet>>() {})
          .toInstance(this.builder.validatorSetMapping()::apply);
    }
  }

  @Provides
  private RoundUpdate initialRoundUpdate(
      BFTConfiguration configuration, ProposerElection proposerElection) {
    HighQC highQC = configuration.getVertexStoreState().getHighQC();
    Round round = highQC.highestQC().getRound().next();
    final BFTNode leader = proposerElection.getProposer(round);
    final BFTNode nextLeader = proposerElection.getProposer(round.next());

    return RoundUpdate.create(round, highQC, leader, nextLeader);
  }

  @Provides
  private BFTValidatorSet validatorSet() {
    return this.builder.validatorSetMapping().apply(1);
  }

  @Provides
  private BFTConfiguration configuration(
      @LastEpochProof LedgerProof proof, BFTValidatorSet validatorSet, Hasher hasher) {
    var accumulatorState = new AccumulatorState(0, this.builder.preGenesisAccumulatorHash);
    VertexWithHash genesisVertex =
        Vertex.createGenesis(LedgerHeader.genesis(accumulatorState, validatorSet, 0))
            .withId(hasher);
    LedgerHeader nextLedgerHeader =
        LedgerHeader.create(
            proof.getNextEpoch(), Round.genesis(), proof.getAccumulatorState(), proof.timestamp());
    var genesisQC = QuorumCertificate.ofGenesis(genesisVertex, nextLedgerHeader);
    var proposerElection = new WeightedRotatingLeaders(validatorSet);
    return new BFTConfiguration(
        proposerElection,
        validatorSet,
        VertexStoreState.create(HighQC.from(genesisQC), genesisVertex, Optional.empty(), hasher));
  }

  public static class Builder {
    private HashCode preGenesisAccumulatorHash;
    private List<BFTNode> nodes;
    private boolean withEpoch;
    private EpochNodeWeightMapping epochNodeWeightMapping;

    public Builder() {
      this(false);
    }

    public Builder(boolean withEpoch) {
      this.withEpoch = withEpoch;
      this.preGenesisAccumulatorHash = HashUtils.zero256();
    }

    public Builder withPreGenesisAccumulatorHash(HashCode preGenesisAccumulatorHash) {
      this.preGenesisAccumulatorHash = preGenesisAccumulatorHash;
      return this;
    }

    public Builder withEpochNodeIndexesMapping(
        Function<Long, IntStream> epochToNodeIndexesMapping) {
      Objects.requireNonNull(epochToNodeIndexesMapping);
      this.epochNodeWeightMapping = epoch -> equalWeight(epochToNodeIndexesMapping.apply(epoch));
      return this;
    }

    private static Stream<NodeIndexAndWeight> equalWeight(IntStream indexes) {
      return indexes.mapToObj(i -> NodeIndexAndWeight.from(i, UInt256.ONE));
    }

    public Builder withEpochNodeWeightMapping(EpochNodeWeightMapping epochNodeWeightMapping) {
      this.epochNodeWeightMapping = Objects.requireNonNull(epochNodeWeightMapping);
      return this;
    }

    LongFunction<BFTValidatorSet> validatorSetMapping() {
      return epochNodeWeightMapping == null
          ? epoch -> completeEqualWeightValidatorSet(this.nodes)
          : epoch -> partialMixedWeightValidatorSet(epoch, this.nodes, this.epochNodeWeightMapping);
    }

    private static BFTValidatorSet completeEqualWeightValidatorSet(List<BFTNode> nodes) {
      return BFTValidatorSet.from(nodes.stream().map(node -> BFTValidator.from(node, UInt256.ONE)));
    }

    private static BFTValidatorSet partialMixedWeightValidatorSet(
        long epoch, List<BFTNode> nodes, EpochNodeWeightMapping mapper) {
      return BFTValidatorSet.from(
          mapper
              .nodesAndWeightFor(epoch)
              .map(niw -> BFTValidator.from(nodes.get(niw.index()), niw.weight())));
    }

    public Builder withNodes(List<BFTNode> nodes) {
      this.nodes = nodes;
      return this;
    }

    public MockedConsensusRecoveryModule build() {
      return new MockedConsensusRecoveryModule(this);
    }
  }
}
