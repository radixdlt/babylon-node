use radix_engine::types::EpochManagerSetEpochInvocation;

use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};

use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::{
    ClockInvocation, ClockSetCurrentTimeInvocation, EpochManagerInvocation, NativeInvocation,
};
use radix_engine_interface::modules::auth::AuthAddresses;
use sbor::*;
use std::collections::BTreeSet;
use transaction::model::{
    AuthZoneParams, Executable, ExecutionContext, FeePayment, Instruction, InstructionList,
};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    EpochUpdate {
        scrypto_epoch: u64,
    },
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

        let instruction = match self {
            ValidatorTransaction::EpochUpdate { scrypto_epoch } => NativeInvocation::EpochManager(
                EpochManagerInvocation::SetEpoch(EpochManagerSetEpochInvocation {
                    receiver: EPOCH_MANAGER,
                    epoch: *scrypto_epoch,
                }),
            ),
            ValidatorTransaction::RoundUpdate {
                proposer_timestamp_ms: timestamp_ms,
                ..
            } => NativeInvocation::Clock(ClockInvocation::SetCurrentTime(
                ClockSetCurrentTimeInvocation {
                    receiver: CLOCK,
                    current_time_ms: *timestamp_ms,
                },
            )),
        };

        PreparedValidatorTransaction {
            hash,
            instructions: vec![Instruction::System(instruction)],
        }
    }
}

#[derive(Debug, Clone, TypeId, PartialEq, Eq)]
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
            InstructionList::AnyOwned(self.instructions),
            ExecutionContext {
                transaction_hash: self.hash,
                payload_size: 0,
                auth_zone_params,
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
            },
        )
    }
}
