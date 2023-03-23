use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use std::collections::BTreeMap;

use radix_engine::errors::RuntimeError;
use radix_engine::ledger::OutputValue;
use radix_engine::state_manager::StateDiff;
use radix_engine::system::kernel_modules::costing::FeeSummary;
use radix_engine::system::kernel_modules::execution_trace::ResourceChange;
use radix_engine::transaction::{
    CommitResult, StateUpdateSummary, TransactionExecutionTrace, TransactionOutcome,
};
use radix_engine::types::{hash, scrypto_encode, Decimal, Hash, Level, ObjectId, SubstateId};
use radix_engine_common::crypto::blake2b_256_hash;
use radix_engine_interface::data::scrypto::model::ComponentAddress;
use radix_engine_interface::*;
use sbor::rust::collections::IndexMap;

use crate::{AccumulatorHash, ConsensusReceipt, SubstateChangeHash};

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommittedTransactionIdentifiers {
    pub state_version: u64,
    pub accumulator_hash: AccumulatorHash,
}

impl CommittedTransactionIdentifiers {
    pub fn pre_genesis() -> Self {
        Self {
            state_version: 0,
            accumulator_hash: AccumulatorHash::pre_genesis(),
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SubstateChanges {
    pub created: BTreeMap<SubstateId, OutputValue>,
    pub updated: BTreeMap<SubstateId, OutputValue>,
    pub deleted: BTreeMap<SubstateId, DeletedSubstateVersion>,
}

impl SubstateChanges {
    pub fn upserted(&self) -> impl Iterator<Item = (&SubstateId, &OutputValue)> {
        self.created.iter().chain(self.updated.iter())
    }

    pub fn deleted_ids(&self) -> impl Iterator<Item = &SubstateId> {
        self.deleted.keys()
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct DeletedSubstateVersion {
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LedgerTransactionOutcome {
    Success,
    Failure,
}

impl LedgerTransactionOutcome {
    fn resolve(outcome: &TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(_) => LedgerTransactionOutcome::Success,
            TransactionOutcome::Failure(_) => LedgerTransactionOutcome::Failure,
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum DetailedTransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl From<TransactionOutcome> for DetailedTransactionOutcome {
    fn from(outcome: TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(output) => {
                Self::Success(
                    output
                        .into_iter()
                        .map(|o| {
                            // TODO: Clean this up
                            match o {
                                InstructionOutput::None => scrypto_encode(&()).unwrap(),
                                InstructionOutput::CallReturn(v) => v,
                            }
                        })
                        .collect(),
                )
            }
            TransactionOutcome::Failure(error) => Self::Failure(error),
        }
    }
}

/// A committed transaction (success or failure), extracted from the Engine's `TransactionReceipt`
/// of any locally-executed transaction (slightly post-processed).
/// It contains all the critical, deterministic pieces of the Engine's receipt, but also some of its
/// other parts - for this reason, it is very clearly split into 2 parts (on-ledger vs off-ledger).
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionReceipt {
    pub on_ledger: LedgerTransactionReceipt,
    pub local_execution: LocalTransactionExecution,
}

/// A part of the `LocalTransactionReceipt` which is completely stored on ledger. It contains only
/// the critical, deterministic pieces of the original Engine's `TransactionReceipt`.
/// All these pieces can be verified against the Receipt Root hash (found in the Ledger Proof).
/// Note: the Ledger Receipt is still a pretty large structure (i.e. containing entire collections,
/// like substate changes) and is not supposed to be hashed directly - it should instead go through
/// a `Consensus Receipt`.
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerTransactionReceipt {
    pub outcome: LedgerTransactionOutcome,
    pub substate_changes: SubstateChanges,
}

/// A computable/non-critical/non-deterministic part of the `LocalTransactionReceipt` (e.g. logs,
/// summaries).
/// It is not verifiable against ledger, but may still be useful for debugging.
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LocalTransactionExecution {
    pub outcome: DetailedTransactionOutcome,
    // The breakdown of the fee
    pub fee_summary: FeeSummary,
    // Which vault/s paid the fee
    pub fee_payments: IndexMap<ObjectId, Decimal>,
    pub application_logs: Vec<(Level, String)>,
    pub state_update_summary: StateUpdateSummary,
    // These will be removed once we have the parent_map for the toolkit to use
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub next_epoch: Option<(BTreeMap<ComponentAddress, Validator>, u64)>,
}

impl LedgerTransactionReceipt {
    pub fn get_consensus_receipt(&self) -> ConsensusReceipt {
        ConsensusReceipt {
            outcome: self.outcome.clone(),
            substate_change_root: Self::compute_substate_change_root(&self.substate_changes),
        }
    }

    fn compute_substate_change_root(substate_changes: &SubstateChanges) -> SubstateChangeHash {
        // TODO: implement using a merkle tree
        SubstateChangeHash::from(blake2b_256_hash(scrypto_encode(substate_changes).unwrap()))
    }
}

impl From<(CommitResult, TransactionExecutionTrace)> for LocalTransactionReceipt {
    fn from((commit_result, execution_trace): (CommitResult, TransactionExecutionTrace)) -> Self {
        let next_epoch = commit_result.next_epoch();
        Self {
            on_ledger: LedgerTransactionReceipt {
                outcome: LedgerTransactionOutcome::resolve(&commit_result.outcome),
                substate_changes: fix_state_updates(commit_result.state_updates),
            },
            local_execution: LocalTransactionExecution {
                outcome: commit_result.outcome.into(),
                fee_summary: commit_result.fee_summary,
                fee_payments: commit_result.fee_payments,
                application_logs: commit_result.application_logs,
                state_update_summary: commit_result.state_update_summary,
                resource_changes: execution_trace.resource_changes,
                next_epoch,
            },
        }
    }
}

fn fix_state_updates(state_updates: StateDiff) -> SubstateChanges {
    // As of end of August 2022, the engine's statediff erroneously includes substate reads
    // (even if the content didn't change) as ups and downs.
    // This needs fixing, but for now, we work around this here, by removing such up/down pairs.
    let mut possible_creations = state_updates.up_substates;
    let mut updated = BTreeMap::<SubstateId, OutputValue>::new();
    let mut deleted = BTreeMap::<SubstateId, DeletedSubstateVersion>::new();

    // We iterate over the downed substates, and attempt to match them with an upped substate
    // > If they match an upped substate, the down and up is erroneous, so we ignore both
    // > If it doesn't match, this is correct and is added to valid_down_substates
    for down_substate_output_id in state_updates.down_substates {
        let substate_id = down_substate_output_id.substate_id;
        let down_substate_hash = down_substate_output_id.substate_hash;
        match possible_creations.remove(&substate_id) {
            Some(up_substate_output_value) => {
                // TODO - this check can be removed when the bug is fixed
                let up_substate_hash =
                    hash(scrypto_encode(&up_substate_output_value.substate).unwrap());
                if up_substate_hash != down_substate_hash {
                    updated.insert(substate_id, up_substate_output_value);
                } else {
                    // Do nothing - this is erroneous
                }
            }
            None => {
                deleted.insert(
                    substate_id,
                    DeletedSubstateVersion {
                        substate_hash: down_substate_hash,
                        version: down_substate_output_id.version,
                    },
                );
            }
        }
    }

    // The remaining up_substates which didn't match with a down_substate are all creates
    let created = possible_creations;

    SubstateChanges {
        created,
        updated,
        deleted,
    }
}
