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

package com.radixdlt.p2p.transport.handshake;

import static com.radixdlt.utils.Bytes.bigIntegerToBytes;
import static com.radixdlt.utils.Bytes.xor;
import static org.junit.Assert.*;

import com.google.common.hash.HashCode;
import com.radixdlt.crypto.*;
import com.radixdlt.p2p.capability.Capabilities;
import com.radixdlt.p2p.capability.LedgerSyncCapability;
import com.radixdlt.p2p.capability.RemotePeerCapability;
import com.radixdlt.p2p.transport.handshake.AuthHandshakeResult.AuthHandshakeError;
import com.radixdlt.p2p.transport.handshake.AuthHandshakeResult.AuthHandshakeSuccess;
import com.radixdlt.serialization.DefaultSerialization;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.Serialization;
import io.netty.buffer.Unpooled;
import java.lang.reflect.Field;
import java.nio.ByteBuffer;
import java.security.SecureRandom;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import org.junit.Assert;
import org.junit.Test;

public final class AuthHandshakerTest {
  private final Serialization serialization = DefaultSerialization.getInstance();
  private final SecureRandom secureRandom = new SecureRandom();
  private final Capabilities capabilities =
      new Capabilities(LedgerSyncCapability.Builder.asDefault().build());

  @Test
  public void test_auth_handshake() throws Exception {
    final var nodeKey1 = ECKeyPair.generateNew();
    final var nodeKey2 = ECKeyPair.generateNew();

    Capabilities peer1Capabilities =
        new Capabilities(new LedgerSyncCapability.Builder(true).build());

    final var handshaker1 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey1),
            (byte) 0x01,
            "fork1",
            peer1Capabilities);
    final var handshaker2 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey2),
            (byte) 0x01,
            "fork1",
            capabilities);

    final var initMessage = handshaker1.initiate(nodeKey2.getPublicKey());
    final var handshaker2ResultPair =
        handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));
    final var handshaker2Result = (AuthHandshakeSuccess) handshaker2ResultPair.getSecond();
    final var responseMessage = handshaker2ResultPair.getFirst();
    final var handshaker1Result =
        (AuthHandshakeSuccess)
            handshaker1.handleResponseMessage(Unpooled.wrappedBuffer(responseMessage));

    assertArrayEquals(handshaker1Result.secrets().aes, handshaker2Result.secrets().aes);
    assertArrayEquals(handshaker1Result.secrets().mac, handshaker2Result.secrets().mac);
    assertArrayEquals(handshaker1Result.secrets().token, handshaker2Result.secrets().token);

    Assert.assertEquals(
        Set.of(new RemotePeerCapability(LedgerSyncCapability.NAME, Map.of())),
        handshaker2Result.remotePeerCapabilities());
  }

  @Test
  public void test_auth_handshake_when_ledger_sync_is_disabled() throws Exception {
    final var nodeKey1 = ECKeyPair.generateNew();
    final var nodeKey2 = ECKeyPair.generateNew();

    Capabilities peer1Capabilities =
        new Capabilities(new LedgerSyncCapability.Builder(false).build());

    final var handshaker1 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey1),
            (byte) 0x01,
            "fork1",
            peer1Capabilities);
    final var handshaker2 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey2),
            (byte) 0x01,
            "fork1",
            capabilities);

    final var initMessage = handshaker1.initiate(nodeKey2.getPublicKey());
    final var handshaker2ResultPair =
        handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));
    final var handshaker2Result = (AuthHandshakeSuccess) handshaker2ResultPair.getSecond();
    final var responseMessage = handshaker2ResultPair.getFirst();
    final var handshaker1Result =
        (AuthHandshakeSuccess)
            handshaker1.handleResponseMessage(Unpooled.wrappedBuffer(responseMessage));

    assertArrayEquals(handshaker1Result.secrets().aes, handshaker2Result.secrets().aes);
    assertArrayEquals(handshaker1Result.secrets().mac, handshaker2Result.secrets().mac);
    assertArrayEquals(handshaker1Result.secrets().token, handshaker2Result.secrets().token);

    Assert.assertEquals(Set.of(), handshaker2Result.remotePeerCapabilities());
  }

  @Test
  public void test_auth_handshake_fail_on_network_id_mismatch() {
    final var nodeKey1 = ECKeyPair.generateNew();
    final var nodeKey2 = ECKeyPair.generateNew();

    final var handshaker1 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey1),
            (byte) 0x01,
            "fork1",
            capabilities);
    final var handshaker2 =
        new AuthHandshaker(
            serialization,
            secureRandom,
            ECKeyOps.fromKeyPair(nodeKey2),
            (byte) 0x02,
            "fork1",
            capabilities);

    final var initMessage = handshaker1.initiate(nodeKey2.getPublicKey());
    final var handshaker2ResultPair =
        handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));
    assertTrue(handshaker2ResultPair.getSecond() instanceof AuthHandshakeError);
    assertArrayEquals(new byte[] {0x02}, handshaker2ResultPair.getFirst());
  }

  @Test
  public void test_auth_handshake_fail_when_capability_count_gt_max_allowed() {
    try {

      var forkName = "fork1";
      var networkId = (byte) 0x01;
      final var nodeKey1 = ECKeyPair.generateNew();
      final var nodeKey2 = ECKeyPair.generateNew();
      var remotePeerCapabilities = new HashSet<RemotePeerCapability>();
      for (var lc = 0; lc < Capabilities.MAX_NUMBER_OF_CAPABILITIES_ACCEPTED + 1; lc++) {
        remotePeerCapabilities.add(new RemotePeerCapability("DUMMY_CAPABILITY" + lc, Map.of()));
      }

      final var handshaker1 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey1),
              networkId,
              forkName,
              capabilities);
      final var handshaker2 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey2),
              networkId,
              forkName,
              capabilities);

      var initMessage =
          initiate(nodeKey2, handshaker1, remotePeerCapabilities, networkId, forkName);
      var handshaker2ResultPair =
          handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));

      assertTrue(
          "Handshake should fail when number of capabilities exceed allowed maximum",
          handshaker2ResultPair.getSecond() instanceof AuthHandshakeError);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    }
  }

  @Test
  public void test_auth_handshake_ok_when_capability_has_config_values() {
    try {
      var forkName = "fork1";
      var networkId = (byte) 0x01;
      final var nodeKey1 = ECKeyPair.generateNew();
      final var nodeKey2 = ECKeyPair.generateNew();
      var remotePeerCapabilities =
          Set.of(
              new RemotePeerCapability(
                  LedgerSyncCapability.NAME,
                  createConfigMap(RemotePeerCapability.CONFIGURATION_MAP_MAX_SIZE)));

      final var handshaker1 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey1),
              networkId,
              forkName,
              capabilities);
      final var handshaker2 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey2),
              networkId,
              forkName,
              capabilities);

      var initMessage =
          initiate(nodeKey2, handshaker1, remotePeerCapabilities, networkId, forkName);
      var handshaker2ResultPair =
          handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));
      var handshaker2Result = (AuthHandshakeSuccess) handshaker2ResultPair.getSecond();
      var capability =
          handshaker2Result.remotePeerCapabilities().stream()
              .filter(c -> c.getName().equals(LedgerSyncCapability.NAME))
              .findFirst()
              .get();

      assertTrue(
          String.format(
              "Capability count incorrect. Expecting (%d), actual (%d)",
              capability.getConfiguration().size(),
              RemotePeerCapability.CONFIGURATION_MAP_MAX_SIZE),
          capability.getConfiguration().size() == RemotePeerCapability.CONFIGURATION_MAP_MAX_SIZE);

    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    }
  }

  @Test
  public void test_auth_handshake_fail_when_capability_config_count_gt_max_allowed() {
    try {

      var forkName = "fork1";
      var networkId = (byte) 0x01;
      final var nodeKey1 = ECKeyPair.generateNew();
      final var nodeKey2 = ECKeyPair.generateNew();
      var remotePeerCapabilities =
          Set.of(
              new RemotePeerCapability(
                  LedgerSyncCapability.NAME,
                  createConfigMap(RemotePeerCapability.CONFIGURATION_MAP_MAX_SIZE + 1)),
              new RemotePeerCapability("DUMMY_CAPABILITY", Map.of()));

      final var handshaker1 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey1),
              networkId,
              forkName,
              capabilities);
      final var handshaker2 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey2),
              networkId,
              forkName,
              capabilities);

      var initMessage =
          initiate(nodeKey2, handshaker1, remotePeerCapabilities, networkId, forkName);
      var handshaker2ResultPair =
          handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));

      assertTrue(
          "Handshake should fail when number of capability configurations exceeds allowed maximum",
          handshaker2ResultPair.getSecond() instanceof AuthHandshakeError);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    }
  }

  @Test
  public void test_auth_handshake_ok_when_capability_config_name_gt_max_allowed() {
    try {

      var forkName = "fork1";
      var networkId = (byte) 0x01;
      final var nodeKey1 = ECKeyPair.generateNew();
      final var nodeKey2 = ECKeyPair.generateNew();
      var remotePeerCapabilities =
          Set.of(
              new RemotePeerCapability(
                  LedgerSyncCapability.NAME,
                  Map.of(
                      "N".repeat(RemotePeerCapability.CONFIGURATION_MAX_NAME_SIZE + 1), "value")));

      final var handshaker1 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey1),
              networkId,
              forkName,
              capabilities);
      final var handshaker2 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey2),
              networkId,
              forkName,
              capabilities);

      var initMessage =
          initiate(nodeKey2, handshaker1, remotePeerCapabilities, networkId, forkName);
      var handshaker2ResultPair =
          handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));

      assertTrue(
          "Handshake should fail when a capability config name exceeds maximum allowed length",
          handshaker2ResultPair.getSecond() instanceof AuthHandshakeError);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    }
  }

  @Test
  public void test_auth_handshake_ok_when_capability_config_value_gt_max_allowed() {
    try {

      var forkName = "fork1";
      var networkId = (byte) 0x01;
      final var nodeKey1 = ECKeyPair.generateNew();
      final var nodeKey2 = ECKeyPair.generateNew();
      var remotePeerCapabilities =
          Set.of(
              new RemotePeerCapability(
                  LedgerSyncCapability.NAME,
                  Map.of(
                      "configName",
                      "V".repeat(RemotePeerCapability.CONFIGURATION_MAX_VALUE_SIZE + 1))));

      final var handshaker1 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey1),
              networkId,
              forkName,
              capabilities);
      final var handshaker2 =
          new AuthHandshaker(
              serialization,
              secureRandom,
              ECKeyOps.fromKeyPair(nodeKey2),
              networkId,
              forkName,
              capabilities);

      var initMessage =
          initiate(nodeKey2, handshaker1, remotePeerCapabilities, networkId, forkName);
      var handshaker2ResultPair =
          handshaker2.handleInitialMessage(Unpooled.wrappedBuffer(initMessage));

      assertTrue(
          "Handshake should fail when a capability config name exceeds maximum allowed length",
          handshaker2ResultPair.getSecond() instanceof AuthHandshakeError);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    }
  }

  /*
   The AuthHandshaker->initiate and createAuthInitiateMessage methods have been copied here and tweaked to allow test cases to create an auth init message
   with a custom RemotePeerCapability set. The init message will be correctly signed, so the target handshaker handleInitialMessage method can validate
   and decode / decrypt it.
  */
  private byte[] initiate(
      ECKeyPair remoteKey,
      AuthHandshaker authHandshaker,
      Set<RemotePeerCapability> remotePeerCapabilities,
      int networkId,
      String forkName)
      throws Exception {
    var remotePubKey = remoteKey.getPublicKey();
    final var message =
        createAuthInitiateMessage(
            remoteKey, authHandshaker, remotePeerCapabilities, networkId, forkName);
    final var encoded = serialization.toDson(message, DsonOutput.Output.WIRE);
    final var MIN_PADDING = 100;
    final var MAX_PADDING = 300;
    final var padding = new byte[secureRandom.nextInt(MAX_PADDING - MIN_PADDING) + MIN_PADDING];
    secureRandom.nextBytes(padding);
    final var padded = new byte[encoded.length + padding.length];
    System.arraycopy(encoded, 0, padded, 0, encoded.length);
    System.arraycopy(padding, 0, padded, encoded.length, padding.length);

    final var encryptedSize = padded.length + ECIESCoder.OVERHEAD_SIZE;
    final var sizePrefix = ByteBuffer.allocate(2).putShort((short) encryptedSize).array();
    final var encryptedPayload = ECIESCoder.encrypt(remotePubKey.getEcPoint(), padded, sizePrefix);
    final var packet = new byte[sizePrefix.length + encryptedPayload.length];
    System.arraycopy(sizePrefix, 0, packet, 0, sizePrefix.length);
    System.arraycopy(encryptedPayload, 0, packet, sizePrefix.length, encryptedPayload.length);

    return packet;
  }

  private AuthInitiateMessage createAuthInitiateMessage(
      ECKeyPair remoteKey,
      AuthHandshaker authHandshaker,
      Set<RemotePeerCapability> remotePeerCapabilities,
      int networkId,
      String forkName)
      throws Exception {
    Field AuthHandshakerField = AuthHandshaker.class.getDeclaredField("nonce");
    AuthHandshakerField.setAccessible(true);
    byte[] nonce = (byte[]) AuthHandshakerField.get(authHandshaker);

    AuthHandshakerField = AuthHandshaker.class.getDeclaredField("ephemeralKey");
    AuthHandshakerField.setAccessible(true);
    ECKeyPair ephemeralKey = (ECKeyPair) AuthHandshakerField.get(authHandshaker);

    var ecKeyOps = ECKeyOps.fromKeyPair(remoteKey);
    var remotePubKey = remoteKey.getPublicKey();

    final var sharedSecret = bigIntegerToBytes(ecKeyOps.ecdhAgreement(remotePubKey), 32);
    final var messageToSign = xor(sharedSecret, nonce);
    final var signature = ephemeralKey.sign(messageToSign);

    // Instantiate with null capabilities.
    var authInitMessage =
        new AuthInitiateMessage(
            signature,
            HashCode.fromBytes(ecKeyOps.nodePubKey().getBytes()),
            HashCode.fromBytes(nonce),
            networkId,
            forkName,
            null);

    // Set the capabilities using reflections. This will bypass the BaseHandshakeMessage class
    // validation, which we want
    // to do when sending capabilities that exceed the specified limits.
    AuthHandshakerField = BaseHandshakeMessage.class.getDeclaredField("capabilities");
    AuthHandshakerField.setAccessible(true);
    AuthHandshakerField.set(authInitMessage, remotePeerCapabilities);
    return authInitMessage;
  }

  private Map<String, String> createConfigMap(int numbOfEntries) {
    var configMap = new HashMap<String, String>();
    for (var lc = 0; lc < numbOfEntries; lc++) {
      configMap.put("config" + lc, "value" + lc);
    }
    return configMap;
  }
}
