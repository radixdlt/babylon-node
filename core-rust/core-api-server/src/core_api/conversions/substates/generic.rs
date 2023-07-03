use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_generic_scrypto_component_state_substate(
    context: &MappingContext,
    substate: &GenericScryptoSborPayload,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        GenericScryptoComponentFieldState,
        {
            data_struct: Box::new(to_api_data_struct_from_bytes(context, substate.data.as_ref())?),
        }
    ))
}

pub fn to_api_generic_scrypto_component_state_substate_from_scrypto_value(
    context: &MappingContext,
    scrypto_value: &ScryptoValue,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        GenericScryptoComponentFieldState,
        {
            data_struct: Box::new(to_api_data_struct_from_scrypto_value(context, scrypto_value)?),
        }
    ))
}

pub fn to_api_generic_key_value_store_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<ScryptoRawValue<'_>>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::GenericKeyValueStoreKey(raw_key)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "GenericKeyValueStoreKey".to_string() });
    };
    let key_data_option = to_api_sbor_data_from_bytes(context, raw_key).ok();
    Ok(key_value_store_optional_substate!(
        substate,
        GenericKeyValueStoreEntry,
        models::GenericKey {
            key_data: key_data_option.map(Box::new),
        },
        value -> {
            data: Box::new(to_api_data_struct_from_scrypto_raw_value(context, value)?),
        },
    ))
}
