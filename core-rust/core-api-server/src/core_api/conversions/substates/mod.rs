mod substate;
mod type_info_module;
mod access_rules_module;
mod royalty_module;
mod metadata_module;
mod package;
mod resource;
mod consensus_manager;
mod account;
mod access_controller;
mod generic;
mod pools;
mod transaction_tracker;

pub use substate::*;
pub use type_info_module::*;
pub use access_rules_module::*;
pub use royalty_module::*;
pub use metadata_module::*;
pub use package::*;
pub use resource::*;
pub use consensus_manager::*;
pub use account::*;
pub use access_controller::*;
pub use generic::*;
pub use pools::*;
pub use transaction_tracker::*;

//====================================
// General substate mapping utilities
//====================================

use radix_engine_queries::typed_substate_layout::KeyValueEntrySubstate;
use super::MappingError;

macro_rules! field_substate {
    (
        $substate:ident,
        $substate_type:ident,
        {
            $($value_key:ident$(: $value_value:expr)?),*$(,)?
        }$(,)?
    ) => {
        paste::paste!{
            models::Substate::[<$substate_type Substate>] {
                is_locked: false,
                $($value_key$(: $value_value)?,)*
            }
        }
    };
}
pub(crate) use field_substate;

macro_rules! key_value_store_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        {
            $($value_key:ident$(: $value_value:expr)?),*$(,)?
        }$(,)?
    ) => {
        paste::paste!{
            models::Substate::[<$substate_type Substate>] {
                is_locked: !$substate.is_mutable(),
                key: Box::new($key),
                $($value_key$(: $value_value)?,)*
            }
        }
    };
}
pub(crate) use key_value_store_substate;

macro_rules! index_substate {
    (
        $substate:ident,
        $substate_type:ident,
        $key:expr,
        {
            $($value_key:ident$(: $value_value:expr)?),*$(,)?
        }$(,)?
    ) => {
        paste::paste!{
            models::Substate::[<$substate_type Substate>] {
                is_locked: false,
                key: Box::new($key),
                $($value_key$(: $value_value)?,)*
            }
        }
    };
}
pub(crate) use index_substate;

trait WrapperMethods {
    type Content;
    fn get_definitely_present_value(&self) -> Result<&Self::Content, MappingError>;
    fn get_optional_value(&self) -> Option<&Self::Content>;
}

impl<Content> WrapperMethods for KeyValueEntrySubstate<Content> {
    type Content = Content;

    fn get_definitely_present_value(&self) -> Result<&Self::Content, MappingError> {
        match self.value.as_ref() {
            Some(value) => Ok(value),
            None => Err(MappingError::KeyValueStoreEntryUnexpectedlyAbsent),
        }
    }

    fn get_optional_value(&self) -> Option<&Self::Content> {
        self.value.as_ref()
    }
}