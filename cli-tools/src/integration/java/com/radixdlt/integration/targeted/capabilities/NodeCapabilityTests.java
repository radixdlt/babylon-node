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

package com.radixdlt.integration.targeted.capabilities;

import static com.radixdlt.shell.RadixShell.nodeBuilder;
import static org.junit.Assert.*;

import com.radixdlt.monitoring.SystemCounters;
import com.radixdlt.monitoring.SystemCountersImpl;
import com.radixdlt.network.messages.StatusRequestMessage;
import com.radixdlt.network.messages.StatusResponseMessage;
import com.radixdlt.shell.RadixShell;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Map;
import org.junit.Test;

public class NodeCapabilityTests {

  static int nextNodePort = 30301;

  @Test
  public void ensure_a_disabled_capability_can_still_send_messages() {
    RadixShell.Node node1 = null;
    RadixShell.Node node2 = null;
    try {
      var expectedResultMap = new HashMap<String, Boolean>();
      // Contains capability messages and whether they are expected or not
      expectedResultMap.put("node2-StatusRequestMessage", true);

      var messagesReceived = new HashSet<String>();

      var n1Port = 30301;
      var n2Port = 30302;

      node1 = startNode(n1Port, false);
      node2 = startNode(n2Port, true);

      connectNodes(node1, node2);

      // Node 1 should send the statusRequestMessage to node 2.
      node2.onMsg(
          StatusRequestMessage.class,
          m -> messagesReceived.add("node2-" + m.message().getClass().getSimpleName()));

      // Has the expected message been received?
      Result result = waitForMessagesReceived(expectedResultMap, messagesReceived, 3);

      assertTrue(result.message, result.testOk);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    } finally {
      node1.stopP2PServer();
      node2.stopP2PServer();
    }
  }

  @Test
  public void ensure_disabled_messages_are_discarded() {
    RadixShell.Node node1 = null;
    RadixShell.Node node2 = null;
    try {
      var n1Port = 30301;
      var n2Port = 30302;

      // Start both nodes with capabilities disabled, so no StatusRequestMessages are sent.
      node1 = startNode(n1Port, false);
      node2 = startNode(n2Port, false);

      connectNodes(node1, node2);

      // Node 1 will not automatically send a status request message to Node2. So send one.
      node1.sendMsg(node2.self().getNodeId(), new StatusRequestMessage());

      var node2Counters = node2.getInstance(SystemCounters.class);

      // verify the discarded count is 1.
      var result =
          waitForCounterValueEquals(
              node2Counters, SystemCounters.CounterType.MESSAGES_INBOUND_DISCARDED, 1, 2);

      assertTrue(result.message, result.testOk);

    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    } finally {
      node1.stopP2PServer();
      node2.stopP2PServer();
    }
  }

  @Test
  public void ensure_messages_for_an_enabled_capability_are_processed() {
    RadixShell.Node node1 = null;
    RadixShell.Node node2 = null;
    try {
      var expectedResultMap = new HashMap<String, Boolean>();
      // Contains capability messages and whether they are expected or not
      expectedResultMap.put("node2-StatusRequestMessage", true);
      expectedResultMap.put("node1-StatusResponseMessage", true);

      var messagesReceived = new HashSet<String>();
      var n1Port = 30301;
      var n2Port = 30302;

      node1 = startNode(n1Port, true);
      node2 = startNode(n2Port, true);

      connectNodes(node1, node2);

      // Node 1 should send the StatusRequestMessage and Node 2 should reply with the
      // StatusResponseMessage
      node2.onMsg(
          StatusRequestMessage.class,
          m -> messagesReceived.add("node2-" + m.message().getClass().getSimpleName()));
      node1.onMsg(
          StatusResponseMessage.class,
          m -> messagesReceived.add("node1-" + m.message().getClass().getSimpleName()));

      // Have the expected messages been received?
      var result = waitForMessagesReceived(expectedResultMap, messagesReceived, 3);

      assertTrue(result.message, result.testOk);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    } finally {
      node1.stopP2PServer();
      node2.stopP2PServer();
    }
  }

