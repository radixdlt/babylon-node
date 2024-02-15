use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_generic_scrypto_component_state_substate(
    context: &MappingContext,
    substate: &FieldSubstate<ScryptoValue>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        GenericScryptoComponentFieldState,
        value,
        Value {
            data_struct: Box::new(to_api_data_struct_from_scrypto_value(context, value)?),
        }
    ))
}

pub fn to_api_generic_key_value_store_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<ScryptoOwnedRawValue>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::GenericKeyValueStoreKey(raw_key))
    );
    let key_data_option = to_api_sbor_data_from_bytes(context, raw_key).ok();
    Ok(key_value_store_optional_substate!(
        substate,
        GenericKeyValueStoreEntry,
        models::GenericKey {
            key_data: key_data_option.map(Box::new),
        },
        value => {
            data: Box::new(to_api_data_struct_from_scrypto_raw_value(context, value)?),
        },
    ))
}
