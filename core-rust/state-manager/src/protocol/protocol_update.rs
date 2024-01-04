// This file contains the protocol update logic for specific protocol versions

use node_common::locks::{LockFactory, RwLock, StateLock};
use radix_engine::track::StateUpdates;
use radix_engine::transaction::CostingParameters;
use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::prelude::Decimal;
use std::ops::Deref;
use std::sync::Arc;
use transaction::prelude::TransactionPayloadPreparable;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};
use utils::btreemap;

use crate::epoch_handling::EpochAwareAccuTreeFactory;
use crate::traits::{
    CommitBundle, CommitStore, CommittedTransactionBundle, HashTreeUpdate, QueryableProofStore,
    ReceiptAccuTreeSliceV1, SubstateStoreUpdate, TransactionAccuTreeSliceV1,
};
use crate::transaction::{
    ExecutionConfigurator, FlashTransactionV1, LedgerTransaction, LedgerTransactionValidator,
    PreparedLedgerTransaction, TransactionSeriesExecutor,
};
use crate::{
    CommittedTransactionIdentifiers, ExecutionCache, LedgerHeader, LedgerProof, LedgerProofOrigin,
    LoggingConfig, ProtocolState, StateManagerDatabase, GENESIS_PROTOCOL_VERSION,
};

pub trait ProtocolUpdaterFactory {
    fn supports_protocol_version(&self, protocol_version_name: &str) -> bool;

    fn updater_for(
        &self,
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Box<dyn ProtocolUpdater>;
}

/// Protocol update consists of two events:
/// 1) Updating the current (state computer) configuration ("transaction processing rules").
///    This includes: transaction validation, execution configuration, etc
/// 2) Executing arbitrary state updates against the current database state.
///    While the abstraction is quite flexible, the only concrete implementation at the moment
///    only modifies the state through committing system transactions (e.g. substate flash).
pub trait ProtocolUpdater {
    fn protocol_version_name(&self) -> String;
    fn state_computer_configurator(&self) -> StateComputerConfigurator;
    fn state_update_executor(&self) -> Box<dyn StateUpdateExecutor>;
}

pub trait StateUpdateExecutor {
    // TODO: document that the implementation must handle node restart / resume
    fn execute_remaining_state_updates(&self);
}

/// A protocol updater implementation that only changes the configuration
/// and does not commit any state updates.
pub struct NoStateUpdatesProtocolUpdater {
    protocol_version_name: String,
    state_computer_configurator: StateComputerConfigurator,
    store: Arc<StateLock<StateManagerDatabase>>,
}

impl NoStateUpdatesProtocolUpdater {
    pub fn default(
        protocol_version_name: String,
        network: NetworkDefinition,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Self {
        Self {
            protocol_version_name,
            state_computer_configurator: StateComputerConfigurator::default(network),
            store,
        }
    }
}

impl ProtocolUpdater for NoStateUpdatesProtocolUpdater {
    fn protocol_version_name(&self) -> String {
        self.protocol_version_name.clone()
    }

    fn state_computer_configurator(&self) -> StateComputerConfigurator {
        self.state_computer_configurator.clone()
    }

