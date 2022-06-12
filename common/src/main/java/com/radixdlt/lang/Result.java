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

import static com.radixdlt.lang.Tuple.*;

import com.radixdlt.lang.Functions.*;
import com.radixdlt.lang.Result.Err;
import com.radixdlt.lang.Result.Ok;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.function.Supplier;

/**
 * Representation of the operation result. The result can be either success (Ok) or failure (Err).
 * In either case, it holds the value returned by the operation.
 *
 * @param <T> Type of value in case of success.
 * @param <E> Type of value in case of error.
 */
@SuppressWarnings("unused")
public sealed interface Result<T, E> permits Ok, Err {
  /**
   * Transform operation result value into value of other type and wrap new value into {@link
   * Result}. Transformation takes place if current instance (this) contains successful result,
   * otherwise current instance remains unchanged and transformation function is not invoked.
   *
   * @param mapper Function to transform successful value
   * @return transformed value (in case of success) or current instance (in case of failure)
   */
  default <R> Result<R, E> map(Func1<? super T, R> mapper) {
    return fold(t -> Result.ok(mapper.apply(t)), Result::err);
  }

  /**
   * Replace value stored in current instance with value of other type. Replacing takes place only
   * if current instance (this) contains successful result, otherwise current instance remains
   * unchanged.
   *
   * @param supplier Source of the replacement value.
   * @return transformed value (in case of success) or current instance (in case of failure)
   */
  default <R> Result<R, E> map(Supplier<R> supplier) {
    return fold(unused -> success(supplier.get()), Result::err);
  }

  /**
   * Transform operation result into another operation result. In case if current instance (this) is
   * an error, transformation function is not invoked and value remains the same.
   *
   * @param mapper Function to apply to result
   * @return transformed value (in case of success) or current instance (in case of failure)
   */
  default <R> Result<R, E> flatMap(Func1<? super T, Result<R, E>> mapper) {
    return fold(mapper, Result::err);
  }

  /**
   * Replace current instance with the instance returned by provided {@link Supplier}. The
   * replacement happens only if current instance contains successful result, otherwise current
   * instance remains unchanged.
   *
   * @param mapper Source of the replacement result.
   * @return replacement result (in case of success) or current instance (in case of failure)
   */
  default <R> Result<R, E> flatMap(Supplier<Result<R, E>> mapper) {
    return fold(unused -> mapper.get(), Result::err);
  }

  /**
   * Apply consumers to result value. Note that depending on the result (success or failure) only
   * one consumer will be applied at a time.
   *
   * @param successConsumer Consumer for success result
   * @param failureConsumer Consumer for failure result
   * @return current instance
   */
  default Result<T, E> apply(
      Consumer<? super T> successConsumer, Consumer<? super E> failureConsumer) {
    return fold(
        t -> {
          successConsumer.accept(t);
          return this;
        },
        t -> {
          failureConsumer.accept(t);
          return this;
        });
  }

  /**
   * Pass successful operation result value into provided consumer.
   *
   * @param consumer Consumer to pass value to
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onSuccess(Consumer<T> consumer) {
    fold(
        v -> {
          consumer.accept(v);
          return null;
        },
        Functions::toNull);
    return this;
  }

  /**
   * Pass successful operation result value into provided consumer. Alias of onSuccess.
   *
   * @param consumer Consumer to pass value to
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onOk(Consumer<T> consumer) {
    fold(
        v -> {
          consumer.accept(v);
          return null;
        },
        Functions::toNull);
    return this;
  }

  /**
   * Run provided action in case of success.
   *
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onSuccessDo(Runnable action) {
    fold(
        v -> {
          action.run();
          return null;
        },
        Functions::toNull);
    return this;
  }

  /**
   * Run provided action in case of success. Alias of onSuccessDo.
   *
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onOkDo(Runnable action) {
    fold(
        v -> {
          action.run();
          return null;
        },
        Functions::toNull);
    return this;
  }

  /**
   * Pass failure operation result value into provided consumer.
   *
   * @param consumer Consumer to pass value to
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onFailure(Consumer<? super E> consumer) {
    fold(
        Functions::toNull,
        v -> {
          consumer.accept(v);
          return null;
        });
    return this;
  }

  /**
   * Pass failure operation result value into provided consumer. Alias of onFailure.
   *
   * @param consumer Consumer to pass value to
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onError(Consumer<? super E> consumer) {
    fold(
        Functions::toNull,
        v -> {
          consumer.accept(v);
          return null;
        });
    return this;
  }

  /**
   * Run provided action in case of failure.
   *
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onFailureDo(Runnable action) {
    fold(
        Functions::toNull,
        v -> {
          action.run();
          return null;
        });
    return this;
  }

  /**
   * Run provided action in case of failure. Alias of onFailureDo.
   *
   * @return current instance for fluent call chaining
   */
  default Result<T, E> onErrorDo(Runnable action) {
    fold(
        Functions::toNull,
        v -> {
          action.run();
          return null;
        });
    return this;
  }

