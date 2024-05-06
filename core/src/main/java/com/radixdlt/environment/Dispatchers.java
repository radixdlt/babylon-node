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

package com.radixdlt.environment;

import com.google.inject.Inject;
import com.google.inject.Provider;
import com.google.inject.TypeLiteral;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.event.CoreEvent;
import com.radixdlt.consensus.event.LocalEvent;
import com.radixdlt.consensus.event.RemoteEvent;
import com.radixdlt.monitoring.Metrics;
import com.radixdlt.p2p.NodeId;
import java.util.Set;
import java.util.stream.Collectors;

/** Helper class to set up environment with dispatched events */
// TODO: get rid of field injection https://radixdlt.atlassian.net/browse/NT-3
public final class Dispatchers {
  private Dispatchers() {
    throw new IllegalStateException("Cannot instantiate.");
  }

  private static class DispatcherProvider<T extends LocalEvent>
      implements Provider<EventDispatcher<T>> {
    @Inject private Provider<Environment> environmentProvider;

    @Inject private Metrics metrics;

    @Inject private Set<EventProcessorOnDispatch<?>> onDispatchProcessors;

    private final Class<T> c;
    private final MetricUpdater<T> metricUpdater;

    DispatcherProvider(Class<T> c, MetricUpdater<T> metricUpdater) {
      this.c = c;
      this.metricUpdater = metricUpdater;
    }

    @Override
    public EventDispatcher<T> get() {
      final EventDispatcher<T> dispatcher = environmentProvider.get().getDispatcher(c);
      final Set<EventProcessor<T>> processors =
          onDispatchProcessors.stream()
              .flatMap(p -> p.getProcessor(c).stream())
              .collect(Collectors.toSet());
      return e -> {
        dispatcher.dispatch(e);
        processors.forEach(p -> p.process(e));
        metricUpdater.update(metrics, e);
      };
    }
  }

  private static final class ScheduledDispatcherProvider<T extends LocalEvent>
      implements Provider<ScheduledEventDispatcher<T>> {
    // TODO: get rid of field injection https://radixdlt.atlassian.net/browse/NT-3
    @Inject private Provider<Environment> environmentProvider;
    private final Class<T> eventClass;
    private final TypeLiteral<T> eventLiteral;

    ScheduledDispatcherProvider(Class<T> eventClass) {
      this.eventClass = eventClass;
      this.eventLiteral = null;
    }

    ScheduledDispatcherProvider(TypeLiteral<T> eventLiteral) {
      this.eventClass = null;
      this.eventLiteral = eventLiteral;
    }

    @Override
    public ScheduledEventDispatcher<T> get() {
      Environment e = environmentProvider.get();
      if (eventClass != null) {
        return e.getScheduledDispatcher(eventClass);
      } else {
        return e.getScheduledDispatcher(eventLiteral);
      }
    }
  }

  private static final class RemoteDispatcherProvider<T extends RemoteEvent>
      implements Provider<RemoteEventDispatcher<NodeId, T>> {
    @Inject private Provider<Environment> environmentProvider;

    @Inject private Metrics metrics;

    @Inject @Self private NodeId self;

    @Inject private Set<EventProcessorOnDispatch<?>> onDispatchProcessors;

    private final Class<T> messageType;

    RemoteDispatcherProvider(Class<T> messageType) {
      this.messageType = messageType;
    }

    @Override
    public RemoteEventDispatcher<NodeId, T> get() {
      var remoteDispatcher = environmentProvider.get().getRemoteDispatcher(messageType);
      var localDispatcher = environmentProvider.get().getDispatcher(messageType);
      final Set<EventProcessor<T>> onDispatch =
          onDispatchProcessors.stream()
              .flatMap(p -> p.getProcessor(messageType).stream())
              .collect(Collectors.toSet());
      return (node, e) -> {
        if (node.equals(self)) {
          localDispatcher.dispatch(e);
        } else {
          remoteDispatcher.dispatch(node, e);
        }
        onDispatch.forEach(p -> p.process(e));
      };
    }
  }

  public static <T extends LocalEvent> Provider<EventDispatcher<T>> dispatcherProvider(Class<T> c) {
    return new DispatcherProvider<>(c, (counter, event) -> {});
  }

  public static <T extends LocalEvent> Provider<EventDispatcher<T>> dispatcherProvider(
      Class<T> c, MetricUpdater<T> metricUpdater) {
    return new DispatcherProvider<>(c, metricUpdater);
  }

  public static <T extends LocalEvent>
      Provider<ScheduledEventDispatcher<T>> scheduledDispatcherProvider(Class<T> c) {
    return new ScheduledDispatcherProvider<>(c);
  }

  public static <T extends LocalEvent>
      Provider<ScheduledEventDispatcher<T>> scheduledDispatcherProvider(TypeLiteral<T> t) {
    return new ScheduledDispatcherProvider<>(t);
  }

  public static <T extends RemoteEvent>
      Provider<RemoteEventDispatcher<NodeId, T>> remoteDispatcherProvider(Class<T> messageType) {
    return new RemoteDispatcherProvider<>(messageType);
  }

  /**
   * A part of {@link #dispatcherProvider(Class, MetricUpdater)}'s logic to be used for metrics'
   * update.
   *
   * @param <T> Event type.
   */
  @FunctionalInterface
  public interface MetricUpdater<T extends CoreEvent> {

    /**
     * Updates the metrics according to the event.
     *
     * @param metrics Metrics.
     * @param event Event.
     */
    void update(Metrics metrics, T event);
  }
}
