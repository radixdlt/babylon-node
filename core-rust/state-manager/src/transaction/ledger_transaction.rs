use radix_engine::types::{scrypto_decode, scrypto_encode};
use radix_engine_interface::scrypto;
use sbor::{Categorize, Decode, DecodeError, Encode, EncodeError};

use crate::transaction::validator_transaction::ValidatorTransaction;
use crate::LedgerPayloadHash;
use transaction::model::{NotarizedTransaction, SystemTransaction};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(Categorize, Encode, Decode)]
pub enum LedgerTransaction {
    User(NotarizedTransaction),
    Validator(ValidatorTransaction),
    System(SystemTransaction),
}

impl LedgerTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn get_hash(&self) -> LedgerPayloadHash {
        LedgerPayloadHash::for_transaction(self)
    }

    pub fn create_payload(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }

    pub fn user(&self) -> Option<&NotarizedTransaction> {
        match self {
            LedgerTransaction::User(tx) => Some(tx),
            LedgerTransaction::Validator(..) => None,
            LedgerTransaction::System(..) => None,
        }
    }
}