  /**
   * Convert instance into {@link Option} of the same value type. Successful instance is converted
   * into present {@link Option} and failure - into empty {@link Option}. Note that during such a
   * conversion error information is lost.
   *
   * @return {@link Option} instance which is present in case of success and missing in case of
   *     failure.
   */
  default Option<T> toOption() {
    return fold(Option::option, t1 -> Option.empty());
  }

  /**
   * Convert instance into {@link Option} of the same value type. Successful instance is converted
   * into empty {@link Option} and successful into empty {@link Option}.
   *
   * @return {@link Option} instance which is present in case of failure and empty in case of
   *     success.
   */
  default Option<E> toOptionErr() {
    return fold(t1 -> Option.empty(), Option::option);
  }

  /**
   * Convert instance into {@link Optional} of the same value type. Successful instance is converted
   * into present {@link Optional} and failure - into empty {@link Optional}. Note that during such
   * a conversion error information is lost.
   *
   * @return {@link Optional} instance which is present in case of success and missing in case of
   *     failure.
   */
  default Optional<T> toOptional() {
    return fold(Optional::of, t1 -> Optional.empty());
  }

  /**
   * Check if instance is success.
   *
   * @return {@code true} if instance is success and {@code false} otherwise
   */
  default boolean isSuccess() {
    return fold(Functions::toTrue, Functions::toFalse);
  }

  /**
   * Check if instance is success.
   * Alias of isSuccess.
   *
   * @return {@code true} if instance is success and {@code false} otherwise
   */
  default boolean isOk() {
    return fold(Functions::toTrue, Functions::toFalse);
  }

  /**
   * Check if instance is failure.
   *
   * @return {@code true} if instance is failure and {@code false} otherwise
   */
  default boolean isFailure() {
    return fold(Functions::toFalse, Functions::toTrue);
  }

  /**
   * Check if instance is failure.
   * Alias of isFailure.
   *
   * @return {@code true} if instance is failure and {@code false} otherwise
   */
  default boolean isErr() {
    return fold(Functions::toFalse, Functions::toTrue);
  }

  /**
   * Filter instance against provided predicate. If predicate returns {@code true} then instance
   * remains unchanged. If predicate returns {@code false}, then failure instance in created using
   * given {@link E}.
   *
   * @param predicate predicate to invoke
   * @param error failure to use in case if predicate returns {@code false}
   * @return current instance if predicate returns {@code true} or {@link Err} instance if predicate
   *     returns {@code false}
   */
  default Result<T, E> filter(Predicate<T> predicate, E error) {
    return fold(v -> predicate.test(v) ? this : failure(error), v -> this);
  }

  /**
   * Filter instance against provided predicate. If predicate returns {@code true} then instance
   * remains unchanged. If predicate returns {@code false}, then failure instance in created using
   * {@link E} created by provided function.
   *
   * @param predicate predicate to invoke
   * @param errorMapper function which transforms the tested value into instance of {@link E} if
   *     predicate returns {@code false}
   * @return current instance if predicate returns {@code true} or {@link Err} instance if predicate
   *     returns {@code false}
   */
  default Result<T, E> filter(Predicate<T> predicate, Func1<T, E> errorMapper) {
    return fold(v -> predicate.test(v) ? this : failure(errorMapper.apply(v)), v -> this);
  }

