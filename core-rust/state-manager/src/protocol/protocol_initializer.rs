// This file contains the protocol update logic for specific protocol versions

use std::io::Empty;
use radix_engine::track::StateUpdates;
use radix_engine::transaction::CostingParameters;
use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::prelude::Decimal;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};

use crate::transaction::{ExecutionConfigurator, LedgerTransactionValidator};
use crate::{LoggingConfig, padded_protocol_version_name, ProtocolState, ReadableStore};
use crate::traits::{CommitBundle, CommitStore, IterableProofStore, QueryableProofStore, QueryableTransactionStore};

pub const PROTO_BABYLON_GENESIS: &str = "babylon-genesis";
pub const PROTO_TESTING_V2: &str = "testing-v2";

pub struct ProtocolConfigurator {
    protocol_version_name: String,
    network: NetworkDefinition,
    logging_config: LoggingConfig,
    validation_config: ValidationConfig,
    costing_parameters: CostingParameters,
}

impl <S: QueryableProofStore + IterableProofStore + QueryableTransactionStore> ProtocolConfigurator {
    pub fn for_protocol_version(
        protocol_version_name: &str,
        network: &NetworkDefinition,
        logging_config: &LoggingConfig,
    ) -> ProtocolConfigurator {
        ProtocolConfigurator {
            protocol_version_name: protocol_version_name.to_string(),
            network: network.clone(),
            logging_config: logging_config.clone(),
            validation_config: ValidationConfig::default(network.id),
            costing_parameters: CostingParameters::default(),
        }
    }
}

impl ProtocolConfigurator {
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
        ExecutionConfigurator::new(&self.network, &self.logging_config, self.costing_parameters)
    }

    pub fn protocol_state(&self) -> ProtocolState {
        self.protocol_state.clone()
    }
}

impl <'s, S: ReadableStore + QueryableProofStore + CommitStore> ProtocolConfigurator {
    pub fn update_executor(&self, store: &'s S) -> Box<dyn ProtocolUpdateExecutor> {
        match self.protocol_version_name {
            PROTO_BABYLON_GENESIS => Box::new(EmptyProtocolUpdateExecutor::new()),
            PROTO_TESTING_V2 => FlashProtocolUpdateExecutor::new(store, vec![]),
            _ => panic!("Unknown protocol version {:?}", self.protocol_version_name)
        }
    }
}

trait ProtocolUpdateExecutor {
    fn commit_remaining_transactions(&self);
}

struct EmptyProtocolUpdateExecutor {}

impl EmptyProtocolUpdateExecutor {
    pub fn new() -> EmptyProtocolUpdateExecutor {
        EmptyProtocolUpdateExecutor {
        }
    }
}

impl ProtocolUpdateExecutor for EmptyProtocolUpdateExecutor {
    fn commit_remaining_transactions(&self) {
        // no-op
    }
}

struct FlashProtocolUpdateExecutor<'s, S: ReadableStore + QueryableProofStore + CommitStore> {
    state_updates_batches: Vec<StateUpdates>,
    store: &'s S,
}

