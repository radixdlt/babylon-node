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

use crate::prelude::*;

/// A configuration for [`LedgerHistoryGc`].
///
/// The GC is **disabled by default**: a Node only ever deletes any ledger history if its operator
/// explicitly asks for that, separately for each kind of history (see the `prune_*` flags).
#[derive(Debug, Clone, Default, Sbor)]
pub struct LedgerHistoryGcConfig {
    /// How often to run the GC, in seconds.
    /// Since this GC operates with an epoch precision, we do not need to run more often than epoch
    /// changes.
    pub interval_sec: u32,
    /// How many most recent *completed* epochs should be left not GC-ed.
    /// Please note that the current epoch is never GC-ed.
    pub retained_epoch_count: u64,
    /// The maximum number of epochs pruned by a single run (for each kind of history).
    /// This only matters when the GC is first enabled on an existing ledger (i.e. when there is a
    /// backlog of many epochs to prune): it spreads the resulting compaction work over time.
    pub max_epochs_per_run: u32,
    /// Whether to prune [`PrunableLedgerHistory::TransactionReceipts`].
    pub prune_transaction_receipts: bool,
    /// Whether to prune [`PrunableLedgerHistory::AccumulatorTreeSlices`].
    pub prune_accumulator_tree_slices: bool,
    /// Whether to prune [`PrunableLedgerHistory::TransactionIdentifiers`].
    pub prune_transaction_identifiers: bool,
    /// Whether to prune [`PrunableLedgerHistory::TransactionHashIndices`].
    pub prune_transaction_hash_indices: bool,
}

impl LedgerHistoryGcConfig {
    /// Returns the kinds of history which are configured to be pruned, in the order in which a
    /// run should prune them.
    pub fn pruned_histories(&self) -> Vec<PrunableLedgerHistory> {
        [
            (
                self.prune_transaction_receipts,
                PrunableLedgerHistory::TransactionReceipts,
            ),
            (
                self.prune_accumulator_tree_slices,
                PrunableLedgerHistory::AccumulatorTreeSlices,
            ),
            // Before the identifiers, so that the hashes are mostly read from them (instead of
            // being re-computed from the raw transactions):
            (
                self.prune_transaction_hash_indices,
                PrunableLedgerHistory::TransactionHashIndices,
            ),
            (
                self.prune_transaction_identifiers,
                PrunableLedgerHistory::TransactionIdentifiers,
            ),
        ]
        .into_iter()
        .filter_map(|(enabled, history)| enabled.then_some(history))
        .collect()
    }
}

/// A garbage collector of sufficiently-old ledger history which is not needed by the Node itself.
///
/// # What can be deleted
///
/// Each of the following is opt-in (see [`LedgerHistoryGcConfig`]), and is deleted only for
/// old-enough *completed* epochs:
///
/// - [`PrunableLedgerHistory::TransactionReceipts`]: the rows of [`TransactionReceiptsCf`] and
///   [`LocalTransactionExecutionsCf`], **except the ones of epoch-changing transactions** (i.e.
///   the last transaction of each epoch).
/// - [`PrunableLedgerHistory::AccumulatorTreeSlices`]: the rows of [`TransactionAccuTreeSlicesCf`]
///   and [`ReceiptAccuTreeSlicesCf`].
/// - [`PrunableLedgerHistory::TransactionIdentifiers`]: the rows of
///   [`CommittedTransactionIdentifiersCf`].
/// - [`PrunableLedgerHistory::TransactionHashIndices`]: the entries of [`LedgerTransactionHashesCf`],
///   [`IntentHashesCf`], [`NotarizedTransactionHashesCf`] and [`FinalizedSubintentHashesCf`] which
///   point at the pruned transactions. These are never pruned within the most recent
///   `max_epoch_range` epochs (see [`PrunableLedgerHistory::min_retained_epoch_count()`]).
///
/// # What is never deleted
///
/// Most importantly: the raw ledger transactions and the ledger proofs needed by the ledger-sync.
/// A Node running this GC remains a fully-capable sync source for its peers, including for the
/// ones syncing from genesis (a syncing Node only ever receives raw transactions with their
/// proof, and re-executes them to compute everything else).
///
/// The current state (i.e. Substates and the state tree) is not touched either. The genesis is
/// never pruned.
///
/// # Why this is safe
///
/// A receipt is an *output* of executing a transaction. The Node needs it at commit time (to
/// compute the receipt root hash, which is persisted separately), and afterwards it only serves:
/// - the Core API (i.e. external clients interested in historical transaction outcomes),
/// - the one-off, on-boot backfills of optional indices (see the "Interactions" section below),
/// - the on-boot computation of [`ProtocolState`], which reads `EpochChangeEvent`s from the
///   receipts of epoch-changing transactions (see [`epoch_change_iter()`]).
///
/// The last point is the reason for retaining the epoch-changing transactions' receipts forever:
/// this GC iterates the very same table of epoch proofs as the [`epoch_change_iter()`] does, and
/// only ever deletes the receipts strictly *between* two consecutive epoch proofs.
///
/// An accumulator tree (of transactions or receipts) is built separately for each epoch, starting
/// from the previous epoch's root (which is read from the epoch proof). Appending to it only reads
/// the single most recent slice (see [`AccuTree::append()`]), which always belongs to the current
/// epoch, and the current epoch is never pruned.
///
/// The transaction identifiers of a past state version are only read by:
/// - the Core API,
/// - the mempool, when it needs to report a (re-)submitted transaction as already committed (it
///   then falls back to re-computing the hashes from the raw transaction, see
///   [`CommittabilityValidator::check_for_rejection()`]),
/// - the historical state reads, which are limited to the state tree's history length. The GC
///   never prunes anything within twice that length from the top of the ledger.
///
/// The hash-based transaction indices are only read by:
/// - the Core API, when looking up a transaction by its intent hash (a transaction committed
///   below the pruned part of the index is then reported as not seen / not found),
/// - the mempool, to reject an intent which is already committed. The Engine refuses such intent
///   too (it tracks the committed intents for as long as they could be valid, i.e. at most
///   `max_epoch_range` epochs), and so the GC keeps the index for at least as long: within that
///   window, a re-submitted intent is still reported precisely as "already committed".
/// - the commit itself, which refuses a duplicate hash (which a duplicate intent could never
///   reach, for the reason above).
///
/// The hashes of the pruned transactions are read from their identifiers, or re-computed from the
/// raw transactions where the identifiers were already pruned.
///
/// # Interactions with other features
///
/// - The Core API endpoints which need a pruned part of the history return a client error
///   (instead of serving a partial history). See
///   [`QueryableTransactionStore::get_transaction_history_pruned_below()`].
/// - The backfills of the account change index and of the entity listing indices need the
///   receipts they have not yet processed. The database refuses to start if such backfill would
///   need receipts which were already pruned (see [`DatabaseConfigValidationError`]).
/// - The [`RestoreDecember2023LostSubstates`] fix replays *all* receipts of a ledger which is
///   still within its first [`MIN_EPOCHS_BEFORE_PRUNING`] epochs. The GC does not prune anything
///   on such a young ledger.
///
/// The implementation is suited for being driven by an external scheduler.
pub struct LedgerHistoryGc {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    pruned_histories: Vec<PrunableLedgerHistory>,
    interval: Duration,
    retained_epoch_count: u64,
    max_epochs_per_run: u64,
    min_epochs_before_pruning: u64,
    /// The source of the current `max_epoch_range` (see
    /// [`Self::min_retained_epoch_count_of_hash_indices()`]). Only ever `None` in tests.
    transaction_validator: Option<Arc<RwLock<TransactionValidator>>>,
    state_version_history_length: u64,
}

