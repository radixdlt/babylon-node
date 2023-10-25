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

use radix_engine::types::{Categorize, Decode, Encode};

use radix_engine_common::types::Epoch;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::store::traits::gc::{
    LedgerProofsGcProgress, LedgerProofsGcProgressV1, LedgerProofsGcStore,
};
use crate::store::traits::proofs::QueryableProofStore;

use crate::store::StateManagerDatabase;

use crate::jni::LedgerSyncLimitsConfig;
use node_common::locks::StateLock;

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
    database: Arc<StateLock<StateManagerDatabase>>,
    interval: Duration,
    most_recent_full_resolution_epoch_count: u64,
    limits_config: LedgerSyncLimitsConfig,
}

impl LedgerProofsGc {
    /// Creates a new GC.
    pub fn new(
        database: Arc<StateLock<StateManagerDatabase>>,
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
    pub fn run(&self) {
        // TODO(locks/snapshots): The GC's operation does not interact with the "current state", and
        // intuitively could use the "historical, non-locked" DB access. However, we have a (very
        // related) use-case, which lists arbitrary past transactions and their proof - and it could
        // happen that a concurrently-running GC deletes the very proof that has to be used to fit
        // within the limits. For this reason, the logic below carefully acquires/releases the DB's
        // read/write lock for relevant stages of the execution.
        // A more robust solution should either use DB snapshots (a development direction we are
        // considering anyway), or a more selective lock over the ledger proofs CF alone (or even
        // over a selected *range* of the ledger proofs CF).

        // Read the GC's persisted state and initialize the run:
        let read_progress_database = self.database.read_current();
        let to_epoch = read_progress_database
            .max_completed_epoch()
            .map(|max_completed_epoch| max_completed_epoch.number())
            .and_then(|number| number.checked_sub(self.most_recent_full_resolution_epoch_count))
            .map(Epoch::of);
        let Some(to_epoch) = to_epoch else {
            // Nothing to GC ever, yet.
            return;
        };
        let progress_started_at: LedgerProofsGcProgress =
            read_progress_database.get_progress().unwrap_or_else(|| {
                LedgerProofsGcProgress::new(
                    read_progress_database
                        .get_post_genesis_epoch_proof()
                        .expect("we checked that there is some completed epoch above")
                        .ledger_header,
                )
            });
        drop(read_progress_database);

        if progress_started_at.last_pruned_epoch >= to_epoch {
            // Nothing to GC during this run.
            return;
        }

        info!(
            "Starting a GC run: pruning ledger proofs up to epoch {}; current progress: {:?}",
            to_epoch.number(),
            progress_started_at,
        );
        let mut last_pruned_state_version = progress_started_at.epoch_proof_state_version;

        let mut retained_proofs = 0; // only for logging purposes
        loop {
            let batch_start_state_version_inclusive = last_pruned_state_version
                .next()
                .expect("state version overflow");

            // Locate the next proof that we need to retain. We use the same method and limits as
            // the business logic responsible for outputting proven transactions:
            let locate_proof_database = self.database.read_current();
            let transactions_and_proof = locate_proof_database.get_txns_and_proof(
                batch_start_state_version_inclusive,
                self.limits_config
                    .max_txns_for_responses_spanning_more_than_one_proof,
                self.limits_config.max_txn_bytes_for_single_response,
            );
            drop(locate_proof_database);

            let Some((_transactions, proof)) = transactions_and_proof else {
                error!(
                    "A chain of transactions-without-proof from state version {} does not fit within the limits {:?}; aborting the GC",
                    batch_start_state_version_inclusive, self.limits_config
                );
                return;
            };
            let header = proof.ledger_header;

            // Delete all the proofs from the beginning of transactions listing up to (but
            // excluding) the returned proof:
            last_pruned_state_version = header.state_version;
            let delete_database = self.database.write_current();
            delete_database.delete_ledger_proofs_range(
                batch_start_state_version_inclusive,
                last_pruned_state_version,
            );

            retained_proofs += 1;
            if let Some(next_epoch) = header.next_epoch {
                info!(
                    "Recording progress of pruned epoch {} (having {} retained proofs)",
                    header.epoch.number(),
                    retained_proofs
                );
                retained_proofs = 0;
                delete_database.set_progress(LedgerProofsGcProgressV1 {
                    last_pruned_epoch: header.epoch,
                    epoch_proof_state_version: last_pruned_state_version,
                });
                if next_epoch.epoch >= to_epoch {
                    break;
                }
            }
            drop(delete_database);
        }

        info!("Ledger proofs' GC run finished");
    }
}
