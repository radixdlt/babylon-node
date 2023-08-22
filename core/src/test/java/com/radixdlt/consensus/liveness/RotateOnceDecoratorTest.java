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

package com.radixdlt.consensus.liveness;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.google.common.collect.Lists;
import com.google.common.primitives.Ints;
import com.radixdlt.consensus.bft.BFTValidator;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.bft.Round;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.rev2.ComponentAddress;
import com.radixdlt.utils.PrivateKeys;
import com.radixdlt.utils.UInt192;
import java.util.Arrays;
import java.util.stream.LongStream;
import org.junit.Test;

public class RotateOnceDecoratorTest {

  @Test
  public void initial_rounds_iterate_through_all_validators_once() {
    final var validators =
        Arrays.asList(
            BFTValidator.from(id(1012, 7), power(22)), // for round 2
            BFTValidator.from(id(1013, 3), power(33)), // for round 0
            BFTValidator.from(id(1018, 7), power(22)), // for round 3
            BFTValidator.from(id(1011, 5), power(22)) // for round 1
            );
    // the underlying mock returns a sequence of `[id(0, 1), id(1, 2), id(2, 3), ...]`
    final var underlying = mock(ProposerElection.class);
    when(underlying.getProposer(any()))
        .then(
            invocation -> {
              final int seed = Ints.checkedCast(invocation.getArgument(0, Round.class).number());
              return id(seed, seed + 1);
            });

    final var subject = new RotateOnceDecorator(BFTValidatorSet.from(validators), underlying);

    // happy-path on initial rounds:
    final int[] iteratedValidatorIndices =
        LongStream.range(0, 4)
            .mapToObj(Round::of)
            .map(subject::getProposer)
            .mapToInt(Lists.transform(validators, BFTValidator::getValidatorId)::indexOf)
            .toArray();
    assertThat(iteratedValidatorIndices)
        .containsExactly(
            1, // wins by stake (33 is before 22)
            3, // wins by key tie-breaker (5 is before 7)
            0, // wins by address tie-breaker (null is before 1018)
            2); // well, does not win

    // right after the initial rounds - the underlying impl returns the first one from its sequence
    assertThat(subject.getProposer(Round.of(4))).isEqualTo(id(0, 1));

    // repeated "initial rounds" query
    assertThat(subject.getProposer(Round.of(2))).isEqualTo(validators.get(0).getValidatorId());
    assertThat(subject.getProposer(Round.of(2))).isEqualTo(validators.get(0).getValidatorId());

    // far outside the initial rounds - the underlying impl returns a `N - initial_rounds`-th one
    assertThat(subject.getProposer(Round.of(907))).isEqualTo(id(903, 904));
  }

  private static BFTValidatorId id(Integer addressBytes, int keySeed) {
    // we do not care about valid address representation (i.e. we do not call Engine), just ordering
    final ComponentAddress address = new ComponentAddress(Ints.toByteArray(addressBytes));
    final ECDSASecp256k1PublicKey key = PrivateKeys.ofNumeric(keySeed).getPublicKey();
    return BFTValidatorId.create(address, key);
  }

  private static UInt192 power(int value) {
    return UInt192.from(value);
  }
}
