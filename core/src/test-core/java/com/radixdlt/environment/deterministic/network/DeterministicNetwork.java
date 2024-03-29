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

package com.radixdlt.environment.deterministic.network;

import com.radixdlt.harness.deterministic.invariants.MessageMonitor;
import io.reactivex.rxjava3.schedulers.Timed;
import java.util.*;
import java.util.concurrent.TimeUnit;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * A BFT network supporting the EventCoordinatorNetworkSender interface which stores each message in
 * a queue until they are synchronously popped.
 */
public final class DeterministicNetwork {
  private static final Logger log = LogManager.getLogger();
  private static final long DEFAULT_LATENCY = 50L; // virtual milliseconds
  private static final long DEFAULT_LOCAL_LATENCY = 5L; // virtual milliseconds
  private final MessageQueue messageQueue = new MessageQueue();
  private final MessageSelector messageSelector;
  private final MessageMutator messageMutator;
  private final MessageMonitor messageMonitor;

  private long currentTime = 0L;
  private long lastTimeLogged = 0L;

  /**
   * Create a BFT test network for deterministic tests.
   *
   * @param messageSelector A {@link MessageSelector} for choosing messages to process next
   * @param messageMutator A {@link MessageMutator} for mutating and queueing messages
   */
  public DeterministicNetwork(MessageSelector messageSelector, MessageMutator messageMutator) {
    this(messageSelector, messageMutator, (m, t) -> {});
  }

  public DeterministicNetwork(
      MessageSelector messageSelector,
      MessageMutator messageMutator,
      MessageMonitor messageMonitor) {
    this.messageSelector = Objects.requireNonNull(messageSelector);
    this.messageMutator = Objects.requireNonNull(messageMutator);
    this.messageMonitor = Objects.requireNonNull(messageMonitor);
  }

  // TODO: use better method than Timed to store time
  public Timed<ControlledMessage> nextMessage() {
    List<ControlledMessage> controlledMessages = this.messageQueue.lowestTimeMessages();
    if (controlledMessages == null || controlledMessages.isEmpty()) {
      throw new IllegalStateException("No messages available (Lost Responsiveness)");
    }
    ControlledMessage controlledMessage = this.messageSelector.select(controlledMessages);

    this.messageQueue.remove(controlledMessage);
    this.currentTime = Math.max(this.currentTime, controlledMessage.arrivalTime());

    return new Timed<>(controlledMessage, this.currentTime, TimeUnit.MILLISECONDS);
  }

  public Timed<ControlledMessage> nextMessage(Predicate<ControlledMessage> predicate) {
    List<ControlledMessage> allMessages = this.messageQueue.allMessages();
    ControlledMessage controlledMessage =
        allMessages.stream()
            .filter(predicate)
            .findFirst()
            .orElseThrow(
                () -> {
                  var msgCount =
                      allMessages.stream()
                          .collect(
                              Collectors.groupingBy(
                                  m -> m.message().getClass(), Collectors.counting()));
                  return new IllegalStateException(
                      String.format("Could not find message. Messages present: %s", msgCount));
                });
    this.messageQueue.remove(controlledMessage);
    this.currentTime = Math.max(this.currentTime, controlledMessage.arrivalTime());
    if ((this.currentTime / 1000) > (this.lastTimeLogged / 1000)) {
      this.lastTimeLogged = this.currentTime;
      log.info("Simulated Time: {}", this.currentTime);
    }

    return new Timed<>(controlledMessage, this.currentTime, TimeUnit.MILLISECONDS);
  }

  public Optional<Timed<ControlledMessage>> nextMessageIfExists(
      Predicate<ControlledMessage> predicate) {
    List<ControlledMessage> allMessages = this.messageQueue.allMessages();
    return allMessages.stream()
        .filter(predicate)
        .findFirst()
        .map(
            (message) -> {
              this.messageQueue.remove(message);
              this.currentTime = Math.max(this.currentTime, message.arrivalTime());

              return new Timed<>(message, this.currentTime, TimeUnit.MILLISECONDS);
            });
  }

  public List<ControlledMessage> allMessages() {
    return this.messageQueue.allMessages();
  }

  public void dropMessages(Predicate<ControlledMessage> controlledMessagePredicate) {
    this.messageQueue.remove(controlledMessagePredicate);
  }

  public void dropAllMessages() {
    this.messageQueue.removeAll();
  }

  public long currentTime() {
    return this.currentTime;
  }

  long delayForChannel(ChannelId channelId) {
    if (channelId.isLocal()) {
      return DEFAULT_LOCAL_LATENCY;
    }
    return DEFAULT_LATENCY;
  }

  void handleMessage(ControlledMessage controlledMessage) {
    log.debug("{}: Dispatch message {}", this.currentTime, controlledMessage);
    messageMonitor.next(controlledMessage, this.currentTime);

    if (!this.messageMutator.mutate(controlledMessage, this.messageQueue)) {
      // If nothing processes this message, we just add it to the queue
      this.messageQueue.add(controlledMessage);
    } else {
      log.debug("Dropping message {}", controlledMessage);
    }
  }
}
