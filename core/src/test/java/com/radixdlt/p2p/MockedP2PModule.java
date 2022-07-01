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

package com.radixdlt.p2p;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.AbstractModule;
import com.radixdlt.consensus.bft.BFTNode;
import com.radixdlt.crypto.ECPublicKey;
import com.radixdlt.network.GetVerticesRequestRateLimit;
import com.radixdlt.network.p2p.NoOpPeerControl;
import com.radixdlt.network.p2p.PeerControl;
import com.radixdlt.network.p2p.addressbook.AddressBookPersistence;
import java.util.List;

public class MockedP2PModule extends AbstractModule {

  private final Builder builder;

  private MockedP2PModule(Builder builder) {
    this.builder = builder;
  }

  @Override
  protected void configure() {
    bind(RateLimiter.class)
        .annotatedWith(GetVerticesRequestRateLimit.class)
        .toInstance(this.builder.rateLimiter);

    bind(PeerControl.class).toInstance(builder.peerControl);

    // Not adding a method to customize it for now as all tests use this version
    var addressBookPersistence = mock(AddressBookPersistence.class);
    when(addressBookPersistence.getAllEntries()).thenReturn(ImmutableList.of());
    bind(AddressBookPersistence.class).toInstance(addressBookPersistence);

    MockedPeersViewModule mockedPeersViewModule;
    if (builder.peersByNode != null) {
      mockedPeersViewModule = new MockedPeersViewModule(this.builder.peersByNode);
    } else if (this.builder.allNodes != null) {
      mockedPeersViewModule = new MockedPeersViewModule(this.builder.allNodes);
    } else {
      mockedPeersViewModule = new MockedPeersViewModule(List.of());
    }

    install(mockedPeersViewModule);
  }

  public static class Builder {
    private RateLimiter rateLimiter;
    private ImmutableMap<ECPublicKey, ImmutableList<ECPublicKey>> peersByNode;
    private List<BFTNode> allNodes;
    private PeerControl peerControl;

    public Builder() {
      this.rateLimiter = unlimitedRateLimiter();
      this.peerControl = new NoOpPeerControl();
    }

    public Builder withDefaultRateLimit() {
      this.rateLimiter = RateLimiter.create(50.0);
      return this;
    }

    public Builder withRateLimit(double permitsPerSecond) {
      this.rateLimiter = RateLimiter.create(permitsPerSecond);
      return this;
    }

    public Builder withPeersByNode(
        ImmutableMap<ECPublicKey, ImmutableList<ECPublicKey>> peersByNode) {
      this.peersByNode = peersByNode;
      return this;
    }

    public Builder withAllNodes(List<BFTNode> allNodes) {
      this.allNodes = allNodes;
      return this;
    }

    public MockedP2PModule build() {
      return new MockedP2PModule(this);
    }
  }

  private static RateLimiter unlimitedRateLimiter() {
    return RateLimiter.create(Double.MAX_VALUE);
  }
}