  /**
   * Return value store in the current instance (if this instance represents successful result) or
   * provided replacement value.
   *
   * @param replacement replacement value returned if current instance represents failure.
   * @return value stored in current instance (in case of success) or replacement value.
   */
  default T or(T replacement) {
    return fold(Functions::id, unused -> replacement);
  }

  /**
   * Return value store in the current instance (if this instance represents successful result) or
   * value returned by provided supplier.
   *
   * @param supplier source of replacement value returned if current instance represents failure.
   * @return value stored in current instance (in case of success) or replacement value.
   */
  default T or(Supplier<T> supplier) {
    return fold(Functions::id, unused -> supplier.get());
  }

  /**
   * Return current instance if this instance represents successful result or replacement instance
   * if current instance represents a failure.
   *
   * @param replacement replacement instance returned if current instance represents failure.
   * @return current instance (in case of success) or replacement instance.
   */
  default Result<T, E> orElse(Result<T, E> replacement) {
    return fold(unused -> this, unused -> replacement);
  }

  /**
   * Return current instance if this instance represents successful result or instance returned by
   * provided supplier if current instance represents a failure.
   *
   * @param supplier source of replacement instance returned if current instance represents failure.
   * @return current instance (in case of success) or replacement instance.
   */
  default Result<T, E> orElse(Supplier<Result<T, E>> supplier) {
    return fold(unused -> this, unused -> supplier.get());
  }

  /**
   * This method allows "unwrapping" the value stored inside the Result instance. If the value is
   * missing then an {@link UnwrapException} is thrown.
   *
   * @return value stored inside present instance.
   */
  default T unwrap() {
    return unwrap(UnwrapException::new);
  }

  /**
   * This method allows "unwrapping" the value stored inside the Result instance. If the value is
   * missing then an exception is created from the error and thrown.
   *
   * @param mapErrorToException a map from the error to a RuntimeException
   * @return value stored inside present instance.
   */
  default T unwrap(Function<? super E, RuntimeException> mapErrorToException) {
    return fold(
        Functions::id,
        v -> {
          throw mapErrorToException.apply(v);
        });
  }

  /**
   * This method allows "unwrapping" the error stored inside the Result instance. If the Result is
   * not an error then an {@link UnwrapException} is thrown.
   *
   * @return error stored inside present instance.
   */
  default E unwrapErr() {
    return unwrapErr(UnwrapException::new);
  }

  /**
   * This method allows "unwrapping" the error stored inside the Result instance. If the Result is
   * not an error then an exception is created from the value and thrown.
   *
   * @param mapValueToException a map from the value to a RuntimeException
   * @return error stored inside present instance.
   */
  default E unwrapErr(Function<? super T, RuntimeException> mapValueToException) {
    return fold(
        v -> {
          throw mapValueToException.apply(v);
        },
        Functions::id);
  }

  /**
   * Handle both possible states (success/failure) and produce single value from it.
   *
   * @param successMapper function to transform success into value
   * @param failureMapper function to transform failure into value
   * @return result of application of one of the mappers.
   */
  <R> R fold(
      Func1<? super T, ? extends R> successMapper, Func1<? super E, ? extends R> failureMapper);

  default Result<T, E> accept(Consumer<E> failureConsumer, Consumer<T> successConsumer) {
    return fold(
        success -> {
          successConsumer.accept(success);
          return this;
        },
        failure -> {
          failureConsumer.accept(failure);
          return this;
        });
  }

  /**
   * Create an instance of successful operation result.
   *
   * @param value Operation result
   * @return created instance
   */
  static <T, E> Result<T, E> success(T value) {
    return new Ok<>(value);
  }

  /**
   * Create an instance of successful operation result.
   *
   * @param value Operation result
   * @return created instance
   */
  static <T, E> Result<T, E> ok(T value) {
    return new Ok<>(value);
  }

  record Ok<T, E>(T value) implements Result<T, E> {
    @Override
    public <R> R fold(
        Func1<? super T, ? extends R> successMapper, Func1<? super E, ? extends R> failureMapper) {
      return successMapper.apply(value);
    }

