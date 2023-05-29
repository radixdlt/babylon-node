use radix_engine_interface::data::manifest::{manifest_decode, manifest_encode};
use radix_engine_interface::*;
use sbor::*;

use crate::transaction::validator_transaction::ValidatorTransaction;
use crate::LedgerPayloadHash;
use transaction::model::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub enum LedgerTransaction {
    User(NotarizedTransaction),
    Validator(ValidatorTransaction),
    System(SystemTransaction),
}

impl LedgerTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        manifest_decode(slice)
    }

    pub fn get_hash(&self) -> LedgerPayloadHash {
        LedgerPayloadHash::for_transaction(self)
    }

    pub fn create_payload(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(self)
    }

    pub fn create_payload_and_hash(&self) -> (Vec<u8>, LedgerPayloadHash) {
        let payload = self.create_payload().unwrap();
        let hash = LedgerPayloadHash::for_ledger_payload_bytes(&payload);
        (payload, hash)
    }

    pub fn user(&self) -> Option<&NotarizedTransaction> {
        match self {
            LedgerTransaction::User(tx) => Some(tx),
            LedgerTransaction::Validator(..) => None,
            LedgerTransaction::System(..) => None,
        }
    }
}