/// A number of epochs (counted from the first epoch known to the ledger) which need to be completed
/// before the GC prunes anything.
/// This is the "problematic epoch" of the [`RestoreDecember2023LostSubstates`] fix (i.e. 19099),
/// plus its tolerance (i.e. 2) - after that point, the fix will never need any receipt again.
pub const MIN_EPOCHS_BEFORE_PRUNING: u64 = 19_099 + 2;

/// A maximum number of prune ranges (i.e. epochs) deleted by a single atomic database write.
const MAX_RANGES_PER_WRITE_BATCH: usize = 100;

/// A maximum number of transactions whose hash index entries are deleted by a single atomic
/// database write (a whole epoch is never split, though).
/// Unlike the other kinds of history, these are deleted entry-by-entry (not by ranges), so their
/// write batches are bounded by size.
const MAX_HASH_INDEXED_TRANSACTIONS_PER_WRITE_BATCH: u64 = 50_000;

/// A range of state versions (from inclusive, to exclusive) pruned for some kind of history.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrunedLedgerHistoryRange {
    pub history: PrunableLedgerHistory,
    pub from_state_version_inclusive: StateVersion,
    pub to_state_version_exclusive: StateVersion,
}

impl LedgerHistoryGc {
    /// Creates a new GC.
    /// The given `state_version_history_length` is the one configured for the [`StateTreeGc`].
    /// The given `transaction_validator` is the Node's current one (which may change at protocol
    /// updates).
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        config: LedgerHistoryGcConfig,
        state_version_history_length: usize,
        transaction_validator: Arc<RwLock<TransactionValidator>>,
    ) -> Self {
        Self {
            database,
            pruned_histories: config.pruned_histories(),
            interval: Duration::from_secs(u64::from(config.interval_sec)),
            retained_epoch_count: config.retained_epoch_count,
            max_epochs_per_run: u64::from(config.max_epochs_per_run),
            min_epochs_before_pruning: MIN_EPOCHS_BEFORE_PRUNING,
            transaction_validator: Some(transaction_validator),
            state_version_history_length: u64::try_from(state_version_history_length)
                .unwrap_or(u64::MAX),
        }
    }

    /// Returns whether this instance should be scheduled at all.
    pub fn is_enabled(&self) -> bool {
        !self.pruned_histories.is_empty()
    }

    /// An interval between [`run()`]s, to be used by this instance's scheduler.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Performs a single GC run, which - for each configured kind of history - permanently deletes
    /// the rows of at most [`LedgerHistoryGcConfig::max_epochs_per_run`] old-enough epochs.
    /// Returns the ranges that have been pruned.
    ///
    /// *Note on concurrent database access:*
    /// Similarly to the [`LedgerProofsGc`], this GC only accesses "old" (i.e. not "top-of-ledger")
    /// DB rows. For this reason, it can use the direct [`DbLock::access_direct()`] and effectively
    /// own these rows, without locking the database. The readers of the history (i.e. the Core API
    /// handlers) use a [`DbLock::snapshot()`], and check the pruning progress within that same
    /// snapshot.
    pub fn run(&self) -> Vec<PrunedLedgerHistoryRange> {
        if self.pruned_histories.is_empty() {
            return vec![];
        }
        let database = self.database.access_direct();

        // Resolve the upper bounds of this run:
        let (Some(first_proof), Some(max_completed_epoch)) =
            (database.get_first_proof(), database.max_completed_epoch())
        else {
            return vec![]; // An empty ledger.
        };
        let first_epoch = first_proof.ledger_header.epoch.number();
        let max_completed_epoch = max_completed_epoch.number();
        if max_completed_epoch <= first_epoch.saturating_add(self.min_epochs_before_pruning) {
            return vec![]; // A ledger too young to be pruned (see `MIN_EPOCHS_BEFORE_PRUNING`).
        }
        // We stay well away from the state versions readable by the state history feature:
        let Some(max_state_version) = database
            .max_state_version()
            .number()
            .checked_sub(self.state_version_history_length.saturating_mul(2))
        else {
            return vec![]; // Nothing to GC ever, yet.
        };
        let max_state_version = StateVersion::of(max_state_version);

        let mut pruned_ranges = vec![];
        for history in &self.pruned_histories {
            let retained_epoch_count = match history {
                PrunableLedgerHistory::TransactionHashIndices => self
                    .retained_epoch_count
                    .max(self.min_retained_epoch_count_of_hash_indices()),
                _ => self.retained_epoch_count,
            };
            let Some(to_epoch) = max_completed_epoch.checked_sub(retained_epoch_count) else {
                continue; // Nothing to GC ever, yet.
            };
            let limits = RunLimits {
                to_epoch,
                max_state_version,
            };
            pruned_ranges.extend(self.run_for(database.deref(), *history, &limits));
        }
        pruned_ranges
    }

    /// Returns the minimum number of most recent completed epochs whose hash indices are never
    /// pruned, regardless of the configured retention.
    ///
    /// An intent can only be committed within its validity window, which ends at most
    /// `max_epoch_range` epochs after the epoch in which it was committed. The Engine refuses an
    /// intent outside of that window before it even looks at its own record of committed intents
    /// (and the mempool refuses to believe the Engine's record if the Node's index does not have
    /// it). Retaining one more epoch than that window means that an intent which can no longer be
    /// found by its hash is always refused as expired.
    fn min_retained_epoch_count_of_hash_indices(&self) -> u64 {
        self.transaction_validator.as_ref().map_or(0, |validator| {
            validator.read().config().max_epoch_range.saturating_add(1)
        })
    }

    fn run_for(
        &self,
        database: &ActualStateManagerDatabase,
        history: PrunableLedgerHistory,
        limits: &RunLimits,
    ) -> Vec<PrunedLedgerHistoryRange> {
        // Read the GC's persisted state:
        let persisted_progress = database.get_ledger_history_gc_progress(history);
        let mut never_pruned_before = persisted_progress.is_none();
        let progress_started_at = persisted_progress.unwrap_or_else(|| {
            // We start right after the genesis (which is thus never pruned):
            let post_genesis_ledger_header = database
                .get_post_genesis_epoch_proof()
                .expect("the caller checked that there is some completed epoch")
                .ledger_header;
            LedgerHistoryGcProgress {
                last_pruned_epoch: post_genesis_ledger_header.epoch,
                epoch_proof_state_version: post_genesis_ledger_header.state_version,
            }
        });
        if progress_started_at.last_pruned_epoch.number() >= limits.to_epoch {
            return vec![]; // Nothing to GC during this run.
        }

        info!(
            "Starting a GC run: pruning {:?} up to epoch {}; current progress: {:?}",
            history, limits.to_epoch, progress_started_at,
        );

        // Note: an epoch proof is keyed by the epoch which it *starts*, while the progress tracks
        // the last epoch which was *ended* by an already-handled epoch proof - hence the `+ 2`.
        let next_epoch_proofs = database.get_next_epoch_proof_iter(Epoch::of(
            progress_started_at
                .last_pruned_epoch
                .number()
                .saturating_add(2),
        ));

        let mut progress = progress_started_at;
        let mut pruned_ranges = vec![];
        let mut pending_ranges: Vec<(StateVersion, StateVersion)> = vec![];
        let mut handled_epochs = 0;
        for epoch_proof in next_epoch_proofs {
            let ended_epoch = epoch_proof.ledger_header.epoch;
            let epoch_proof_state_version = epoch_proof.ledger_header.state_version;
            if epoch_proof_state_version <= progress.epoch_proof_state_version {
                continue; // Defensive only: an epoch proof which we have already handled.
            }
            if ended_epoch.number() > limits.to_epoch
                || epoch_proof_state_version > limits.max_state_version
                || handled_epochs >= self.max_epochs_per_run
            {
                break;
            }

            // We delete everything between the previously-handled epoch proof and the current one
            // (exclusive). The row of the previous epoch-changing transaction is kept if this kind
            // of history needs it (or, on the very first run, if it is the last row of genesis).
            let previous_epoch_proof_state_version = progress.epoch_proof_state_version;
            let from_state_version_inclusive =
                if history.retains_rows_of_epoch_changes() || never_pruned_before {
                    previous_epoch_proof_state_version
                        .next()
                        .expect("state version overflow")
                } else {
                    previous_epoch_proof_state_version
                };
            let to_state_version_exclusive = epoch_proof_state_version;
            if from_state_version_inclusive < to_state_version_exclusive {
                match pending_ranges.last_mut() {
                    Some((_, previous_to)) if *previous_to == from_state_version_inclusive => {
                        // A continuation of the previous range:
                        *previous_to = to_state_version_exclusive;
                    }
                    _ => {
                        pending_ranges
                            .push((from_state_version_inclusive, to_state_version_exclusive));
                    }
                }
            }
            never_pruned_before = false;
            progress = LedgerHistoryGcProgress {
                last_pruned_epoch: ended_epoch,
                epoch_proof_state_version,
            };
            handled_epochs += 1;

            if pending_ranges.len() >= MAX_RANGES_PER_WRITE_BATCH
                || (history == PrunableLedgerHistory::TransactionHashIndices
                    && pending_ranges
                        .iter()
                        .map(|(from, to)| to.number() - from.number())
                        .sum::<u64>()
                        >= MAX_HASH_INDEXED_TRANSACTIONS_PER_WRITE_BATCH)
            {
                database.prune_ledger_history(history, &pending_ranges, progress.clone());
                pruned_ranges.append(&mut pending_ranges);
            }
        }
        if handled_epochs > 0 {
            // This also records the progress of the (theoretical) empty epochs:
            database.prune_ledger_history(history, &pending_ranges, progress.clone());
            pruned_ranges.append(&mut pending_ranges);
        }

        info!(
            "GC run of {:?} finished: pruned {} epochs; current progress: {:?}",
            history, handled_epochs, progress
        );
        pruned_ranges
            .into_iter()
            .map(|(from, to)| PrunedLedgerHistoryRange {
                history,
                from_state_version_inclusive: from,
                to_state_version_exclusive: to,
            })
            .collect()
    }
}

