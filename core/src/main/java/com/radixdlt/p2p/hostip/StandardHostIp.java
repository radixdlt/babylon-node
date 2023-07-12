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

package com.radixdlt.p2p.hostip;

import com.radixdlt.utils.properties.RuntimeProperties;
import java.util.HashSet;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/** Provides a standard {@link HostIp} retriever. */
public final class StandardHostIp {
  private static final Logger log = LogManager.getLogger();

  private StandardHostIp() {
    throw new IllegalStateException("Can't construct");
  }

  /**
   * Uses well-known web services to determine a public IP address and verifies a configured
   * environment variable / property against it.
   */
  public static HostIp defaultHostIp(RuntimeProperties properties) {
    final var networkQueryResult = NetworkQueryHostIp.create(properties).queryNetworkHosts();
    final var maybeHostIpFromEnv = new EnvironmentHostIp().hostIp();
    final var maybeHostIpFromProperties = new RuntimePropertiesHostIp(properties).hostIp();

    final var configuredHostIps = new HashSet<HostIp>();
    maybeHostIpFromEnv.ifPresent(configuredHostIps::add);
    maybeHostIpFromProperties.ifPresent(configuredHostIps::add);
    if (configuredHostIps.size() > 1) {
      throw new RuntimeException(
          String.format(
              "A host IP address of this node has been configured in both properties"
                  + " (network.host_ip=%s) and environment (RADIXDLT_HOST_IP_ADDRESS=%s) and they"
                  + " differ. Make sure you configure an unambiguous host IP address.",
              maybeHostIpFromProperties.orElseThrow(), maybeHostIpFromEnv.orElseThrow()));
    } else if (configuredHostIps.size() == 0) {
      if (networkQueryResult.maybeHostIp().isPresent()) {
        // All good, we have an IP address from network query
        log.info(
            "Host's public IP address has been acquired from an external oracle (services queried:"
                + " {}). Consider setting a `network.host_ip` property instead to lessen reliance"
                + " on external services.",
            networkQueryResult.hostsQueried());
        return networkQueryResult.maybeHostIp().orElseThrow();
      } else {
        throw new RuntimeException(
            String.format(
                "An IP address of this node hasn't been configured. "
                    + "Make sure you set your `network.host_ip` property. "
                    + "An attempt was made to acquire it from an external oracle, "
                    + "but that also failed (services queried: %s).",
                networkQueryResult.hostsQueried()));
      }
    } else {
      // We've got a configured IP and possibly also an IP address from an external oracle
      // we're going to use a configured IP, but issue a warning if it doesn't match what
      // we got from an oracle.

      final var configuredHostIp = configuredHostIps.iterator().next();

      if (networkQueryResult.maybeHostIp().stream()
          .anyMatch(hostIpFromNetwork -> !hostIpFromNetwork.equals(configuredHostIp))) {
        log.warn(
            "An IP address that was configured for this node ({}) differs from a public IP "
                + "address reported by an external oracle ({}, services queried: {}). "
                + "This indicates a likely misconfiguration. "
                + "Make sure your `network.host_ip` property or "
                + "`RADIXDLT_HOST_IP_ADDRESS` environment variable are set correctly.",
            configuredHostIp,
            networkQueryResult.maybeHostIp().orElseThrow(),
            networkQueryResult.hostsQueried());
      }

      log.info("Using a configured host IP address: {}", configuredHostIp);

      return configuredHostIp;
    }
  }
}
