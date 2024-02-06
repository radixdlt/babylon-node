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

import static org.junit.Assert.assertEquals;
import static org.mockito.Mockito.*;

import com.google.common.hash.HashCode;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.utils.UInt192;
import java.util.List;
import org.junit.Test;

public final class DivergentVertexExecutionDetectorTest {
  @Test
  public void when_divergent_execution__then_should_update_the_metrics() {
    final var metrics = new MetricsInitializer().initialize();
    final var author1 = BFTValidatorId.random();
    final var author2 = BFTValidatorId.random();
    final var author3 = BFTValidatorId.random();
    final var validatorSet =
        BFTValidatorSet.from(
            List.of(
                BFTValidator.from(author1, UInt192.ONE),
                BFTValidator.from(author2, UInt192.ONE),
                BFTValidator.from(author3, UInt192.ONE)));
    final var detector = new DivergentVertexExecutionDetector(metrics, validatorSet);
    final var vertex1 = HashUtils.random256();
    final var vertex2 = HashUtils.random256();

    // No divergence for vertex1
    detector.processVote(createVoteForVertexId(author1, vertex1, 1L));
    detector.processVote(createVoteForVertexId(author2, vertex1, 1L));

    // Divergence for vertex2
    detector.processVote(createVoteForVertexId(author1, vertex2, 2L));
    detector.processVote(createVoteForVertexId(author2, vertex2, 2L));
    detector.processVote(createVoteForVertexId(author3, vertex2, 3L));

    detector.summarizeAfterRoundAndReset(Round.epochInitial());

    assertEquals(
        1, // Expecting 1 divergent execution with 2 conflicting results (label)
        (int)
            metrics
                .bft()
                .divergentVertexExecutions()
                .label(new Metrics.Bft.DivergentVertexExecution(2))
                .get());

    assertEquals(
        1, // Expecting no more results from other labels
        (int) metrics.bft().divergentVertexExecutions().getSum());
  }

  private Vote createVoteForVertexId(
      BFTValidatorId author, HashCode vertexId, long resultDiscriminator) {
    final var ledgerHeader =
        LedgerHeader.create(
            0L,
            Round.epochInitial(),
            resultDiscriminator, // Using a different state version as a discriminator
            LedgerHashes.zero(),
            0L,
            0L);

    final var bftHeader = mock(BFTHeader.class);
    when(bftHeader.getVertexId()).thenReturn(vertexId);
    when(bftHeader.getLedgerHeader()).thenReturn(ledgerHeader);

    final var voteData = mock(VoteData.class);
    when(voteData.getProposed()).thenReturn(bftHeader);

    final var vote = mock(Vote.class);
    when(vote.getAuthor()).thenReturn(author);
    when(vote.getVoteData()).thenReturn(voteData);

    return vote;
  }
}