    @Override
    public String toString() {
      return "Ok(" + value.toString() + ")";
    }
  }

  /**
   * Create an instance of failure result.
   *
   * @param error Operation error value
   * @return created instance
   */
  static <T, E> Result<T, E> failure(E error) {
    return new Err<>(error);
  }

  /**
   * Create an instance of failure result.
   *
   * @param error Operation error value
   * @return created instance
   */
  static <T, E> Result<T, E> err(E error) {
    return new Err<>(error);
  }

  record Err<T, E>(E error) implements Result<T, E> {
    @Override
    public <R> R fold(
        Func1<? super T, ? extends R> successMapper, Func1<? super E, ? extends R> failureMapper) {
      return failureMapper.apply(error);
    }

    @Override
    public String toString() {
      return "Err(" + error + ")";
    }
  }

  @SuppressWarnings("OptionalUsedAsFieldOrParameterType")
  static <T, E> Result<T, E> fromOptional(Optional<T> source, E error) {
    return source.map(Result::<T, E>ok).orElseGet(() -> Result.err(error));
  }

  @SuppressWarnings("OptionalUsedAsFieldOrParameterType")
  static <T, E> Result<T, E> fromOptional(Optional<T> source, Supplier<E> errorSupplier) {
    return source.map(Result::<T, E>ok).orElseGet(() -> Result.err(errorSupplier.get()));
  }

  /**
   * Wrap value returned by provided lambda into success {@link Result} if call succeeds or into
   * failure {@link Result} if call throws exception.
   *
   * @param supplier the call to wrap
   * @param exceptionMapper the function which will transform exception into instance of {@link E}
   * @return result of execution of the provided lambda wrapped into {@link Result}
   */
  static <R, E> Result<R, E> lift(
      ThrowingSupplier<R> supplier, Func1<? super Throwable, ? extends E> exceptionMapper) {
    try {
      return success(supplier.get());
    } catch (Throwable e) {
      return failure(exceptionMapper.apply(e));
    }
  }

  /**
   * Transform list of {@link Result} instances into {@link Result} with list of values. If there
   * are any failures, the first failure is returned.
   *
   * @param resultList input list
   * @return success instance if all {@link Result} instances in list are successes or failure
   *     instance with any instances in list is a failure
   */
  static <T, E> Result<List<T>, E> allOf(List<Result<T, E>> resultList) {
    var values = new ArrayList<T>(resultList.size());

    for (var result : resultList) {
      if (result.isFailure()) {
        return failure(result.unwrapErr());
      }
      values.add(result.unwrap());
    }

    return ok(values);
  }

  /**
   * Transform list of {@link Result} instances into {@link Result} with list of values. If there
   * are any failures, all the failures are returned.
   *
   * @param resultList input list
   * @return success instance if all {@link Result} instances in list are successes or failure
   *     instance with any instances in list is a failure
   */
  static <T, E> Result<List<T>, List<E>> allOfOrAllErrors(List<Result<T, E>> resultList) {
    var values = new ArrayList<T>(resultList.size());
    var failures = new ArrayList<E>(resultList.size());

    resultList.forEach(val -> val.fold(values::add, failures::add));

    return failures.isEmpty() ? ok(values) : failure(failures);
  }

  /**
   * Find and return first success instance among provided.
   *
   * @param first first input result
   * @param results remaining input results
   * @return first success instance among provided, else the first if none are successful
   */
  @SafeVarargs
  static <T, E> Result<T, E> any(Result<T, E> first, Result<T, E>... results) {
    if (first.isSuccess()) {
      return first;
    }

    for (var result : results) {
      if (result.isSuccess()) {
        return result;
      }
    }

    return first;
  }

  /**
   * Lazy version of the {@link #any(Result, Result[])}.
   *
   * @param first first instance to check
   * @param suppliers suppliers which provide remaining instances for check
   * @return first success instance among provided, else the first if none are successful
   */
  @SafeVarargs
  static <T, E> Result<T, E> any(Result<T, E> first, Supplier<Result<T, E>>... suppliers) {
    if (first.isSuccess()) {
      return first;
    }

    for (var supplier : suppliers) {
      var result = supplier.get();

      if (result.isSuccess()) {
        return result;
      }
    }

    return first;
  }

