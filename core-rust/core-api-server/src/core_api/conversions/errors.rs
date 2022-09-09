use sbor::DecodeError;

#[derive(Debug, Clone)]
pub enum MappingError {
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