    fn state_update_executor(&self) -> Box<dyn StateUpdateExecutor> {
        Box::new(NoOpStateUpdateExecutor {
            protocol_version_name: self.protocol_version_name.clone(),
            store: self.store.clone(),
            state_computer_configurator: self.state_computer_configurator(),
        })
    }
}

struct NoOpStateUpdateExecutor {
    protocol_version_name: String,
    store: Arc<StateLock<StateManagerDatabase>>,
    state_computer_configurator: StateComputerConfigurator,
}

impl StateUpdateExecutor for NoOpStateUpdateExecutor {
    fn execute_remaining_state_updates(&self) {
        FlashStateUpdateExecutor::new(
            self.protocol_version_name.clone(),
            self.store.clone(),
            vec![StateUpdates {
                by_node: Default::default(),
            }],
            self.state_computer_configurator.clone(),
        )
        .execute_remaining_state_updates();
    }
}

pub struct FlashProtocolUpdater {
    protocol_version_name: String,
    store: Arc<StateLock<StateManagerDatabase>>,
    state_computer_configurator: StateComputerConfigurator,
    flash_transactions_updates: Vec<StateUpdates>,
}

impl FlashProtocolUpdater {
    pub fn new_with_default_configurator(
        protocol_version_name: String,
        store: Arc<StateLock<StateManagerDatabase>>,
        network: NetworkDefinition,
        flash_transactions_updates: Vec<StateUpdates>,
    ) -> Self {
        Self {
            protocol_version_name,
            store,
            state_computer_configurator: StateComputerConfigurator::default(network),
            flash_transactions_updates,
        }
    }
}

impl ProtocolUpdater for FlashProtocolUpdater {
    fn protocol_version_name(&self) -> String {
        self.protocol_version_name.clone()
    }

    fn state_computer_configurator(&self) -> StateComputerConfigurator {
        self.state_computer_configurator.clone()
    }

