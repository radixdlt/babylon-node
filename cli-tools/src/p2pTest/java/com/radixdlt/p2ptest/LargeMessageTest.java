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

package com.radixdlt.p2ptest;

import static com.radixdlt.shell.RadixShell.nodeBuilder;
import static org.awaitility.Awaitility.await;

import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.crypto.RadixKeyStore;
import com.radixdlt.crypto.exception.KeyStoreException;
import com.radixdlt.networks.Network;
import com.radixdlt.p2p.RadixNodeUri;
import java.io.File;
import java.io.IOException;
import java.net.URI;
import java.nio.file.Files;
import java.security.Security;
import java.time.Duration;
import java.util.List;
import java.util.Random;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.awaitility.core.ConditionTimeoutException;
import org.bouncycastle.jce.provider.BouncyCastleProvider;

public final class LargeMessageTest {
  private static final Logger log = LogManager.getLogger();

  public static final class TargetNodeMain {
    private static final Logger log = LogManager.getLogger();

    public static void main(String[] args) throws Exception {
      Security.insertProviderAt(new BouncyCastleProvider(), 1);

      log.warn("Starting the target node...");

      nodeBuilder().p2pServer(Integer.parseInt(args[0])).prop("node.key.path", args[1]).build();
    }
  }

  public static final class AttackerNodeMain {
    private static final Logger log = LogManager.getLogger();

    public static void main(String[] args) throws Exception {
      Security.insertProviderAt(new BouncyCastleProvider(), 1);

      log.warn("Starting the attacker node...");

      final var node =
          nodeBuilder().p2pServer(Integer.parseInt(args[0])).prop("node.key.path", args[1]).build();

      final var targetUri = RadixNodeUri.fromUri(URI.create(args[2]));
      log.warn("Targeting: " + targetUri);

      // Connect to the target
      node.connectTo(targetUri);

      // Await for the connection
      await().until(() -> node.peers().size() != 1);
      log.warn("Target node connected");

      log.warn("Allocating 1G of random bytes...");
      final var rnd = new Random();
      final var bytes = new byte[1_000_000_000];
      rnd.nextBytes(bytes);
      log.warn("Allocated, sending to target...");
      node.peers().get(0).unsafeRawWriteToChannel(bytes);
      log.warn("Sent");
    }
  }

  public static void main(String[] args) throws Exception {
    // Target node process
    final var targetNodePort = FreePortFinder.findFreeLocalPort();
    final var targetKeyPair = ECKeyPair.generateNew();
    final var targetKeyPath = writeKeyPairToTempFolder(targetKeyPair);
    final var targetProc =
        JavaProcess.exec(
            TargetNodeMain.class,
            "128m", // Just 128m of memory
            List.of(Integer.toString(targetNodePort), targetKeyPath.getAbsolutePath()));

    final var targetUri =
        RadixNodeUri.fromPubKeyAndAddress(
            Network.LOCALNET.getId(), targetKeyPair.getPublicKey(), "127.0.0.1", targetNodePort);

    // Attacker node process
    final var attackerNodePort = FreePortFinder.findFreeLocalPort();
    final var attackerKeyPair = ECKeyPair.generateNew();
    final var attackerKeyPath = writeKeyPairToTempFolder(attackerKeyPair);
    final var attackerProc =
        JavaProcess.exec(
            AttackerNodeMain.class,
            "3g",
            List.of(
                Integer.toString(attackerNodePort),
                attackerKeyPath.getAbsolutePath(),
                targetUri.toString()));

    Runtime.getRuntime()
        .addShutdownHook(
            new Thread(
                () -> {
                  targetProc.destroy();
                  attackerProc.destroy();
                }));

    // The target process should survive
    try {
      // TODO: 15s is enough, but need to find a better way to know when the attack has finished
      await().atMost(Duration.ofSeconds(15)).until(() -> !targetProc.isAlive());
      log.error("TEST FAILED: TARGET PROCESS KILLED!");
    } catch (ConditionTimeoutException e) {
      log.warn("The target node didn't crash! The test succeeded.");
    }
  }

  private static File writeKeyPairToTempFolder(ECKeyPair keyPair)
      throws IOException, KeyStoreException {
    final var keystorePath =
        new File(Files.createTempDirectory("radix-shell-node-key-").toString(), "node-keystore.ks");
    try (var ks = RadixKeyStore.fromFile(keystorePath, "radix".toCharArray(), true)) {
      ks.writeKeyPair("node", keyPair);
    }
    return keystorePath;
  }
}