  @Test
  public void ensure_capabilities_are_sent_in_the_handshake_message() {
    RadixShell.Node node1 = null;
    RadixShell.Node node2 = null;
    try {
      var capabilityName = "ledger-sync";
      var peerCapabilityFound = false;
      var messagesReceived = new HashSet<String>();
      var n1Port = 30301;
      var n2Port = 30302;

      node1 = startNode(n1Port, true);
      node2 = startNode(n2Port, true);

      var counters = node1.getInstance(SystemCountersImpl.class);
      connectNodes(node1, node2);

      // Verify that Node1 has ledger-sync as a remote peer capability for node2
      var node1Capabilities =
          node1.peers().stream()
              .filter(peer -> peer.getPort() == n2Port)
              .findFirst()
              .get()
              .getRemotePeerCapabilities();

      var capability =
          node1Capabilities.stream().filter(c -> c.getName().equals(capabilityName)).findFirst();

      if (!capability.isEmpty() && capability.get().getName().equals(capabilityName)) {
        peerCapabilityFound = true;
      }

      assertTrue(
          String.format("%s capability is not in node2 peer capability list", capabilityName),
          peerCapabilityFound);
    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    } finally {
      node1.stopP2PServer();
      node2.stopP2PServer();
    }
  }

  @Test
  public void ensure_nodes_are_aware_of_and_respect_other_node_capabilities() {
    RadixShell.Node node1 = null;
    RadixShell.Node node2 = null;
    RadixShell.Node node3 = null;
    try {
      // Contains capability messages and whether they are expected or not
      var expectedResultMap = new HashMap<String, Boolean>();
      expectedResultMap.put("node1-StatusRequestMessage", true);
      expectedResultMap.put("node2-StatusRequestMessage", true);
      expectedResultMap.put("node3-StatusRequestMessage", false);

      var messagesReceived = new HashSet<String>();
      var n1Port = 30301;
      var n2Port = 30302;
      var n3Port = 30303;

      node1 = startNode(n1Port, true);
      node2 = startNode(n2Port, true);
      node3 = startNode(n3Port, false);

      connectNodes(node1, node2);
      connectNodes(node1, node3);
      connectNodes(node2, node3);

      // Node's 1 and 2 should receive the StatusRequest message but node3 shouldn't.
      node1.onMsg(
          StatusRequestMessage.class,
          m -> messagesReceived.add("node1-" + m.message().getClass().getSimpleName()));
      node2.onMsg(
          StatusRequestMessage.class,
          m -> messagesReceived.add("node2-" + m.message().getClass().getSimpleName()));
      node3.onMsg(
          StatusRequestMessage.class,
          m -> messagesReceived.add("node3-" + m.message().getClass().getSimpleName()));

      // Have the expected messages been received?
      Result result = waitForMessagesReceived(expectedResultMap, messagesReceived, 2);
      assertTrue(result.message, result.testOk);

      var node3Counters = node2.getInstance(SystemCounters.class);

      // Ensure Node3 doesn't have a discarded message (i.e. Node's 1 and 2 shouldn't send a status
      // request message to node 3)
      result =
          waitForCounterValueEquals(
              node3Counters, SystemCounters.CounterType.MESSAGES_INBOUND_DISCARDED, 0, 1);

    } catch (Exception ex) {
      fail(String.format("Exception %s", ex.getMessage()));
    } finally {
      node1.stopP2PServer();
      node2.stopP2PServer();
      node3.stopP2PServer();
    }
  }

  private RadixShell.Node startNode(int port, boolean ledgerSyncEnabled) throws Exception {
    return nodeBuilder()
        .p2pServer(port)
        .ledgerSync()
        .prop("network.genesis_txn", "01")
        .prop("capabilities.ledger_sync.enabled", Boolean.toString(ledgerSyncEnabled))
        .build();
  }