    fn state_update_executor(&self) -> Box<dyn StateUpdateExecutor> {
        // We're reusing a flash updater to commit a single empty flash
        // transactions and a corresponding proof to get us
        // to the next (post update) header.
        Box::new(FlashStateUpdateExecutor::new(
            self.protocol_version_name.clone(),
            self.store.clone(),
            self.flash_transactions_updates.clone(),
            self.state_computer_configurator(),
        ))
    }
}

struct FlashStateUpdateExecutor {
    protocol_version_name: String,
    store: Arc<StateLock<StateManagerDatabase>>,
    flash_transactions_updates: Vec<StateUpdates>,
    state_computer_configurator: StateComputerConfigurator,
}

impl FlashStateUpdateExecutor {
    pub fn new(
        protocol_version_name: String,
        store: Arc<StateLock<StateManagerDatabase>>,
        flash_transactions_updates: Vec<StateUpdates>,
        state_computer_configurator: StateComputerConfigurator,
    ) -> Self {
        Self {
            protocol_version_name,
            store,
            flash_transactions_updates,
            state_computer_configurator,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StateComputerConfigurator {
    network: NetworkDefinition,
    logging_config: LoggingConfig,
    validation_config: ValidationConfig,
    costing_parameters: CostingParameters,
}

impl StateComputerConfigurator {
    pub fn default(network: NetworkDefinition) -> StateComputerConfigurator {
        let network_id = network.id;
        StateComputerConfigurator {
            network,
            logging_config: LoggingConfig::default(),
            validation_config: ValidationConfig::default(network_id),
            costing_parameters: CostingParameters::default(),
        }
    }
}

impl StateComputerConfigurator {
    pub fn ledger_transaction_validator(&self) -> LedgerTransactionValidator {
        LedgerTransactionValidator::default_from_validation_config(self.validation_config)
    }

    pub fn user_transaction_validator(&self) -> NotarizedTransactionValidator {
        NotarizedTransactionValidator::new(self.validation_config)
    }

    pub fn validation_config(&self) -> ValidationConfig {
        self.validation_config
    }

    pub fn execution_configurator(&self, no_fees: bool) -> ExecutionConfigurator {
        let mut costing_parameters = self.costing_parameters;
        if no_fees {
            costing_parameters.execution_cost_unit_price = Decimal::ZERO;
            costing_parameters.finalization_cost_unit_price = Decimal::ZERO;
            costing_parameters.state_storage_price = Decimal::ZERO;
            costing_parameters.archive_storage_price = Decimal::ZERO;
        }
        ExecutionConfigurator::new(&self.network, &self.logging_config, costing_parameters)
    }
}

enum TxnCommitExecutorState {
    UpdateInitiatedButNothingCommitted,
    UpdateInProgress {
        last_batch_idx: u32,
    },
    /// This means that the last proof contains no notion of a protocol update,
    /// which in practice almost always means that it has already executed in full.
    /// But we leave this interpretation to the caller,
    /// so here we just call it "not updating".
    NotUpdating,
}

/// A helper (to the) `StateUpdateExecutor` that manages
/// transaction committing-based state updates.
/// It handles the logic to fulfill the resumability contract of "execute_remaining_state_updates"
/// by storing the index of a previously committed transaction batch in the `opaque` hash
/// of the proof. This hash is usually used to store the (consensus) vertex hash, but since
/// we're not working with vertices here, we repurpose it for protocol update needs.
struct TxnCommitStateUpdateExecutorHelper {
    protocol_version_name: String,
    store: Arc<StateLock<StateManagerDatabase>>,
    execution_configurator: RwLock<ExecutionConfigurator>,
    ledger_transaction_validator: LedgerTransactionValidator,
}

impl TxnCommitStateUpdateExecutorHelper {
    pub fn new(
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
        state_computer_configurator: StateComputerConfigurator,
    ) -> Self {
        Self {
            protocol_version_name: protocol_version_name.to_string(),
            store,
            execution_configurator: LockFactory::new("protocol_update")
                .new_rwlock(state_computer_configurator.execution_configurator(true)),
            ledger_transaction_validator: state_computer_configurator
                .ledger_transaction_validator(),
        }
    }

    fn read_state_from_last_proof(&self) -> TxnCommitExecutorState {
        let Some(last_proof) = self.store.read_current().get_last_proof() else {
            return TxnCommitExecutorState::NotUpdating;
        };

        match &last_proof.origin {
            LedgerProofOrigin::Genesis { .. } => TxnCommitExecutorState::NotUpdating,
            LedgerProofOrigin::Consensus { .. } => {
                if let Some(last_proof_protocol_version) =
                    last_proof.ledger_header.next_protocol_version
                {
                    // We've just committed a protocol update header. No protocol update transactions
                    // have been committed yet. Next batch idx is 0.

                    // Just a sanity check to double check that the header matches initial_protocol_state.
                    if last_proof_protocol_version != self.protocol_version_name {
                        panic!("Protocol state mismatch: last proof doesn't  match initial_protocol_state");
                    }

                    TxnCommitExecutorState::UpdateInitiatedButNothingCommitted
                } else {
                    TxnCommitExecutorState::NotUpdating
                }
            }
            LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name,
                batch_idx,
            } => {
                if *protocol_version_name == self.protocol_version_name {
                    TxnCommitExecutorState::UpdateInProgress {
                        last_batch_idx: *batch_idx,
                    }
                } else {
                    TxnCommitExecutorState::NotUpdating
                }
            }
        }
    }

    pub fn is_protocol_update_in_progress(&self) -> bool {
        match self.read_state_from_last_proof() {
            TxnCommitExecutorState::UpdateInitiatedButNothingCommitted => true,
            TxnCommitExecutorState::UpdateInProgress { .. } => true,
            TxnCommitExecutorState::NotUpdating => false,
        }
    }

    pub fn next_batch_idx(&self) -> u32 {
        match self.read_state_from_last_proof() {
            TxnCommitExecutorState::UpdateInitiatedButNothingCommitted => 0,
            TxnCommitExecutorState::UpdateInProgress { last_batch_idx } => last_batch_idx + 1,
            TxnCommitExecutorState::NotUpdating => panic!("Protocol update isn't in progress"),
        }
    }

    pub fn commit_batch(&mut self, transactions: Vec<LedgerTransaction>) {
        let batch_idx = self.next_batch_idx();

        let read_store = self.store.read_current();
        let last_proof: LedgerProof = read_store
            .get_last_proof()
            .expect("Pre-genesis protocol updates are currently not supported");

        println!("Committing protocol update batch {:?}, initial state version = {:?}, state hash = {:?}, txn hash = {:?}",
                 batch_idx, last_proof.ledger_header.state_version,
        last_proof.ledger_header.hashes.state_root,
        last_proof.ledger_header.hashes.transaction_root
        );

        let lock_factory = LockFactory::new("protocol_update");
        let execution_cache = lock_factory.new_mutex(ExecutionCache::new(
            last_proof.ledger_header.hashes.transaction_root,
        ));
        // For the purpose of executing protocol update transactions we're just going to use
        // a dummy protocol state with no configured updates and the name of this (in progress)
        // protocol update as the current version (although that could really be any string,
        // it doesn't matter here).
        let dummy_protocol_state = ProtocolState {
            current_epoch: None,
            current_protocol_version: self.protocol_version_name.clone(),
            enacted_protocol_updates: btreemap!(),
            unenacted_protocol_updates: vec![],
        };
        let mut series_executor = TransactionSeriesExecutor::new(
            read_store.deref(),
            &execution_cache,
            &self.execution_configurator,
            dummy_protocol_state,
        );

        let mut committed_transaction_bundles = Vec::new();
        let mut substate_store_update = SubstateStoreUpdate::new();
        let mut state_tree_update = HashTreeUpdate::new();
        let mut new_node_ancestry_records = Vec::new();
        let epoch_accu_trees = EpochAwareAccuTreeFactory::new(
            series_executor.epoch_identifiers().state_version,
            series_executor.latest_state_version(),
        );
        let mut transaction_tree_slice_merger = epoch_accu_trees.create_merger();
        let mut receipt_tree_slice_merger = epoch_accu_trees.create_merger();

        for transaction in transactions {
            let raw = transaction.to_raw().unwrap();
            let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw).unwrap();
            let validated = self.ledger_transaction_validator.validate_flash(prepared);

            let commit = series_executor
                .execute_and_update_state(&validated, "flash protocol update")
                .expect("protocol update not committable")
                .expect_success("protocol update");

            substate_store_update.apply(commit.database_updates);
            let hash_structures_diff = commit.hash_structures_diff;
            state_tree_update.add(
                series_executor.latest_state_version(),
                hash_structures_diff.state_hash_tree_diff,
            );
            new_node_ancestry_records.extend(commit.new_substate_node_ancestry_records);
            transaction_tree_slice_merger.append(hash_structures_diff.transaction_tree_diff.slice);
            receipt_tree_slice_merger.append(hash_structures_diff.receipt_tree_diff.slice);

            let proposer_timestamp_ms = last_proof.ledger_header.proposer_timestamp_ms;
            committed_transaction_bundles.push(CommittedTransactionBundle {
                state_version: series_executor.latest_state_version(),
                raw,
                receipt: commit.local_receipt,
                identifiers: CommittedTransactionIdentifiers {
                    payload: validated.create_identifiers(),
                    resultant_ledger_hashes: *series_executor.latest_ledger_hashes(),
                    proposer_timestamp_ms,
                },
            });
        }

        let resultant_state_version = series_executor.latest_state_version();
        let resultant_ledger_hashes = *series_executor.latest_ledger_hashes();
        let proof = LedgerProof {
            ledger_header: LedgerHeader {
                epoch: last_proof.ledger_header.epoch,
                round: last_proof.ledger_header.round,
                state_version: resultant_state_version,
                hashes: resultant_ledger_hashes,
                consensus_parent_round_timestamp_ms: last_proof
                    .ledger_header
                    .consensus_parent_round_timestamp_ms,
                proposer_timestamp_ms: last_proof.ledger_header.proposer_timestamp_ms,
                next_epoch: series_executor.next_epoch().cloned(),
                next_protocol_version: None,
            },
            origin: LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name: self.protocol_version_name.clone(),
                batch_idx,
            },
        };

        println!("Protocol update batch {:?} committed, proof state version = {:?}, proof state hash = {:?}, txn hash = {:?}", batch_idx,
                 proof.ledger_header.state_version,
                 proof.ledger_header.hashes.state_root,
                 proof.ledger_header.hashes.transaction_root,
        );

        let commit_bundle = CommitBundle {
            transactions: committed_transaction_bundles,
            proof,
            substate_store_update,
            vertex_store: None,
            state_tree_update,
            transaction_tree_slice: TransactionAccuTreeSliceV1(
                transaction_tree_slice_merger.into_slice(),
            ),
            receipt_tree_slice: ReceiptAccuTreeSliceV1(receipt_tree_slice_merger.into_slice()),
            new_substate_node_ancestry_records: new_node_ancestry_records,
        };

        drop(read_store);

        self.store.write_current().commit(commit_bundle);
    }
}

impl StateUpdateExecutor for FlashStateUpdateExecutor {
    fn execute_remaining_state_updates(&self) {
        let mut helper = TxnCommitStateUpdateExecutorHelper::new(
            self.protocol_version_name.as_str(),
            self.store.clone(),
            self.state_computer_configurator.clone(),
        );

        if !helper.is_protocol_update_in_progress() {
            // Nothing to do if we're not expecting any more commits
            return;
        }

        let mut next_batch_idx = helper.next_batch_idx();
        // TODO: add a comment (we need a unique nonce to be able to commit
        // the same state updates twice (e.g. in future protocol updates)
        // and we're just reusing the current state version as a nonce
        let mut nonce = self.store.read_current().max_state_version().number();
        while (next_batch_idx as usize) < self.flash_transactions_updates.len() {
            let next_flash_updates: &StateUpdates = self
                .flash_transactions_updates
                .get(next_batch_idx as usize)
                .unwrap();
            //
            let next_txn = LedgerTransaction::FlashV1(Box::new(FlashTransactionV1 {
                nonce,
                state_updates: next_flash_updates.clone(),
            }));
            nonce = nonce
                .checked_add(1)
                .expect("Nonce (state version) overflow");
            helper.commit_batch(vec![next_txn]);
            next_batch_idx = helper.next_batch_idx();
        }
    }
}

pub struct MainnetProtocolUpdaterFactory {}

impl MainnetProtocolUpdaterFactory {
    pub fn new() -> MainnetProtocolUpdaterFactory {
        MainnetProtocolUpdaterFactory {}
    }
}

impl Default for MainnetProtocolUpdaterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolUpdaterFactory for MainnetProtocolUpdaterFactory {
    fn supports_protocol_version(&self, protocol_version_name: &str) -> bool {
        [GENESIS_PROTOCOL_VERSION].contains(&protocol_version_name)
    }

