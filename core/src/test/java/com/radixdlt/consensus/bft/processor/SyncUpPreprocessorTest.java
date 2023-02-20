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

package com.radixdlt.consensus.bft.processor;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import com.radixdlt.consensus.BFTHeader;
import com.radixdlt.consensus.HighQC;
import com.radixdlt.consensus.LedgerHeader;
import com.radixdlt.consensus.LedgerProof;
import com.radixdlt.consensus.Proposal;
import com.radixdlt.consensus.QuorumCertificate;
import com.radixdlt.consensus.bft.BFTSyncer;
import com.radixdlt.consensus.bft.BFTSyncer.SyncResult;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.bft.RoundUpdate;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.MetricsInitializer;
import com.radixdlt.utils.PrivateKeys;
import java.util.Optional;
import org.junit.Before;
import org.junit.Test;

public class SyncUpPreprocessorTest {

  private SyncUpPreprocessor syncUpPreprocessor;

  private final BFTEventProcessorAtCurrentRound forwardTo =
      mock(BFTEventProcessorAtCurrentRound.class);
  private final BFTSyncer bftSyncer = mock(BFTSyncer.class);
  private final Metrics metrics = new MetricsInitializer().initialize();
  private final RoundUpdate initialRoundUpdate = mock(RoundUpdate.class);

  @Before
  public void setUp() {
    this.syncUpPreprocessor =
        new SyncUpPreprocessor(forwardTo, bftSyncer, metrics, initialRoundUpdate);
  }

  @Test
  public void when_round_update__then_should_process_cached_events() {
    final var proposal = mock(Proposal.class);
    final var proposalHighQc = mock(HighQC.class);
    final var proposalHighestCommittedQc = mock(QuorumCertificate.class);
    final var proposalLedgerProof = mock(LedgerProof.class);
    when(initialRoundUpdate.getCurrentRound()).thenReturn(Round.of(2));
    when(proposal.getAuthor())
        .thenReturn(BFTValidatorId.create(PrivateKeys.ofNumeric(1).getPublicKey()));
    when(proposal.getRound()).thenReturn(Round.of(4));
    when(proposal.highQC()).thenReturn(proposalHighQc);
    when(proposalHighQc.highestCommittedQC()).thenReturn(proposalHighestCommittedQc);
    var header = mock(BFTHeader.class);
    when(header.getLedgerHeader()).thenReturn(mock(LedgerHeader.class));
    when(proposalHighestCommittedQc.getCommittedHeader()).thenReturn(Optional.of(header));
    when(proposalLedgerProof.isEndOfEpoch()).thenReturn(false);
    when(bftSyncer.syncToQC(any(), any(), any())).thenReturn(SyncResult.IN_PROGRESS);

    // we're at v2, proposal for v4 should get cached as sync returns IN_PROGRESS
    this.syncUpPreprocessor.processProposal(proposal);

    final var newRoundUpdate = mock(RoundUpdate.class);
    when(newRoundUpdate.getCurrentRound()).thenReturn(Round.of(4));
    when(bftSyncer.syncToQC(any(), any(), any())).thenReturn(SyncResult.SYNCED);

    // we're going straight to v4, cached proposal should get processed
    this.syncUpPreprocessor.processRoundUpdate(newRoundUpdate);
    verify(forwardTo, times(1)).processProposal(proposal);
  }
}
