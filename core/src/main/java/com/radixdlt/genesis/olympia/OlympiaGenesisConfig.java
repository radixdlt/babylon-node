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

package com.radixdlt.genesis.olympia;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.genesis.olympia.bech32.OlympiaBech32;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.net.MalformedURLException;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import org.bouncycastle.util.encoders.Base64;

public record OlympiaGenesisConfig(
    URL nodeCoreApiUrl,
    Optional<String> basicAuthCredentialsBase64,
    ECDSASecp256k1PublicKey nodePublicKey) {
  private static final List<String> VALID_NODE_HRPS =
      List.of("rn", "tn", "tn3", "tn4", "tn5", "tn6", "tn7", "dn");
  private static final String PREFIX = "genesis.olympia";

  public static OlympiaGenesisConfig fromRuntimeProperties(RuntimeProperties properties) {
    final URL nodeCoreApiUrl;

    try {
      nodeCoreApiUrl = new URL(properties.get(String.format("%s.node_core_api_url", PREFIX)));
    } catch (MalformedURLException e) {
      throw new RuntimeException(
          """
              Olympia genesis was configured, but the provided genesis.olympia.node_core_api_url value \
              is invalid (expected a valid URL)""",
          e);
    }

    final ECDSASecp256k1PublicKey nodePublicKey;
    try {
      final var nodeBech32Address = properties.get(String.format("%s.node_bech32_address", PREFIX));
      final var decodedBech32 = OlympiaBech32.decode(nodeBech32Address);
      if (!VALID_NODE_HRPS.contains(decodedBech32.hrp)) {
        throw new RuntimeException(
            String.format(
                "The property genesis.olympia.node_bech32_address is not a valid Olympia node"
                    + " address. Expected one of HRPs: %s, but got %s",
                VALID_NODE_HRPS, decodedBech32.hrp));
      }
      nodePublicKey = ECDSASecp256k1PublicKey.fromBytes(decodedBech32.data);
    } catch (Exception e) {
      throw new RuntimeException(
          """
              Olympia genesis was configured, but the provided genesis.olympia.node_bech32_address value \
              is invalid (expected a Bech32-encoded Olympia node address)""",
          e);
    }

    final var maybeAuthUser =
        Optional.ofNullable(properties.get(String.format("%s.node_core_api_auth_user", PREFIX)));
    final Optional<String> maybeBasicAuthCredentialsBase64;
    if (maybeAuthUser.isPresent()) {
      final var authUser = maybeAuthUser.get();
      final var authPassword =
          Optional.ofNullable(
                  properties.get(String.format("%s.node_core_api_auth_password", PREFIX)))
              .orElseThrow(
                  () ->
                      new RuntimeException(
                          "Olympia genesis auth user was specified, but the password is missing."
                              + " Make sure both genesis.olympia.node_core_api_auth_user and"
                              + " genesis.olympia.node_core_api_auth_password are set correctly."));
      maybeBasicAuthCredentialsBase64 =
          Optional.of(
              Base64.toBase64String(
                  String.format("%s:%s", authUser, authPassword).getBytes(StandardCharsets.UTF_8)));
    } else {
      maybeBasicAuthCredentialsBase64 = Optional.empty();
    }

    return new OlympiaGenesisConfig(nodeCoreApiUrl, maybeBasicAuthCredentialsBase64, nodePublicKey);
  }
}
