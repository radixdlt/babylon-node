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

package com.radixdlt.api.system;

import static com.radixdlt.lang.Tuple.tuple;
import static java.util.Objects.requireNonNull;
import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.inject.AbstractModule;
import com.google.inject.Inject;
import com.radixdlt.api.SystemApiTestBase;
import com.radixdlt.api.system.generated.models.HealthResponse;
import com.radixdlt.api.system.generated.models.PendingProtocolUpdate;
import com.radixdlt.api.system.generated.models.SignalledReadinessPendingProtocolUpdateState;
import com.radixdlt.api.system.routes.HealthHandler;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.consensus.epoch.EpochRound;
import com.radixdlt.monitoring.InMemorySystemInfo;
import com.radixdlt.monitoring.InMemorySystemInfoState;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateTrigger;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.statecomputer.ProtocolState;
import com.radixdlt.utils.UInt64;
import java.util.Set;
import org.junit.Test;

public class HealthHandlerTest extends SystemApiTestBase {

  // Test setup parameters
  // Threshold: 70% for 20 epochs.
  // At epoch 100 we already have 5 started epochs of support.
  // Expected enactment at start of epoch 116.
  private static final long REQUIRED_EPOCHS = 20L;
  private static final long STARTED_EPOCHS_OF_SUPPORT = 5L;
  private static final long CURRENT_EPOCH = 100L;
  private static final long EPOCH_TARGET_DURATION_MS = 5 * 60 * 1000; /* 5 minutes */
  private static final long EPOCH_EFFECTIVE_START_MS = 1709835872431L;

  // Test assertion parameters
  private static final long EXPECTED_PROJECTED_ENACTMENT_EPOCH = 116;
  private static final long EXPECTED_PROJECTED_ENACTMENT_TIMESTAMP =
      EPOCH_EFFECTIVE_START_MS + (EPOCH_TARGET_DURATION_MS * 16);

  @Inject private HealthHandler sut;

  public HealthHandlerTest() {
    super(
        new AbstractModule() {
          @Override
          public void configure() {
            final var threshold =
                new ProtocolUpdateEnactmentCondition.SignalledReadinessThreshold(
                    Decimal.ofNonNegativeFraction(7, 10),
                    UInt64.fromNonNegativeLong(REQUIRED_EPOCHS));
            final var thresholdState =
                new ProtocolState.PendingProtocolUpdateState.SignalledReadinessThresholdState(
                    UInt64.fromNonNegativeLong(STARTED_EPOCHS_OF_SUPPORT));
            final var pendingProtocolUpdate =
                new ProtocolState.PendingProtocolUpdate(
                    new ProtocolUpdateTrigger(
                        "custom-pending-3",
                        new ProtocolUpdateEnactmentCondition.EnactAtStartOfEpochIfValidatorsReady(
                            UInt64.fromNonNegativeLong(10L),
                            UInt64.fromNonNegativeLong(200L),
                            ImmutableList.of(threshold))),
                    new ProtocolState.PendingProtocolUpdateState
                        .ForSignalledReadinessSupportCondition(
                        ImmutableList.of(tuple(threshold, thresholdState))));
            final var pendingProtocolUpdateTwo =
                new ProtocolState.PendingProtocolUpdate(
                    new ProtocolUpdateTrigger(
                        "custom-pending-4",
                        new ProtocolUpdateEnactmentCondition
                            .EnactImmediatelyAfterEndOfProtocolUpdate("custom-pending-3")),
                    new ProtocolState.PendingProtocolUpdateState.Empty());

            final var protocolState =
                new ProtocolState(
                    ImmutableMap.of(UInt64.fromNonNegativeLong(5L), "custom-enacted-2"),
                    ImmutableMap.of(
                        pendingProtocolUpdate.protocolUpdateTrigger().nextProtocolVersion(),
                        pendingProtocolUpdate,
                        pendingProtocolUpdateTwo.protocolUpdateTrigger().nextProtocolVersion(),
                        pendingProtocolUpdateTwo));

            final var systemInfo = mock(InMemorySystemInfo.class);
            when(systemInfo.getCurrentRound())
                .thenReturn(EpochRound.of(CURRENT_EPOCH, Round.epochInitial()));
            when(systemInfo.getState())
                .thenReturn(
                    new InMemorySystemInfoState(
                        protocolState,
                        EpochRound.of(CURRENT_EPOCH, Round.epochInitial()),
                        Set.of() /* unused in this test */,
                        EPOCH_TARGET_DURATION_MS,
                        EPOCH_EFFECTIVE_START_MS));
            when(systemInfo.ledgerUpdateEventProcessor()).thenReturn(ignored -> {});
            bind(InMemorySystemInfo.class).toInstance(systemInfo);
          }
        });
  }

