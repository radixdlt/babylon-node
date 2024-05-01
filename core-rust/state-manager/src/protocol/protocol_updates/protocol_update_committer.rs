// This file contains the protocol update logic for specific protocol versions

use crate::engine_prelude::*;

use crate::protocol::*;
use crate::query::TransactionIdentifierLoader;
use crate::traits::*;
use crate::transaction::*;
use crate::{LedgerHeader, LedgerProof, LedgerProofOrigin, ReadableStore};

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum UpdateTransaction {
    FlashTransactionV1(FlashTransactionV1),
}

impl From<FlashTransactionV1> for UpdateTransaction {
    fn from(value: FlashTransactionV1) -> Self {
        Self::FlashTransactionV1(value)
    }
}

impl From<UpdateTransaction> for LedgerTransaction {
    fn from(value: UpdateTransaction) -> Self {
        match value {
            UpdateTransaction::FlashTransactionV1(flash_transaction) => {
                LedgerTransaction::FlashV1(Box::new(flash_transaction))
            }
        }
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
    transaction_executor_factory: &'s TransactionExecutorFactory,
    ledger_transaction_validator: &'s LedgerTransactionValidator,
}

impl<'s, S> ProtocolUpdateTransactionCommitter<'s, S>
where
    S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader + CommitStore,
{
    pub fn new(
        protocol_version_name: ProtocolVersionName,
        database: &'s S,
        transaction_executor_factory: &'s TransactionExecutorFactory,
        ledger_transaction_validator: &'s LedgerTransactionValidator,
    ) -> Self {
        Self {
            protocol_version_name,
            database,
            transaction_executor_factory,
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

    /// Commits a batch of flash transactions, followed by a single proof (of protocol update origin).
    pub fn commit_batch(&mut self, batch_idx: u32, update_transactions: Vec<UpdateTransaction>) {
        let latest_header = self
            .database
            .get_latest_proof()
            .expect("Pre-genesis protocol updates are currently not supported")
            .ledger_header;

        // Currently protocol updates are always executed at epoch boundary,
        // so the first batch's proof will use (next_epoch, 0) - based on the latest
        // consensus proof, and subsequent batches will use the same values based on
        // the proof for the previous batch.
        let (epoch, round) = if let Some(next_epoch) = latest_header.next_epoch {
            (next_epoch.epoch, Round::zero())
        } else {
            (latest_header.epoch, latest_header.round)
        };

        let mut series_executor = self
            .transaction_executor_factory
            .start_series_execution(self.database);
        let mut commit_bundle_builder = series_executor.start_commit_builder();

        for transaction in update_transactions {
            let raw = LedgerTransaction::from(transaction).to_raw().unwrap();
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
