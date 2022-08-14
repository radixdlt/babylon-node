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

package com.radixdlt.rev2;

import org.bouncycastle.util.encoders.Hex;

public final class REv2ExampleTransactions {
  // These are copied from the `construct_sign_and_notarize `test in `model/transaction.rs`
  public static final byte[] VALID_TXN_BYTES_0 =
      Hex.decode(
          "10020000001002000000100200000010090000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0700000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010010000003011010000000d000000436c656172417574685a6f6e65000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000024bcbce546542fc6af07d9a74f2f0051f150b9b1b3db2ee6aa8c527f36246053c78c2fa7dfe982cc528f62dca74beb7a5bc3f9240e025777ce7091005b377af02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee592400000008103c26608a17c042cf536ff2767a85a844b3a555f217ce1f8c7bd6fa4d9619131e52b60bf436a203c112228df1915f5c2956ab7e7ef9cced5fff16d2f91d57d9240000000919af67b11c4272f2509ed2fab7f609bb761662bbf9986c9e49ef3b7574a902c0c67929b98ff75e7d97d9bc02470ea9c8b8ab6fa8e0ab4b5510fb0bc845836fd");
  public static final byte[] VALID_TXN_BYTES_1 =
      Hex.decode(
          "10020000001002000000100200000010090000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0800000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010010000003011010000000d000000436c656172417574685a6f6e65000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000d35b40323e7552f006f430088ad80db3d2ab8cd185879acdd3017ee2a3738e2f11caf6435c3150ebaa5a51248a05644c0bd7f3f6d05f6f6c2c47314dd1da7d9d02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee59240000000b4d3de92645f87f1489c9802e42cd80248c6603f6914160f5a507c423027cefb0f8192bd6d955be3699ebb430fa6d3cf17b29ab8482e0d7a3ba179ca2b1898089240000000802421b5741b1d4a1cd5382047a41eb71100534b13b3ebdb51fc09b968e1eae439e0c2b97f2129a1918a9d48917847d608ef4f3321fb6e4d362cc92eee6f1c67");
  public static final byte[] VALID_TXN_BYTES_2 =
      Hex.decode(
          "10020000001002000000100200000010090000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0900000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010010000003011010000000d000000436c656172417574685a6f6e65000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000fb8a35182be8f6462aafd8f4bb22a10fc80303ff49f52cd0f34f5a6f661411fb3ed0fba58fc08b0667e58dc1edbb18c64d0eb3692ee5291097b81945df5a27e602000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5924000000074429593a363dd604881e2f847abd40a7e98f85c1210d691f6f9491033323ba630bfe47dfb0d24c3bac499ea81384e788ad65ab249f4ecf80fc2244fd16644de92400000009eed40f54876262e3d73f55f32bef9ada6c3b9564654f6aa82828c1d3008ee141b39e0a58c06f0b328e152a7d46484f0f6ac14b06809a9b6d4ba97f6f3a49db8");

  private REv2ExampleTransactions() {
    throw new IllegalStateException("Cannot instantiate.");
  }
}
