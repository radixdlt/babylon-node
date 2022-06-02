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

import java.util.Optional;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Collectors;
import org.junit.Assert;
import org.junit.Test;

public class OptionTest {
  @Test
  public void emptyOptionsAreEqual() {
    assertEquals(Option.empty(), Option.empty());
    assertEquals("None()", Option.empty().toString());
  }

  @Test
  public void presentOptionsAreEqualIfContentEqual() {
    assertEquals(Option.present(123), Option.present(123));
    assertNotEquals(Option.present(321), Option.present(123));
    assertNotEquals(Option.empty(), Option.present(1));
    assertNotEquals(Option.present(1), Option.empty());
    assertEquals("Some(1)", Option.present(1).toString());
  }

  @Test
  public void presentOptionCanBeTransformed() {
    Option.present(123)
        .onPresent(value -> assertEquals(123, value.intValue()))
        .onEmpty(Assert::fail)
        .map(Object::toString)
        .onPresent(value -> assertEquals("123", value))
        .onEmpty(Assert::fail);
  }

  @Test
  public void emptyOptionRemainsEmptyAfterTransformation() {
    Option.empty().onPresent(this::fail).map(Object::toString).onPresent(this::fail);
  }

  @Test
  public void presentOptionCanBeFlatMapped() {
    Option.present(123L)
        .onPresent(value -> assertEquals(123L, value.longValue()))
        .onEmpty(Assert::fail)
        .flatMap(value -> Option.present(value.toString()))
        .onPresent(value -> assertEquals("123", value))
        .onEmpty(Assert::fail);
  }

  @Test
  public void emptyOptionRemainsEmptyAfterFlatMap() {
    Option.empty()
        .onPresent(this::fail)
        .flatMap(value -> Option.present(value.toString()))
        .onPresent(this::fail);
  }

  @Test
  public void presentOptionCanBeFilteredToPresentOption() {
    Option.present(123L)
        .onPresent(value -> assertEquals(123L, value.longValue()))
        .onEmpty(Assert::fail)
        .filter(value -> value > 120L)
        .onPresent(value -> assertEquals(123L, value.longValue()))
        .onEmpty(Assert::fail);
  }

  @Test
  public void presentOptionCanBeFilteredToEmptyOption() {
    Option.present(123L)
        .onPresent(value -> assertEquals(123L, value.longValue()))
        .onEmpty(Assert::fail)
        .filter(value -> value < 120L)
        .onPresent(this::fail);
  }

  @Test
  public void whenOptionPresentThenPresentSideEffectIsTriggered() {
    var flag = new AtomicBoolean(false);

    Option.present(123L).onPresent(value -> flag.set(true));

    assertTrue(flag.get());
  }

  @Test
  public void whenOptionEmptyThenPresentSideEffectIsNotTriggered() {
    var flag = new AtomicBoolean(false);

    Option.empty().onPresent(value -> flag.set(true));

    assertFalse(flag.get());
  }

  @Test
  public void whenOptionEmptyThenEmptySideEffectIsTriggered() {
    var flag = new AtomicBoolean(false);

    Option.empty().onEmpty(() -> flag.set(true));

    assertTrue(flag.get());
  }

  @Test
  public void whenOptionPresentThenEmptySideEffectIsNotTriggered() {
    var flag = new AtomicBoolean(false);

    Option.present(123L).onEmpty(() -> flag.set(true));

    assertFalse(flag.get());
  }

  @Test
  public void presentSideEffectIsInvokedForPresentOption() {
    var flagPresent = new AtomicLong(0L);
    var flagEmpty = new AtomicBoolean(false);

    Option.present(123L).apply(() -> flagEmpty.set(true), flagPresent::set);

    assertEquals(123L, flagPresent.get());
    assertFalse(flagEmpty.get());
  }

  @Test
  public void emptySideEffectIsInvokedForEmptyOption() {
    var flagPresent = new AtomicLong(0L);
    var flagEmpty = new AtomicBoolean(false);

    Option.<Long>empty().apply(() -> flagEmpty.set(true), flagPresent::set);

    assertEquals(0L, flagPresent.get());
    assertTrue(flagEmpty.get());
  }

