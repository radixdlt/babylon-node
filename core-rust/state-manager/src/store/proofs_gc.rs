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

use crate::engine_prelude::*;
use node_common::locks::DbLock;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::store::consensus::traits::gc::{
    LedgerProofsGcProgress, LedgerProofsGcProgressV1, LedgerProofsGcStore,
};
use crate::store::consensus::traits::proofs::QueryableProofStore;

use crate::jni::LedgerSyncLimitsConfig;
use crate::store::consensus::rocks_db::{ActualStateManagerDatabase, StateManagerDatabase};
use crate::store::common::rocks_db::ReadableRocks;
use crate::consensus::traits::GetSyncableTxnsAndProofError::{
    FailedToPrepareAResponseWithinLimits, NothingToServeAtTheGivenStateVersion,
    RefusedToServeGenesis, RefusedToServeProtocolUpdate,
};
use crate::{LedgerProof, StateVersion};

/// A configuration for [`LedgerProofsGc`].
#[derive(Debug, Categorize, Encode, Decode, Clone, Default)]
pub struct LedgerProofsGcConfig {
    /// How often to run the GC, in seconds.
    /// Since this GC operates with an epoch precision, we do not need to run more often than epoch
    /// changes.
    // TODO(after having some event-driven Rust infra): The entire `LedgerProofsGc` could be
    // migrated away from `Scheduler` into `EventListener<EpochChangeCommittedEvent>` (as noted
    // above - it wants to run async exactly once after each epoch).
    pub interval_sec: u32,
    /// How many most recent *completed* epochs should be left not GC-ed.
    /// Please note that the current epoch is never GC-ed.
    pub most_recent_full_resolution_epoch_count: usize,
}

/// A garbage collector of sufficiently-old, non-critical ledger proofs.
/// A ledger proof is "non-critical" when it is not listed by our "get transactions with their
/// proof" logic (used e.g. for ledger-sync responses).
/// The implementation is suited for being driven by an external scheduler.
pub struct LedgerProofsGc {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    interval: Duration,
    most_recent_full_resolution_epoch_count: u64,
    limits_config: LedgerSyncLimitsConfig,
}

