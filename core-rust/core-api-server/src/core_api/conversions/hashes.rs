use state_manager::{
    AccumulatorHash, IntentHash, LedgerPayloadHash, SignaturesHash, UserPayloadHash,
};

use crate::core_api::*;

#[allow(dead_code)]
pub fn to_api_intent_hash(intent_hash: &IntentHash) -> String {
    to_hex(intent_hash)
}

pub fn to_api_signed_intent_hash(signatures_hash: &SignaturesHash) -> String {
    to_hex(signatures_hash)
}

pub fn to_api_payload_hash(payload_hash: &UserPayloadHash) -> String {
    to_hex(payload_hash)
}

pub fn to_api_ledger_hash(ledger_hash: &LedgerPayloadHash) -> String {
    to_hex(ledger_hash)
}

pub fn to_api_accumulator_hash(accumulator_hash: &AccumulatorHash) -> String {
    to_hex(accumulator_hash)
}

pub fn extract_intent_hash(intent_hash_str: String) -> Result<IntentHash, ExtractionError> {
    let intent_hash_bytes = from_hex(intent_hash_str)?;
    Ok(IntentHash::from_raw_bytes(
        intent_hash_bytes
            .try_into()
            .map_err(|_| ExtractionError::InvalidHash)?,
    ))
}

#[allow(dead_code)]
pub fn extract_payload_hash(intent_hash_str: String) -> Result<UserPayloadHash, ExtractionError> {
    let intent_hash_bytes = from_hex(intent_hash_str)?;
    Ok(UserPayloadHash::from_raw_bytes(
        intent_hash_bytes
            .try_into()
            .map_err(|_| ExtractionError::InvalidHash)?,
    ))
}
