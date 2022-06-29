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

import java.io.IOException;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.util.NoSuchElementException;
import java.util.concurrent.atomic.AtomicInteger;

/** Finds currently available server ports. */
public final class FreePortFinder {

  /**
   * The minimum server currentMinPort number for IPv4. Set at 1100 to avoid returning privileged
   * currentMinPort numbers.
   */
  public static final int MIN_PORT_NUMBER = 1100;

  /** The maximum server currentMinPort number for IPv4. */
  public static final int MAX_PORT_NUMBER = 65535;

  /**
   * We'll hold open the lowest port in this process so parallel processes won't use the same block
   * of ports. They'll go up to the next block.
   */
  private static final ServerSocket LOCK;

  /** Incremented to the next lowest available port when findFreeLocalPort() is called. */
  private static AtomicInteger currentMinPort = new AtomicInteger(MIN_PORT_NUMBER);

  /** Creates a new instance. */
  private FreePortFinder() {
    // Do nothing
  }

  static {
    int port = MIN_PORT_NUMBER;
    ServerSocket ss = null;

    while (ss == null) {
      try {
        ss = new ServerSocket(port);
      } catch (Exception e) {
        ss = null;
        port += 200;
      }
    }
    LOCK = ss;
    Runtime.getRuntime()
        .addShutdownHook(
            new Thread() {
              public void run() {
                try {
                  LOCK.close();
                } catch (Exception ex) {
                  // ignore
                }
              }
            });
    currentMinPort.set(port + 1);
  }

  /**
   * Gets the next available port starting at the lowest number. This is the preferred method to
   * use. The port return is immediately marked in use and doesn't rely on the caller actually
   * opening the port.
   *
   * @throws IllegalArgumentException is thrown if the port number is out of range
   * @throws NoSuchElementException if there are no ports available
   * @return the available port
   */
  public static synchronized int findFreeLocalPort() {
    return findFreeLocalPort(null);
  }

  /**
   * Gets the next available port starting at the lowest number. This is the preferred method to
   * use. The port return is immediately marked in use and doesn't rely on the caller actually
   * opening the port.
   *
   * @param bindAddress the address that will try to bind
   * @throws IllegalArgumentException is thrown if the port number is out of range
   * @throws NoSuchElementException if there are no ports available
   * @return the available port
   */
  public static synchronized int findFreeLocalPort(InetAddress bindAddress) {
    int next = findFreeLocalPort(currentMinPort.get(), bindAddress);
    currentMinPort.set(next + 1);
    return next;
  }

  /**
   * Gets the next available port starting at the lowest number. This is the preferred method to
   * use. The port return is immediately marked in use and doesn't rely on the caller actually
   * opening the port.
   *
   * @throws IllegalArgumentException is thrown if the port number is out of range
   * @throws NoSuchElementException if there are no ports available
   * @return the available port
   */
  public static synchronized int findFreeLocalPort(int fromPort) {
    return findFreeLocalPort(fromPort, null);
  }

  /**
   * Gets the next available port starting at a given from port.
   *
   * @param fromPort the from port to scan for availability
   * @param bindAddress the address that will try to bind
   * @throws IllegalArgumentException is thrown if the port number is out of range
   * @throws NoSuchElementException if there are no ports available
   * @return the available port
   */
  public static synchronized int findFreeLocalPort(int fromPort, InetAddress bindAddress) {
    if (fromPort < currentMinPort.get() || fromPort > MAX_PORT_NUMBER) {
      throw new IllegalArgumentException("From port number not in valid range: " + fromPort);
    }

    for (int i = fromPort; i <= MAX_PORT_NUMBER; i++) {
      if (available(i, bindAddress)) {
        return i;
      }
    }

    throw new NoSuchElementException("Could not find an available port above " + fromPort);
  }

  /**
   * Gets the next available port starting at a given from port.
   *
   * @param bindAddresses the addresses that will try to bind
   * @throws IllegalArgumentException is thrown if the port number is out of range
   * @throws NoSuchElementException if there are no ports available
   * @return the available port
   */
  public static synchronized int findFreeLocalPortOnAddresses(InetAddress... bindAddresses) {
    int fromPort = currentMinPort.get();
    if (fromPort < currentMinPort.get() || fromPort > MAX_PORT_NUMBER) {
      throw new IllegalArgumentException("From port number not in valid range: " + fromPort);
    }
    if (bindAddresses != null) {
      for (int j = fromPort; j <= MAX_PORT_NUMBER; j++) {
        for (int i = 0; i < bindAddresses.length; i++) {
          if (available(j, bindAddresses[i])) {
            currentMinPort.set(j + 1);
            return j;
          }
        }
      }
    }

    throw new NoSuchElementException("Could not find an available port above " + fromPort);
  }
  /**
   * Checks to see if a specific port is available.
   *
   * @param port the port number to check for availability
   * @return <tt>true</tt> if the port is available, or <tt>false</tt> if not
   * @throws IllegalArgumentException is thrown if the port number is out of range
   */
  public static boolean available(int port) throws IllegalArgumentException {
    return available(port, null);
  }

  /**
   * Checks to see if a specific port is available.
   *
   * @param port the port number to check for availability
   * @param bindAddress the address that will try to bind
   * @return <tt>true</tt> if the port is available, or <tt>false</tt> if not
   * @throws IllegalArgumentException is thrown if the port number is out of range
   */
  public static boolean available(int port, InetAddress bindAddress)
      throws IllegalArgumentException {
    if (port < currentMinPort.get() || port > MAX_PORT_NUMBER) {
      throw new IllegalArgumentException("Invalid start currentMinPort: " + port);
    }

    ServerSocket ss = null;
    DatagramSocket ds = null;
    try {
      ss = (bindAddress != null) ? new ServerSocket(port, 50, bindAddress) : new ServerSocket(port);
      ss.setReuseAddress(true);
      ds = (bindAddress != null) ? new DatagramSocket(port, bindAddress) : new DatagramSocket(port);
      ds.setReuseAddress(true);
      return true;
    } catch (IOException e) {
      // Do nothing
    } finally {
      if (ds != null) {
        ds.close();
      }

      if (ss != null) {
        try {
          ss.close();
        } catch (IOException e) {
          /* should not be thrown */
        }
      }
    }

    return false;
  }
}
