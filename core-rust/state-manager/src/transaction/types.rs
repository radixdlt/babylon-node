use sbor::{Decode, Encode, TypeId};

use crate::transaction::validator::ValidatorTransaction;
use transaction::model::NotarizedTransaction;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Transaction {
    User(NotarizedTransaction),
    Validator(ValidatorTransaction),
}
