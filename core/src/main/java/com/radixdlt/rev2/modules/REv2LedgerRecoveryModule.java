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

package com.radixdlt.rev2.modules;

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Singleton;
import com.radixdlt.consensus.*;
import com.radixdlt.consensus.bft.*;
import com.radixdlt.consensus.vertexstore.VertexStoreState;
import com.radixdlt.crypto.Hasher;
import com.radixdlt.lang.Option;
import com.radixdlt.ledger.AccumulatorState;
import com.radixdlt.ledger.LedgerAccumulator;
import com.radixdlt.recovery.VertexStoreRecovery;
import com.radixdlt.rev2.REv2ToConsensus;
import com.radixdlt.serialization.DeserializeException;
import com.radixdlt.serialization.DsonOutput;
import com.radixdlt.serialization.Serialization;
import com.radixdlt.statecomputer.RustStateComputer;
import com.radixdlt.statecomputer.commit.CommitRequest;
import com.radixdlt.statecomputer.commit.PrepareGenesisRequest;
import com.radixdlt.store.LastEpochProof;
import com.radixdlt.store.LastProof;
import com.radixdlt.store.LastStoredProof;
import com.radixdlt.sync.TransactionsAndProofReader;
import com.radixdlt.transactions.RawLedgerTransaction;
import com.radixdlt.utils.UInt64;
import java.util.List;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class REv2LedgerRecoveryModule extends AbstractModule {

  private static final Logger log = LogManager.getLogger();
  private final AccumulatorState initialAccumulatorState;
  private final RawLedgerTransaction genesis;

  public REv2LedgerRecoveryModule(
      AccumulatorState initialAccumulatorState, RawLedgerTransaction genesis) {
    this.initialAccumulatorState = initialAccumulatorState;
    this.genesis = genesis;
  }

  @Provides
  @Singleton
  @LastStoredProof
  private LedgerProof lastProof(
      RustStateComputer stateComputer,
      Serialization serialization,
      TransactionsAndProofReader transactionsAndProofReader,
      LedgerAccumulator ledgerAccumulator) {
    final var timestamp = 0L; /* TODO: use Olympia end-state timestamp */
    return transactionsAndProofReader
        .getLastProof()
        .orElseGet(
            () -> {
              /* TODO: Move this into StateManager init() when Proof generation is supported by state_manager */
              var result = stateComputer.prepareGenesis(new PrepareGenesisRequest(genesis));
              var validatorSet =
                  result
                      .validatorSet()
                      .map(REv2ToConsensus::validatorSet)
                      .or((BFTValidatorSet) null);
              var accumulatorState =
                  ledgerAccumulator.accumulate(initialAccumulatorState, genesis.getPayloadHash());
              var proof =
                  LedgerProof.genesis(
                      accumulatorState, result.stateHash(), validatorSet, timestamp, timestamp);
              var commitRequest =
                  new CommitRequest(
                      List.of(genesis),
                      UInt64.fromNonNegativeLong(proof.getStateVersion()),
                      proof.getStateHash(),
                      serialization.toDson(proof, DsonOutput.Output.ALL),
                      Option.none());
              var commitResult = stateComputer.commit(commitRequest);
              commitResult.unwrap();

              return proof;
            });
  }

  @Provides
  @Singleton
  @LastProof
  LedgerProof lastProof(
      VertexStoreState vertexStoreState, @LastStoredProof LedgerProof lastStoredProof) {
    if (lastStoredProof.isEndOfEpoch()) {
      return vertexStoreState.getRootHeader();
    } else {
      return lastStoredProof;
    }
  }

  @Provides
  @Singleton
  @LastEpochProof
  LedgerProof lastEpochProof(
      TransactionsAndProofReader transactionsAndProofReader,
      @LastStoredProof LedgerProof lastStoredProof) {
    if (lastStoredProof.isEndOfEpoch()) {
      return lastStoredProof;
    }
    var lastEpoch = lastStoredProof.getEpoch();
    return transactionsAndProofReader.getEpochProof(lastEpoch).orElseThrow();
  }

  private static VertexStoreState genesisEpochProofToGenesisVertexStore(
      LedgerProof lastEpochProof, Hasher hasher) {
    return VertexStoreState.createNewForNextEpoch(lastEpochProof, hasher);
  }

  @Provides
  @Singleton
  private VertexStoreState vertexStoreState(
      @LastEpochProof LedgerProof lastEpochProof,
      VertexStoreRecovery vertexStoreRecovery,
      Serialization serialization,
      Hasher hasher) {

    var currentEpoch = lastEpochProof.getNextEpoch().orElseThrow();
    var vertexStoreState =
        vertexStoreRecovery
            .recoverVertexStore()
            .map(
                bytes -> {
                  try {
                    return serialization.fromDson(
                        bytes, VertexStoreState.SerializedVertexStoreState.class);
                  } catch (DeserializeException e) {
                    throw new RuntimeException("Unable to recover VertexStore", e);
                  }
                })
            .filter(state -> state.isForEpoch(currentEpoch.getEpoch()))
            .map(v -> v.toVertexStoreState(hasher))
            .orElseGet(() -> genesisEpochProofToGenesisVertexStore(lastEpochProof, hasher));

    log.info("Initial Vertex Store State: {}", vertexStoreState);

    return vertexStoreState;
  }
}
