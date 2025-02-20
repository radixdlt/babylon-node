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

package com.radixdlt.consensus.liveness;

import com.google.common.primitives.Doubles;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.Inject;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Main timeout calculator implementation, which uses two factors to calculate the timeout:
 *
 * <ol>
 *   <li>- the number of consecutive timeout occurrences
 *   <li>- and the current capacity of the vertex store
 */
public final class MultiFactorPacemakerTimeoutCalculator implements PacemakerTimeoutCalculator {
  private static final Logger logger = LogManager.getLogger();

  private final PacemakerTimeoutCalculatorConfig config;

  private final RateLimiter logRatelimiter =
      RateLimiter.create(0.016); // At most one log every ~minute

  @Inject
  public MultiFactorPacemakerTimeoutCalculator(PacemakerTimeoutCalculatorConfig config) {
    this.config = config;
  }

  @Override
  @SuppressWarnings("UnstableApiUsage")
  public long calculateTimeoutMs(long timeoutOccurrences, double vertexStoreUtilizationRatio) {
    final var consecutiveTimeoutFactor =
        Math.pow(
            config.consecutiveTimeoutFactorRate(),
            Math.min(config.consecutiveTimeoutFactorMaxExponent(), timeoutOccurrences));

    // It should already be in the [0, 1] range, but we're nonetheless sanitizing the input
    final var vertexStoreUtilizationRatioClamped =
        Doubles.constrainToRange(vertexStoreUtilizationRatio, 0, 1);

    final double vertexStoreUtilizationFactor;
    if (vertexStoreUtilizationRatioClamped <= config.vertexStoreUtilizationFactorThreshold()) {
      vertexStoreUtilizationFactor = 1;
    } else {
      // We're linearly transforming the current utilization
      // from [threshold, 1] to [1, maxExponent] to get the exponent
      // for the vertex store utilization factor.
      final var exponent =
          lerp(
              config.vertexStoreUtilizationFactorThreshold(),
              1.0,
              0.0,
              config.vertexStoreUtilizationFactorMaxExponent(),
              vertexStoreUtilizationRatioClamped);
      vertexStoreUtilizationFactor = Math.pow(config.vertexStoreUtilizationFactorRate(), exponent);
    }

    final var res =
        Math.round(
            config.baseTimeoutMs() * consecutiveTimeoutFactor * vertexStoreUtilizationFactor);

    if (vertexStoreUtilizationRatioClamped >= config.vertexStoreUtilizationFactorThreshold()
        && logRatelimiter.tryAcquire()) {
      logger.warn(
          "Vertex store is currently at {} of its maximum byte capacity. Consensus timeouts are"
              + " being slowed down to slow down pressure accumulation on the vertex store."
              + " [base_timeout ({} ms) * consecutive_timeout_factor ({}) *"
              + " vertex_store_utilization_factor ({}) = resultant_timeout ({} ms)]",
          vertexStoreUtilizationRatioClamped,
          config.baseTimeoutMs(),
          consecutiveTimeoutFactor,
          vertexStoreUtilizationFactor,
          res);
    }

    return res;
  }

  // Computes a linear interpolation (or extrapolation).
  // The input value `z` is interpolated from the source range [x, y]
  // to the target range [p, q].
  // E.g. if [x, y] = [10, 20], z = 15 and [p, q] = [0, 1], returns 0.5.
  // If z is outside [x, y] then it's extrapolated (linearly) and can produce
  // a value outside [p, q]. This should be handled by the caller.
  private static double lerp(double x, double y, double p, double q, double z) {
    return p + (q - p) * (z - x) / (y - x);
  }

  @Override
  public long additionalRoundTimeIfProposalReceivedMs() {
    return config.additionalRoundTimeIfProposalReceivedMs();
  }
}