/// The upper bounds (both inclusive) of a single GC run.
struct RunLimits {
    /// The last epoch which is old enough to be pruned.
    to_epoch: u64,
    /// The last state version which is old enough to be pruned.
    max_state_version: StateVersion,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{
        commit_round_updates_until_epoch, create_bootstrapped_state_manager,
        prepare_and_commit_round_update_with_transactions,
    };
    use PrunableLedgerHistory::*;

    /// Creates a state manager progressing through 10 rounds (i.e. state versions) per epoch.
    fn create_state_manager(config: StateManagerConfig) -> StateManager {
        let babylon_settings =
            BabylonSettings::test_default().with_consensus_manager_config(ConsensusManagerConfig {
                max_validators: 10,
                epoch_change_condition: EpochChangeCondition {
                    min_round_count: 10,
                    max_round_count: 10,
                    target_duration_millis: 0,
                },
                num_unstake_epochs: 1,
                total_emission_xrd_per_epoch: Decimal::one(),
                min_validator_reliability: Decimal::one(),
                num_owner_stake_units_unlock_epochs: 2,
                num_fee_increase_delay_epochs: 1,
                validator_creation_usd_cost: Decimal::one(),
            });
        create_bootstrapped_state_manager(config, babylon_settings)
    }

    fn create_default_state_manager(tmp: &tempfile::TempDir) -> StateManager {
        create_state_manager(StateManagerConfig::new_for_testing(
            tmp.path().to_str().unwrap(),
        ))
    }

