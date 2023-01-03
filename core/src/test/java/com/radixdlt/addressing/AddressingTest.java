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

package com.radixdlt.addressing;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.crypto.exception.PublicKeyException;
import com.radixdlt.exceptions.Bech32DecodeException;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.ScryptoConstants;
import com.radixdlt.serialization.DeserializeException;
import org.junit.Test;

public class AddressingTest {
  @Test
  public void test_system_faucet_address_encoded_correctly() {
    assertThat(
            Addressing.ofNetwork(Network.INTEGRATIONTESTNET)
                .encodeNormalComponentAddress(ScryptoConstants.FAUCET_COMPONENT_ADDRESS))
        .isEqualTo("component_test1qgehpqdhhr62xh76wh6gppnyn88a0uau68epljprvj3s83tzxc");
  }

  @Test
  public void test_system_faucet_address_decoded_correctly() {
    assertThat(
            Addressing.ofNetwork(Network.INTEGRATIONTESTNET)
                .decodeNormalComponentAddress(
                    "component_test1qgehpqdhhr62xh76wh6gppnyn88a0uau68epljprvj3s83tzxc"))
        .isEqualTo(ScryptoConstants.FAUCET_COMPONENT_ADDRESS);
  }

  @Test
  public void can_encode_and_decode_a_node_address()
      throws PublicKeyException, DeserializeException {
    var pubKey =
        ECDSASecp256k1PublicKey.fromHex(
            "0236856ea9fa8c243e45fc94ec27c29cf3f17e3a9e19a410ee4a41f4858e379918");
    var address = Addressing.ofNetwork(Network.INTEGRATIONTESTNET).encodeNodeAddress(pubKey);
    var decoded = Addressing.ofNetwork(Network.INTEGRATIONTESTNET).decodeNodeAddress(address);

    assertThat(decoded).isEqualTo(pubKey);
  }

  @Test
  public void node_address_for_enkinet_is_decoded_correctly()
      throws PublicKeyException, DeserializeException {
    var address = "node_tdx_21_1qfk895krd3l8t8z7z7p9sxpjdszpal24f6y2sjtqe7mdkhdele5az658ak2";
    var expected =
        ECDSASecp256k1PublicKey.fromHex(
            "026c72d2c36c7e759c5e17825818326c041efd554e88a84960cfb6db5db9fe69d1");
    var decoded = Addressing.ofNetwork(Network.ENKINET).decodeNodeAddress(address);

    assertThat(decoded).isEqualTo(expected);
  }

  @Test
  public void non_bech32m_addresses_are_not_permitted() {
    var address = "tn211qg42kem99gpw3esdt7avcncugfl89aq4uzke8l4rakq05u99c0x86qt94jr";
    assertThatThrownBy(() -> Addressing.decodeNodeAddressUnknownHrp(address))
        .isInstanceOf(DeserializeException.class)
        .hasRootCauseInstanceOf(Bech32DecodeException.class)
        .hasRootCauseMessage("Address was bech32 encoded, not bech32m");
  }
}
