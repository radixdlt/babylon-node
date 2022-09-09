use std::num::TryFromIntError;

use sbor::DecodeError;

#[derive(Debug, Clone)]
pub enum MappingError {
    Integer {
        message: String,
        error: TryFromIntError,
    },
    VirtualRootSubstatePersisted {
        message: String,
    },
    VirtualSubstateDownedWithInvalidParent {
        message: String,
    },
    TransientSubstatePersisted {
        message: String,
    },
    InvalidRootEntity {
        message: String,
    },
    InvalidDataStruct {
        decode_error: DecodeError,
        bytes: Vec<u8>,
    },
    InvalidComponentStateEntities {
        message: String,
    },
    MismatchedSubstateId {
        message: String,
    },
}
