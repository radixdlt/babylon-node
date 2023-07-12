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

package com.radixdlt.p2p.hostip;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Suppliers;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;
import com.google.common.net.HostAndPort;
import com.radixdlt.utils.properties.RuntimeProperties;
import java.net.MalformedURLException;
import java.net.URL;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Supplier;
import okhttp3.*;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Query for a public IP address using an oracle. This class can be used to query a single oracle,
 * or if a number of oracles are provided, a simple majority vote is used.
 */
final class NetworkQueryHostIp {
  private static final Logger log = LogManager.getLogger();

  public record Result(Optional<HostIp> maybeHostIp, ImmutableList<URL> hostsQueried) {}

  @VisibleForTesting static final String QUERY_URLS_PROPERTY = "network.host_ip_query_urls";

  @VisibleForTesting
  static final ImmutableList<URL> DEFAULT_QUERY_URLS =
      ImmutableList.of(
          makeurl("https://checkip.amazonaws.com/"),
          makeurl("https://ipv4.icanhazip.com/"),
          makeurl("https://myexternalip.com/raw"),
          makeurl("https://ipecho.net/plain"),
          makeurl("https://ifconfig.me"),
          makeurl("https://www.trackip.net/ip"),
          makeurl("https://ifconfig.co/ip"));

  static NetworkQueryHostIp create(Collection<URL> urls) {
    return new NetworkQueryHostIp(urls);
  }

  static NetworkQueryHostIp create(RuntimeProperties properties) {
    String urlsProperty = properties.get(QUERY_URLS_PROPERTY, "");
    if (urlsProperty == null || urlsProperty.trim().isEmpty()) {
      return create(DEFAULT_QUERY_URLS);
    }
    ImmutableList<URL> urls =
        Arrays.asList(urlsProperty.split(",")).stream()
            .map(NetworkQueryHostIp::makeurl)
            .collect(ImmutableList.toImmutableList());
    return create(urls);
  }

  private final List<URL> hosts;
  private final OkHttpClient okHttpClient;
  private final Supplier<Result> result = Suppliers.memoize(this::get);

  NetworkQueryHostIp(Collection<URL> urls) {
    if (urls.isEmpty()) {
      throw new IllegalArgumentException("At least one URL must be specified");
    }
    this.hosts = new ArrayList<>(urls);
    this.okHttpClient = new OkHttpClient.Builder().build();
  }

  int count() {
    return this.hosts.size();
  }

  public Result queryNetworkHosts() {
    return result.get();
  }

  Result get() {
    return publicIp((count() + 1) / 2); // Round up
  }

  Result publicIp(int threshold) {
    // Make sure we don't DoS the first one on the list
    Collections.shuffle(this.hosts);
    log.debug("Using hosts {}", this.hosts);
    final Map<HostAndPort, AtomicInteger> ips = Maps.newHashMap();
    final ImmutableList.Builder<URL> hostsQueried = ImmutableList.builder();
    for (URL url : this.hosts) {
      HostAndPort q = query(url);
      if (q != null) {
        hostsQueried.add(url);
        int newValue = ips.computeIfAbsent(q, k -> new AtomicInteger()).incrementAndGet();
        if (newValue >= threshold) {
          log.debug("Found address {}", q);
          return new Result(Optional.of(new HostIp(q.getHost())), hostsQueried.build());
        }
      }
    }
    log.debug("No suitable address found");
    return new Result(Optional.empty(), hostsQueried.build());
  }

  HostAndPort query(URL url) {
    try {
      final var response =
          okHttpClient
              .newCall(
                  new Request.Builder()
                      .url(url)
                      .header(
                          "User-Agent", "curl/7.58.0") // User agent is required by some services
                      .header(
                          "Accept", "*/*") // Similarly, this seems to be required by some services
                      .get()
                      .build())
              .execute();
      return HostAndPort.fromHost(response.body().string().trim());
    } catch (Exception ex) {
      // We don't want any single query to throw an uncaught exception
      // (e.g. if they return an invalid response), so we're just catching all here.
      return null;
    }
  }

  private static URL makeurl(String s) {
    try {
      return new URL(s);
    } catch (MalformedURLException ex) {
      throw new IllegalStateException("While constructing URL for " + s, ex);
    }
  }
}
