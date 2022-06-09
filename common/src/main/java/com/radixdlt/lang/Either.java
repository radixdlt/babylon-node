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

package com.radixdlt.lang;

import com.radixdlt.lang.Functions.FN1;
import java.util.function.Consumer;
import java.util.function.Function;

/** The type which can hold one of two values of different types. */
// TODO: extend API
public sealed interface Either<L, R> {
  /**
   * Handle both possible states (left/right) and produce single value from it. Depending on which
   * (left or right) value is stored in particular instance, respective mapping function is invoked.
   *
   * @param leftMapper function to transform left value into output value
   * @param rightMapper function to transform right value into output value
   * @return result of application of one of the mappers.
   */
  <T> T fold(FN1<? extends T, ? super L> leftMapper, FN1<? extends T, ? super R> rightMapper);

  /**
   * Invoke side-effect-producing consumer to the Either container. Depending on which (left or
   * right) value is stored in particular instance, respective consumer is invoked.
   *
   * @param leftConsumer function to process left value
   * @param rightConsumer function to process right value
   * @return result of application of one of the mappers.
   */
  default void apply(Consumer<? super L> leftConsumer, Consumer<? super R> rightConsumer) {
    fold(
        left -> {
          leftConsumer.accept(left);
          return null;
        },
        right -> {
          rightConsumer.accept(right);
          return null;
        });
  }

  default boolean isLeft() {
    return fold(
        left -> true,
        right -> false);
  }

  default boolean isRight() {
    return fold(
        left -> false,
        right -> true);
  }

  /**
   * This method allows "unwrapping" the left value stored inside the Either instance. If the value
   * is a right then an {@link IllegalStateException} is thrown.
   *
   * @return value stored inside present instance.
   */
  default L unwrapLeft() {
    return unwrapLeft(
        r ->
            new IllegalStateException(
                "Unwrap error (not a left) - the Either was a right, containing " + r.toString()));
  }

  /**
   * This method allows "unwrapping" the left value stored inside the Either instance. If the value
   * is a right then an exception is created from the right value and thrown.
   *
   * @param mapToException a map from the cause to a RuntimeException
   * @return value stored inside present instance.
   */
  default L unwrapLeft(Function<? super R, RuntimeException> mapToException) {
    return fold(
        Functions::id,
        r -> {
          throw mapToException.apply(r);
        });
  }

  /**
   * This method allows "unwrapping" the right value stored inside the Either instance. If the value
   * is a left then an {@link IllegalStateException} is thrown.
   *
   * @return value stored inside present instance.
   */
  default R unwrapRight() {
    return unwrapRight(
        l ->
            new IllegalStateException(
                "Unwrap error (not a right) - the Either was a left, containing " + l.toString()));
  }

  /**
   * This method allows "unwrapping" the right value stored inside the Either instance. If the value
   * is a left then an exception is created from the left value and thrown.
   *
   * @param mapToException a map from the cause to a RuntimeException
   * @return value stored inside present instance.
   */
  default R unwrapRight(Function<? super L, RuntimeException> mapToException) {
    return fold(
        l -> {
          throw mapToException.apply(l);
        },
        Functions::id);
  }

  record left<L, R>(L value) implements Either<L, R> {
    @Override
    public <T> T fold(
        FN1<? extends T, ? super L> leftMapper, FN1<? extends T, ? super R> rightMapper) {
      return leftMapper.apply(value());
    }
  }

  /**
   * Create instance which contains left value.
   *
   * @return created instance.
   */
  static <L, R> Either<L, R> left(L left) {
    return new left<>(left);
  }

  record right<L, R>(R value) implements Either<L, R> {
    @Override
    public <T> T fold(
        FN1<? extends T, ? super L> leftMapper, FN1<? extends T, ? super R> rightMapper) {
      return rightMapper.apply(value());
    }
  }

  /**
   * Create instance which contains right value.
   *
   * @return created instance.
   */
  static <L, R> Either<L, R> right(R right) {
    return new right<>(right);
  }
}
