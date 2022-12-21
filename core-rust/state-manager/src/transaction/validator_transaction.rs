use radix_engine::types::GlobalAddress;
use radix_engine::types::NativeMethodIdent;
use radix_engine::types::RENodeId;

use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};

use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::{ClockSetCurrentTimeInvocation, EpochManagerNextRoundInvocation};
use radix_engine_interface::modules::auth::AuthAddresses;
use sbor::*;
use std::collections::BTreeSet;
use transaction::model::{
    AuthZoneParams, Executable, ExecutionContext, FeePayment, Instruction, InstructionList,
    SystemInstruction,
};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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
                let update_time = SystemInstruction::CallNativeMethod {
                    method_ident: NativeMethodIdent {
                        receiver: RENodeId::Global(GlobalAddress::System(CLOCK)),
                        method_name: "set_current_time".to_string(),
                    },
                    args: scrypto_encode(&ClockSetCurrentTimeInvocation {
                        receiver: CLOCK,
                        current_time_ms: *timestamp_ms,
                    })
                        .unwrap(),
                };
                let update_round = SystemInstruction::CallNativeMethod {
                    method_ident: NativeMethodIdent {
                        receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
                        method_name: "next_round".to_string(),
                    },
                    args: scrypto_encode(&EpochManagerNextRoundInvocation {
                        receiver: EPOCH_MANAGER,
                        round: *round_in_epoch,
                    })
                        .unwrap(),
                };

                vec![Instruction::System(update_time), Instruction::System(update_round)]
            },
        };

        PreparedValidatorTransaction {
            hash,
            instructions,
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