  @SafeVarargs
  static <E> Result<Unit, E> allOf(Result<Unit, E>... values) {
    for (var value : values) {
      if (value.isFailure()) {
        return value;
      }
    }
    return Unit.unitResult();
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper1} prepared for further transformation.
   */
  static <T1, E> Mapper1<T1, E> all(Result<T1, E> value) {
    return () -> value.flatMap(vv1 -> success(tuple(vv1)));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper2} prepared for further transformation.
   */
  static <T1, T2, E> Mapper2<T1, T2, E> all(Result<T1, E> value1, Result<T2, E> value2) {
    return () -> value1.flatMap(vv1 -> value2.flatMap(vv2 -> success(tuple(vv1, vv2))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper3} prepared for further transformation.
   */
  static <T1, T2, T3, E> Mapper3<T1, T2, T3, E> all(
      Result<T1, E> value1, Result<T2, E> value2, Result<T3, E> value3) {
    return () ->
        value1.flatMap(
            vv1 -> value2.flatMap(vv2 -> value3.flatMap(vv3 -> success(tuple(vv1, vv2, vv3)))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper4} prepared for further transformation.
   */
  static <T1, T2, T3, T4, E> Mapper4<T1, T2, T3, T4, E> all(
      Result<T1, E> value1, Result<T2, E> value2, Result<T3, E> value3, Result<T4, E> value4) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 -> value4.flatMap(vv4 -> success(tuple(vv1, vv2, vv3, vv4))))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper5} prepared for further transformation.
   */
  static <T1, T2, T3, T4, T5, E> Mapper5<T1, T2, T3, T4, T5, E> all(
      Result<T1, E> value1,
      Result<T2, E> value2,
      Result<T3, E> value3,
      Result<T4, E> value4,
      Result<T5, E> value5) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 -> success(tuple(vv1, vv2, vv3, vv4, vv5)))))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper6} prepared for further transformation.
   */
  static <T1, T2, T3, T4, T5, T6, E> Mapper6<T1, T2, T3, T4, T5, T6, E> all(
      Result<T1, E> value1,
      Result<T2, E> value2,
      Result<T3, E> value3,
      Result<T4, E> value4,
      Result<T5, E> value5,
      Result<T6, E> value6) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        success(
                                                            tuple(
                                                                vv1, vv2, vv3, vv4, vv5,
                                                                vv6))))))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper7} prepared for further transformation.
   */
  static <T1, T2, T3, T4, T5, T6, T7, E> Mapper7<T1, T2, T3, T4, T5, T6, T7, E> all(
      Result<T1, E> value1,
      Result<T2, E> value2,
      Result<T3, E> value3,
      Result<T4, E> value4,
      Result<T5, E> value5,
      Result<T6, E> value6,
      Result<T7, E> value7) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                success(
                                                                    tuple(
                                                                        vv1, vv2, vv3, vv4, vv5,
                                                                        vv6, vv7)))))))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper8} prepared for further transformation.
   */
  static <T1, T2, T3, T4, T5, T6, T7, T8, E> Mapper8<T1, T2, T3, T4, T5, T6, T7, T8, E> all(
      Result<T1, E> value1,
      Result<T2, E> value2,
      Result<T3, E> value3,
      Result<T4, E> value4,
      Result<T5, E> value5,
      Result<T6, E> value6,
      Result<T7, E> value7,
      Result<T8, E> value8) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                value8.flatMap(
                                                                    vv8 ->
                                                                        success(
                                                                            tuple(
                                                                                vv1, vv2, vv3, vv4,
                                                                                vv5, vv6, vv7,
                                                                                vv8))))))))));
  }

  /**
   * Transform provided results into single result containing tuple of values. The result is failure
   * if any input result is failure. Otherwise returned instance contains tuple with values from
   * input results.
   *
   * @return {@link Mapper9} prepared for further transformation.
   */
  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, E> Mapper9<T1, T2, T3, T4, T5, T6, T7, T8, T9, E> all(
      Result<T1, E> value1,
      Result<T2, E> value2,
      Result<T3, E> value3,
      Result<T4, E> value4,
      Result<T5, E> value5,
      Result<T6, E> value6,
      Result<T7, E> value7,
      Result<T8, E> value8,
      Result<T9, E> value9) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                value8.flatMap(
                                                                    vv8 ->
                                                                        value9.flatMap(
                                                                            vv9 ->
                                                                                success(
                                                                                    tuple(
                                                                                        vv1, vv2,
                                                                                        vv3, vv4,
                                                                                        vv5, vv6,
                                                                                        vv7, vv8,
                                                                                        vv9)))))))))));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, E>
      Mapper10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, E> all(
          Result<T1, E> value1,
          Result<T2, E> value2,
          Result<T3, E> value3,
          Result<T4, E> value4,
          Result<T5, E> value5,
          Result<T6, E> value6,
          Result<T7, E> value7,
          Result<T8, E> value8,
          Result<T9, E> value9,
          Result<T10, E> value10) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                value8.flatMap(
                                                                    vv8 ->
                                                                        value9.flatMap(
                                                                            vv9 ->
                                                                                value10.flatMap(
                                                                                    vv10 ->
                                                                                        success(
                                                                                            tuple(
                                                                                                vv1,
                                                                                                vv2,
                                                                                                vv3,
                                                                                                vv4,
                                                                                                vv5,
                                                                                                vv6,
                                                                                                vv7,
                                                                                                vv8,
                                                                                                vv9,
                                                                                                vv10))))))))))));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, E>
      Mapper11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, E> all(
          Result<T1, E> value1,
          Result<T2, E> value2,
          Result<T3, E> value3,
          Result<T4, E> value4,
          Result<T5, E> value5,
          Result<T6, E> value6,
          Result<T7, E> value7,
          Result<T8, E> value8,
          Result<T9, E> value9,
          Result<T10, E> value10,
          Result<T11, E> value11) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                value8.flatMap(
                                                                    vv8 ->
                                                                        value9.flatMap(
                                                                            vv9 ->
                                                                                value10.flatMap(
                                                                                    vv10 ->
                                                                                        value11
                                                                                            .flatMap(
                                                                                                vv11 ->
                                                                                                    success(
                                                                                                        tuple(
                                                                                                            vv1,
                                                                                                            vv2,
                                                                                                            vv3,
                                                                                                            vv4,
                                                                                                            vv5,
                                                                                                            vv6,
                                                                                                            vv7,
                                                                                                            vv8,
                                                                                                            vv9,
                                                                                                            vv10,
                                                                                                            vv11)))))))))))));
  }

  static <T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, E>
      Mapper12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, E> all(
          Result<T1, E> value1,
          Result<T2, E> value2,
          Result<T3, E> value3,
          Result<T4, E> value4,
          Result<T5, E> value5,
          Result<T6, E> value6,
          Result<T7, E> value7,
          Result<T8, E> value8,
          Result<T9, E> value9,
          Result<T10, E> value10,
          Result<T11, E> value11,
          Result<T12, E> value12) {
    return () ->
        value1.flatMap(
            vv1 ->
                value2.flatMap(
                    vv2 ->
                        value3.flatMap(
                            vv3 ->
                                value4.flatMap(
                                    vv4 ->
                                        value5.flatMap(
                                            vv5 ->
                                                value6.flatMap(
                                                    vv6 ->
                                                        value7.flatMap(
                                                            vv7 ->
                                                                value8.flatMap(
                                                                    vv8 ->
                                                                        value9.flatMap(
                                                                            vv9 ->
                                                                                value10.flatMap(
                                                                                    vv10 ->
                                                                                        value11
                                                                                            .flatMap(
                                                                                                vv11 ->
                                                                                                    value12
                                                                                                        .flatMap(
                                                                                                            vv12 ->
                                                                                                                success(
                                                                                                                    tuple(
                                                                                                                        vv1,
                                                                                                                        vv2,
                                                                                                                        vv3,
                                                                                                                        vv4,
                                                                                                                        vv5,
                                                                                                                        vv6,
                                                                                                                        vv7,
                                                                                                                        vv8,
                                                                                                                        vv9,
                                                                                                                        vv10,
                                                                                                                        vv11,
                                                                                                                        vv12))))))))))))));
  }

  /**
   * Helper interface for convenient {@link Tuple1} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper1<T1, E> {
    Result<Tuple1<T1>, E> id();

    default <R> Result<R, E> map(Func1<T1, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func1<T1, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple2} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper2<T1, T2, E> {
    Result<Tuple2<T1, T2>, E> id();

    default <R> Result<R, E> map(Func2<T1, T2, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func2<T1, T2, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple3} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper3<T1, T2, T3, E> {
    Result<Tuple3<T1, T2, T3>, E> id();

    default <R> Result<R, E> map(Func3<T1, T2, T3, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func3<T1, T2, T3, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple4} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper4<T1, T2, T3, T4, E> {
    Result<Tuple4<T1, T2, T3, T4>, E> id();

    default <R> Result<R, E> map(Func4<T1, T2, T3, T4, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func4<T1, T2, T3, T4, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple5} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper5<T1, T2, T3, T4, T5, E> {
    Result<Tuple5<T1, T2, T3, T4, T5>, E> id();

    default <R> Result<R, E> map(Func5<T1, T2, T3, T4, T5, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func5<T1, T2, T3, T4, T5, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple6} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper6<T1, T2, T3, T4, T5, T6, E> {
    Result<Tuple6<T1, T2, T3, T4, T5, T6>, E> id();

    default <R> Result<R, E> map(Func6<T1, T2, T3, T4, T5, T6, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func6<T1, T2, T3, T4, T5, T6, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple7} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper7<T1, T2, T3, T4, T5, T6, T7, E> {
    Result<Tuple7<T1, T2, T3, T4, T5, T6, T7>, E> id();

    default <R> Result<R, E> map(Func7<T1, T2, T3, T4, T5, T6, T7, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func7<T1, T2, T3, T4, T5, T6, T7, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple8} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper8<T1, T2, T3, T4, T5, T6, T7, T8, E> {
    Result<Tuple8<T1, T2, T3, T4, T5, T6, T7, T8>, E> id();

    default <R> Result<R, E> map(Func8<T1, T2, T3, T4, T5, T6, T7, T8, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(Func8<T1, T2, T3, T4, T5, T6, T7, T8, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  /**
   * Helper interface for convenient {@link Tuple9} transformation. In case if you need to return a
   * tuple, it might be more convenient to return this interface instead. For example, instead of
   * this:
   *
   * <blockquote>
   *
   * <pre>
   *     return tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   *
   * return this:
   *
   * <blockquote>
   *
   * <pre>
   *     return () -> tuple(value, ...);
   * </pre>
   *
   * </blockquote>
   */
  interface Mapper9<T1, T2, T3, T4, T5, T6, T7, T8, T9, E> {
    Result<Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9>, E> id();

    default <R> Result<R, E> map(Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(
        Func9<T1, T2, T3, T4, T5, T6, T7, T8, T9, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  interface Mapper10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, E> {
    Result<Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>, E> id();

    default <R> Result<R, E> map(Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(
        Func10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  interface Mapper11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, E> {
    Result<Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>, E> id();

    default <R> Result<R, E> map(Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(
        Func11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  interface Mapper12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, E> {
    Result<Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>, E> id();

    default <R> Result<R, E> map(
        Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, R> mapper) {
      return id().map(tuple -> tuple.map(mapper));
    }

    default <R> Result<R, E> flatMap(
        Func12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, Result<R, E>> mapper) {
      return id().flatMap(tuple -> tuple.map(mapper));
    }
  }

  class UnwrapException extends RuntimeException {
    private final Object error;

    public UnwrapException(Object error) {
      super("Unwrap failed as the Result is an Err. The error value is: " + error);
      this.error = error;
    }

    public Object error() {
      return error;
    }
  }
}
