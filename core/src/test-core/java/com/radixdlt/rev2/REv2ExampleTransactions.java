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
  public static final byte[] VALID_TXN_BYTES_0 =
      Hex.decode(
          "10020000001002000000100200000010070000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0500000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9010010010000003011010000000d000000436c656172417574685a6f6e65000000003023020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179892400000006cf35fe75e8cf4cc7db93e2d0b5e5f17efe0768cc2eb3db9d1e9d4bb8c6df6d95446cc78c550c68a91217f75266dc8ec14b1c2324637ea49cc99119d782f3a4b02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee592400000000350a245e2df1143d5a97433cc640601e725fc342d3ba9ebd74052757526695432ff1c321c001ab11f01943a9da312333b78f4bcbadfac89754ec111c2cf5ea1924000000024bd869215c36f4291ea48ac7e1378758bef43a56088446d441f99509cec06f9516089eb7040d1bb9455be59455084c232ecc85becb496cb59b7c156a1206917");
  public static final byte[] VALID_TXN_BYTES_1 =
      Hex.decode(
          "10020000001002000000100200000010070000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0600000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9010010010000003011010000000d000000436c656172417574685a6f6e65000000003023020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000d6f37bebb4c67ebb0844dd48e447c415a13b47fafdf13495f58b21826dc044a043fb00243cfe573bbb38b8ae9371801c2b91ec92ae764238e4ff40d857e58a3002000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5924000000047da0da82cdceed2a227ebd305ece670cf12aaedad6863ebce173d0952eea73b3a1b136bae431d82bae822ceb11eaed406dddc1a4a94756201cb7292139584bf9240000000a767554290bd2cba8e63bc1feeefc1534ebcd33fe345f9a8d0ac76abc1d3bd5968e847ec5ca55d6e9fe18227f13c5c114463751e9bc5a38f563ba8819d7fc882");
  public static final byte[] VALID_TXN_BYTES_2 =
      Hex.decode(
          "10020000001002000000100200000010070000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0700000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9010010010000003011010000000d000000436c656172417574685a6f6e65000000003023020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000c16e23aeeb803da90c19e58d082e099bece503a6a0f81e8ea78b464c06a2bc7109d6a00154f1aef82a6a5076040d04def0f3dbbdb269a917d4b04bdb62c94df102000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee59240000000db9f306a835f78fcc99c9ad9bbea8f1ea3f63114309bfbf7854a5bd73e89516c5954fa6296919ebd4ac3d826fedf8a7cd13e8dcf55cb535d2f204a9bbf15d3f192400000006e1f17b5ce613a746a56c6cbf67beb4549dcd65f237d4e11145ccb976b1e69042b010cf6d1b90c2d3db4f917a4a30eaf364077b762ccdd55534b6790ad248d34");

  private REv2ExampleTransactions() {
    throw new IllegalStateException("Cannot instantiate.");
  }
}
