use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_metadata_value_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &MetadataEntryEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(
            entry_name
        ))
    );
    Ok(key_value_store_optional_substate_versioned!(
        substate,
        MetadataModuleEntry,
        models::MetadataKey {
            name: entry_name.to_string(),
        },
        value => {
            data_struct: Box::new(to_api_data_struct_from_bytes(context, &scrypto_encode(value).unwrap())?),
        }
    ))
}
