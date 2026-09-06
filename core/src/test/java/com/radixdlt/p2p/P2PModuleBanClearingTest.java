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

package com.radixdlt.p2p;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import com.google.common.collect.ImmutableMap;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.statecomputer.ProtocolState;
import org.junit.Test;

public final class P2PModuleBanClearingTest {
  private static final long ENACTMENT_EPOCH = 339898;

  @Test
  public void clears_bans_in_the_epoch_right_before_an_unconditional_enactment() {
    // Arrange
    final var protocolState =
        pending(ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(ENACTMENT_EPOCH));

    // Act
    final var oneEpochBefore =
        P2PModule.shouldClearNearProtocolUpdateBans(protocolState, ENACTMENT_EPOCH - 1);
    final var twoEpochsBefore =
        P2PModule.shouldClearNearProtocolUpdateBans(protocolState, ENACTMENT_EPOCH - 2);
    final var atEnactmentEpoch =
        P2PModule.shouldClearNearProtocolUpdateBans(protocolState, ENACTMENT_EPOCH);

    // Assert
    assertTrue(oneEpochBefore);
    assertFalse(twoEpochsBefore);
    assertFalse(atEnactmentEpoch);
  }

  @Test
  public void clears_bans_in_the_epoch_right_before_a_moratorium_enactment() {
    // Arrange
    final var protocolState =
        pending(ProtocolUpdateEnactmentCondition.unconditionallyAtEpoch(ENACTMENT_EPOCH));

    // Act
    final var oneEpochBefore =
        P2PModule.shouldClearNearProtocolUpdateBans(protocolState, ENACTMENT_EPOCH - 1);
    final var twoEpochsBefore =
        P2PModule.shouldClearNearProtocolUpdateBans(protocolState, ENACTMENT_EPOCH - 2);

    // Assert
    assertTrue(oneEpochBefore);
    assertFalse(twoEpochsBefore);
  }

  @Test
  public void clears_bans_from_one_epoch_before_the_lower_bound_of_a_readiness_enactment() {
    // Arrange
    final var protocolState =
        pending(
            ProtocolUpdateEnactmentCondition.singleReadinessThresholdBetweenEpochs(
                100, 200, Decimal.ofNonNegativeFraction(3, 4), 1));

    // Act
    final var beforeWindow = P2PModule.shouldClearNearProtocolUpdateBans(protocolState, 98);
    final var oneBeforeLowerBound = P2PModule.shouldClearNearProtocolUpdateBans(protocolState, 99);
    final var insideWindow = P2PModule.shouldClearNearProtocolUpdateBans(protocolState, 150);
    final var atUpperBound = P2PModule.shouldClearNearProtocolUpdateBans(protocolState, 200);

    // Assert
    assertFalse(beforeWindow);
    assertTrue(oneBeforeLowerBound);
    assertTrue(insideWindow);
    assertFalse(atUpperBound);
  }

  @Test
  public void never_clears_bans_for_an_update_chained_after_another() {
    // Arrange
    final var protocolState = pending(ProtocolUpdateEnactmentCondition.immediatelyAfter("test-v1"));

    // Act
    final var result = P2PModule.shouldClearNearProtocolUpdateBans(protocolState, 5);

    // Assert
    assertFalse(result);
  }

  private static ProtocolState pending(ProtocolUpdateEnactmentCondition condition) {
    final var trigger = new ProtocolUpdateTrigger("test-v2", condition);
    return new ProtocolState(
        ImmutableMap.of(),
        ImmutableMap.of(
            "test-v2",
            new ProtocolState.PendingProtocolUpdate(
                trigger, new ProtocolState.PendingProtocolUpdateState.Empty())));
  }
}