  @Test
  public void valueCanBeObtainedFromOption() {
    assertEquals(321L, Option.present(321L).or(123L).longValue());
    assertEquals(123L, Option.empty().or(123L));
  }

  @Test
  public void valueCanBeLazilyObtainedFromOption() {
    var flag = new AtomicBoolean(false);
    assertEquals(
        321L,
        Option.present(321L)
            .or(
                () -> {
                  flag.set(true);
                  return 123L;
                })
            .longValue());
    assertFalse(flag.get());

    assertEquals(
        123L,
        Option.empty()
            .or(
                () -> {
                  flag.set(true);
                  return 123L;
                }));
    assertTrue(flag.get());
  }

  @Test
  public void presentOptionCanBeStreamed() {
    assertEquals(
        1L,
        Option.present(1).stream().collect(Collectors.summarizingInt(Integer::intValue)).getSum());
  }

  @Test
  public void emptyOptionCanBeStreamedToEmptyStream() {
    assertEquals(0L, Option.empty().stream().count());
  }

  @Test
  public void presentOptionCanBeConvertedToSuccessResult() {
    Option.option(1)
        .toResult(Causes.cause("Not expected"))
        .onSuccess(value -> assertEquals(1, value.intValue()))
        .onFailureDo(Assert::fail);
  }

  @Test
  public void emptyOptionCanBeConvertedToFailureResult() {
    Option.option(null)
        .toResult(Causes.cause("Expected"))
        .onSuccess(value -> fail("Should not be a success"))
        .onFailure(cause -> assertEquals("Expected", cause.message()));
  }

  @Test
  public void optionCanBeConvertedToOptional() {
    assertEquals(Optional.of(321), Option.present(321).toOptional());
    assertEquals(Optional.empty(), Option.empty().toOptional());
  }

  @Test
  public void optionalCanBeConvertedToOption() {
    assertEquals(Option.option(123), Option.from(Optional.of(123)));
    assertEquals(Option.empty(), Option.from(Optional.empty()));
  }

  @Test
  public void anyReturnsFirstPresentOption() {
    Option.any(Option.present(1), Option.present(2))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(1, value.intValue()));

