use radix_engine::types::EpochManagerSetEpochInvocation;
use radix_engine::types::GlobalAddress;
use radix_engine::types::NativeMethodIdent;
use radix_engine::types::RENodeId;

use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};

use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::ClockSetCurrentTimeInvocation;
use sbor::*;
use std::collections::BTreeSet;
use transaction::model::{
    AuthModule, AuthZoneParams, Executable, ExecutionContext, FeePayment, Instruction,
    TransactionManifest,
};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    EpochUpdate(u64),
    TimeUpdate(u64),
}

impl ValidatorTransaction {
    pub fn prepare(&self) -> PreparedValidatorTransaction {
        // TODO: Figure out better way to do this or if we even do need it
        let validator_role_nf_address = hash(scrypto_encode(self).unwrap());

        let instruction = match self {
            ValidatorTransaction::EpochUpdate(epoch) => Instruction::CallNativeMethod {
                method_ident: NativeMethodIdent {
                    receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
                    method_name: "set_epoch".to_string(),
                },
                args: scrypto_encode(&EpochManagerSetEpochInvocation {
                    receiver: EPOCH_MANAGER,
                    epoch: *epoch,
                })
                .unwrap(),
            },
            ValidatorTransaction::TimeUpdate(current_time_ms) => Instruction::CallNativeMethod {
                method_ident: NativeMethodIdent {
                    receiver: RENodeId::Global(GlobalAddress::System(CLOCK)),
                    method_name: "set_current_time".to_string(),
                },
                args: scrypto_encode(&ClockSetCurrentTimeInvocation {
                    receiver: CLOCK,
                    current_time_ms: *current_time_ms,
                })
                .unwrap(),
            },
        };

        PreparedValidatorTransaction {
            hash: validator_role_nf_address,
            manifest: TransactionManifest {
                instructions: vec![instruction],
                blobs: vec![],
            },
        }
    }
}

#[derive(Debug, Clone, TypeId, PartialEq, Eq)]
pub struct PreparedValidatorTransaction {
    hash: Hash,
    manifest: TransactionManifest,
}

impl PreparedValidatorTransaction {
    pub fn get_executable(&self) -> Executable {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);

        let auth_zone_params = AuthZoneParams {
            initial_proofs: vec![AuthModule::validator_role_non_fungible_address()],
            virtualizable_proofs_resource_addresses: BTreeSet::new(),
        };

        Executable::new(
            &self.manifest.instructions,
            &self.manifest.blobs,
            ExecutionContext {
                transaction_hash,
                auth_zone_params,
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
            },
        )
    }
}
