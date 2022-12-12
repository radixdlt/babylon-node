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

package com.radixdlt.monitoring;

import com.google.common.base.Joiner;
import io.prometheus.client.*;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.stream.Stream;
import javax.annotation.Nullable;

/** An initializer for {@link Metrics}. */
public class MetricsInitializer {

  /** A way of joining name parts in the final metric. */
  private static final Joiner NAME_JOINER = Joiner.on('_').skipNulls();

  /** A Prometheus registry. */
  private final CollectorRegistry registry;

  /** An ad-hoc constructor (with an internal Prometheus registry); to be used mostly for test. */
  public MetricsInitializer() {
    this(new CollectorRegistry());
  }

  /**
   * A direct constructor.
   *
   * @param registry A Prometheus registry.
   */
  public MetricsInitializer(CollectorRegistry registry) {
    this.registry = registry;
  }

  /**
   * Instantiates a complete hierarchy of a {@link Metrics} record and registers its collectors with
   * Prometheus.
   *
   * @return Initialized instance.
   */
  public Metrics initialize() {
    return createCollectorHierarchy("node", Metrics.class);
  }

  /**
   * Instantiates a complete hierarchy of the given record class and registers its collectors with
   * Prometheus.
   *
   * @param namePrefix A prefix to apply for metric names down the hierarchy tree.
   * @param recordClass A class to instantiate.
   * @return Initialized instance.
   * @param <R> A record type.
   */
  @SuppressWarnings("unchecked")
  private <R extends Record> R createCollectorHierarchy(
      @Nullable String namePrefix, Class<R> recordClass) {
    Constructor<?> constructor = recordClass.getConstructors()[0];
    Object[] rowValues =
        Stream.of(recordClass.getRecordComponents())
            .map(
                component ->
                    createComponentValue(
                        NAME_JOINER.join(namePrefix, NameRenderer.render(component.getName())),
                        component.getGenericType()))
            .toArray();
    try {
      return (R) constructor.newInstance(rowValues);
    } catch (InstantiationException | IllegalAccessException | InvocationTargetException e) {
      throw new IllegalStateException(
          "cannot instantiate %s(%s)".formatted(recordClass, Joiner.on(", ").join(rowValues)), e);
    }
  }

  /**
   * Instantiates a specific component of a record, which may either be:
   *
   * <ul>
   *   <li>a sub-record (i.e. recursing into {@link #createCollectorHierarchy(String, Class)});
   *   <li>or a leaf collector, to be registered with Prometheus under the given name.
   * </ul>
   *
   * @param name A name.
   * @param componentType A component's type.
   * @return Instantiated component.
   */
  @SuppressWarnings("unchecked")
  private Object createComponentValue(String name, Type componentType) {
    if (componentType instanceof Class<?> rowClass && rowClass.isRecord()) {
      return createCollectorHierarchy(name, (Class<? extends Record>) rowClass);
    }
    Collector collector = instantiateCollector(name, componentType);
    registry.register(collector);
    return collector;
  }

  /**
   * Resolves a required collector from the given type and instantiates it with the given name.
   * Supports a subset of standard Prometheus collectors and our type-safe label-support wrappers
   * (see {@link Metrics}).
   *
   * @param name A name.
   * @param type A type.
   * @return Collector.
   */
  @SuppressWarnings("unchecked")
  private Collector instantiateCollector(String name, Type type) {
    if (type == Counter.class) {
      return Counter.build(name, name).create();
    }
    if (type == Gauge.class) {
      return Gauge.build(name, name).create();
    }
    if (type == Summary.class) {
      return Summary.build(name, name).create();
    }
    if (type instanceof ParameterizedType generic) {
      Class<? extends Record> labelClass =
          (Class<? extends Record>) generic.getActualTypeArguments()[0];
      if (generic.getRawType() == LabelledCounter.class) {
        return new LabelledCounter<>(name, labelClass);
      }
      if (generic.getRawType() == LabelledGauge.class) {
        return new LabelledGauge<>(name, labelClass);
      }
    }
    throw new IllegalArgumentException(
        "unknown collector type %s used for metric %s".formatted(type.getTypeName(), name));
  }
}
