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

package com.radixdlt;

import com.google.common.base.Stopwatch;
import com.google.inject.Guice;
import com.radixdlt.bootstrap.RadixNodeBootstrapper;
import com.radixdlt.bootstrap.RadixNodeBootstrapperModule;
import com.radixdlt.monitoring.ApplicationVersion;
import com.radixdlt.utils.MemoryLeakDetector;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.net.URISyntaxException;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class RadixNodeApplication {
  private static final Logger log = LogManager.getLogger();

  private RadixNodeApplication() {}

  static {
    System.setProperty("java.net.preferIPv4Stack", "true");
  }

  public static void main(String[] args) {
    try {
      MemoryLeakDetector.start();
      logVersion();
      dumpExecutionLocation();
      final var properties = RuntimeProperties.fromCommandLineArgs(args);
      bootstrapRadixNode(properties);
    } catch (Exception ex) {
      log.fatal("Unable to start", ex);
      LogManager.shutdown(); // Flush any async logs
      exitWithError();
    }
  }

  private static void bootstrapRadixNode(RuntimeProperties properties) {
    final var nodeBootStopwatch = Stopwatch.createStarted();
    final var bootstrapperModule =
        Guice.createInjector(new RadixNodeBootstrapperModule(properties));
    final var bootstrapper = bootstrapperModule.getInstance(RadixNodeBootstrapper.class);
    final var radixNodeBootstrapperHandle = bootstrapper.bootstrapRadixNode();
    /* Note that because some modules obtain the resources at construction (ORAC paradigm), this
     shutdown hook doesn't guarantee that these resources will be correctly freed up.
     For example, when an error occurs while Guice is building its object graph,
     we haven't yet received a reference (Injector) to the modules that have already been initialized,
     and thus we can't clean them up.
     TODO: consider refactoring the modules to follow a DNORAC (do NOT obtain resources at construction) paradigm
           and then re-evaluate if the shutdown hook below (and/or the need for RadixNodeBootstrapperHandle which
           provides the shutdown functionality) is still needed - for both happy (no errors during initialization)
           and unhappy (errors during initialization) paths.
    */
    Runtime.getRuntime().addShutdownHook(new Thread(radixNodeBootstrapperHandle::shutdown));
    radixNodeBootstrapperHandle
        .radixNodeFuture()
        .thenAccept(
            (unstartedRadixNode) -> {
              final var startupTime = nodeBootStopwatch.elapsed();
              final var runningNode = RunningRadixNode.run(unstartedRadixNode);
              log.info(
                  "Radix node {} started successfully in {} ms",
                  runningNode.self(),
                  startupTime.toMillis());
              runningNode.reportSelfStartupTime(startupTime);
              runningNode.installShutdownHook();
            })
        // Call .join() to block on the future completing, ensuring that errors during
        // bootstrapping are not swallowed, and propagate to the "Unable to start" handler.
        // In particular, errors can come from running genesis during guice initiation in
        // RunningRadixNode.run(..);
        .join();
  }

  private static void exitWithError() {
    // When this happens, the integration test errors look like:
    // "Process 'Gradle Test Executor 1' finished with non-zero exit value 255"
    java.lang.System.exit(-1);
  }

  private static void logVersion() {
    log.always()
        .log(
            "Radix distributed ledger '{}' from branch '{}' commit '{}'",
            ApplicationVersion.INSTANCE.display(),
            ApplicationVersion.INSTANCE.branch(),
            ApplicationVersion.INSTANCE.commit());
  }

  private static void dumpExecutionLocation() {
    try {
      String jarFile =
          RadixNodeApplication.class
              .getProtectionDomain()
              .getCodeSource()
              .getLocation()
              .toURI()
              .getPath();
      System.setProperty("radix.jar", jarFile);

      String jarPath = jarFile;

      if (jarPath.toLowerCase().endsWith(".jar")) {
        jarPath = jarPath.substring(0, jarPath.lastIndexOf('/'));
      }
      System.setProperty("radix.jar.path", jarPath);

      log.debug("Execution file: {}", System.getProperty("radix.jar"));
      log.debug("Execution path: {}", System.getProperty("radix.jar.path"));
    } catch (URISyntaxException e) {
      throw new IllegalStateException("Error while fetching execution location", e);
    }
  }
}