    fn updater_for(
        &self,
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Box<dyn ProtocolUpdater> {
        match protocol_version_name {
            GENESIS_PROTOCOL_VERSION => Box::new(NoStateUpdatesProtocolUpdater::default(
                protocol_version_name.to_owned(),
                NetworkDefinition::mainnet(),
                store,
            )),
            _ => panic!("Unknown protocol version {:?}", protocol_version_name),
        }
    }
}

pub struct TestingDefaultProtocolUpdaterFactory {
    network: NetworkDefinition,
}

impl TestingDefaultProtocolUpdaterFactory {
    pub fn new(network: NetworkDefinition) -> TestingDefaultProtocolUpdaterFactory {
        TestingDefaultProtocolUpdaterFactory { network }
    }
}

impl ProtocolUpdaterFactory for TestingDefaultProtocolUpdaterFactory {
    fn supports_protocol_version(&self, _protocol_version_name: &str) -> bool {
        true
    }

    fn updater_for(
        &self,
        protocol_version_name: &str,
        store: Arc<StateLock<StateManagerDatabase>>,
    ) -> Box<dyn ProtocolUpdater> {
        // All default testing protocol updates are no-op
        Box::new(NoStateUpdatesProtocolUpdater::default(
            protocol_version_name.to_owned(),
            self.network.clone(),
            store,
        ))
    }
}
