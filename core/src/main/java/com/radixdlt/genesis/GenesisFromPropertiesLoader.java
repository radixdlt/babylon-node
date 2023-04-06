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

package com.radixdlt.genesis;

import com.google.common.base.Splitter;
import com.google.common.base.Strings;
import com.google.common.collect.Streams;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.identifiers.Address;
import com.radixdlt.lang.Tuple;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.utils.IOUtils;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.io.FileInputStream;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.json.JSONObject;

/**
 * Responsible for loading the genesis data from a configured file, or directly from a property
 * value
 */
public record GenesisFromPropertiesLoader(RuntimeProperties properties, Network network) {
  private static final Logger log = LogManager.getLogger();

  // Genesis parameters for XRD allocation for testnets
  private static final Set<Network> GENESIS_NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT =
      Set.of(
          Network.GILGANET,
          Network.ENKINET,
          Network.HAMMUNET,
          Network.MARDUNET,
          Network.NERGALNET,
          Network.NEBUNET,
          Network.KISHARNET,
          Network.ANSHARNET);
  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE =
      Decimal.of(700_000_000_000L); // 70% XRD_MAX_SUPPLY
  private static final Decimal GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1_000_000_000L); // 0.1% XRD_MAX_SUPPLY
  private static final ECDSASecp256k1PublicKey GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY =
      ECDSASecp256k1PublicKey.tryFromHex(
              "026f08db98ef1d0231eb15580da9123db8e25aa1747c8c32e5fd2ec47b8db73d5c")
          .unwrap();
  private static final Decimal GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR =
      Decimal.of(1); // Allow it to be easily changed in eg tests

  public Optional<GenesisData> loadGenesisDataFromProperties() {
    final var genesisFileProp = properties.get("network.genesis_file");
    if (genesisFileProp != null && !genesisFileProp.isBlank()) {
      log.info("Loading genesis from file: {}", genesisFileProp);
      return Optional.of(createGenesisDataFromHex(loadRawGenesisFromFile(genesisFileProp)));
    } else if (!Strings.isNullOrEmpty(properties.get("network.genesis_txn"))) {
      log.info("Loading genesis from genesis_txn property");
      return Optional.of(createGenesisDataFromHex(properties.get("network.genesis_txn")));
    } else {
      return Optional.empty();
    }
  }

  private GenesisData createGenesisDataFromHex(String input) {
    final var initialVset =
        Streams.stream(
                Splitter.fixedLength(ECDSASecp256k1PublicKey.COMPRESSED_BYTES * 2).split(input))
            .map(
                pubKeyBytes -> {
                  log.info("Initial vset validator: {}", pubKeyBytes);
                  try {
                    return ECDSASecp256k1PublicKey.fromHex(pubKeyBytes);
                  } catch (PublicKeyException e) {
                    throw new RuntimeException(e);
                  }
                })
            .toList();
    var validatorSet =
        new HashMap<ECDSASecp256k1PublicKey, Tuple.Tuple2<Decimal, ComponentAddress>>();
    final var usePowerfulStakingAccount =
        GENESIS_NETWORKS_TO_USE_POWERFUL_STAKING_ACCOUNT.contains(network);

    final var stakingAccount =
        usePowerfulStakingAccount
            ? Address.virtualAccountAddress(GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY)
            : Address.virtualAccountAddress(PrivateKeys.ofNumeric(1).getPublicKey());
    final var stakeAmount =
        usePowerfulStakingAccount
            ? GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR
            : GENESIS_NO_STAKING_ACCOUNT_INITIAL_XRD_STAKE_PER_VALIDATOR;

    initialVset.forEach(k -> validatorSet.put(k, Tuple.tuple(stakeAmount, stakingAccount)));

    final Map<ECDSASecp256k1PublicKey, Decimal> xrdAllocations =
        usePowerfulStakingAccount
            ? Map.of(
                GENESIS_POWERFUL_STAKING_ACCOUNT_PUBLIC_KEY,
                GENESIS_POWERFUL_STAKING_ACCOUNT_INITIAL_XRD_BALANCE)
            : Map.of();

    log.info("Genesis XRD allocations: {}", xrdAllocations.isEmpty() ? "(empty)" : "");
    xrdAllocations.forEach((k, v) -> log.info("{}: {}", k, v));

    return new GenesisData(validatorSet, xrdAllocations);
  }

  private String loadRawGenesisFromFile(String genesisFile) {
    try (var genesisJsonString = new FileInputStream(genesisFile)) {
      var genesisJson = new JSONObject(IOUtils.toString(genesisJsonString));
      return genesisJson.getString("genesis");
    } catch (IOException e) {
      throw new IllegalStateException(e);
    }
  }
}
