use crate::engine_prelude::*;
use crate::prelude::*;

#[derive(Debug, Clone, EnumIter, Display, FromRepr)]
#[repr(i64)]
pub(crate) enum MeshApiOperationTypes {
    Withdraw,
    Deposit,
    // LockFee,
    // Mint,
    // Burn,
}

#[derive(Debug, Clone, EnumIter, Display)]
pub(crate) enum MeshApiOperationStatus {
    #[strum(serialize = "Success")]
    Success,
    #[strum(serialize = "Failure")]
    Failure,
}

impl From<DetailedTransactionOutcome> for MeshApiOperationStatus {
    fn from(value: DetailedTransactionOutcome) -> Self {
        match value {
            DetailedTransactionOutcome::Success(..) => Self::Success,
            DetailedTransactionOutcome::Failure(..) => Self::Failure,
        }
    }
}

impl From<MeshApiOperationStatus> for bool {
    fn from(value: MeshApiOperationStatus) -> Self {
        match value {
            MeshApiOperationStatus::Success => true,
            MeshApiOperationStatus::Failure => false,
        }
    }
}
