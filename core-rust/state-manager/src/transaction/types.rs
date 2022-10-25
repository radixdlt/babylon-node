use sbor::{Decode, Encode, TypeId};
use scrypto::buffer::scrypto_encode;

use crate::transaction::validator::ValidatorTransaction;
use crate::TransactionPayloadHash;
use transaction::model::NotarizedTransaction;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum LedgerTransaction {
    User(NotarizedTransaction),
    Validator(ValidatorTransaction),
}

impl LedgerTransaction {
    pub fn get_hash(&self) -> TransactionPayloadHash {
        TransactionPayloadHash::for_transaction(self)
    }

    pub fn into_payload(self) -> Vec<u8> {
        scrypto_encode(&self)
    }
}
