use std::num::TryFromIntError;

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
}
