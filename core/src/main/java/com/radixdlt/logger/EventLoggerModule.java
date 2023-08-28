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

package com.radixdlt.logger;

import static org.apache.logging.log4j.Level.*;

import com.google.common.hash.HashCode;
import com.google.common.util.concurrent.RateLimiter;
import com.google.inject.AbstractModule;
import com.google.inject.Provider;
import com.google.inject.Provides;
import com.google.inject.Singleton;
import com.google.inject.multibindings.ProvidesIntoSet;
import com.radixdlt.consensus.ConsensusByzantineEvent;
import com.radixdlt.consensus.bft.BFTValidatorId;
import com.radixdlt.consensus.bft.Self;
import com.radixdlt.consensus.bft.SelfValidatorInfo;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.consensus.epoch.EpochRoundUpdate;
import com.radixdlt.consensus.liveness.EpochLocalTimeoutOccurrence;
import com.radixdlt.crypto.ECDSASecp256k1PublicKey;
import com.radixdlt.environment.EventProcessorOnDispatch;
import com.radixdlt.ledger.LedgerUpdate;
import com.radixdlt.statecomputer.commit.CommitSummary;
import com.radixdlt.utils.Bytes;
import java.util.Optional;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public final class EventLoggerModule extends AbstractModule {
  private static final Logger logger = LogManager.getLogger();

  private final EventLoggerConfig eventLoggerConfig;

  public EventLoggerModule(EventLoggerConfig eventLoggerConfig) {
    this.eventLoggerConfig = eventLoggerConfig;
  }

  @Override
  public void configure() {
    bind(EventLoggerConfig.class).toInstance(eventLoggerConfig);
  }

  @Provides
  @Self
  String name(@Self ECDSASecp256k1PublicKey selfKey) {
    return eventLoggerConfig.formatNodeAddress().apply(selfKey);
  }

  @ProvidesIntoSet
  EventProcessorOnDispatch<?> logByzantineEvents() {
    return new EventProcessorOnDispatch<>(
        ConsensusByzantineEvent.class,
        event -> {
          final var authorStr =
              switch (event) {
                case ConsensusByzantineEvent.ConflictingGenesis
                conflictingGenesis -> eventLoggerConfig
                    .formatNodeAddress()
                    .apply(conflictingGenesis.author().getPublicKey());
                case ConsensusByzantineEvent.DoubleVote doubleVote -> eventLoggerConfig
                    .formatBftValidatorId()
                    .apply(doubleVote.author());
              };
          logger.warn("Byzantine Behavior detected: {} (author: {})", event, authorStr);
        });
  }

  @ProvidesIntoSet
  EventProcessorOnDispatch<?> logTimeouts() {
    return new EventProcessorOnDispatch<>(
        EpochLocalTimeoutOccurrence.class,
        t ->
            logger.warn(
                "bft_timeout{epoch={} round={} leader={} next_leader={} count={}}",
                t.getEpochRound().getEpoch(),
                t.getEpochRound().getRound().number(),
                eventLoggerConfig.formatBftValidatorId().apply(t.getLeader()),
                eventLoggerConfig.formatBftValidatorId().apply(t.getNextLeader()),
                t.getBase().timeout().count()));
  }

  @ProvidesIntoSet
  @SuppressWarnings("UnstableApiUsage")
  EventProcessorOnDispatch<?> logRounds() {
    final var logLimiter = RateLimiter.create(1.0);
    return new EventProcessorOnDispatch<>(
        EpochRoundUpdate.class,
        u -> {
          var logLevel = logLimiter.tryAcquire() ? INFO : TRACE;
          logger.log(
              logLevel,
              "bft_nxtrnd{epoch={} round={} leader={} next_leader={}}",
              u.getEpoch(),
              u.getEpochRound().getRound().number(),
              eventLoggerConfig.formatBftValidatorId().apply(u.getRoundUpdate().getLeader()),
              eventLoggerConfig.formatBftValidatorId().apply(u.getRoundUpdate().getNextLeader()));
        });
  }

  @ProvidesIntoSet
  @Singleton
  @SuppressWarnings("UnstableApiUsage")
  EventProcessorOnDispatch<?> ledgerUpdate(
      // The `Provider` indirection is needed here to break an unexpected circular dependency.
      Provider<SelfValidatorInfo> self) {
    final var ledgerUpdateLogLimtier = RateLimiter.create(1.0);
    final var missedProposalsLogLimtier = RateLimiter.create(1.0);
    return new EventProcessorOnDispatch<>(
        LedgerUpdate.class,
        ledgerUpdate ->
            processLedgerUpdate(
                self.get(), ledgerUpdateLogLimtier, missedProposalsLogLimtier, ledgerUpdate));
  }

  @SuppressWarnings("UnstableApiUsage")
  private void processLedgerUpdate(
      SelfValidatorInfo self,
      RateLimiter ledgerUpdateLogLimiter,
      RateLimiter missedProposalsLogLimiter,
      LedgerUpdate ledgerUpdate) {

    logLedgerUpdate(
        ledgerUpdate,
        ledgerUpdate.commitSummary().numUserTransactions().toInt(),
        calculateLoggingLevel(ledgerUpdateLogLimiter, ledgerUpdate.epochChange()));

    ledgerUpdate.epochChange().ifPresent(epochChange -> logEpochChange(self, epochChange));

    self.bftValidatorId()
        .ifPresent(
            selfValidatorId -> {
              logSelfMissedProposals(
                  selfValidatorId, ledgerUpdate.commitSummary(), missedProposalsLogLimiter);
            });
  }

  private static void logEpochChange(SelfValidatorInfo self, EpochChange epochChange) {
    var validatorSet = epochChange.getBFTConfiguration().getValidatorSet();
    final var included = self.bftValidatorId().stream().anyMatch(validatorSet::containsValidator);
    logger.info(
        "lgr_nepoch{epoch={} included={} num_validators={} total_stake={}}",
        epochChange.getNextEpoch(),
        included,
        validatorSet.getValidators().size(),
        validatorSet.getTotalPower());
  }

  private static void logLedgerUpdate(LedgerUpdate ledgerUpdate, long txnCount, Level logLevel) {
    if (!logger.isEnabled(logLevel)) {
      return;
    }

    final var proof = ledgerUpdate.proof();
    final var ledgerHashes = proof.getLedgerHashes();
    logger.log(
        logLevel,
        "lgr_commit{epoch={} round={} version={} num_txns={}, ts={}, state_root={}, txn_root={},"
            + " receipt_root={}}",
        proof.getEpoch(),
        proof.getRound().number(),
        proof.getStateVersion(),
        txnCount,
        proof.getProposerTimestamp(),
        shortFormatLedgerHash(ledgerHashes.getStateRoot()),
        shortFormatLedgerHash(ledgerHashes.getTransactionRoot()),
        shortFormatLedgerHash(ledgerHashes.getReceiptRoot()));
  }

  private static String shortFormatLedgerHash(HashCode hash) {
    return Bytes.toHexString(hash.asBytes()).substring(0, 16);
  }

  @SuppressWarnings("UnstableApiUsage")
  private static Level calculateLoggingLevel(
      RateLimiter logLimiter, Optional<EpochChange> epochChange) {
    return (epochChange.isPresent() || logLimiter.tryAcquire()) ? INFO : TRACE;
  }

  @SuppressWarnings("UnstableApiUsage")
  private void logSelfMissedProposals(
      BFTValidatorId self, CommitSummary commitSummary, RateLimiter logLimiter) {
    final var maybeSelfCounters =
        commitSummary.validatorRoundCounters().stream()
            .filter(c -> c.first().equals(self.getValidatorAddress()))
            .findFirst();
    if (maybeSelfCounters.isEmpty()) {
      return;
    }
    final var selfCounters = maybeSelfCounters.orElseThrow().last();

    final var totalMissed =
        selfCounters.missedByFallback().toLong() + selfCounters.missedByGap().toLong();

    if (totalMissed > 0 && logLimiter.tryAcquire()) {
      logger.warn(
          "proposals_missed{total_missed={} by_fallback_qc={} by_timeout={} validator={}",
          totalMissed,
          selfCounters.missedByFallback().toLong(),
          selfCounters.missedByGap().toLong(),
          eventLoggerConfig.formatBftValidatorId().apply(self));
    }
  }
}
