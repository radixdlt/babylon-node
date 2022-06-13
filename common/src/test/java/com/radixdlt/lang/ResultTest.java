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

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import java.util.Objects;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.Assert;
import org.junit.Test;

public class ResultTest {
  @Test
  public void successResultsAreEqualIfValueEqual() {
    assertEquals(Result.success("123"), Result.success(123).map(Objects::toString));
    assertNotEquals(Result.success("321"), Result.success(123).map(Objects::toString));
  }

  @Test
  public void failureResultsAreEqualIfFailureIsEqual() {
    assertEquals(
        Result.<Integer, Cause>failure(Causes.cause("123")),
        Result.<Integer, Cause>success(123).filterOrElse(v -> v < 0, Causes.with1("{0}")));
    assertNotEquals(
        Result.<Integer, Cause>failure(Causes.cause("321")),
        Result.<Integer, Cause>success(123).filterOrElse(v -> v < 0, Causes.with1("{0}")));
  }

  @Test
  public void successResultCanBeTransformedWithMap() {
    Result.success(123)
        .map(Objects::toString)
        .onFailureDo(Assert::fail)
        .onSuccess(value -> assertEquals("123", value));
  }

  @Test
  public void successResultCanBeTransformedWithFlatMap() {
    Result.success(123)
        .flatMap(v -> Result.success(v.toString()))
        .onFailureDo(Assert::fail)
        .onSuccess(value -> assertEquals("123", value));
  }

  @Test
  public void failureResultRemainsUnchangedAfterMap() {
    Result.<Integer, Cause>failure(Causes.cause("Some error"))
        .map(Objects::toString)
        .onFailure(cause -> assertEquals("Some error", cause.message()))
        .onSuccessDo(Assert::fail);
  }

  @Test
  public void failureResultRemainsUnchangedAfterFlatMap() {
    Result.<Integer, Cause>failure(Causes.cause("Some error"))
        .flatMap(v -> Result.success(v.toString()))
        .onFailure(cause -> assertEquals("Some error", cause.message()))
        .onSuccessDo(Assert::fail);
  }

  @Test
  public void onlyOneMethodIsInvokedOnApply() {
    Result.<Integer, Cause>success(321)
        .apply(Functions::unitFn, failure -> fail(failure.message()));

    Result.failure(Causes.cause("Some error"))
        .apply(value -> fail(value.toString()), Functions::unitFn);
  }

  @Test
  public void onSuccessIsInvokedForSuccessResult() {
    Result.<Integer, Cause>success(123)
        .onFailureDo(Assert::fail)
        .onSuccess(value -> assertEquals(123, value.intValue()));
    Result.<Integer, Cause>failure(Causes.cause("123"))
        .onFailure(cause -> assertEquals("123", cause.message()))
        .onSuccess(value -> fail(value.toString()));
  }

  @Test
  public void onSuccessDoIsInvokedForSuccessResult() {
    var flag1 = new AtomicBoolean(false);

    Result.success(123).onFailureDo(Assert::fail).onSuccessDo(() -> flag1.set(true));

    assertTrue(flag1.get());

    var flag2 = new AtomicBoolean(false);

    Result.<Integer, Cause>failure(Causes.cause("123"))
        .onFailureDo(() -> flag2.set(true))
        .onSuccessDo(Assert::fail);

    assertTrue(flag2.get());
  }

  @Test
  public void onFailureIsInvokedForFailure() {
    Result.<Integer, Cause>success(123)
        .onFailure(cause -> fail(cause.message()))
        .onSuccess(value -> assertEquals(123, value.intValue()));
    Result.<Integer, Cause>failure(Causes.cause("123"))
        .onFailure(cause -> assertEquals("123", cause.message()))
        .onSuccess(value -> fail(value.toString()));
  }

  @Test
  public void onFailureDoIsInvokedForFailureResult() {
    var flag1 = new AtomicBoolean(false);

    Result.success(123).onFailureDo(Assert::fail).onSuccessDo(() -> flag1.set(true));

    assertTrue(flag1.get());

    var flag2 = new AtomicBoolean(false);

    Result.<Integer, Cause>failure(Causes.cause("123"))
        .onFailureDo(() -> flag2.set(true))
        .onSuccessDo(Assert::fail);

    assertTrue(flag2.get());
  }

  @Test
  public void resultCanBeConvertedToOption() {
    Result.success(123)
        .toOption()
        .onPresent(value -> assertEquals(123, value.intValue()))
        .onEmpty(Assert::fail);

    var flag1 = new AtomicBoolean(false);

    Result.<Integer, Cause>failure(Causes.cause("123"))
        .toOption()
        .onPresent(__ -> fail("Should not happen"))
        .onEmpty(() -> flag1.set(true));

    assertTrue(flag1.get());
  }

  @Test
  public void resultStatusCanBeChecked() {
    assertTrue(Result.success(321).isSuccess());
    assertFalse(Result.success(321).isFailure());
    assertFalse(Result.failure(Causes.cause("321")).isSuccess());
    assertTrue(Result.failure(Causes.cause("321")).isFailure());
  }

  @Test
  public void successResultCanBeFiltered() {
    Result.<Integer, Cause>success(231)
        .onSuccess(value -> assertEquals(231, value.intValue()))
        .onFailureDo(Assert::fail)
        .filterOrElse(value -> value > 321, Causes.with1("Value {0} is below threshold"))
        .onSuccessDo(Assert::fail)
        .onFailure(cause -> assertEquals("Value 231 is below threshold", cause.message()));
  }

  @Test
  public void liftWrapsCodeWhichCanThrowExceptions() {
    Result.lift(() -> throwingFunction(3), Causes::fromThrowable)
        .onFailure(
            cause ->
                assertTrue(
                    cause
                        .message()
                        .startsWith("java.lang.IllegalStateException: Just throw exception 3")))
        .onSuccess(value -> fail("Expecting failure"));

    Result.lift(() -> throwingFunction(4), Causes::fromThrowable)
        .onFailure(cause -> fail(cause.message()))
        .onSuccess(value -> assertEquals("Input:4", value));
  }

  static String throwingFunction(int i) {
    if (i == 3) {
      throw new IllegalStateException("Just throw exception " + i);
    }

    return "Input:" + i;
  }
}
