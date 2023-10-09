use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use radix_engine::system::bootstrap::create_substate_flash_for_genesis;
use radix_engine::track::StateUpdates;
use radix_engine::transaction::{CostingParameters, execute_transaction, ExecutionConfig};
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine_common::prelude::{NetworkDefinition, scrypto_decode};
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use rocksdb::{DB, Direction, IteratorMode, Options};
use flume;
use sbor::HasLatestVersion;
use transaction::model::Executable;
use crate::query::TransactionIdentifierLoader;
use crate::{ProcessedCommitResult, StateHash, StateVersion, VersionedCommittedTransactionIdentifiers, VersionedLedgerTransactionReceipt};
use crate::store::{DatabaseFlags, RocksDBStore, StateManagerDatabase};
use crate::store::traits::{CommitStore, SubstateStoreUpdate};
use crate::transaction::{LedgerTransactionValidator, PreparedGenesisTransaction, PreparedLedgerTransactionInner, RawLedgerTransaction};

fn should_log(state_version: StateVersion) -> bool {
    state_version.number() < 700 || state_version.number() % 1000 < 5
}

pub struct StateComputerLite {
    network: NetworkDefinition,
    store: StateManagerDatabase,
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    tx_validator: LedgerTransactionValidator,
}

impl StateComputerLite {
    pub fn new_with_rocksdb(network: NetworkDefinition, db_root: &str) -> Self {
        let db = {
            match RocksDBStore::new(PathBuf::from(db_root), DatabaseFlags::default()) {
                Ok(db) => db,
                Err(_error) => panic!("Couldn't create a DB")
            }
        };
        Self::new(
            network,
            StateManagerDatabase::RocksDB(db))
    }

    pub fn new(network: NetworkDefinition, db: StateManagerDatabase) -> Self {
        Self {
            network: network.clone(),
            store: db,
            scrypto_vm: ScryptoVm::<DefaultWasmEngine>::default(),
            tx_validator: LedgerTransactionValidator::new(&network),
        }
    }

    pub fn latest_committed_state_version(&mut self) -> StateVersion {
        self.store.get_state_computer_lite_latest_state_version()
            .unwrap_or(StateVersion::pre_genesis())
    }

    pub fn commit(
        &mut self,
        state_version: StateVersion,
        raw_ledger_transaction: RawLedgerTransaction,
        identifiers: VersionedCommittedTransactionIdentifiers,
    ) {
        let prepared = self
            .tx_validator
            .prepare_from_raw(&raw_ledger_transaction)
            .expect("Couldn't prepare raw ledger transaction");

        match &prepared.inner {
            PreparedLedgerTransactionInner::Genesis(prepared_genesis_tx) => {
                match prepared_genesis_tx.as_ref() {
                    PreparedGenesisTransaction::Flash(_) => {
                        self.flash_and_commit(state_version);
                    }
                    PreparedGenesisTransaction::Transaction(_) => {
                        self.execute_and_commit(
                            state_version,
                            self.tx_validator.validate_genesis(prepared).get_executable(),
                            ExecutionConfig::for_genesis_transaction(self.network.clone()),
                            identifiers,
                        );
                    }
                }
            }
            PreparedLedgerTransactionInner::UserV1(_) =>
                {
                    self.execute_and_commit(
                        state_version,
                        self.tx_validator.validate_user_or_round_update(prepared)
                            .expect("User transaction validation failed")
                            .get_executable(),
                        ExecutionConfig::for_notarized_transaction(self.network.clone()),
                        identifiers,
                    );
                },
            PreparedLedgerTransactionInner::RoundUpdateV1(_) =>
                {
                    self.execute_and_commit(
                        state_version,
                        self.tx_validator.validate_user_or_round_update(prepared)
                            .expect("Round update transaction validation failed")
                            .get_executable(),
                        ExecutionConfig::for_system_transaction(self.network.clone()),
                        identifiers,
                    );
                }
        };
    }

    fn flash_and_commit(&mut self, state_version: StateVersion) {
        let flash_receipt = create_substate_flash_for_genesis();
        self.commit_state_updates(state_version, &flash_receipt.state_updates);
    }

