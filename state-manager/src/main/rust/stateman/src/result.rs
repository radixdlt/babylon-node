use crate::types::JavaStructure;
use sbor::{Decode, Encode, TypeId};

// System Errors.
pub const ERRCODE_JNI: i16 = 0;
pub const ERRCODE_SBOR: i16 = 1;
// Mempool Errors.
pub const ERRCODE_MEMPOOL_FULL: i16 = 0x10;
pub const ERRCODE_MEMPOOL_DUPLICATE: i16 = 0x11;

#[derive(TypeId, Encode, Decode)]
pub struct StateManagerError {
    error_code: i16,
    error_msg: String,
}

impl StateManagerError {
    pub fn create(error_code: i16, error_msg: String) -> StateManagerError {
        StateManagerError {
            error_code,
            error_msg,
        }
    }
}

pub trait ToStateManagerError {
    fn to_state_manager_error(&self) -> StateManagerError;
}

pub type StateManagerResult<T> = Result<T, StateManagerError>;

impl<T: Encode + Decode> JavaStructure for StateManagerResult<T> {}
