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

package com.radixdlt.api.system.routes;

import static com.radixdlt.api.system.generated.models.PendingProtocolUpdate.ReadinessSignalStatusEnum.NO_SIGNAL_REQUIRED;

import com.google.inject.Inject;
import com.radixdlt.api.system.SystemJsonHandler;
import com.radixdlt.api.system.generated.models.*;
import com.radixdlt.api.system.health.HealthInfoService;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple;
import com.radixdlt.monitoring.InMemorySystemInfoState;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition.EnactAtStartOfEpochIfValidatorsReady;
import com.radixdlt.protocol.ProtocolUpdateEnactmentCondition.EnactAtStartOfEpochUnconditionally;
import com.radixdlt.statecomputer.ProtocolState;
import com.radixdlt.statecomputer.ProtocolState.PendingProtocolUpdateState.ForSignalledReadinessSupportCondition;
import java.util.stream.Collectors;

public final class HealthHandler extends SystemJsonHandler<HealthResponse> {
  private final HealthInfoService healthInfoService;

  @Inject
  HealthHandler(HealthInfoService healthInfoService) {
    super();
    this.healthInfoService = healthInfoService;
  }

  @Override
  public HealthResponse handleRequest() {
    final var nodeStatus = healthInfoService.nodeStatus();
    final var statusEnum =
        switch (nodeStatus) {
          case BOOTING_PRE_GENESIS -> HealthResponse.StatusEnum.BOOTING_PRE_GENESIS;
          case SYNCING -> HealthResponse.StatusEnum.SYNCING;
          case UP -> HealthResponse.StatusEnum.UP;
          case OUT_OF_SYNC -> HealthResponse.StatusEnum.OUT_OF_SYNC;
        };

    final var statistic = healthInfoService.recentSelfProposalMissStatistic();

    final var readinessSignalStatuses = healthInfoService.readinessSignalStatuses();

    final var systemInfoState = healthInfoService.systemInfoState();
    final var protocolState = systemInfoState.protocolState();

    return new HealthResponse()
        .status(statusEnum)
        .detail(nodeStatus.detail())
        .recentSelfProposalMissStatistic(
            new RecentSelfProposalMissStatistic()
                .missedCount(statistic.missedCount().toLong())
                .recentProposalsTrackedCount(statistic.recentProposalsTrackedCount().toLong()))
        .currentProtocolVersion(protocolState.currentProtocolVersion())
        .enactedProtocolUpdates(
            protocolState.enactedProtocolUpdates().entrySet().stream()
                .map(
                    e ->
                        new EnactedProtocolUpdate()
                            .stateVersion(e.getKey().toLong())
                            .resultantProtocolVersion(e.getValue()))
                .toList())
        .pendingProtocolUpdates(
            protocolState.pendingProtocolUpdates().values().stream()
                .map(
                    p ->
                        pendingProtocolUpdate(
                            p,
                            readinessSignalStatuses.getOrDefault(
                                p.protocolUpdateTrigger().nextProtocolVersion(),
                                NO_SIGNAL_REQUIRED),
                            systemInfoState))
                .toList());
  }