impl LedgerProofsGc {
    /// Creates a new GC.
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        gc_config: LedgerProofsGcConfig,
        limits_config: LedgerSyncLimitsConfig,
    ) -> Self {
        Self {
            database,
            interval: Duration::from_secs(u64::from(gc_config.interval_sec)),
            most_recent_full_resolution_epoch_count: u64::try_from(
                gc_config.most_recent_full_resolution_epoch_count,
            )
            .unwrap(),
            limits_config,
        }
    }

    /// An interval between [`run()`]s, to be used by this instance's scheduler.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Performs a single GC run, which is supposed to permanently delete *all* non-critical ledger
    /// proofs of configured old-enough epochs.
    /// Returns proof ranges that have been pruned.
    /// TODO: the return value is only used in tests, consider refactoring
    ///
    /// *Note on concurrent database access:*
    /// The proofs' GC process, by its nature, only accesses "old" (i.e. not "top-of-ledger" new)
    /// proof DB rows. For this reason, it can use the direct [`DbLock::access_direct()`] and
    /// effectively own these rows (for reads and deletes), without locking the database.
    /// Of course, a concurrent ledger sync may request a range of "old" proofs too, and thus it
    /// should use a [`DbLock::snapshot()`] to avoid relying on proofs which are about to be
    /// deleted.
    pub fn run(&self) -> Vec<ProofPruneRange> {
        // Read the GC's persisted state and initialize the run:
        let database = self.database.access_direct();
        let to_epoch = database
            .max_completed_epoch()
            .map(|max_completed_epoch| max_completed_epoch.number())
            .and_then(|number| number.checked_sub(self.most_recent_full_resolution_epoch_count))
            .map(Epoch::of);
        let Some(to_epoch) = to_epoch else {
            // Nothing to GC ever, yet.
            return vec![];
        };
        let progress_started_at: LedgerProofsGcProgress =
            database.get_progress().unwrap_or_else(|| {
                LedgerProofsGcProgress::new(
                    database
                        .get_post_genesis_epoch_proof()
                        .expect("we checked that there is some completed epoch above")
                        .ledger_header,
                )
            });

        if progress_started_at.last_pruned_epoch >= to_epoch {
            // Nothing to GC during this run.
            return vec![];
        }

        info!(
            "Starting a GC run: pruning ledger proofs up to epoch {}; current progress: {:?}",
            to_epoch.number(),
            progress_started_at,
        );
        let mut last_pruned_state_version = progress_started_at.epoch_proof_state_version;

        let mut pruned_proof_ranges = vec![];
        let mut retained_proofs = 0; // only for logging purposes
        while let Some(next_prune_range) = Self::locate_next_prune_range(
            database.deref(),
            last_pruned_state_version,
            &self.limits_config,
        ) {
            database.delete_ledger_proofs_range(
                next_prune_range.from_state_version_inclusive,
                next_prune_range.to_state_version_exclusive(),
            );
            pruned_proof_ranges.push(next_prune_range.clone());
            last_pruned_state_version = next_prune_range.to_state_version_exclusive();

            let retained_header = next_prune_range.to_proof_exclusive.ledger_header;

            retained_proofs += next_prune_range.skipped_proofs;
            retained_proofs += 1;
            if let Some(next_epoch) = retained_header.next_epoch {
                info!(
                    "Recording progress of pruned epoch {} (having {} retained proofs)",
                    retained_header.epoch.number(),
                    retained_proofs
                );
                retained_proofs = 0;
                database.set_progress(LedgerProofsGcProgressV1 {
                    last_pruned_epoch: retained_header.epoch,
                    epoch_proof_state_version: last_pruned_state_version,
                });
                if next_epoch.epoch > to_epoch {
                    break;
                }
            }
        }

        info!("Ledger proofs' GC run finished");
        pruned_proof_ranges
    }

    /// Returns a range of proofs to delete.
    /// A return value of `None` means that (for now) no more proofs should be deleted and the
    /// current run should end.
    fn locate_next_prune_range(
        database: &StateManagerDatabase<impl ReadableRocks>,
        last_pruned_state_version: StateVersion,
        limits_config: &LedgerSyncLimitsConfig,
    ) -> Option<ProofPruneRange> {
        let mut from_state_version_inclusive = last_pruned_state_version
            .next()
            .expect("state version overflow");

        let mut skipped_proofs = 0;
        loop {
            let syncable_txns_and_proof_result = database.get_syncable_txns_and_proof(
                from_state_version_inclusive,
                limits_config.max_txns_for_responses_spanning_more_than_one_proof,
                limits_config.max_txn_bytes_for_single_response,
            );

            match syncable_txns_and_proof_result {
                Ok(syncable_txns_and_proof) => {
                    // All good, we know the next proof to retain
                    return Some(ProofPruneRange {
                        from_state_version_inclusive,
                        to_proof_exclusive: syncable_txns_and_proof.proof,
                        skipped_proofs,
                    });
                }
                Err(err) => {
                    match err {
                        NothingToServeAtTheGivenStateVersion => {
                            // No more proofs
                            return None;
                        }
                        RefusedToServeGenesis { refused_proof } => {
                            // We have encountered the genesis proof, which shouldn't be pruned.
                            // Skipping to the next (post-proof) state version.
                            skipped_proofs += 1;
                            from_state_version_inclusive = refused_proof
                                .ledger_header
                                .state_version
                                .next()
                                .expect("state version overflow");
                            continue;
                        }
                        RefusedToServeProtocolUpdate { refused_proof } => {
                            // Similarly here, we're skipping all protocol update
                            // state versions and retrying from the next (post-update)
                            // version in the next iteration.
                            skipped_proofs += 1;
                            from_state_version_inclusive = refused_proof
                                .ledger_header
                                .state_version
                                .next()
                                .expect("state version overflow");
                            continue;
                        }
                        FailedToPrepareAResponseWithinLimits => {
                            // That's an error
                            error!(
                                "A chain of transactions-without-proof from state version {} does not fit within the limits {:?}; aborting the GC",
                                from_state_version_inclusive,
                                limits_config
                            );
                            return None;
                        }
                    }
                }
            }
        }
    }
}

