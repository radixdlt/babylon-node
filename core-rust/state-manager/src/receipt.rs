use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use std::collections::BTreeMap;

use radix_engine::errors::RuntimeError;
use radix_engine::ledger::OutputValue;
use radix_engine::state_manager::StateDiff;
use radix_engine::system::kernel_modules::costing::FeeSummary;
use radix_engine::system::kernel_modules::execution_trace::ResourceChange;
use radix_engine::transaction::{
    StateUpdateSummary, TransactionOutcome, TransactionReceipt as EngineTransactionReceipt,
    TransactionResult,
};
use radix_engine::types::{hash, scrypto_encode, Decimal, Hash, Level, ObjectId, SubstateId};
use radix_engine_interface::api::types::EventTypeIdentifier;
use radix_engine_interface::data::scrypto::model::ComponentAddress;
use radix_engine_interface::*;
use sbor::rust::collections::IndexMap;

use crate::{AccumulatorHash, LedgerReceiptHash};

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

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CommittedTransactionStatus {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SubstateChanges {
    pub created: BTreeMap<SubstateId, OutputValue>,
    pub updated: BTreeMap<SubstateId, OutputValue>,
    pub deleted: BTreeMap<SubstateId, DeletedSubstateVersion>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct DeletedSubstateVersion {
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LedgerTransactionOutcome {
    Success(Vec<Vec<u8>>),
    Failure(RuntimeError),
}

impl From<TransactionOutcome> for LedgerTransactionOutcome {
    fn from(outcome: TransactionOutcome) -> Self {
        match outcome {
            TransactionOutcome::Success(output) => {
                LedgerTransactionOutcome::Success(
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
            TransactionOutcome::Failure(error) => LedgerTransactionOutcome::Failure(error),
        }
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerTransactionReceipt {
    pub outcome: LedgerTransactionOutcome,
    // The breakdown of the fee
    pub fee_summary: FeeSummary,
    // Which vault/s paid the fee
    pub fee_payments: IndexMap<ObjectId, Decimal>,
    pub application_logs: Vec<(Level, String)>,
    pub application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub substate_changes: SubstateChanges,
    pub state_update_summary: StateUpdateSummary,
    // These will be removed once we have the parent_map for the toolkit to use
    pub resource_changes: IndexMap<usize, Vec<ResourceChange>>,
    pub next_epoch: Option<(BTreeMap<ComponentAddress, Validator>, u64)>,
}

impl LedgerTransactionReceipt {
    pub fn get_hash(&self) -> LedgerReceiptHash {
        LedgerReceiptHash::for_receipt(self)
    }
}

impl TryFrom<EngineTransactionReceipt> for LedgerTransactionReceipt {
    type Error = String;

    fn try_from(engine_receipt: EngineTransactionReceipt) -> Result<Self, Self::Error> {
        match engine_receipt.result {
            TransactionResult::Commit(commit_result) => {
                let next_epoch = commit_result.next_epoch();
                let ledger_receipt = LedgerTransactionReceipt {
                    outcome: commit_result.outcome.into(),
                    fee_summary: commit_result.fee_summary,
                    fee_payments: commit_result.fee_payments,
                    application_logs: commit_result.application_logs,
                    application_events: commit_result.application_events,
                    substate_changes: fix_state_updates(commit_result.state_updates),
                    state_update_summary: commit_result.state_update_summary,
                    resource_changes: engine_receipt.execution_trace.resource_changes,
                    next_epoch,
                };
                Ok(ledger_receipt)
            }
            TransactionResult::Reject(error) => Err(format!(
                "Can't create a ledger receipt for rejected txn: {error:?}"
            )),
            TransactionResult::Abort(result) => Err(format!(
                "Can't create a ledger receipt for aborted txn: {result:?}"
            )),
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