    Option.any(Option.empty(), Option.present(2))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(2, value.intValue()));
  }

  @Test
  public void anyLazilyEvaluatesOtherOptions() {
    var flag = new AtomicBoolean(false);

    Option.any(
            Option.present(1),
            () -> {
              flag.set(true);
              return Option.present(2);
            })
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(1, value.intValue()));

    assertFalse(flag.get());

    Option.any(
            Option.empty(),
            () -> {
              flag.set(true);
              return Option.present(2);
            })
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(2, value.intValue()));

    assertTrue(flag.get());
  }

  @Test
  public void anyFindsFirstNonEmptyOption() {
    Option.any(Option.empty(), Option.present(2))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(2, value.intValue()));

    Option.any(Option.empty(), Option.empty(), Option.present(3))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(3, value.intValue()));

    Option.any(Option.empty(), Option.empty(), Option.empty(), Option.present(4))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(4, value.intValue()));

    Option.any(Option.empty(), Option.empty(), Option.empty(), Option.empty(), Option.present(5))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(5, value.intValue()));

    Option.any(
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.present(6))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(6, value.intValue()));

    Option.any(
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.present(7))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(7, value.intValue()));

    Option.any(
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.present(8))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(8, value.intValue()));

    Option.any(
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.empty(),
            Option.present(9))
        .onEmpty(Assert::fail)
        .onPresent(value -> assertEquals(9, value.intValue()));
  }

  @Test
  public void allIsPresentIfAllInputsArePresent() {
    Option.all(Option.present(1))
        .map(v1 -> v1)
        .onPresent(value -> assertEquals(1, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1))
        .map((v1, v2) -> v1 + v2)
        .onPresent(value -> assertEquals(2, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1), Option.present(1))
        .map((v1, v2, v3) -> v1 + v2 + v3)
        .onPresent(value -> assertEquals(3, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1), Option.present(1), Option.present(1))
        .map((v1, v2, v3, v4) -> v1 + v2 + v3 + v4)
        .onPresent(value -> assertEquals(4, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .map((v1, v2, v3, v4, v5) -> v1 + v2 + v3 + v4 + v5)
        .onPresent(value -> assertEquals(5, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .map((v1, v2, v3, v4, v5, v6) -> v1 + v2 + v3 + v4 + v5 + v6)
        .onPresent(value -> assertEquals(6, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .map((v1, v2, v3, v4, v5, v6, v7) -> v1 + v2 + v3 + v4 + v5 + v6 + v7)
        .onPresent(value -> assertEquals(7, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .map((v1, v2, v3, v4, v5, v6, v7, v8) -> v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8)
        .onPresent(value -> assertEquals(8, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .map((v1, v2, v3, v4, v5, v6, v7, v8, v9) -> v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8 + v9)
        .onPresent(value -> assertEquals(9, value.intValue()))
        .onEmpty(Assert::fail);
  }

  @Test
  public void allCanBeFlatMappedIfAllInputsArePresent() {
    Option.all(Option.present(1))
        .flatMap(v1 -> Option.present(v1))
        .onPresent(value -> assertEquals(1, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1))
        .flatMap((v1, v2) -> Option.present(v1 + v2))
        .onPresent(value -> assertEquals(2, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1), Option.present(1))
        .flatMap((v1, v2, v3) -> Option.present(v1 + v2 + v3))
        .onPresent(value -> assertEquals(3, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(Option.present(1), Option.present(1), Option.present(1), Option.present(1))
        .flatMap((v1, v2, v3, v4) -> Option.present(v1 + v2 + v3 + v4))
        .onPresent(value -> assertEquals(4, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .flatMap((v1, v2, v3, v4, v5) -> Option.present(v1 + v2 + v3 + v4 + v5))
        .onPresent(value -> assertEquals(5, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .flatMap((v1, v2, v3, v4, v5, v6) -> Option.present(v1 + v2 + v3 + v4 + v5 + v6))
        .onPresent(value -> assertEquals(6, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .flatMap((v1, v2, v3, v4, v5, v6, v7) -> Option.present(v1 + v2 + v3 + v4 + v5 + v6 + v7))
        .onPresent(value -> assertEquals(7, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .flatMap(
            (v1, v2, v3, v4, v5, v6, v7, v8) ->
                Option.present(v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8))
        .onPresent(value -> assertEquals(8, value.intValue()))
        .onEmpty(Assert::fail);

    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .flatMap(
            (v1, v2, v3, v4, v5, v6, v7, v8, v9) ->
                Option.present(v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8 + v9))
        .onPresent(value -> assertEquals(9, value.intValue()))
        .onEmpty(Assert::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing1() {
    Option.all(Option.empty()).id().onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing2() {
    Option.all(Option.empty(), Option.present(1)).id().onPresent(this::fail);
    Option.all(Option.present(1), Option.empty()).id().onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing3() {
    Option.all(Option.empty(), Option.present(1), Option.present(1)).id().onPresent(this::fail);
    Option.all(Option.present(1), Option.empty(), Option.present(1)).id().onPresent(this::fail);
    Option.all(Option.present(1), Option.present(1), Option.empty()).id().onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing4() {
    Option.all(Option.empty(), Option.present(1), Option.present(1), Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(Option.present(1), Option.empty(), Option.present(1), Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(Option.present(1), Option.present(1), Option.empty(), Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(Option.present(1), Option.present(1), Option.present(1), Option.empty())
        .id()
        .onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing5() {
    Option.all(
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty())
        .id()
        .onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing6() {
    Option.all(
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty())
        .id()
        .onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing7() {
    Option.all(
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty())
        .id()
        .onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing8() {
    Option.all(
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty())
        .id()
        .onPresent(this::fail);
  }

  @Test
  public void allIsMissingIfAnyInputIsMissing9() {
    Option.all(
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty(),
            Option.present(1))
        .id()
        .onPresent(this::fail);
    Option.all(
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.present(1),
            Option.empty())
        .id()
        .onPresent(this::fail);
  }

  private <T> void fail(T unused) {
    Assert.fail();
  }
}