  private PendingProtocolUpdate pendingProtocolUpdate(
      ProtocolState.PendingProtocolUpdate pendingProtocolUpdate,
      PendingProtocolUpdate.ReadinessSignalStatusEnum readinessSignalStatus,
      InMemorySystemInfoState systemInfoState) {
    final var currentEpoch = systemInfoState.currentEpochRound().getEpoch();
    final var epochStartTimeCalculator =
        new EpochStartTimeCalculator(
            systemInfoState.consensusManagerConfigEpochTargetDurationMs(),
            currentEpoch,
            systemInfoState.consensusManagerStateEpochEffectiveStartMs());

    return switch (pendingProtocolUpdate.protocolUpdateTrigger().enactmentCondition()) {
      case EnactAtStartOfEpochIfValidatorsReady atStartOfEpochCondition -> {
        final var state = (ForSignalledReadinessSupportCondition) pendingProtocolUpdate.state();

        final var projectedFulfillmentEpochByThreshold =
            state.thresholdsState().stream()
                .collect(
                    Collectors.toMap(
                        Tuple.Tuple2::first,
                        thresholdState -> {
                          final var consecutiveStartedEpochsOfSupport =
                              thresholdState.last().consecutiveStartedEpochsOfSupport().toLong();
                          if (consecutiveStartedEpochsOfSupport <= 0) {
                            return Option.<Long>empty();
                          } else {
                            final var requiredConsecutiveCompletedEpochsOfSupport =
                                thresholdState
                                    .first()
                                    .requiredConsecutiveCompletedEpochsOfSupport()
                                    .toLong();
                            final var projectedFulfillmentAtStartOfEpoch =
                                currentEpoch
                                    + requiredConsecutiveCompletedEpochsOfSupport
                                    - consecutiveStartedEpochsOfSupport
                                    + 1;

                            if (projectedFulfillmentAtStartOfEpoch
                                    >= atStartOfEpochCondition.lowerBoundInclusive().toLong()
                                && projectedFulfillmentAtStartOfEpoch
                                    < atStartOfEpochCondition.upperBoundExclusive().toLong()) {
                              return Option.some(projectedFulfillmentAtStartOfEpoch);
                            } else {
                              return Option.<Long>none();
                            }
                          }
                        }));

        final var apiState =
            new SignalledReadinessPendingProtocolUpdateState()
                .thresholdsState(
                    state.thresholdsState().stream()
                        .map(
                            thresholdState -> {
                              final var apiThresholdState =
                                  new SignalledReadinessThresholdState()
                                      .consecutiveStartedEpochsOfSupport(
                                          thresholdState
                                              .last()
                                              .consecutiveStartedEpochsOfSupport()
                                              .toLong());

                              projectedFulfillmentEpochByThreshold
                                  .getOrDefault(thresholdState.first(), Option.none())
                                  .ifPresent(
                                      projectedEnactmentEpoch -> {
                                        apiThresholdState.setProjectedFulfillmentAtStartOfEpoch(
                                            projectedEnactmentEpoch);
                                        apiThresholdState.setProjectedFulfillmentTimestamp(
                                            epochStartTimeCalculator.estimateStartOfEpoch(
                                                projectedEnactmentEpoch));
                                      });

                              return new SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState()
                                  .threshold(
                                      new SignalledReadinessThreshold()
                                          .requiredRatioOfStakeSupported(
                                              thresholdState
                                                  .first()
                                                  .requiredRatioOfStakeSupported()
                                                  .toString())
                                          .requiredConsecutiveCompletedEpochsOfSupport(
                                              thresholdState
                                                  .first()
                                                  .requiredConsecutiveCompletedEpochsOfSupport()
                                                  .toLong()))
                                  .thresholdState(apiThresholdState);
                            })
                        .toList())
                .type(PendingProtocolUpdateStateType.FORSIGNALLEDREADINESSSUPPORTCONDITION);

        final var res =
            new PendingProtocolUpdate()
                .protocolVersion(
                    pendingProtocolUpdate.protocolUpdateTrigger().nextProtocolVersion())
                .state(apiState)
                .readinessSignalName(
                    pendingProtocolUpdate.protocolUpdateTrigger().readinessSignalName())
                .readinessSignalStatus(readinessSignalStatus);

        projectedFulfillmentEpochByThreshold.values().stream()
            .flatMap(Option::stream)
            .mapToLong(l -> l)
            .min()
            .ifPresent(
                earliestFulfillmentEpoch -> {
                  res.setProjectedEnactmentAtStartOfEpoch(earliestFulfillmentEpoch);
                  res.setProjectedEnactmentTimestamp(
                      epochStartTimeCalculator.estimateStartOfEpoch(earliestFulfillmentEpoch));
                });

        yield res;
      }
      case EnactAtStartOfEpochUnconditionally condition -> {
        final var enactmentEpoch = condition.epoch().toLong();
        yield new PendingProtocolUpdate()
            .protocolVersion(pendingProtocolUpdate.protocolUpdateTrigger().nextProtocolVersion())
            .state(new EmptyPendingProtocolUpdateState().type(PendingProtocolUpdateStateType.EMPTY))
            .projectedEnactmentAtStartOfEpoch(enactmentEpoch)
            .projectedEnactmentTimestamp(
                epochStartTimeCalculator.estimateStartOfEpoch(enactmentEpoch))
            .readinessSignalStatus(readinessSignalStatus);
      }
      case ProtocolUpdateEnactmentCondition.EnactImmediatelyAfterEndOfProtocolUpdate condition -> {
        yield new PendingProtocolUpdate()
            .protocolVersion(pendingProtocolUpdate.protocolUpdateTrigger().nextProtocolVersion())
            .state(new EmptyPendingProtocolUpdateState().type(PendingProtocolUpdateStateType.EMPTY))
            .readinessSignalStatus(readinessSignalStatus);
      }
    };
  }

  private record EpochStartTimeCalculator(
      long expectedEpochLengthMs, long baseEpoch, long baseEpochStartTimestamp) {

    long estimateStartOfEpoch(long epoch) {
      return baseEpochStartTimestamp + expectedEpochLengthMs * (epoch - baseEpoch);
    }
  }
}