    /// Creates a GC instance which is not limited by the "young ledger" protection.
    fn create_gc(
        state_manager: &StateManager,
        pruned_histories: Vec<PrunableLedgerHistory>,
        retained_epoch_count: u64,
        max_epochs_per_run: u64,
    ) -> LedgerHistoryGc {
        LedgerHistoryGc {
            database: state_manager.database.clone(),
            pruned_histories,
            interval: Duration::MAX,
            retained_epoch_count,
            max_epochs_per_run,
            min_epochs_before_pruning: 0,
            transaction_validator: None,
            state_version_history_length: 0,
        }
    }

    fn all_histories() -> Vec<PrunableLedgerHistory> {
        LedgerHistoryGcConfig {
            prune_transaction_receipts: true,
            prune_accumulator_tree_slices: true,
            prune_transaction_identifiers: true,
            prune_transaction_hash_indices: true,
            ..Default::default()
        }
        .pruned_histories()
    }

    /// Returns the state versions of all epoch-changing transactions.
    fn epoch_change_state_versions(state_manager: &StateManager) -> BTreeSet<StateVersion> {
        state_manager
            .database
            .access_direct()
            .get_next_epoch_proof_iter(Epoch::of(0))
            .map(|proof| proof.ledger_header.state_version)
            .collect()
    }

    fn post_genesis_state_version(state_manager: &StateManager) -> StateVersion {
        state_manager
            .database
            .access_direct()
            .get_post_genesis_epoch_proof()
            .unwrap()
            .ledger_header
            .state_version
    }

    /// Returns the state versions for which the given predicate holds.
    fn state_versions_where(
        state_manager: &StateManager,
        predicate: impl Fn(&ActualStateManagerDatabase, StateVersion) -> bool,
    ) -> BTreeSet<StateVersion> {
        let database = state_manager.database.access_direct();
        (1..=database.max_state_version().number())
            .map(StateVersion::of)
            .filter(|state_version| predicate(database.deref(), *state_version))
            .collect()
    }

    fn state_versions_having_receipts(state_manager: &StateManager) -> BTreeSet<StateVersion> {
        state_versions_where(state_manager, |database, state_version| {
            database
                .get_committed_ledger_transaction_receipt(state_version)
                .is_some()
        })
    }

    fn state_versions_having_identifiers(state_manager: &StateManager) -> BTreeSet<StateVersion> {
        state_versions_where(state_manager, |database, state_version| {
            database
                .get_committed_transaction_identifiers(state_version)
                .is_some()
        })
    }

    fn state_versions_having_raw_transactions(
        state_manager: &StateManager,
    ) -> BTreeSet<StateVersion> {
        state_versions_where(state_manager, |database, state_version| {
            database.get_committed_transaction(state_version).is_some()
        })
    }

    /// Returns the state versions having both the transaction- and receipt- accu tree slices, after
    /// asserting that they are always stored (and pruned) together.
    fn state_versions_having_tree_slices(state_manager: &StateManager) -> BTreeSet<StateVersion> {
        state_versions_where(state_manager, |database, state_version| {
            let transaction_tree_slice =
                ReadableAccuTreeStore::<StateVersion, TransactionTreeHash>::get_tree_slice(
                    database,
                    &state_version,
                );
            let receipt_tree_slice =
                ReadableAccuTreeStore::<StateVersion, ReceiptTreeHash>::get_tree_slice(
                    database,
                    &state_version,
                );
            assert_eq!(
                transaction_tree_slice.is_some(),
                receipt_tree_slice.is_some()
            );
            transaction_tree_slice.is_some()
        })
    }

    fn top_ledger_hashes(state_manager: &StateManager) -> (StateVersion, LedgerHashes) {
        let (state_version, identifiers) = state_manager
            .database
            .access_direct()
            .get_top_transaction_identifiers()
            .unwrap();
        (state_version, identifiers.resultant_ledger_hashes)
    }

    fn progress(
        state_manager: &StateManager,
        history: PrunableLedgerHistory,
    ) -> Option<LedgerHistoryGcProgress> {
        state_manager
            .database
            .access_direct()
            .get_ledger_history_gc_progress(history)
    }

    #[test]
    fn is_disabled_by_default_and_then_deletes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let receipts_before = state_versions_having_receipts(&state_manager);
        let identifiers_before = state_versions_having_identifiers(&state_manager);
        let tree_slices_before = state_versions_having_tree_slices(&state_manager);

        let gc = LedgerHistoryGc::new(
            state_manager.database.clone(),
            LedgerHistoryGcConfig::default(),
            0,
            state_manager.transaction_validator.clone(),
        );

