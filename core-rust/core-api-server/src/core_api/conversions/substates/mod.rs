mod access_controller;
mod access_rules_module;
mod account;
mod account_locker;
mod boot_loader_module;
mod consensus_manager;
mod generic;
mod metadata_module;
mod package;
mod pools;
mod resource;
mod royalty_module;
mod substate;
mod transaction_tracker;
mod type_info_module;

pub use access_controller::*;
pub use access_rules_module::*;
pub use account::*;
pub use account_locker::*;
pub use boot_loader_module::*;
pub use consensus_manager::*;
pub use generic::*;
pub use metadata_module::*;
pub use package::*;
pub use pools::*;
pub use resource::*;
pub use royalty_module::*;
pub use substate::*;
pub use transaction_tracker::*;
pub use type_info_module::*;

//====================================
// General substate mapping utilities
//====================================

use super::MappingError;
use crate::engine_prelude::*;

macro_rules! assert_key_type {
    (
        $typed_key:ident,
        $value_unpacking:pat $(=> {
            $($mapping:stmt)+
        })?
    ) => {
        paste::paste! {
            let $value_unpacking = $typed_key else {
                return Err(MappingError::MismatchedSubstateKeyType {
                    expected_match: stringify!($value_unpacking).to_owned(),
                    actual: format!("{:?}", $typed_key)
                });
            };
        }
    };
}
pub(crate) use assert_key_type;

macro_rules! field_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $value_unpacking:pat $(=> {
            $($mapping:stmt)+
        })?,
        Value $fields:tt$(,)?
    ) => {
        paste::paste! {
            // The trailing semicolon after the $($($mapping)+)?; output is occasionally needed,
            // eg if the mapping ends with a match statement. But otherwise it's reported as a
            // redundant - so add this allow statement to allow us just to include it regardless.
            #[allow(redundant_semicolons)]
            models::Substate::[<$substate_type Substate>] {
                is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                value: {
                    // NB: We should use compiler to unpack to ensure we map all fields
                    let $value_unpacking = $substate.payload();
                    $($($mapping)+)?;
                    Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use field_substate;

macro_rules! field_substate_versioned {
    (
        $substate:ident,
        $substate_type:ident,
        $value_unpacking:pat $(=> {
            $($mapping:stmt)+
        })?,
        Value $fields:tt$(,)?
    ) => {
        paste::paste! {
            // The trailing semicolon after the $($($mapping)+)?; output is occasionally needed,
            // eg if the mapping ends with a match statement. But otherwise it's reported as a
            // redundant - so add this allow statement to allow us just to include it regardless.
            #[allow(redundant_semicolons)]
            models::Substate::[<$substate_type Substate>] {
                is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                value: {
                    // NB: We should use compiler to unpack to ensure we map all fields
                    let $value_unpacking = &scrypto_clone($substate.payload())
                        .fully_update_and_into_latest_version();
                    $($($mapping)+)?;
                    Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use field_substate_versioned;

macro_rules! system_field_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $value_unpacking:pat $(=> {
            $($mapping:stmt)+
        })?,
        Value $fields:tt$(,)?
    ) => {
        paste::paste! {
            // The trailing semicolon after the $($($mapping)+)?; output is occasionally needed,
            // eg if the mapping ends with a match statement. But otherwise it's reported as a
            // redundant - so add this allow statement to allow us just to include it regardless.
            #[allow(redundant_semicolons)]
            models::Substate::[<$substate_type Substate>] {
                is_locked: false,
                value: {
                    // NB: We should use compiler to unpack to ensure we map all fields
                    let $value_unpacking = &$substate;
                    $($($mapping)+)?;
                    Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use system_field_substate;

macro_rules! key_value_store_optional_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $value_unpacking:pat => $fields:tt$(,)?
    ) => {
        paste::paste! {
            models::Substate::[<$substate_type Substate>] {
                is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                key: Box::new($key),
                value: $substate
                    .get_optional_value()
                    .map(|$value_unpacking| -> Result<_, MappingError> {
                        Ok(Box::new(models::[<$substate_type Value>] $fields))
                    })
                    .transpose()?,
            }
        }
    };
}
pub(crate) use key_value_store_optional_substate;

macro_rules! key_value_store_optional_substate_versioned {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $value_unpacking:pat => $fields:tt$(,)?
    ) => {
        paste::paste! {
            models::Substate::[<$substate_type Substate>] {
                is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                key: Box::new($key),
                value: $substate
                    .get_optional_value()
                    .map(|opt| -> Result<_, MappingError> {
                        #[allow(clippy::let_unit_value)]
                        let $value_unpacking = &scrypto_clone(opt)
                            .fully_update_and_into_latest_version();
                        Ok(Box::new(models::[<$substate_type Value>] $fields))
                    })
                    .transpose()?,
            }
        }
    };
}
pub(crate) use key_value_store_optional_substate_versioned;

macro_rules! key_value_store_mandatory_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $value_unpacking:pat => $fields:tt$(,)?
    ) => {
        paste::paste! {
            {
                let $value_unpacking = $substate.get_definitely_present_value()?;
                models::Substate::[<$substate_type Substate>] {
                    is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                    key: Box::new($key),
                    value: Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use key_value_store_mandatory_substate;

macro_rules! key_value_store_mandatory_substate_versioned {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $value_unpacking:pat => $fields:tt$(,)?
    ) => {
        paste::paste! {
            {
                let $value_unpacking = &scrypto_clone($substate.get_definitely_present_value()?)
                    .fully_update_and_into_latest_version();
                models::Substate::[<$substate_type Substate>] {
                    is_locked: matches!($substate.lock_status(), LockStatus::Locked),
                    key: Box::new($key),
                    value: Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use key_value_store_mandatory_substate_versioned;

macro_rules! index_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $fields:tt$(,)?
    ) => {
        paste::paste! {
            models::Substate::[<$substate_type Substate>] {
                is_locked: false,
                key: Box::new($key),
                value: Box::new(models::[<$substate_type Value>] $fields)
            }
        }
    };
}
pub(crate) use index_substate;

macro_rules! index_substate_versioned {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        $value_unpacking:pat => $fields:tt$(,)?
    ) => {
        paste::paste! {
            {
                let $value_unpacking = &scrypto_clone($substate.value())
                    .fully_update_and_into_latest_version();
                models::Substate::[<$substate_type Substate>] {
                    is_locked: false,
                    key: Box::new($key),
                    value: Box::new(models::[<$substate_type Value>] $fields)
                }
            }
        }
    };
}
pub(crate) use index_substate_versioned;

trait WrapperMethods<C> {
    fn get_definitely_present_value(&self) -> Result<&C, MappingError> {
        self.get_optional_value()
            .ok_or(MappingError::KeyValueStoreEntryUnexpectedlyAbsent)
    }
    fn get_optional_value(&self) -> Option<&C>;
}

impl<C> WrapperMethods<C> for KeyValueEntrySubstate<C> {
    fn get_optional_value(&self) -> Option<&C> {
        match self {
            KeyValueEntrySubstate::V1(v1) => v1.value.as_ref(),
        }
    }
}

/// "Clones" an SBOR-encodable value.
/// Note: this hack is required since many of the Substate value structs do not support [`Clone`],
/// and the convenient "into latest" method (which we need in order to output the value according to
/// the "current" Core API schema) needs an owned instance. A true fix should involve either cloning
/// or some other way of "get latest from reference" on the Engine's side.
fn scrypto_clone<T: ScryptoSbor>(value: &T) -> T {
    let bytes = scrypto_encode(value).expect("cannot encode");
    scrypto_decode(&bytes).expect("cannot decode")
}
