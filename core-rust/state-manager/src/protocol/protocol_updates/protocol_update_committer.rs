// This file contains the protocol update logic for specific protocol versions

use crate::engine_prelude::*;
use node_common::locks::LockFactory;

use crate::commit_bundle::CommitBundleBuilder;
use crate::protocol::*;
use crate::query::TransactionIdentifierLoader;
use crate::traits::*;
use crate::transaction::*;
use crate::{ExecutionCache, LedgerHeader, LedgerProof, LedgerProofOrigin, ReadableStore};

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum UpdateTransaction {
    FlashTransactionV1(FlashTransactionV1),
}

impl From<FlashTransactionV1> for UpdateTransaction {
    fn from(value: FlashTransactionV1) -> Self {
        Self::FlashTransactionV1(value)
    }
}

enum ProtocolUpdateProgress {
    UpdateInitiatedButNothingCommitted {
        protocol_version_name: ProtocolVersionName,
    },
    UpdateInProgress {
        protocol_version_name: ProtocolVersionName,
        last_batch_idx: u32,
    },
    /// This means that the last proof contains no notion of a protocol update,
    /// which in practice almost always means that it has already executed in full.
    /// But we leave this interpretation to the caller,
    /// so here we just call it "not updating".
    NotUpdating,
}

/// A helper that manages committing flash transactions state updates.
/// It handles the logic to fulfill the resumability contract of "execute_remaining_state_updates"
/// by storing the index of a previously committed transaction batch in the ledger proof.
pub struct ProtocolUpdateTransactionCommitter<'s, S> {
    protocol_version_name: ProtocolVersionName,
    database: &'s S,
    execution_configurator: ExecutionConfigurator,
    ledger_transaction_validator: LedgerTransactionValidator,
}

impl<'s, S> ProtocolUpdateTransactionCommitter<'s, S>
where
    S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader + CommitStore,
{
    pub fn new(
        protocol_version_name: ProtocolVersionName,
        database: &'s S,
        execution_configurator: ExecutionConfigurator,
        ledger_transaction_validator: LedgerTransactionValidator,
    ) -> Self {
        Self {
            protocol_version_name,
            database,
            execution_configurator,
            ledger_transaction_validator,
        }
    }

    fn read_protocol_update_progress(&self) -> ProtocolUpdateProgress {
        let Some(latest_proof) = self.database.get_latest_proof() else {
            return ProtocolUpdateProgress::NotUpdating;
        };

        match &latest_proof.origin {
            LedgerProofOrigin::Genesis { .. } => ProtocolUpdateProgress::NotUpdating,
            LedgerProofOrigin::Consensus { .. } => {
                if let Some(latest_proof_protocol_version) =
                    latest_proof.ledger_header.next_protocol_version
                {
                    ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                        protocol_version_name: ProtocolVersionName::of_unchecked(
                            latest_proof_protocol_version,
                        ),
                    }
                } else {
                    ProtocolUpdateProgress::NotUpdating
                }
            }
            LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name,
                batch_idx,
            } => ProtocolUpdateProgress::UpdateInProgress {
                protocol_version_name: protocol_version_name.clone(),
                last_batch_idx: *batch_idx,
            },
        }
    }

    pub fn next_committable_batch_idx(&self) -> Option<u32> {
        match self.read_protocol_update_progress() {
            ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                protocol_version_name: state_protocol_version_name,
            } => {
                if self.protocol_version_name == state_protocol_version_name {
                    Some(0)
                } else {
                    None
                }
            }
            ProtocolUpdateProgress::UpdateInProgress {
                protocol_version_name: state_protocol_version_name,
                last_batch_idx,
            } => {
                if self.protocol_version_name == state_protocol_version_name {
                    Some(last_batch_idx.checked_add(1).unwrap())
                } else {
                    None
                }
            }
            ProtocolUpdateProgress::NotUpdating => None,
        }
    }

    pub fn commit_single(&mut self, transaction: UpdateTransaction) {
        self.commit_batch(vec![transaction]);
    }

    /// Commits a batch of flash transactions, followed by a single
    /// proof (of protocol update origin).
    pub fn commit_batch(&mut self, update_transactions: Vec<UpdateTransaction>) {
        let ledger_transactions = update_transactions
            .into_iter()
            .map(|update_txn| match update_txn {
                UpdateTransaction::FlashTransactionV1(flash_txn) => {
                    LedgerTransaction::FlashV1(Box::new(flash_txn))
                }
            })
            .collect();
        self.commit_txn_batch(ledger_transactions);
    }

    fn commit_txn_batch(&mut self, transactions: Vec<LedgerTransaction>) {
        let batch_idx = self
            .next_committable_batch_idx()
            .expect("Can't commit next protocol update batch");

        let latest_proof: LedgerProof = self
            .database
            .get_latest_proof()
            .expect("Pre-genesis protocol updates are currently not supported");
        let latest_header = latest_proof.ledger_header;

        // Currently protocol updates are always executed at epoch boundary,
        // so the first batch's proof will use (next_epoch, 0) - based on the latest
        // consensus proof, and subsequent batches will use the same values based on
        // the proof for the previous batch.
        let (epoch, round) = if let Some(next_epoch) = latest_header.next_epoch {
            (next_epoch.epoch, Round::zero())
        } else {
            (latest_header.epoch, latest_header.round)
        };

        let lock_factory = LockFactory::new("protocol_update");
        let execution_cache =
            lock_factory.new_mutex(ExecutionCache::new(latest_header.hashes.transaction_root));
        // For the purpose of executing protocol update transactions we're just going to use
        // a dummy protocol state with no configured updates and the name of this (in progress)
        // protocol update as the current version (although that could really be any string,
        // it doesn't matter here).
        let dummy_protocol_state = ProtocolState {
            current_protocol_version: self.protocol_version_name.clone(),
            enacted_protocol_updates: btreemap!(),
            pending_protocol_updates: vec![],
        };

        let mut series_executor = TransactionSeriesExecutor::new(
            self.database,
            &execution_cache,
            &self.execution_configurator,
            dummy_protocol_state,
        );

        let mut commit_bundle_builder = CommitBundleBuilder::new(
            series_executor.epoch_identifiers().state_version,
            series_executor.latest_state_version(),
        );

        for transaction in transactions {
            let raw = transaction.to_raw().unwrap();
            let prepared = PreparedLedgerTransaction::prepare_from_raw(&raw).unwrap();
            let validated = self.ledger_transaction_validator.validate_flash(prepared);

            let commit = series_executor
                .execute_and_update_state(&validated, "flash protocol update")
                .expect("protocol update not committable")
                .expect_success("protocol update");

            commit_bundle_builder.add_executed_transaction(
                series_executor.latest_state_version(),
                latest_header.proposer_timestamp_ms,
                raw,
                validated,
                commit,
            );
        }

        let resultant_state_version = series_executor.latest_state_version();
        let resultant_ledger_hashes = *series_executor.latest_ledger_hashes();
        let proof = LedgerProof {
            ledger_header: LedgerHeader {
                epoch,
                round,
                state_version: resultant_state_version,
                hashes: resultant_ledger_hashes,
                consensus_parent_round_timestamp_ms: latest_header
                    .consensus_parent_round_timestamp_ms,
                proposer_timestamp_ms: latest_header.proposer_timestamp_ms,
                next_epoch: series_executor.epoch_change().map(|ev| ev.into()),
                next_protocol_version: None,
            },
            origin: LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name: self.protocol_version_name.clone(),
                batch_idx,
            },
        };

        self.database
            .commit(commit_bundle_builder.build(proof, None));
    }
}