  // Check whether the expected messages have been received / not received.
  private Result checkMessages(
      HashMap<String, Boolean> expectedResultMap, HashSet<String> messagesReceived) {
    Result result = new Result();
    result.testOk = true;
    Iterator it = expectedResultMap.entrySet().iterator();
    while (it.hasNext()) {
      Map.Entry pair = (Map.Entry) it.next();
      String message = pair.getKey().toString();
      Boolean shouldBeReceived = Boolean.valueOf(pair.getValue().toString());
      if (!messagesReceived.contains(message) && shouldBeReceived) {
        result.testOk = false;
        result.message += String.format("%s was NOT received but should have been; ", message);

      } else if (messagesReceived.contains(message) && !shouldBeReceived) {
        result.testOk = false;
        result.message += String.format("%s was received but should NOT have been; ", message);
      }
    }
    return result;
  }

  /*
  Repeatedly check to see if the expected messages have been received. Will retry every 100ms
  until either the messages have been found or the maxWaitTimeSecs is exceeded.
  If the expectedResultMap contains any messages that shouldn't be received, then the
  maxWaitTimeSecs becomes the actual time the method will retry before returning.
  In these cases, the maxWaitTime should be set to enough time to be confident a
  message isn't going to be received.
  */
  private Result waitForMessagesReceived(
      HashMap<String, Boolean> expectedResultMap,
      HashSet<String> messagesReceived,
      int maxWaitTimeSecs)
      throws InterruptedException {
    Result result = new Result();
    result.testOk = false;
    boolean maxWaitTimeIsActualWaitTime = false;

    // Are there any messages we want to ensure do not arrive? If so, the maxWaitTimeSecs becomes
    // the actual time to wait.
    if (expectedResultMap.containsValue(false)) {
      maxWaitTimeIsActualWaitTime = true;
    }

    int currentWaitTimeMs = 0;
    int sleepIntervalMs = 100;
    while (true) {
      result = checkMessages(expectedResultMap, messagesReceived);

      // If maxWaitTimeIsActualWaitTime is true, we want to ignore the result and only verify the
      // message(s) do not exist after the wait period.
      if (result.testOk && !maxWaitTimeIsActualWaitTime) {
        break;
      } else {
        if (currentWaitTimeMs >= (maxWaitTimeSecs * 1000)) {
          if (!maxWaitTimeIsActualWaitTime) {
            result.message = "Max duration (%s seconds) exceeded. " + result.message;
          }
          break;
        }
        Thread.sleep(sleepIntervalMs);
        currentWaitTimeMs += sleepIntervalMs;
      }
    }

    return result;
  }

  // Check a specified counter value. The value is checked every 100ms until either the value
  // matches, or the maxWaitTimeSecs expires
  private Result waitForCounterValueEquals(
      SystemCounters counters,
      SystemCounters.CounterType counterType,
      long expectedValue,
      int maxWaitTimeSecs)
      throws InterruptedException {
    Result result = new Result();
    result.testOk = false;
    int currentWaitTimeMs = 0;
    int sleepIntervalMs = 100;
    while (true) {
      if (counters.get(counterType) == expectedValue) {
        result.testOk = true;
        result.message =
            String.format("%s equals expected value: %s", counterType.name(), expectedValue);
        break;
      } else {
        if (currentWaitTimeMs >= (maxWaitTimeSecs * 1000)) {
          result.message =
              String.format(
                  "%s (%d) does NOT equal expected value: %d within the specified time %d seconds",
                  counterType.name(), counters.get(counterType), expectedValue, maxWaitTimeSecs);
          break;
        }
        Thread.sleep(sleepIntervalMs);
        currentWaitTimeMs += sleepIntervalMs;
      }
    }

    return result;
  }

  // Connect node1 to node2 and wait until the connection is complete (or fail if not completed
  // within x seconds)
  private void connectNodes(RadixShell.Node node1, RadixShell.Node node2)
      throws InterruptedException {
    node1.connectTo(node2.self());
    Boolean connected = false;
    var node2Port = node2.self().getPort();
    for (int lc = 0; lc < 5; lc++) {
      try {
        node1.peers().stream()
            .filter(peer -> peer.getPort() == node2Port)
            .findFirst()
            .get()
            .getRemotePeerCapabilities();
        connected = true;
        break;
      } catch (Exception ex) {
        Thread.sleep(500);
      }
    }
    if (!connected) {
      fail("Failed to connect within allocated time");
    }
  }

  private class Result {
    Boolean testOk = false;
    String message = "";
  }
}
