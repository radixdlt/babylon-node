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

use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;

use crate::commit_bundle::CommitBundleBuilder;
use crate::protocol::*;
use crate::query::*;
use crate::staging::ReadableStore;
use crate::store::traits::*;
use crate::transaction::*;
use crate::*;

use crate::engine_prelude::*;

pub struct TransactionExecutorFactory {
    execution_configurator: Arc<ExecutionConfigurator>,
    execution_cache_manager: Arc<ExecutionCacheManager>,
    protocol_state_manager: Arc<ProtocolStateManager>,
}

impl TransactionExecutorFactory {
    pub fn new(
        execution_configurator: Arc<ExecutionConfigurator>,
        execution_cache_manager: Arc<ExecutionCacheManager>,
        protocol_state_manager: Arc<ProtocolStateManager>,
    ) -> Self {
        Self {
            execution_configurator,
            execution_cache_manager,
            protocol_state_manager,
        }
    }

    pub fn execute_isolated<S: ReadableStore>(
        &self,
        store: &S,
        transaction: &ValidatedLedgerTransaction,
        description: &'static str,
    ) -> TransactionReceipt {
        self.execution_configurator
            .wrap_ledger_transaction(transaction, description)
            .execute_on(store)
    }

    pub fn start_series_execution<'s, S>(&'s self, store: &'s S) -> TransactionSeriesExecutor<'s, S>
    where
        S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader,
    {
        TransactionSeriesExecutor::new(
            store,
            &self.execution_cache_manager,
            self.execution_configurator.deref(),
            self.protocol_state_manager.current_protocol_state(),
        )
    }
}

/// An internal delegate for executing a series of consecutive transactions while tracking their
/// progress.
pub struct TransactionSeriesExecutor<'s, S> {
    store: &'s S,
    execution_cache_manager: &'s ExecutionCacheManager,
    execution_configurator: &'s ExecutionConfigurator,
    epoch_identifiers: EpochTransactionIdentifiers,
    epoch_header: Option<LedgerHeader>,
    state_tracker: StateTracker,
}

impl<'s, S> TransactionSeriesExecutor<'s, S>
where
    S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader,
{
    /// Creates a new executor for a lifetime of entire transaction batch execution (i.e. for all
    /// transactions in a prepared vertex, or in a commit request).
    /// The borrowed `store` should be already locked (i.e. final database writes, if any, should be
    /// performed under the same lock).
    /// The locking of the borrowed `execution_cache` will be handled by this executor.
    pub fn new(
        store: &'s S,
        execution_cache_manager: &'s ExecutionCacheManager,
        execution_configurator: &'s ExecutionConfigurator,
        protocol_state: ProtocolState,
    ) -> Self {
        let epoch_header = store
            .get_latest_epoch_proof()
            .map(|epoch_proof| epoch_proof.ledger_header);
        Self {
            store,
            execution_cache_manager,
            execution_configurator,
            epoch_identifiers: epoch_header
                .as_ref()
                .map(EpochTransactionIdentifiers::from)
                .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis),
            epoch_header,
            state_tracker: StateTracker::new(store.get_top_ledger_hashes(), protocol_state),
        }
    }

    /// Executes the given already-validated ledger transaction (against the borrowed `store` and
    /// `execution_cache`).
    /// Uses an internal [`StateTracker`] to track the progression of *committable* transactions.
    /// Note that this method should NOT be used if a *committable* transaction is to be in
    /// some other way rejected by an upper layer (because then the subsequent
    /// execute_* calls may use an invalid state, e.g. ledger hashes may include the
    /// hash of the transaction, which the upper layer decided to discard).
    /// The passed description will only be used for logging/errors/panics (and will be augmented by
    /// the transaction's ledger hash).
    pub fn execute_and_update_state(
        &mut self,
        transaction: &ValidatedLedgerTransaction,
        description: &'static str,
    ) -> Result<ProcessedCommitResult, ProcessedRejectResult> {
        let result = self.execute_no_state_update(transaction, description);
        if let Ok(commit) = &result {
            self.update_state(commit);
        }
        result
    }

    /// Executes the given already-validated ledger transaction (against the borrowed `store` and
    /// `execution_cache`).
    /// Uses an internal [`StateTracker`] in a read-only mode. Specifically, does NOT
    /// update it with commit result (which can later be done by calling [`TransactionSeriesExecutor::update_state()`]).
    /// The passed description will only be used for logging/errors/panics (and will be augmented by
    /// the transaction's ledger hash).
    pub fn execute_no_state_update(
        &mut self,
        transaction: &ValidatedLedgerTransaction,
        description: &'static str,
    ) -> Result<ProcessedCommitResult, ProcessedRejectResult> {
        let description = DescribedTransactionHash {
            ledger_hash: transaction.ledger_transaction_hash(),
            description,
        };
        self.execute_wrapped_no_state_update(
            &description,
            self.execution_configurator
                .wrap_ledger_transaction(transaction, &description),
        )
    }

    /// Creates an empty [`CommitBundleBuilder`] ready to collect commits from the current state
    /// version reached by this executor.
    pub fn start_commit_builder(&self) -> CommitBundleBuilder {
        CommitBundleBuilder::new(
            self.epoch_identifiers.state_version,
            self.state_tracker.state_version,
        )
    }

    fn execute_wrapped_no_state_update<T: for<'l> TransactionLogic<StagedStore<'l, S>>>(
        &mut self,
        description: &DescribedTransactionHash,
        wrapped_executable: T,
    ) -> Result<ProcessedCommitResult, ProcessedRejectResult> {
        let mut execution_cache = self.execution_cache_manager.access_exclusively();
        let processed = execution_cache.execute_transaction(
            self.store,
            self.epoch_identifiers(),
            self.state_tracker.state_version,
            &self.state_tracker.ledger_hashes.transaction_root,
            &description.ledger_hash,
            wrapped_executable,
        );
        processed.expect_commit_or_reject(&description).cloned()
    }

    pub fn update_state(&mut self, commit: &ProcessedCommitResult) {
        self.state_tracker.update(commit);
    }

    /// Returns a ledger header which started the current epoch (i.e. in which the transactions are
    /// being executed), or [`None`] if the ledger state is pre-genesis.
    pub fn epoch_header(&self) -> Option<&LedgerHeader> {
        self.epoch_header.as_ref()
    }

    /// Returns transaction identifiers at the beginning of the current epoch (i.e. in which the
    /// transactions are being executed), or [`EpochTransactionIdentifiers::pre_genesis`].
    pub fn epoch_identifiers(&self) -> &EpochTransactionIdentifiers {
        &self.epoch_identifiers
    }

    /// Returns the ledger hashes resulting from the most recent `execute()` call.
    pub fn latest_ledger_hashes(&self) -> &LedgerHashes {
        &self.state_tracker.ledger_hashes
    }

    /// Returns the state version after the most recent `execute()` call.
    pub fn latest_state_version(&self) -> StateVersion {
        self.state_tracker.state_version
    }

    /// Returns the epoch change indication resulting from the most recent `execute()` call, or
    /// [`None`] if that call did not change the epoch.
    /// Please note that it is illegal (and enforced by this executor) to execute transactions after
    /// the epoch change.
    pub fn epoch_change(&self) -> Option<EpochChangeEvent> {
        self.state_tracker.epoch_change.clone()
    }

    /// Returns the protocol state resulting from the most recent `execute()` call.
    pub fn protocol_state(&self) -> ProtocolState {
        self.state_tracker.protocol_state.clone()
    }

    pub fn next_protocol_version(&self) -> Option<ProtocolVersionName> {
        self.state_tracker.next_protocol_version()
    }
}