        assert!(!gc.is_enabled());
        assert!(gc.run().is_empty());
        assert_eq!(
            state_versions_having_receipts(&state_manager),
            receipts_before
        );
        assert_eq!(
            state_versions_having_identifiers(&state_manager),
            identifiers_before
        );
        assert_eq!(
            state_versions_having_tree_slices(&state_manager),
            tree_slices_before
        );
        let database = state_manager.database.access_direct();
        assert_eq!(database.get_transaction_receipts_pruned_below(), None);
        assert_eq!(database.get_transaction_history_pruned_below(), None);
        assert_eq!(database.get_transaction_hash_indices_pruned_below(), None);
    }

    #[test]
    fn does_not_prune_a_young_ledger() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let receipts_before = state_versions_having_receipts(&state_manager);
        let identifiers_before = state_versions_having_identifiers(&state_manager);
        let tree_slices_before = state_versions_having_tree_slices(&state_manager);

        // A GC of everything, created the same way as the production one:
        let gc = LedgerHistoryGc::new(
            state_manager.database.clone(),
            LedgerHistoryGcConfig {
                interval_sec: 1,
                retained_epoch_count: 0,
                max_epochs_per_run: 1000,
                prune_transaction_receipts: true,
                prune_accumulator_tree_slices: true,
                prune_transaction_identifiers: true,
                prune_transaction_hash_indices: true,
            },
            0,
            state_manager.transaction_validator.clone(),
        );

        assert!(gc.is_enabled());
        assert!(gc.run().is_empty());
        assert_eq!(
            state_versions_having_receipts(&state_manager),
            receipts_before
        );
        assert_eq!(
            state_versions_having_identifiers(&state_manager),
            identifiers_before
        );
        assert_eq!(
            state_versions_having_tree_slices(&state_manager),
            tree_slices_before
        );
    }

    #[test]
    fn prunes_old_receipts_but_retains_epoch_change_receipts() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));

        let epoch_changes = epoch_change_state_versions(&state_manager);
        let receipts_before = state_versions_having_receipts(&state_manager);
        let identifiers_before = state_versions_having_identifiers(&state_manager);
        let tree_slices_before = state_versions_having_tree_slices(&state_manager);
        let max_state_version = state_manager.database.access_direct().max_state_version();
        // Sanity check of the test setup: before the GC, every transaction has its receipt.
        assert_eq!(receipts_before.len() as u64, max_state_version.number());

        // We retain 2 completed epochs (i.e. 6 and 7; the max completed one is 7):
        let gc = create_gc(&state_manager, vec![TransactionReceipts], 2, 1000);
        let pruned_ranges = gc.run();

        let post_genesis_state_version = post_genesis_state_version(&state_manager);
        let database = state_manager.database.access_direct();
        // The epoch proof keyed by 6 is the one which ended the epoch 5 (i.e. the last pruned one):
        let pruned_below = database
            .get_epoch_proof(Epoch::of(6))
            .unwrap()
            .ledger_header
            .state_version;
        assert_eq!(
            database.get_transaction_receipts_pruned_below(),
            Some(pruned_below)
        );
        assert_eq!(
            database.get_transaction_history_pruned_below(),
            Some(pruned_below)
        );
        assert_eq!(
            database.get_ledger_history_gc_progress(TransactionReceipts),
            Some(LedgerHistoryGcProgress {
                last_pruned_epoch: Epoch::of(5),
                epoch_proof_state_version: pruned_below,
            })
        );
        drop(database);

        // The exact expectation: a receipt remains if and only if it is a genesis one, or an
        // epoch-changing one, or one that is not old enough.
        let expected_receipts_after = receipts_before
            .iter()
            .filter(|state_version| {
                **state_version <= post_genesis_state_version
                    || epoch_changes.contains(*state_version)
                    || **state_version >= pruned_below
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        let receipts_after = state_versions_having_receipts(&state_manager);
        assert_eq!(receipts_after, expected_receipts_after);
        // ... and the GC really did delete something (4 epochs, 9 non-epoch-changing txns each):
        assert_eq!(receipts_before.len() - receipts_after.len(), 4 * 9);
        assert_eq!(pruned_ranges.len(), 4);
        for range in &pruned_ranges {
            assert_eq!(range.history, TransactionReceipts);
            let (from, to) = (
                range.from_state_version_inclusive,
                range.to_state_version_exclusive,
            );
            assert!(from < to);
            assert!(!epoch_changes.iter().any(|v| from <= *v && *v < to));
        }

        // Nothing else was touched:
        assert_eq!(
            state_versions_having_raw_transactions(&state_manager).len() as u64,
            max_state_version.number()
        );
        assert_eq!(
            state_versions_having_identifiers(&state_manager),
            identifiers_before
        );
        assert_eq!(
            state_versions_having_tree_slices(&state_manager),
            tree_slices_before
        );
        assert_eq!(progress(&state_manager, AccumulatorTreeSlices), None);
        assert_eq!(progress(&state_manager, TransactionIdentifiers), None);
        assert_eq!(progress(&state_manager, TransactionHashIndices), None);

        // A repeated run has nothing more to do...
        assert!(gc.run().is_empty());
        assert_eq!(
            state_versions_having_receipts(&state_manager),
            receipts_after
        );

        // ... until the ledger progresses by further epochs:
        commit_round_updates_until_epoch(&state_manager, Epoch::of(10));
        assert_eq!(gc.run().len(), 2);
        assert_eq!(
            progress(&state_manager, TransactionReceipts)
                .unwrap()
                .last_pruned_epoch,
            Epoch::of(7)
        );
    }

    #[test]
    fn paces_the_backlog_by_max_epochs_per_run() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));

        // No retained epochs, but only 2 epochs pruned per run (the max completed epoch is 7):
        let gc = create_gc(&state_manager, all_histories(), 0, 2);
        let first_epoch_to_prune = state_manager
            .database
            .access_direct()
            .get_post_genesis_epoch_proof()
            .unwrap()
            .ledger_header
            .epoch
            .number()
            + 1;

        let mut expected_last_pruned_epoch = first_epoch_to_prune - 1;
        while expected_last_pruned_epoch < 7 {
            let pruned_ranges = gc.run();
            let expected_pruned_epochs = (7 - expected_last_pruned_epoch).min(2);
            // The receipts are pruned in per-epoch ranges; the other histories in a single range:
            assert_eq!(
                pruned_ranges
                    .iter()
                    .filter(|range| range.history == TransactionReceipts)
                    .count() as u64,
                expected_pruned_epochs
            );
            for history in [
                AccumulatorTreeSlices,
                TransactionHashIndices,
                TransactionIdentifiers,
            ] {
                assert_eq!(
                    pruned_ranges
                        .iter()
                        .filter(|range| range.history == history)
                        .count(),
                    1
                );
            }
            expected_last_pruned_epoch += expected_pruned_epochs;
            for history in all_histories() {
                assert_eq!(
                    progress(&state_manager, history)
                        .unwrap()
                        .last_pruned_epoch
                        .number(),
                    expected_last_pruned_epoch
                );
            }
        }
        assert!(gc.run().is_empty());
    }

    /// On every boot, the Node re-computes its [`ProtocolState`] from the receipts of the
    /// epoch-changing transactions. This must give the very same result on a pruned ledger.
    #[test]
    fn protocol_state_can_still_be_computed_after_pruning() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(12));

        // A (never enacted) protocol update depending on the validators' readiness signals - i.e.
        // the kind of condition which needs to look at the past `EpochChangeEvent`s:
        let triggers = vec![ProtocolUpdateTrigger::of(
            TestProtocolUpdateDefinition::subnamed("v2"),
            ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
                lower_bound_inclusive: Epoch::of(5),
                upper_bound_exclusive: Epoch::of(1000),
                readiness_thresholds: vec![SignalledReadinessThreshold {
                    required_ratio_of_stake_supported: Decimal::one(),
                    required_consecutive_completed_epochs_of_support: 3,
                }],
            },
        )];
        let network_definition = NetworkDefinition::simulator();
        let genesis_data_resolver: Arc<dyn ResolveGenesisData> =
            Arc::new(FixedGenesisDataResolver::new(JavaGenesisData::new_from(
                BabylonSettings::test_default(),
                vec![],
            )));
        let scenarios_execution_config = ScenariosExecutionConfig::default();
        let compute_protocol_state = || {
            ProtocolState::compute_initial(
                &ProtocolUpdateContentOverrides::empty().into(),
                ProtocolUpdateContext {
                    network: &network_definition,
                    database: &state_manager.database,
                    genesis_data_resolver: &genesis_data_resolver,
                    scenario_config: &scenarios_execution_config,
                },
                &triggers,
            )
        };
        let collect_epoch_change_events = || {
            let database = state_manager.database.access_direct();
            let events = epoch_change_iter(database.deref(), Epoch::of(0))
                .map(|(state_version, event)| (state_version, event.epoch))
                .collect::<Vec<_>>();
            events
        };

        let protocol_state_before = compute_protocol_state();
        let epoch_change_events_before = collect_epoch_change_events();
        assert_eq!(protocol_state_before.pending_protocol_updates.len(), 1);

        // Prune everything, as aggressively as possible:
        let pruned_ranges = create_gc(&state_manager, all_histories(), 0, 1000).run();
        assert!(!pruned_ranges.is_empty());

        assert_eq!(compute_protocol_state(), protocol_state_before);
        assert_eq!(collect_epoch_change_events(), epoch_change_events_before);

        // The ledger can progress, and the GC can run again, with the same guarantee:
        commit_round_updates_until_epoch(&state_manager, Epoch::of(14));
        let protocol_state_before = compute_protocol_state();
        assert!(!create_gc(&state_manager, all_histories(), 0, 1000)
            .run()
            .is_empty());
        assert_eq!(compute_protocol_state(), protocol_state_before);
    }

    /// The "local transaction executions" are pruned together with the receipts.
    #[test]
    fn prunes_local_transaction_executions_in_lockstep() {
        let tmp = tempfile::tempdir().unwrap();
        let mut config = StateManagerConfig::new_for_testing(tmp.path().to_str().unwrap());
        config
            .database_config
            .enable_local_transaction_execution_index = true;
        config.database_config.enable_account_change_index = true;
        let state_manager = create_state_manager(config);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));

        assert!(
            !create_gc(&state_manager, vec![TransactionReceipts], 2, 1000)
                .run()
                .is_empty()
        );

        let database = state_manager.database.access_direct();
        let pruned_below = database.get_transaction_receipts_pruned_below().unwrap();
        let mut pruned_count = 0;
        for state_version in (1..=database.max_state_version().number()).map(StateVersion::of) {
            let receipt = database.get_committed_ledger_transaction_receipt(state_version);
            let local_execution = database.get_committed_local_transaction_execution(state_version);
            assert_eq!(receipt.is_some(), local_execution.is_some());
            // This is the method used by the Core API, which panics on any inconsistency:
            let local_receipt = database.get_committed_local_transaction_receipt(state_version);
            assert_eq!(local_receipt.is_some(), receipt.is_some());
            if state_version >= pruned_below {
                assert!(receipt.is_some());
            }
            if receipt.is_none() {
                pruned_count += 1;
            }
        }
        assert_eq!(pruned_count, 4 * 9);

        // The full bundles can still be iterated from the "pruned below" state version onwards:
        let bundle_count = database
            .get_committed_transaction_bundle_iter(pruned_below)
            .count() as u64;
        assert_eq!(
            bundle_count,
            database.max_state_version().number() - pruned_below.number() + 1
        );
    }

    /// The accumulator trees are a part of the ledger hashes, so this test compares a pruned
    /// ledger against its never-pruned twin.
    #[test]
    fn pruned_ledger_progresses_to_the_same_ledger_hashes_as_a_not_pruned_one() {
        let pruned_tmp = tempfile::tempdir().unwrap();
        let pruned = create_default_state_manager(&pruned_tmp);
        let reference_tmp = tempfile::tempdir().unwrap();
        let reference = create_default_state_manager(&reference_tmp);

        commit_round_updates_until_epoch(&pruned, Epoch::of(8));
        commit_round_updates_until_epoch(&reference, Epoch::of(8));
        // Sanity check of the test setup: the two ledgers are deterministic twins.
        assert_eq!(top_ledger_hashes(&pruned), top_ledger_hashes(&reference));

        let tree_slices_before = state_versions_having_tree_slices(&pruned);
        let identifiers_before = state_versions_having_identifiers(&pruned);
        let post_genesis_state_version = post_genesis_state_version(&pruned);

        // Prune everything, as aggressively as possible (i.e. all 7 completed epochs)...
        let gc = create_gc(&pruned, all_histories(), 0, 1000);
        assert!(!gc.run().is_empty());
        let pruned_below = progress(&pruned, AccumulatorTreeSlices)
            .unwrap()
            .epoch_proof_state_version;
        assert_eq!(
            pruned_below,
            pruned
                .database
                .access_direct()
                .get_latest_epoch_proof()
                .unwrap()
                .ledger_header
                .state_version
        );
        assert_eq!(
            progress(&pruned, TransactionIdentifiers)
                .unwrap()
                .epoch_proof_state_version,
            pruned_below
        );

        // ... which leaves exactly the genesis and the not-old-enough rows:
        let is_retained = |state_version: &&StateVersion| {
            **state_version <= post_genesis_state_version || **state_version >= pruned_below
        };
        let expected_tree_slices_after = tree_slices_before
            .iter()
            .filter(is_retained)
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            state_versions_having_tree_slices(&pruned),
            expected_tree_slices_after
        );
        assert!(expected_tree_slices_after.len() < tree_slices_before.len());
        let expected_identifiers_after = identifiers_before
            .iter()
            .filter(is_retained)
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            state_versions_having_identifiers(&pruned),
            expected_identifiers_after
        );
        assert!(expected_identifiers_after.len() < identifiers_before.len());
        // The raw transactions (i.e. what a ledger-sync serves) are all intact:
        assert_eq!(
            state_versions_having_raw_transactions(&pruned),
            state_versions_having_raw_transactions(&reference)
        );

        // Both ledgers progress (within an epoch, and then through epoch changes) identically,
        // with the GC running after every epoch of the pruned one:
        for epoch in 9..=12 {
            commit_round_updates_until_epoch(&pruned, Epoch::of(epoch));
            commit_round_updates_until_epoch(&reference, Epoch::of(epoch));
            assert_eq!(top_ledger_hashes(&pruned), top_ledger_hashes(&reference));
            assert!(!gc.run().is_empty());
        }
        commit_round_updates_until_epoch(&pruned, Epoch::of(13));
        commit_round_updates_until_epoch(&reference, Epoch::of(13));
        assert_eq!(top_ledger_hashes(&pruned), top_ledger_hashes(&reference));
    }

    #[test]
    fn does_not_prune_within_the_state_history_length() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let max_state_version = state_manager.database.access_direct().max_state_version();

        // A state history longer than the ledger itself:
        let mut gc = create_gc(&state_manager, all_histories(), 0, 1000);
        gc.state_version_history_length = max_state_version.number();
        assert!(gc.run().is_empty());

        // A state history of 15 state versions (i.e. 1.5 epochs): we stay 30 versions away.
        gc.state_version_history_length = 15;
        assert!(!gc.run().is_empty());
        for history in all_histories() {
            let pruned_below = progress(&state_manager, history)
                .unwrap()
                .epoch_proof_state_version;
            assert!(pruned_below.number() <= max_state_version.number() - 30);
            // ... but not needlessly far away (i.e. less than 1 epoch further):
            assert!(pruned_below.number() > max_state_version.number() - 30 - 10);
        }
    }

    /// Anyone can (re-)submit an old, already committed transaction. The mempool must be able to
    /// report this properly, even if the identifiers of that transaction were pruned.
    #[test]
    fn mempool_reports_an_already_committed_transaction_after_pruning_its_identifiers() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(3));

        // Commit a user transaction:
        let notary = Ed25519PrivateKey::from_u64(7).unwrap();
        let raw_user_transaction = TransactionV1Builder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(100),
                nonce: 1,
                notary_public_key: notary.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new_v1().lock_fee_from_faucet().build())
            .notarize(&notary)
            .build()
            .to_raw()
            .unwrap();
        let validator = &state_manager.committability_validator;
        let prepared = validator.prepare_from_raw(&raw_user_transaction).unwrap();
        let validated = validator.validate(prepared).unwrap();
        let user_hashes = validated.hashes();
        let executable = validated.create_executable();
        prepare_and_commit_round_update_with_transactions(
            &state_manager,
            vec![raw_user_transaction],
        );
        let committed_state_version = state_manager
            .database
            .access_direct()
            .get_txn_state_version_by_identifier(&user_hashes.transaction_intent_hash)
            .expect("the user transaction should have been committed");

        // The transaction gets re-submitted with a different notarization (we simply fake its
        // hash here), so that the test can tell where the reported hash comes from:
        let mut resubmitted_hashes = user_hashes.clone();
        resubmitted_hashes.notarized_transaction_hash =
            NotarizedTransactionHash::from_hash(hash("a different notarization"));
        let check_for_rejection =
            || validator.check_for_rejection(&executable, &resubmitted_hashes, SystemTime::now());
        let attempt_before = check_for_rejection();
        let expected_rejection = Some(MempoolRejectionReason::TransactionIntentAlreadyCommitted(
            AlreadyCommittedError {
                committed_state_version,
                committed_notarized_transaction_hash: user_hashes.notarized_transaction_hash,
            },
        ));
        assert_eq!(attempt_before.rejection, expected_rejection);

        // Let the transaction become old enough, and prune its identifiers:
        commit_round_updates_until_epoch(&state_manager, Epoch::of(6));
        assert!(
            !create_gc(&state_manager, vec![TransactionIdentifiers], 0, 1000)
                .run()
                .is_empty()
        );
        assert!(state_manager
            .database
            .access_direct()
            .get_committed_transaction_identifiers(committed_state_version)
            .is_none());

        assert_eq!(check_for_rejection().rejection, expected_rejection);
    }

    /// Commits a single user transaction (with the given nonce, valid from epoch 1 until the given
    /// epoch), and returns its hashes and its executable (for a later re-submission).
    fn commit_user_transaction(
        state_manager: &StateManager,
        nonce: u32,
        end_epoch_exclusive: Epoch,
    ) -> (UserTransactionHashes, ExecutableTransaction) {
        let notary = Ed25519PrivateKey::from_u64(7).unwrap();
        let raw_user_transaction = TransactionV1Builder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive,
                nonce,
                notary_public_key: notary.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new_v1().lock_fee_from_faucet().build())
            .notarize(&notary)
            .build()
            .to_raw()
            .unwrap();
        let validator = &state_manager.committability_validator;
        let prepared = validator.prepare_from_raw(&raw_user_transaction).unwrap();
        let validated = validator.validate(prepared).unwrap();
        let user_hashes = validated.hashes();
        let executable = validated.create_executable();
        prepare_and_commit_round_update_with_transactions(
            state_manager,
            vec![raw_user_transaction],
        );
        (user_hashes, executable)
    }

    /// Returns the hashes of every committed transaction (to be called before any pruning).
    fn all_transaction_hashes(
        state_manager: &StateManager,
    ) -> Vec<(StateVersion, LedgerTransactionHashes)> {
        let database = state_manager.database.access_direct();
        (1..=database.max_state_version().number())
            .map(StateVersion::of)
            .map(|state_version| {
                let hashes = database
                    .get_committed_transaction_identifiers(state_version)
                    .unwrap()
                    .transaction_hashes;
                (state_version, hashes)
            })
            .collect()
    }

    /// Asserts that exactly the transactions below `pruned_below` (except the genesis ones) can no
    /// longer be found by any of their hashes.
    fn assert_hash_indices_pruned_below(
        state_manager: &StateManager,
        hashes: &[(StateVersion, LedgerTransactionHashes)],
        pruned_below: StateVersion,
    ) {
        let post_genesis_state_version = post_genesis_state_version(state_manager);
        let database = state_manager.database.access_direct();
        let mut found_count = 0;
        for (state_version, hashes) in hashes {
            let expected = (*state_version <= post_genesis_state_version
                || *state_version >= pruned_below)
                .then_some(*state_version);
            assert_eq!(
                database.get_txn_state_version_by_identifier(&hashes.ledger_transaction_hash),
                expected,
                "ledger transaction hash at {state_version}"
            );
            if let Some(user_hashes) = hashes.as_user() {
                assert_eq!(
                    database
                        .get_txn_state_version_by_identifier(&user_hashes.transaction_intent_hash),
                    expected,
                    "intent hash at {state_version}"
                );
                assert_eq!(
                    database.get_txn_state_version_by_identifier(
                        &user_hashes.notarized_transaction_hash
                    ),
                    expected,
                    "notarized transaction hash at {state_version}"
                );
            }
            if expected.is_some() {
                found_count += 1;
            }
        }
        // Sanity check: the assertion is not vacuous.
        assert!(found_count < hashes.len());
    }

    #[test]
    fn prunes_hash_index_entries_of_old_transactions() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(3));
        let (old_user_hashes, _) = commit_user_transaction(&state_manager, 1, Epoch::of(100));
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let (recent_user_hashes, _) = commit_user_transaction(&state_manager, 2, Epoch::of(100));
        let hashes = all_transaction_hashes(&state_manager);
        let identifiers_before = state_versions_having_identifiers(&state_manager);

        // We retain 2 completed epochs (i.e. 6 and 7):
        assert!(
            !create_gc(&state_manager, vec![TransactionHashIndices], 2, 1000)
                .run()
                .is_empty()
        );

        let database = state_manager.database.access_direct();
        let pruned_below = database
            .get_transaction_hash_indices_pruned_below()
            .unwrap();
        assert_eq!(
            pruned_below,
            database
                .get_epoch_proof(Epoch::of(6))
                .unwrap()
                .ledger_header
                .state_version
        );
        // Only the hash indices are pruned (i.e. the transaction history itself is complete):
        assert_eq!(database.get_transaction_history_pruned_below(), None);
        assert_eq!(
            database.get_txn_state_version_by_identifier(&old_user_hashes.transaction_intent_hash),
            None
        );
        assert!(database
            .get_txn_state_version_by_identifier(&recent_user_hashes.transaction_intent_hash)
            .is_some());
        drop(database);
        assert_hash_indices_pruned_below(&state_manager, &hashes, pruned_below);
        assert_eq!(
            state_versions_having_identifiers(&state_manager),
            identifiers_before
        );

        // The ledger progresses (i.e. new hashes still get indexed and checked for duplicates):
        commit_round_updates_until_epoch(&state_manager, Epoch::of(10));
        let (newest_user_hashes, _) = commit_user_transaction(&state_manager, 3, Epoch::of(100));
        assert!(state_manager
            .database
            .access_direct()
            .get_txn_state_version_by_identifier(&newest_user_hashes.transaction_intent_hash)
            .is_some());
    }

    /// A Node which already pruned its identifiers re-computes the hashes from the raw transactions.
    #[test]
    fn prunes_hash_index_entries_of_transactions_with_pruned_identifiers() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(3));
        let (old_user_hashes, _) = commit_user_transaction(&state_manager, 1, Epoch::of(100));
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let hashes = all_transaction_hashes(&state_manager);

        // First the identifiers only (e.g. a Node which enables the hash indices pruning later)...
        assert!(
            !create_gc(&state_manager, vec![TransactionIdentifiers], 2, 1000)
                .run()
                .is_empty()
        );
        let identifiers_pruned_below = progress(&state_manager, TransactionIdentifiers)
            .unwrap()
            .epoch_proof_state_version;
        let old_state_version = state_manager
            .database
            .access_direct()
            .get_txn_state_version_by_identifier(&old_user_hashes.transaction_intent_hash)
            .unwrap();
        assert!(old_state_version < identifiers_pruned_below);

        // ... and then the hash indices, which have no identifiers to read the hashes from:
        assert!(
            !create_gc(&state_manager, vec![TransactionHashIndices], 2, 1000)
                .run()
                .is_empty()
        );
        let pruned_below = state_manager
            .database
            .access_direct()
            .get_transaction_hash_indices_pruned_below()
            .unwrap();
        assert_eq!(pruned_below, identifiers_pruned_below);
        assert_hash_indices_pruned_below(&state_manager, &hashes, pruned_below);
    }

    /// The production GC keeps the hash indices for at least `max_epoch_range` epochs, whatever
    /// the configured retention.
    #[test]
    fn retains_hash_indices_for_the_max_epoch_range() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));
        let receipts_before = state_versions_having_receipts(&state_manager);

        let mut gc = create_gc(
            &state_manager,
            vec![TransactionReceipts, TransactionHashIndices],
            0,
            1000,
        );
        gc.transaction_validator = Some(state_manager.transaction_validator.clone());
        let pruned_ranges = gc.run();

        assert!(!pruned_ranges.is_empty());
        assert!(pruned_ranges
            .iter()
            .all(|range| range.history == TransactionReceipts));
        assert!(state_versions_having_receipts(&state_manager).len() < receipts_before.len());
        assert_eq!(progress(&state_manager, TransactionHashIndices), None);
        assert_eq!(
            gc.min_retained_epoch_count_of_hash_indices(),
            12 * 24 * 30 + 1
        );
    }

    /// The minimum retention of the hash indices (see
    /// [`LedgerHistoryGc::min_retained_epoch_count_of_hash_indices()`]) relies on the Engine
    /// refusing an expired intent before it looks at its own record of committed intents. Without
    /// that, a re-submitted intent missing from the Node's index would trip the mempool's invariant
    /// check.
    #[test]
    fn engine_rejects_an_expired_intent_after_pruning_its_hash_index_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_default_state_manager(&tmp);
        commit_round_updates_until_epoch(&state_manager, Epoch::of(3));
        // An intent valid in epochs 1 to 4 only:
        let (user_hashes, executable) = commit_user_transaction(&state_manager, 1, Epoch::of(5));
        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));

        assert!(
            !create_gc(&state_manager, vec![TransactionHashIndices], 0, 1000)
                .run()
                .is_empty()
        );
        assert_eq!(
            state_manager
                .database
                .access_direct()
                .get_txn_state_version_by_identifier(&user_hashes.transaction_intent_hash),
            None
        );

        let rejection = state_manager
            .committability_validator
            .check_for_rejection(&executable, &user_hashes, SystemTime::now())
            .rejection
            .expect("an expired intent must be rejected");
        let MempoolRejectionReason::FromExecution(reason) = &rejection else {
            panic!("unexpected rejection: {rejection:?}");
        };
        assert!(
            matches!(
                **reason,
                ExecutionRejectionReason::TransactionEpochNoLongerValid { .. }
            ),
            "unexpected rejection: {rejection:?}"
        );
    }
}
