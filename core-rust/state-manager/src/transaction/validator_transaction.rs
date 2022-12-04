use radix_engine::types::Decimal;
use radix_engine::types::EpochManagerSetEpochInvocation;
use radix_engine::types::GlobalAddress;
use radix_engine::types::NativeMethodIdent;
use radix_engine::types::RENodeId;
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::api::types::{ScryptoMethodIdent, ScryptoReceiver};
use radix_engine_interface::constants::{EPOCH_MANAGER, FAUCET_COMPONENT};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::args;
use radix_engine_interface::data::scrypto_encode;
use sbor::*;
use std::collections::BTreeSet;
use transaction::model::{
    AuthModule, AuthZoneParams, Executable, ExecutionContext, FeePayment, Instruction,
    TransactionManifest,
};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    EpochUpdate(u64),
}

impl ValidatorTransaction {
    pub fn prepare(&self) -> PreparedValidatorTransaction {
        // TODO: Figure out better way to do this or if we even do need it
        let validator_role_nf_address = hash(scrypto_encode(self).unwrap());

        let main_instruction = match self {
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
        };

        let instructions = vec![
            // TODO - given that we use the system fee reserve to run this
            // We should be able to try to remove the lock fee here?
            Instruction::CallMethod {
                method_ident: ScryptoMethodIdent {
                    receiver: ScryptoReceiver::Global(FAUCET_COMPONENT),
                    method_name: "lock_fee".to_string(),
                },
                args: args!(Decimal::from(100u32)),
            },
            main_instruction,
        ];

        PreparedValidatorTransaction {
            hash: validator_role_nf_address,
            manifest: TransactionManifest {
                instructions,
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
                fee_payment: FeePayment {
                    cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
                    tip_percentage: 0,
                },
                runtime_validations: vec![],
            },
        )
    }
}