/// A simple `Display` augmenting the human-readable transaction description with its ledger hash.
struct DescribedTransactionHash {
    ledger_hash: LedgerTransactionHash,
    description: &'static str,
}

impl Display for DescribedTransactionHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (ledger hash {:?})",
            self.description, self.ledger_hash
        )
    }
}

/// A low-level tracker of consecutive state version / ledger hashes /
/// epoch changes / protocol state changes.
struct StateTracker {
    state_version: StateVersion,
    ledger_hashes: LedgerHashes,
    epoch_change: Option<EpochChangeEvent>,
    protocol_state: ProtocolState,
    next_protocol_version: Option<ProtocolVersionName>,
}

impl StateTracker {
    /// Initializes the tracker to a known state (assuming it is not an end-state of an epoch).
    pub fn new(
        ledger_hashes_entry: (StateVersion, LedgerHashes),
        protocol_state: ProtocolState,
    ) -> Self {
        Self {
            state_version: ledger_hashes_entry.0,
            ledger_hashes: ledger_hashes_entry.1,
            epoch_change: None,
            protocol_state,
            next_protocol_version: None,
        }
    }

    /// Updates the internal state of this state tracker according to commit result.
    /// This includes:
    /// * bumping the state version
    /// * recording the next ledger hashes (from the given transaction results)
    /// * updating the protocol state
    ///
    /// This method validates that no further transaction should happen after an epoch change.
    pub fn update(&mut self, result: &ProcessedCommitResult) {
        if let Some(epoch_change) = &self.epoch_change {
            panic!(
                "the {:?} has happened at {:?} (version {}) and must not be followed by {:?}",
                epoch_change,
                self.ledger_hashes,
                self.state_version,
                result.hash_structures_diff.ledger_hashes
            );
        }

        if let Some(next_protocol_version) = &self.next_protocol_version {
            panic!(
                "the protocol update {:?} has happened at {:?} (version {}) and must not be followed by {:?}",
                next_protocol_version,
                self.ledger_hashes,
                self.state_version,
                result.hash_structures_diff.ledger_hashes
            );
        }

        self.state_version = self
            .state_version
            .next()
            .expect("Invalid next state version!");
        self.ledger_hashes = result.hash_structures_diff.ledger_hashes;
        self.epoch_change = result.epoch_change();

        let (protocol_state, next_protocol_version) =
            self.protocol_state.compute_next(&result.local_receipt, self.state_version);
        self.protocol_state = protocol_state;
        self.next_protocol_version = next_protocol_version;
    }

    pub fn next_protocol_version(&self) -> Option<ProtocolVersionName> {
        self.next_protocol_version.clone()
    }
}
