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

package com.radixdlt.rev2.nofaucet;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static com.radixdlt.lang.Tuple.tuple;
import static org.junit.Assert.assertEquals;

import com.google.common.collect.ImmutableList;
import com.google.inject.Module;
import com.radixdlt.addressing.Addressing;
import com.radixdlt.api.core.generated.api.TransactionApi;
import com.radixdlt.api.core.generated.client.ApiException;
import com.radixdlt.api.core.generated.models.*;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.*;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.identifiers.Address;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.ScryptoConstants;
import com.radixdlt.rev2.protocol.ProtocolUpdateTestUtils;
import com.radixdlt.utils.UInt32;
import com.radixdlt.utils.UInt64;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

public final class CallPreviewWithoutFaucetTest {
  @Rule public TemporaryFolder folder = new TemporaryFolder();

  private DeterministicTest createTest(Module... extraModules) {
    final var validatorKey = ECKeyPair.fromSeed(new byte[] {0x02}).getPublicKey();

    var genesisData =
        new GenesisData(
            UInt64.fromNonNegativeLong(1L),
            0,
            GenesisConsensusManagerConfig.testingDefaultEmpty(),
            ImmutableList.of(
                new GenesisDataChunk.Validators(
                    ImmutableList.of(
                        new GenesisValidator(
                            validatorKey,
                            true,
                            true,
                            Decimal.ZERO,
                            ImmutableList.of(),
                            Address.virtualAccountAddress(validatorKey)))),
                new GenesisDataChunk.Stakes(
                    ImmutableList.of(Address.virtualAccountAddress(validatorKey)),
                    ImmutableList.of(
                        tuple(
                            validatorKey,
                            ImmutableList.of(
                                new GenesisStakeAllocation(
                                    UInt32.fromNonNegativeInt(0), Decimal.ONE)))))),
            Decimal.ofNonNegative(0L),
            ImmutableList.of());

    return DeterministicTest.builder()
        .addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
        .messageSelector(firstSelector())
        .messageMutator(MessageMutator.dropTimeouts())
        .addModules(extraModules)
        .functionalNodeModule(
            new FunctionalRadixNodeModule(
                FunctionalRadixNodeModule.NodeStorageConfig.tempFolder(folder),
                true,
                FunctionalRadixNodeModule.SafetyRecoveryConfig.BERKELEY_DB,
                FunctionalRadixNodeModule.ConsensusConfig.of(1000),
                FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
                    StateComputerConfig.rev2().withGenesis(genesisData))));
  }

  @Test
  public void call_preview_works_without_faucet() throws ApiException {
    // Parameters corresponding to the following struct passed as input:
    //    FungibleResourceManagerCreateWithInitialSupplyInput {
    //      owner_role: Default::default(),
    //      track_total_supply: Default::default(),
    //      divisibility: Default::default(),
    //      resource_roles: Default::default(),
    //      metadata: Default::default()
    //      address_reservation: Default::default(),
    //      initial_supply: Default::default(),
    //    }

    final var addressing = Addressing.ofNetwork(Network.INTEGRATIONTESTNET);
    final var accountLockerCall =
        new TransactionCallPreviewRequest()
            .network(Network.INTEGRATIONTESTNET.getLogicalName())
            .target(
                new BlueprintFunctionTargetIdentifier()
                    .packageAddress(addressing.encode(ScryptoConstants.RESOURCE_PACKAGE_ADDRESS))
                    .blueprintName("FungibleResourceManager")
                    .functionName("create")
                    .type(TargetIdentifierType.FUNCTION))
            .addArgumentsItem("4d220000")
            .addArgumentsItem("4d0100")
            .addArgumentsItem("4d0700")
            .addArgumentsItem("4d2106220000220000220000220000220000220000")
            .addArgumentsItem("4d2102230c2100230c2200")
            .addArgumentsItem("4d220000");

    final var coreApiHelper = new ProtocolUpdateTestUtils.CoreApiHelper();
    try (var test = createTest(coreApiHelper.module())) {
      // Arrange: Start a single node network
      test.startAllNodes();

      // Act: Preview a transaction
      final var callBeforeBottlenose =
          new TransactionApi(coreApiHelper.client()).transactionCallPreviewPost(accountLockerCall);

      // Assert: It should succeed despite empty faucet
      assertEquals(TransactionStatus.SUCCEEDED, callBeforeBottlenose.getStatus());
    }
  }
}
