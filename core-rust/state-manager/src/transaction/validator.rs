use sbor::*;
use scrypto::args;
use scrypto::buffer::scrypto_encode;
use scrypto::constants::SYS_FAUCET_COMPONENT;
use scrypto::core::{NativeFnIdentifier, Receiver, SystemFnIdentifier};
use scrypto::crypto::hash;
use scrypto::engine::types::RENodeId;
use scrypto::math::Decimal;
use transaction::model::{AuthModule, Instruction, MethodIdentifier, Validated};

#[derive(Debug, Copy, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ValidatorTransaction {
    EpochUpdate(u64),
}

impl From<ValidatorTransaction> for Validated<ValidatorTransaction> {
    fn from(validator_transaction: ValidatorTransaction) -> Self {
        let transaction_hash = hash(scrypto_encode(&validator_transaction)); // TODO: Figure out better way to do this or if we even do need it

        let instruction = match validator_transaction {
            ValidatorTransaction::EpochUpdate(epoch) => Instruction::CallMethod {
                method_identifier: MethodIdentifier::Native {
                    receiver: Receiver::Ref(RENodeId::System),
                    native_fn_identifier: NativeFnIdentifier::System(SystemFnIdentifier::SetEpoch),
                },
                args: args!(epoch),
            },
        };

        let instructions = vec![
            // TODO: Remove lock fee requirement
            Instruction::CallMethod {
                method_identifier: MethodIdentifier::Scrypto {
                    component_address: SYS_FAUCET_COMPONENT,
                    ident: "lock_fee".to_string(),
                },
                args: args!(Decimal::from(1000u32)),
            },
            instruction,
        ];
        Validated::new(
            validator_transaction,
            transaction_hash,
            instructions,
            vec![AuthModule::validator_role_nf_address()],
            10_000_000,
            0,
            vec![],
        )
    }
}
