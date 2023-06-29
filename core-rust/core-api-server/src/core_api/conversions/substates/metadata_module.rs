use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_metadata_value_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &MetadataEntrySubstate,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(entry_name)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "MetadataEntryKey".to_string() });
    };
    Ok(key_value_store_substate!(
        substate,
        MetadataModuleEntry,
        models::MetadataKey {
            name: entry_name.to_string(),
        },
        {
            data_struct: substate
                .get_optional_value()
                .map(|metadata_value| -> Result<_, MappingError> {
                    Ok(Box::new(to_api_data_struct_from_bytes(context, &scrypto_encode(metadata_value).unwrap())?))
                })
                .transpose()?,
        }
    ))
}