  @Test
  public void can_retrieve_health_response() throws Exception {
    // Arrange
    start();

    // Act
    var response = handleRequestWithExpectedResponse(sut, HealthResponse.class);

    // Assert
    assertThat(response.getStatus()).isNotNull();
  }

  @Test
  public void test_protocol_update_enactment_projection() throws Exception {
    // Arrange
    start();

    // Act
    final var response = handleRequestWithExpectedResponse(sut, HealthResponse.class);

    // Assert first pending enactment
    assertEquals(
        "custom-pending-3", response.getPendingProtocolUpdates().get(0).getProtocolVersion());
    assertEquals(
        "5bfdcc71f883edd0custom-pending-3",
        response.getPendingProtocolUpdates().get(0).getReadinessSignalName());
    assertEquals(
        EXPECTED_PROJECTED_ENACTMENT_EPOCH,
        requireNonNull(
                response.getPendingProtocolUpdates().get(0).getProjectedEnactmentAtStartOfEpoch())
            .longValue());

    assertEquals(
        EXPECTED_PROJECTED_ENACTMENT_TIMESTAMP,
        requireNonNull(response.getPendingProtocolUpdates().get(0).getProjectedEnactmentTimestamp())
            .longValue());

    final var thresholdState =
        ((SignalledReadinessPendingProtocolUpdateState)
                response.getPendingProtocolUpdates().get(0).getState())
            .getThresholdsState()
            .get(0)
            .getThresholdState();

    assertEquals(
        STARTED_EPOCHS_OF_SUPPORT,
        requireNonNull(thresholdState.getConsecutiveStartedEpochsOfSupport()).longValue());
    assertEquals(
        EXPECTED_PROJECTED_ENACTMENT_EPOCH,
        requireNonNull(thresholdState.getProjectedFulfillmentAtStartOfEpoch()).longValue());
    assertEquals(
        EXPECTED_PROJECTED_ENACTMENT_TIMESTAMP,
        requireNonNull(thresholdState.getProjectedFulfillmentTimestamp()).longValue());

    // Assert second pending enactment (immediately after custom-pending-3)
    assertEquals(
        "custom-pending-4", response.getPendingProtocolUpdates().get(1).getProtocolVersion());
    assertEquals(
        PendingProtocolUpdate.ReadinessSignalStatusEnum.NO_SIGNAL_REQUIRED,
        response.getPendingProtocolUpdates().get(1).getReadinessSignalStatus());
    assertNull(response.getPendingProtocolUpdates().get(1).getReadinessSignalName());
    // It could be argued for us to copy the custom-pending-3 values, but for now let's just
    // use "null" because:
    // (A) it's not ~technically~ at the start of this epoch
    // (B) this will only be used on testnets
    // (C) this will likely not even be observable on testnets because
    //     this setting will be used to trigger updates straight off of genesis,
    //     before the System API boots, so we'll never be able to observe it as "pending"
    assertNull(response.getPendingProtocolUpdates().get(1).getProjectedEnactmentAtStartOfEpoch());
    assertNull(response.getPendingProtocolUpdates().get(1).getProjectedEnactmentTimestamp());
  }
}