    fn execute_and_commit(
        &mut self,
        state_version: StateVersion,
        executable: Executable,
        execution_config: ExecutionConfig,
        identifiers: VersionedCommittedTransactionIdentifiers,
    ) {
        let receipt = execute_transaction(
            &self.store,
            Vm {
                scrypto_vm: &self.scrypto_vm,
                native_vm: DefaultNativeVm::new(),
            },
            &CostingParameters::default(),
            &execution_config,
            &executable,
        );
        let commit = receipt.expect_commit_ignore_outcome();
        let new_root = self.commit_state_updates(state_version, &commit.state_updates);

        if new_root != identifiers.into_latest().resultant_ledger_hashes.state_root {
            panic!("State hash mismatch at state version {}", state_version);
        }
    }

    fn commit_state_updates(
        &mut self,
        state_version: StateVersion,
        state_updates: &StateUpdates
    ) -> StateHash {
        let database_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();

        let state_tree_update = ProcessedCommitResult::compute_state_tree_update(
            &self.store,
            state_version.previous().unwrap_or(StateVersion::pre_genesis()),
            &database_updates);

        let substate_store_update = SubstateStoreUpdate {
            updates: database_updates,
        };

        let new_root = state_tree_update.new_root;

        self.store.commit_lite(state_version, substate_store_update, state_tree_update);

        if should_log(state_version) {
            println!("Committed {} (state root = {})", state_version, new_root.0);
        }

        new_root
    }
}

pub fn main() {
    // Source ledger directory (e.g. `/.../RADIXDB/state_manager`)
    let source_db_path = "...";
    // Working directory for the secondary database (I believe it's just for logs)
    let source_db_secondary_workdir_path = "...";
    // "Lite" ledger directory
    let state_computer_lite_db_path = "...";

    let mut state_computer_lite = StateComputerLite::new_with_rocksdb(
        NetworkDefinition::mainnet(),
        state_computer_lite_db_path,
    );

    let latest_committed_state_version: StateVersion = state_computer_lite
        .latest_committed_state_version();

    let (tx, rx) = flume::bounded(10);

    // One thread reads from the source DB (which is opened as a secondary DB)
    // and adds the transactions to the queue (blocks if full).
    let txn_read_thread_handle = thread::spawn(move || {
        println!("Starting txn reader thread...");
        println!("Opening secondary DB...");
        let db = DB::open_cf_as_secondary(
            &Options::default(),
            PathBuf::from(source_db_path).as_path(),
            PathBuf::from(source_db_secondary_workdir_path).as_path(),
            vec!["raw_ledger_transactions", "committed_transaction_identifiers"]
        ).unwrap();

        let mut iterator_start_state_version = latest_committed_state_version.next();
        loop {
            db.try_catch_up_with_primary()
                .expect("DB catch up with primary failed");
            let mut txn_iter = db.iterator_cf(
                &db.cf_handle("raw_ledger_transactions").unwrap(),
                IteratorMode::From(
                    &iterator_start_state_version.unwrap().to_bytes(),
                    Direction::Forward
                ),
            );
            let mut identifiers_iter = db.iterator_cf(
                &db.cf_handle("committed_transaction_identifiers").unwrap(),
                IteratorMode::From(
                    &iterator_start_state_version.unwrap().to_bytes(),
                    Direction::Forward
                ),
            );
            while let Some(next_txn) = txn_iter.next() {
                let next_txn = next_txn.unwrap();
                let next_state_version = StateVersion::from_bytes(next_txn.0.as_ref());
                let next_raw_ledger_transaction =
                    RawLedgerTransaction(next_txn.1.to_vec());
                let next_identifiers_bytes = identifiers_iter.next()
                    .expect("Missing txn identifiers")
                    .unwrap();

                let next_identifiers: VersionedCommittedTransactionIdentifiers
                    = scrypto_decode(next_identifiers_bytes.1.as_ref()).unwrap();

                tx.send((next_state_version, next_raw_ledger_transaction, next_identifiers)).unwrap();
                iterator_start_state_version = next_state_version.next();
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Another thread reads the transaction from the queue
    // and commits them into its own DB.
    let state_computer_thread_handle = thread::spawn(move || {
        println!("Starting txn processing thread...");
        let mut iter = rx.iter();
        loop {
            let (next_state_version, next_raw_ledger_transaction, next_identifiers) = iter.next().unwrap();
            state_computer_lite.commit(next_state_version, next_raw_ledger_transaction, next_identifiers);
        }
    });

    txn_read_thread_handle.join().unwrap();
    state_computer_thread_handle.join().unwrap();
}

#[test]
fn test_run() {
    main();
}
