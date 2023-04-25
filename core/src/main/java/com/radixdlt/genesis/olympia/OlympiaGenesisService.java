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

import static java.util.concurrent.Executors.newSingleThreadScheduledExecutor;

import com.google.common.hash.HashCode;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.Inject;
import com.radixdlt.crypto.ECDSASecp256k1Signature;
import com.radixdlt.crypto.HashUtils;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisData2;
import com.radixdlt.genesis.olympia.OlympiaEndStateApiClient.OlympiaEndStateResponse;
import com.radixdlt.genesis.olympia.state.OlympiaStateIRDeserializer;
import com.radixdlt.genesis.olympia.state.OlympiaStateIRSerializationException;
import com.radixdlt.networks.Network;
import com.radixdlt.utils.Bytes;
import com.radixdlt.utils.ThreadFactories;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.xerial.snappy.Snappy;

@SuppressWarnings({"UnstableApiUsage", "OptionalUsedAsFieldOrParameterType"})
public final class OlympiaGenesisService {
  private static final Logger log = LogManager.getLogger();

  // A placeholder hash included in the "not ready" response
  // used to be able to verify that the configured Olympia node public key
  // is correct, even before the network has shutdown. Used as an early
  // feedback mechanism in case of a misconfiguration. Must be the same
  // on Olympia and Babylon nodes.
  private static final HashCode PLACEHOLDER_HASH_FOR_NOT_READY_RESPONSE =
      HashUtils.sha256Twice(new byte[] {1});

  // Polling interval when connection to Olympia node succeeds
  private static final long POLL_INTERVAL_AFTER_NOT_READY_MS = 1000;

  // Polling interval when there was an error
  private static final long POLL_INTERVAL_AFTER_ERROR_MS = 10000;

  private final Network network;
  private final OlympiaGenesisConfig olympiaGenesisConfig;
  private final OlympiaEndStateApiClient olympiaEndStateApiClient;
  private final RateLimiter notReadyLogRateLimiter = RateLimiter.create(0.03);

  private Optional<ScheduledExecutorService> executor = Optional.empty();

  @Inject
  public OlympiaGenesisService(
      Network network,
      OlympiaGenesisConfig olympiaGenesisConfig,
      OlympiaEndStateApiClient olympiaEndStateApiClient) {
    this.network = network;
    this.olympiaGenesisConfig = olympiaGenesisConfig;
    this.olympiaEndStateApiClient = olympiaEndStateApiClient;
  }

  public CompletableFuture<GenesisData> start() {
    if (this.executor.isPresent()) {
      throw new IllegalStateException("OlympiaGenesisService already running");
    }

    this.executor =
        Optional.of(
            newSingleThreadScheduledExecutor(ThreadFactories.threads("OlympiaGenesisService")));

    final var completableFuture = new CompletableFuture<GenesisData>();
    // TODO: Fixme :)
//    this.executor.orElseThrow().execute(() -> poll(completableFuture));
    return completableFuture;
  }

  private void poll(CompletableFuture<GenesisData2> completableFuture) {
    final OlympiaEndStateResponse response;
    try {
      response = olympiaEndStateApiClient.getOlympiaEndState();
    } catch (Exception ex /* just catch anything */) {
      log.warn(
          """
              An error occurred while querying the Olympia node for the genesis state. \
              Retrying in {} ms... ({})""",
          POLL_INTERVAL_AFTER_ERROR_MS,
          ex.getMessage());
      this.executor
          .orElseThrow()
          .schedule(
              () -> poll(completableFuture), POLL_INTERVAL_AFTER_ERROR_MS, TimeUnit.MILLISECONDS);
      return;
    }

    switch (response) {
      case OlympiaEndStateResponse.Ready readyResponse -> {
        final var contentBytes = Bytes.fromBase64String(readyResponse.base64Contents());
        final var contentHash = HashUtils.sha256Twice(contentBytes);
        final var receivedHashBytes = HashCode.fromBytes(Bytes.fromHexString(readyResponse.hash()));

        if (!contentHash.equals(receivedHashBytes)) {
          completableFuture.completeExceptionally(hashMismatchErr("content"));
          return;
        }

        final var signature = ECDSASecp256k1Signature.decodeFromHexDer(readyResponse.signature());
        if (!this.olympiaGenesisConfig.nodePublicKey().verify(contentHash, signature)) {
          completableFuture.completeExceptionally(signatureErr());
          return;
        }

        final byte[] uncompressedBytes;
        try {
          uncompressedBytes = Snappy.uncompress(contentBytes);
        } catch (IOException e) {
          completableFuture.completeExceptionally(
              new RuntimeException(
                  """
                      Successfully connected to the Olympia node, but the received genesis data \
                      couldn't be uncompressed.""",
                  e));
          return;
        }

        try (final var bais = new ByteArrayInputStream(uncompressedBytes)) {
          final var parsedEndState = new OlympiaStateIRDeserializer().deserialize(bais);
          OlympiaStateToBabylonGenesisMapper.toGenesisData(parsedEndState)
              .fold(
                  completableFuture::complete,
                  error -> completableFuture.completeExceptionally(new RuntimeException(error)));
        } catch (OlympiaStateIRSerializationException | IOException ex) {
          completableFuture.completeExceptionally(
              new RuntimeException("Failed to deserialize the Olympia end state", ex));
        }
      }
      case OlympiaEndStateResponse.NotReady notReadyResponse -> {
        final var receivedPlaceholderHash =
            HashCode.fromBytes(Bytes.fromHexString(notReadyResponse.placeholderHash()));

        if (!receivedPlaceholderHash.equals(PLACEHOLDER_HASH_FOR_NOT_READY_RESPONSE)) {
          completableFuture.completeExceptionally(hashMismatchErr("placeholder"));
          return;
        }

        final var signature =
            ECDSASecp256k1Signature.decodeFromHexDer(notReadyResponse.signature());
        if (!this.olympiaGenesisConfig
            .nodePublicKey()
            .verify(PLACEHOLDER_HASH_FOR_NOT_READY_RESPONSE, signature)) {
          completableFuture.completeExceptionally(signatureErr());
          return;
        }

        // All good, the node has been verified and the configuration is correct, continue polling

        if (notReadyLogRateLimiter.tryAcquire()) {
          log.info(
              """
                  Successfully connected to the Olympia node ({}), \
                  but the end state hasn't yet been generated (will keep polling)...""",
              network.getLogicalName());
        }

        this.executor
            .orElseThrow()
            .schedule(
                () -> poll(completableFuture),
                POLL_INTERVAL_AFTER_NOT_READY_MS,
                TimeUnit.MILLISECONDS);
      }
    }
  }

  private RuntimeException signatureErr() {
    return new RuntimeException(
        String.format(
            """
            Successfully connected to the Olympia node, but the signature received along with the \
            response doesn't match a configured value. Double check that the \
            genesis.olympia.node_public_key configuration matches the public key of Olympia \
            node running at %s""",
            olympiaGenesisConfig.nodeCoreApiUrl()));
  }

  private RuntimeException hashMismatchErr(String hashType) {
    return new RuntimeException(
        String.format(
            """
            Successfully connected to the Olympia node, but the %s hash \
            received along with the response doesn't match expected value.""",
            hashType));
  }

  public void shutdown() {
    this.executor.ifPresent(ScheduledExecutorService::shutdown);
  }
}