/// Specifies a range of proofs to delete.
/// `to_proof_exclusive`'s state version specifies the upper bound for deletion (exclusive).
#[derive(Clone, Debug)]
pub struct ProofPruneRange {
    from_state_version_inclusive: StateVersion,
    to_proof_exclusive: LedgerProof,
    skipped_proofs: usize,
}

impl ProofPruneRange {
    pub fn to_state_version_exclusive(&self) -> StateVersion {
        self.to_proof_exclusive.ledger_header.state_version
    }
}

#[cfg(test)]
mod tests {
    use crate::engine_prelude::*;
    use crate::jni::LedgerSyncLimitsConfig;
    use crate::proofs_gc::{LedgerProofsGc, LedgerProofsGcConfig};
    use crate::protocol::*;
    use crate::store::consensus::traits::proofs::QueryableProofStore;
    use crate::test::{commit_round_updates_until_epoch, create_state_manager};
    use crate::consensus::traits::GetSyncableTxnsAndProofError;
    use crate::{StateManagerConfig, StateVersion};

    use std::time::Duration;

    #[test]
    fn test_retain_protocol_update_proofs() {
        let tmp = tempfile::tempdir().unwrap();
        let mut config = StateManagerConfig::new_for_testing(tmp.path().to_str().unwrap());
        // Disable scheduled proof GC
        config.ledger_proofs_gc_config = LedgerProofsGcConfig {
            interval_sec: 0,
            most_recent_full_resolution_epoch_count: 0,
        };
        // An unconditional protocol update at epoch 5
        config.protocol_config = ProtocolConfig::new_with_triggers(hashmap! {
            TestProtocolUpdateDefinition::subnamed("v2") =>
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(Epoch::of(
                    5,
                ))
        });
        let state_manager = create_state_manager(config);

        let db = state_manager.database.clone();

        // Testing config with 10 rounds per epoch
        let consensus_manager_config = ConsensusManagerConfig {
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
        };

        state_manager
            .system_executor
            .execute_genesis_for_unit_tests_with_config(consensus_manager_config);

        let sync_limits_config = LedgerSyncLimitsConfig {
            // Max 8 txns (0.8 epoch worth of round changes)
            max_txns_for_responses_spanning_more_than_one_proof: 8,
            // 1 MiB; unused in this test
            max_txn_bytes_for_single_response: 1_000_000,
        };

        // We're using our own GC instance in this test
        let gc = LedgerProofsGc {
            database: db.clone(),
            interval: Duration::MAX,
            most_recent_full_resolution_epoch_count: 2,
            limits_config: sync_limits_config.clone(),
        };

        commit_round_updates_until_epoch(&state_manager, Epoch::of(8));

        let database = db.access_direct();
        let post_genesis_epoch_proof = database.get_post_genesis_epoch_proof().unwrap();
        // The calculations below rely on this
        let first_post_genesis_state_version = post_genesis_epoch_proof
            .ledger_header
            .state_version
            .number()
            .checked_add(1)
            .unwrap();
        assert_eq!(6, first_post_genesis_state_version);

        // Calculated manually given: 10 rounds (txns) per epoch and a sync limit of 8 txns.
        // The values are: (start_state_version_inclusive, end_state_version_exclusive, expected_skipped_proofs)
        let expected_deleted_state_version_ranges = vec![
            (6, 13, 0),  // Ends mid epoch (8 txn response limit)
            (14, 15, 0), // Up to the end of epoch
            (16, 23, 0), // The pattern repeats
            (24, 25, 0), // -||-
            (26, 33, 0), // -||-
            (34, 35, 0), // This ends at an epoch change with a protocol update
            (37, 44, 1), // We're skipping the protocol update proof (version 36)
            (45, 46, 0), // Up to the end of final (5) epoch
                         // We're keeping 2 completed epochs of proofs (that is 6 and 7), so nothing more pruned
        ];

        // Run the GC and verify that the result matches expectations
        let pruned = gc.run();
        assert_eq!(pruned.len(), expected_deleted_state_version_ranges.len());
        for (pruned, expected) in pruned.iter().zip(expected_deleted_state_version_ranges) {
            assert_eq!(pruned.from_state_version_inclusive.number(), expected.0);
            assert_eq!(pruned.to_state_version_exclusive().number(), expected.1);
            assert_eq!(pruned.skipped_proofs, expected.2);
        }

        // Verify that sync works as expected
        let protocol_update_init_state_version = database
            .get_latest_protocol_update_init_proof()
            .unwrap()
            .ledger_header
            .state_version
            .number();

        let first_protocol_update_txn = protocol_update_init_state_version.checked_add(1).unwrap();

        let last_protocol_update_state_version = database
            .get_latest_protocol_update_execution_proof()
            .unwrap()
            .ledger_header
            .state_version
            .number();

        let first_post_protocol_update_state_version =
            last_protocol_update_state_version.checked_add(1).unwrap();

        let latest_state_version = database.max_state_version().number();

        let mut total_state_versions_tried = 0;
        // 0 -> post genesis version (exclusive): unsyncable
        for state_version in 0..first_post_genesis_state_version {
            let res = database.get_syncable_txns_and_proof(
                StateVersion::of(state_version),
                sync_limits_config.max_txns_for_responses_spanning_more_than_one_proof,
                sync_limits_config.max_txn_bytes_for_single_response,
            );
            assert!(matches!(
                res,
                Err(GetSyncableTxnsAndProofError::RefusedToServeGenesis { .. })
            ));
            total_state_versions_tried += 1;
        }

        // post genesis -> protocol update trigger (inclusive): syncable
        for state_version in first_post_genesis_state_version..=protocol_update_init_state_version {
            let res = database.get_syncable_txns_and_proof(
                StateVersion::of(state_version),
                sync_limits_config.max_txns_for_responses_spanning_more_than_one_proof,
                sync_limits_config.max_txn_bytes_for_single_response,
            );
            assert!(res.is_ok());
            assert!(!res.unwrap().txns.is_empty());
            total_state_versions_tried += 1;
        }

        // protocol update transactions (1): unsyncable
        for state_version in first_protocol_update_txn..=last_protocol_update_state_version {
            let res = database.get_syncable_txns_and_proof(
                StateVersion::of(state_version),
                sync_limits_config.max_txns_for_responses_spanning_more_than_one_proof,
                sync_limits_config.max_txn_bytes_for_single_response,
            );
            assert!(matches!(
                res,
                Err(GetSyncableTxnsAndProofError::RefusedToServeProtocolUpdate { .. })
            ));
            total_state_versions_tried += 1;
        }

        // post-protocol update transactions: syncable
        for state_version in first_post_protocol_update_state_version..=latest_state_version {
            let res = database.get_syncable_txns_and_proof(
                StateVersion::of(state_version),
                sync_limits_config.max_txns_for_responses_spanning_more_than_one_proof,
                sync_limits_config.max_txn_bytes_for_single_response,
            );
            assert!(res.is_ok());
            assert!(!res.unwrap().txns.is_empty());
            total_state_versions_tried += 1;
        }

        // Just to be sure we've covered all versions in the above groups
        assert_eq!(
            total_state_versions_tried,
            latest_state_version + 1 /* starting at 0, so +1 here */
        );
    }
}
