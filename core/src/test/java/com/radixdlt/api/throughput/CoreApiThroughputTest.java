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

package com.radixdlt.api.throughput;

import com.radixdlt.api.DeterministicCoreApiTestBase;
import com.radixdlt.api.RealisticCoreApiTestBase;
import com.radixdlt.api.core.generated.models.TransactionSubmitRequest;
import com.radixdlt.api.core.generated.models.V0TransactionStatusRequest;
import com.radixdlt.api.core.generated.models.V0TransactionStatusResponse;
import com.radixdlt.rev2.REv2TestTransactions;
import com.radixdlt.utils.Bytes;
import org.junit.Test;

import static com.radixdlt.harness.predicates.NodesPredicate.allCommittedTransaction;
import static org.assertj.core.api.AssertionsForClassTypes.assertThat;

public class CoreApiThroughputTest extends RealisticCoreApiTestBase {
  @Test
  public void fill_mempool_and_empty_test() throws Exception {
    // Need to change this to a non-deterministic test.
    // Try to use the default node setup for things like:
    // * Proposal size
    // * Other things like mempool sync delay
    try (var test = buildRunningServerTest()) {
      // PART 1 - Populate mempool with 1000 transactions

      // PART 2 - Start stop-watch and Start consensus

      // PART 3 - wait till mempool empties and record time and TPS
    }
  }

  @Test
  public void concurrent_actors_test() throws Exception {
    // Need to change this to a non-deterministic test.
    // Try to use the default node setup for things like:
    // * Proposal size
    // * Other things like mempool sync delay
    try (var test = buildRunningServerTest()) {
      // PART 1 - Start stopwatch + node, and count of 10000 transactions
      // PART 2 - Spin up 1000 concurrent actors
      // PART 3 - Each actor tries to submit transaction every few seconds, and waits for commit
      //          As actors complete, replace with new transaction
      // PART 4 - Once all transactions complete, record time and TPS
    }
  }
}