impl <'s, S: ReadableStore + QueryableProofStore + CommitStore> FlashProtocolUpdateExecutor<'s, S> {
    pub fn new(store: &'s S, state_updates_batches: Vec<StateUpdates>) -> FlashProtocolUpdateExecutor<'s, S> {
        FlashProtocolUpdateExecutor {
            state_updates_batches,
            store,
        }
    }

    fn prepare_next_commit_bundle(&self, bundle_idx: u32) -> Option<CommitBundle> {
        // execute next bundle using self.store
        // and populate the proof / header
        None
        /*
        CommitBundle {
            transactions: vec![],
            proof: LedgerProofV1 {
                opaque: Hash(),
                ledger_header: LedgerHeader {
                    epoch: (),
                    round: (),
                    state_version: (),
                    hashes: LedgerHashes {
                        state_root: StateHash(),
                        transaction_root: TransactionTreeHash(),
                        receipt_root: ReceiptTreeHash(),
                    },
                    consensus_parent_round_timestamp_ms: 0,
                    proposer_timestamp_ms: 0,
                    next_epoch: None,
                    next_protocol_version: None,
                },
                timestamped_signatures: vec![],
            },
            substate_store_update: Default::default(),
            vertex_store: None,
            state_tree_update: Default::default(),
            transaction_tree_slice: TransactionAccuTreeSliceV1(),
            receipt_tree_slice: ReceiptAccuTreeSliceV1(),
            new_substate_node_ancestry_records: vec![],
        }
         */
    }
}

impl <'s, S> ProtocolUpdateExecutor for FlashProtocolUpdateExecutor<'s, S> {
    fn commit_remaining_transactions(&self) {
        let Some(last_proof) = self.store.get_last_proof() else {
            return;
        };

        let mut next_batch_idx =
            if let Some(last_proof_protocol_version) = last_proof.ledger_header.next_protocol_version {
                // We've just committed a protocol update header. No protocol update transactions
                // have been committed yet. Next batch idx is 0.

                // Just a sanity check to double check that the header matches initial_protocol_state.
                if last_proof_protocol_version != self.protocol_version_name {
                    panic!("Protocol state mismatch: last proof doesn't  match initial_protocol_state");
                }

                0
            } else {
                // We're either mid-protocol update (resume after reboot) or already done.
                // We're reusing the opaque hash in the ledger proof to store the protocol update progress.
                // In case of mid-protocol update proofs the "hash" (32 bytes) consists of:
                // - bytes 0-11: 0 padding
                // - bytes 12-15: u32 index of the last committed batch (zero padded, big endian)
                // - bytes 16-32: 16 bytes of protocol_version_name (left padded with 0s)
                // For example a hash of:
                // 00000000000000000000000000000005000000000000626162796C6F6E2D7632
                // means the we've last committed batch 5 of the "babylon-v2" protocol update.
                // If it doesn't match the above pattern for our current protocol_version_name
                // we consider it to be a "regular" proof (with a consensus-populated opaque hash)
                // and, consequently, consider the protocol update completed.
                if last_proof.opaque.0[16..32] ==
                    padded_protocol_version_name(&self.protocol_state.current_protocol_version).as_bytes() {
                    // Protocol version name matches, we're mid-protocol update
                    // or we've just committed the last batch.
                    u32::from_be_bytes(last_proof.opaque.0[12..15].try_into().unwrap())
                } else {
                    return;
                }
            };

        while let Some(next_commit_bundle) = self.prepare_next_commit_bundle(next_batch_idx) {
            next_batch_idx += 1;
            self.store.commit(next_commit_bundle);
        }
    }
}





/*
to miałem w state computer:


        // Run (or resume) protocol update transactions
        while let Some(transactions) = protocol_initializer.init_transactions(next_checkpoint_id) {
            let read_store = self.store.read_current();
            let mut series_executor = self.start_series_execution(&read_store);

            let mut committed_transaction_bundles = vec![];
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
                let validated = self.ledger_transaction_validator.read()
                    .validate_genesis(prepared);
                let commit = series_executor.execute_and_update_state(&validated, "protocol update")
                    .expect("Protocol update transaction failed");
                substate_store_update.apply(commit.database_updates);
                let hash_structures_diff = commit.hash_structures_diff;
                state_tree_update.add(
                    series_executor.latest_state_version(),
                    hash_structures_diff.state_hash_tree_diff,
                );
                new_node_ancestry_records.extend(commit.new_substate_node_ancestry_records);
                transaction_tree_slice_merger.append(hash_structures_diff.transaction_tree_diff.slice);
                receipt_tree_slice_merger.append(hash_structures_diff.receipt_tree_diff.slice);

                committed_transaction_bundles.push(CommittedTransactionBundle {
                    state_version: series_executor.latest_state_version(),
                    raw,
                    receipt: commit.local_receipt,
                    identifiers: CommittedTransactionIdentifiers {
                        payload: validated.create_identifiers(),
                        resultant_ledger_hashes: *series_executor.latest_ledger_hashes(),
                        proposer_timestamp_ms: current_header.proposer_timestamp_ms,
                    },
                });
            }

            let proof = LedgerProof {
                opaque: Hash([0u8; 32]), // checkpoint ID?
                ledger_header: LedgerHeader {
                    epoch: current_header.epoch,
                    round: current_header.round,
                    state_version: series_executor.latest_state_version(),
                    hashes: series_executor.latest_ledger_hashes().clone(),
                    consensus_parent_round_timestamp_ms: current_header.consensus_parent_round_timestamp_ms,
                    proposer_timestamp_ms: current_header.proposer_timestamp_ms,
                    next_epoch: None, // no epoch changes mid-protocol update
                    next_protocol_version: None, // no protocol updates mid-protocol update
                },
                timestamped_signatures: vec![], // no signatures for protocol updates
            };
            drop(read_store);

            let mut write_store = self.store.write_current();
            // PROOF opaque can contain the checkpoint ID! no need for a separate param
            write_store.commit(CommitBundle {
                transactions: committed_transaction_bundles,
                proof,
                substate_store_update,
                vertex_store: None, // no vertex store for protocol updates
                state_tree_update,
                transaction_tree_slice: TransactionAccuTreeSliceV1(
                    transaction_tree_slice_merger.into_slice(),
                ),
                receipt_tree_slice: ReceiptAccuTreeSliceV1(receipt_tree_slice_merger.into_slice()),
                new_substate_node_ancestry_records: new_node_ancestry_records,
            });
            drop(write_store);

            // execute all and then

            // commit (reuse the same commit as real commit and add an optional checkpoint id??)




            next_checkpoint_id += 1;
        }

 */