use radix_engine::types::*;

use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::clock::{
    ClockSetCurrentTimeInput, CLOCK_SET_CURRENT_TIME_IDENT,
};
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerNextRoundInput, EPOCH_MANAGER_NEXT_ROUND_IDENT,
};
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use std::collections::BTreeSet;
use transaction::model::{
    AuthZoneParams, Executable, ExecutionContext, FeePayment, Instruction,
};

#[derive(Debug, Copy, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    RoundUpdate {
        proposer_timestamp_ms: i64,
        // We include epoch because our current database implementation needs
        // to ensure all ledger payloads are unique.
        // Currently scrypto epoch != consensus epoch, but this will change
        consensus_epoch: u64,
        round_in_epoch: u64,
    },
}

impl ValidatorTransaction {
    pub fn prepare(&self) -> PreparedValidatorTransaction {
        // TODO: Figure out better way to do this or if we even do need it
        let hash = hash(scrypto_encode(self).unwrap());

        let instructions = match self {
            ValidatorTransaction::RoundUpdate {
                proposer_timestamp_ms: timestamp_ms,
                round_in_epoch,
                ..
            } => {
                let update_time = Instruction::CallMethod {
                    component_address: CLOCK,
                    method_name: CLOCK_SET_CURRENT_TIME_IDENT.to_string(),
                    args: scrypto_encode(&ClockSetCurrentTimeInput {
                        current_time_ms: *timestamp_ms,
                    })
                    .unwrap(),
                };

                let update_round = Instruction::CallMethod {
                    component_address: EPOCH_MANAGER,
                    method_name: EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
                    args: scrypto_encode(&EpochManagerNextRoundInput {
                        round: *round_in_epoch,
                    })
                    .unwrap(),
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
            initial_proofs: vec![AuthAddresses::validator_role()],
            virtualizable_proofs_resource_addresses: BTreeSet::new(),
        };

        Executable::new_no_blobs(
            self.instructions,
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
