use radix_engine::types::*;

use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use transaction::model::*;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    RoundUpdate {
        proposer_timestamp_ms: i64,
        // We include epoch because our current database implementation needs
        // to ensure all ledger payloads are unique.
        epoch: u64,
        round: u64,
        leader_proposal_history: LeaderProposalHistory,
    },
}

impl ValidatorTransaction {
    pub fn prepare(&self) -> PreparedValidatorTransaction {
        // TODO: Figure out better way to do this or if we even do need it
        let hash = hash(manifest_encode(self).unwrap());

        let instructions = match self {
            ValidatorTransaction::RoundUpdate {
                proposer_timestamp_ms,
                epoch: _, // we deliberately ignore this bit, which is only needed for transaction uniqueness
                round,
                leader_proposal_history,
            } => {
                let update_time = Instruction::CallMethod {
                    component_address: CLOCK,
                    method_name: CLOCK_SET_CURRENT_TIME_IDENT.to_string(),
                    args: to_manifest_value(&ClockSetCurrentTimeInput {
                        current_time_ms: *proposer_timestamp_ms,
                    }),
                };

                let update_round = Instruction::CallMethod {
                    component_address: EPOCH_MANAGER,
                    method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
                    args: to_manifest_value(&EpochManagerNextRoundInput {
                        round: *round,
                        leader_proposal_history: leader_proposal_history.clone(),
                    }),
                };

                vec![update_time, update_round]
            }
        };

        PreparedValidatorTransaction { hash, instructions }
    }
}

#[derive(Debug, Clone, Categorize, PartialEq, Eq)]
pub struct PreparedValidatorTransaction {
    hash: Hash,
    instructions: Vec<Instruction>,
}

impl PreparedValidatorTransaction {
    pub fn to_executable(self) -> Executable<'static> {
        let auth_zone_params = AuthZoneParams {
            initial_proofs: btreeset!(AuthAddresses::validator_role()),
            virtual_resources: BTreeSet::new(),
        };

        Executable::new_no_blobs(
            self.instructions.as_ref(),
            ExecutionContext {
                transaction_hash: self.hash,
                payload_size: 0,
                auth_zone_params,
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
                pre_allocated_ids: BTreeSet::new(),
            },
        )
    }
}
