use sbor::*;
use scrypto::args;
use scrypto::buffer::scrypto_encode;
use scrypto::constants::{SYS_FAUCET_COMPONENT, SYS_SYSTEM_COMPONENT};
use scrypto::core::{MethodIdent, NativeMethod, Receiver, ReceiverMethodIdent, SystemMethod};
use scrypto::crypto::hash;
use scrypto::engine::types::{GlobalAddress, RENodeId};
use scrypto::math::Decimal;
use std::collections::BTreeSet;
use transaction::model::{AuthModule, AuthZoneParams, Executable, Instruction};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    EpochUpdate(u64),
}

impl From<ValidatorTransaction> for Executable {
    fn from(validator_transaction: ValidatorTransaction) -> Self {
        let transaction_hash = hash(scrypto_encode(&validator_transaction)); // TODO: Figure out better way to do this or if we even do need it

        let instruction = match validator_transaction {
            ValidatorTransaction::EpochUpdate(epoch) => Instruction::CallMethod {
                method_ident: ReceiverMethodIdent {
                    receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                        SYS_SYSTEM_COMPONENT,
                    ))),
                    method_ident: MethodIdent::Native(NativeMethod::System(SystemMethod::SetEpoch)),
                },
                args: args!(epoch),
            },
        };

        let instructions = vec![
            // TODO: Remove lock fee requirement
            Instruction::CallMethod {
                method_ident: ReceiverMethodIdent {
                    receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                        SYS_FAUCET_COMPONENT,
                    ))),
                    method_ident: MethodIdent::Scrypto("lock_fee".to_string()),
                },
                args: args!(Decimal::from(100u32)),
            },
            instruction,
        ];

        Executable::new(
            transaction_hash,
            instructions,
            AuthZoneParams {
                initial_proofs: vec![AuthModule::validator_role_nf_address()],
                virtualizable_proofs_resource_addresses: BTreeSet::new(),
            },
            10_000_000,
            0,
            vec![],
        )
    }
}
