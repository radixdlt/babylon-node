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

package com.radixdlt.messaging.core;

import static com.radixdlt.messaging.core.MessagingErrors.*;

import com.radixdlt.addressing.Addressing;
import com.radixdlt.lang.Cause;
import com.radixdlt.lang.Result;
import com.radixdlt.lang.Unit;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.monitoring.Metrics.Messages.AbortedOutboundMessage;
import com.radixdlt.monitoring.Metrics.Messages.OutboundMessageAbortedReason;
import com.radixdlt.p2p.NodeId;
import com.radixdlt.p2p.PeerManager;
import com.radixdlt.p2p.transport.PeerChannel;
import com.radixdlt.serialization.DsonOutput.Output;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.utils.*;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/*
 * This could be moved into MessageCentralImpl at some stage, but has been
 * separated out so that we can check if all the functionality here is
 * required, and remove the stuff we don't want to keep.
 */
class MessageDispatcher {
  private static final Logger log = LogManager.getLogger();

  private final long messageTtlMs;
  private final Metrics metrics;
  private final Serialization serialization;
  private final TimeSupplier timeSource;
  private final PeerManager peerManager;
  private final Addressing addressing;

  // A rate of 0.05 == 1 message every 20s
  private final RejectionCountingRateLimiter ttlExpiredLogRateLimiter =
      new RejectionCountingRateLimiter(0.05);
  private final LRUCache<NodeId, RejectionCountingRateLimiter> sendErrorLogRateLimitersByReceiver =
      new LRUCache<>(150);

  private final int maxMessageSize;

  MessageDispatcher(
      Metrics metrics,
      MessageCentralConfiguration config,
      Serialization serialization,
      TimeSupplier timeSource,
      PeerManager peerManager,
      Addressing addressing,
      int maxMessageSize) {
    this.messageTtlMs = Objects.requireNonNull(config).messagingTimeToLive(30_000L);
    this.metrics = Objects.requireNonNull(metrics);
    this.serialization = Objects.requireNonNull(serialization);
    this.timeSource = Objects.requireNonNull(timeSource);
    this.peerManager = Objects.requireNonNull(peerManager);
    this.addressing = Objects.requireNonNull(addressing);
    this.maxMessageSize = maxMessageSize;
  }

  CompletableFuture<Result<Unit, Cause>> send(final OutboundMessageEvent outboundMessage) {
    final var message = outboundMessage.message();
    final var receiver = outboundMessage.receiver();

    if (timeSource.currentTime() - message.getTimestamp() > messageTtlMs) {
      this.metrics
          .messages()
          .outbound()
          .aborted()
          .label(new AbortedOutboundMessage(OutboundMessageAbortedReason.MESSAGE_EXPIRED))
          .inc();
      logTtlExpired(message, receiver);
      return CompletableFuture.completedFuture(MESSAGE_EXPIRED.result());
    }

    final var serializedMessageBytes = serialize(message);

    if (serializedMessageBytes.length > maxMessageSize) {
      this.metrics
          .messages()
          .outbound()
          .aborted()
          .label(new AbortedOutboundMessage(OutboundMessageAbortedReason.MESSAGE_TOO_LARGE))
          .inc();
      logSendError(
          message,
          receiver,
          String.format("message too large (length exceeds %s bytes)", maxMessageSize));
      return CompletableFuture.completedFuture(MESSAGE_TOO_LARGE.result());
    }

    return peerManager
        .findOrCreateChannel(outboundMessage.receiver())
        .thenApply(channel -> send(channel, serializedMessageBytes))
        .thenApply(this::updateStatistics)
        .thenApply(
            result -> {
              result.onError(err -> logSendError(message, receiver, err.toString()));
              return result;
            })
        .exceptionally(
            t -> {
              final var cause = t.getCause() != null ? t.getCause().getMessage() : t.getMessage();
              logSendError(message, receiver, cause);
              return IO_ERROR.result();
            });
  }

  private void logSendError(Message message, NodeId receiver, String cause) {
    final RejectionCountingRateLimiter rateLimiter;
    synchronized (sendErrorLogRateLimitersByReceiver) {
      if (sendErrorLogRateLimitersByReceiver.contains(receiver)) {
        rateLimiter = sendErrorLogRateLimitersByReceiver.get(receiver).orElseThrow();
      } else {
        // 1/60 permits a second == 1 message every minute
        rateLimiter = new RejectionCountingRateLimiter((double) 1 / 60);
        sendErrorLogRateLimitersByReceiver.put(receiver, rateLimiter);
      }
    }
    rateLimiter.tryAcquire(
        countSinceLastPermit -> {
          final var baseMsg =
              String.format(
                  "An outbound message of type %s couldn't be sent to %s because of: \"%s\".",
                  message.getClass().getSimpleName(),
                  addressing.encodeNodeAddress(receiver.getPublicKey()),
                  cause);
          if (countSinceLastPermit > 0) {
            log.warn(
                "{} {} more messages couldn't be send to this peer since "
                    + "the previous log message (likely for the same reason).",
                baseMsg,
                countSinceLastPermit);
          } else {
            log.warn(baseMsg);
          }
        });
  }

  private void logTtlExpired(Message message, NodeId receiver) {
    ttlExpiredLogRateLimiter.tryAcquire(
        countSinceLastPermit -> {
          final var baseMsg =
              String.format(
                  "TTL (of %s ms) has expired for an outbound message of type %s destined to %s and"
                      + " it will be dropped. This is likely caused by an overgrown message"
                      + " backlog, which might be caused by slow network speed and/or excessive"
                      + " processing load, in which case it's likely a transient issue.",
                  messageTtlMs,
                  message.getClass().getSimpleName(),
                  addressing.encodeNodeAddress(receiver.getPublicKey()));
          if (countSinceLastPermit > 0) {
            log.warn(
                "{} {} more messages were dropped due to TTL expiration "
                    + "since the previous log message (possibly targeted to different peers).",
                baseMsg,
                countSinceLastPermit);
          } else {
            log.warn(baseMsg);
          }
        });
  }

  private Result<Unit, Cause> send(PeerChannel channel, byte[] bytes) {
    this.metrics.networking().bytesSent().inc(bytes.length);
    return channel.send(bytes);
  }

  private Result<Unit, Cause> updateStatistics(Result<Unit, Cause> result) {
    this.metrics.messages().outbound().processed().inc();
    if (result.isSuccess()) {
      this.metrics.messages().outbound().sent().inc();
    }
    return result;
  }

  private byte[] serialize(Message out) {
    try {
      byte[] uncompressed = serialization.toDson(out, Output.WIRE);
      return Compress.compress(uncompressed);
    } catch (IOException e) {
      throw new UncheckedIOException("While serializing message", e);
    }
  }
}
