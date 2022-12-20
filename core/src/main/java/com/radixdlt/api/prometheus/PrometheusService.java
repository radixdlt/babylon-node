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

package com.radixdlt.api.prometheus;

import static com.radixdlt.RadixNodeApplication.SYSTEM_VERSION_KEY;
import static com.radixdlt.RadixNodeApplication.VERSION_STRING_KEY;

import com.google.inject.Inject;
import com.radixdlt.RadixNodeApplication;
import com.radixdlt.api.system.health.HealthInfoService;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.monitoring.InMemorySystemInfo;
import com.radixdlt.p2p.PeersView;
import com.radixdlt.prometheus.StateManagerPrometheus;
import java.util.AbstractCollection;

public class PrometheusService {

  private final JavaPrometheus javaPrometheus;
  private final StateManagerPrometheus stateManagerPrometheus;
  private final HealthInfoService healthInfoService;
  private final InMemorySystemInfo inMemorySystemInfo;
  private final BFTNode self;
  private final PeersView peersView;

  @Inject
  public PrometheusService(
      PeersView peersView,
      HealthInfoService healthInfoService,
      InMemorySystemInfo inMemorySystemInfo,
      @Self BFTNode self,
      StateManagerPrometheus stateManagerPrometheus,
      JavaPrometheus javaPrometheus) {
    this.peersView = peersView;
    this.healthInfoService = healthInfoService;
    this.inMemorySystemInfo = inMemorySystemInfo;
    this.self = self;
    this.stateManagerPrometheus = stateManagerPrometheus;
    this.javaPrometheus = javaPrometheus;
  }

  public String getMetrics() {
    var builder = new StringBuilder();
    builder.append(this.stateManagerPrometheus.prometheusMetrics());
    builder.append(this.javaPrometheus.prometheusMetrics());
    exportSystemInfo(builder);
    return builder.append('\n').toString();
  }

  private void exportSystemInfo(StringBuilder builder) {
    var currentEpochRound = inMemorySystemInfo.getCurrentRound();

    appendCounter(
        builder, "info_epochmanager_currentround_round", currentEpochRound.getRound().number());
    appendCounter(builder, "info_epochmanager_currentround_epoch", currentEpochRound.getEpoch());
    appendCounter(builder, "total_peers", peersView.peers().count());

    var totalValidators =
        inMemorySystemInfo
            .getEpochProof()
            .getNextValidatorSet()
            .map(BFTValidatorSet::getValidators)
            .map(AbstractCollection::size)
            .orElse(0);

    appendCounter(builder, "total_validators", totalValidators);

    appendCounterExtended(
        builder,
        prepareNodeInfo(),
        "nodeinfo",
        "Special metric used to convey information about the current node using labels. Value will"
            + " always be 0.",
        0.0);
  }

  private String prepareNodeInfo() {
    var builder = new StringBuilder("nodeinfo{");
    addBranchAndCommit(builder);
    addValidatorAddress(builder);
    appendField(builder, "health", healthInfoService.nodeStatus().name());
    appendField(builder, "key", self.getKey().toHex());
    return builder.append("}").toString();
  }

  private void addValidatorAddress(StringBuilder builder) {
    // TODO - add back when validators have addresses in the engine
    // appendField(builder, "own_validator_address", addressing.forValidators().of(self.getKey()));

    var inSet =
        inMemorySystemInfo
            .getEpochProof()
            .getNextValidatorSet()
            .map(set -> set.containsNode(self))
            .orElse(false);

    appendField(builder, "is_in_validator_set", inSet);
  }

  private void addBranchAndCommit(StringBuilder builder) {
    var branchAndCommit =
        RadixNodeApplication.systemVersionInfo().get(SYSTEM_VERSION_KEY).get(VERSION_STRING_KEY);
    appendField(builder, "branch_and_commit", branchAndCommit);
  }

  private void appendField(StringBuilder builder, String name, Object value) {
    builder.append(name).append("=\"").append(value).append("\",");
  }

  private static void appendCounter(StringBuilder builder, String name, Number value) {
    appendCounterExtended(builder, name, name, name, value.doubleValue());
  }

  private static void appendCounterExtended(
      StringBuilder builder, String name, String type, String help, Object value) {
    builder
        .append("# HELP ")
        .append(help)
        .append('\n')
        .append("# TYPE ")
        .append(type)
        .append(' ')
        .append("counter")
        .append('\n')
        .append(name)
        .append(' ')
        .append(value)
        .append('\n');
  }
}
