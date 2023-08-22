use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_type_info_substate(
    context: &MappingContext,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(system_field_substate!(
        substate,
        TypeInfoModuleFieldTypeInfo,
        value => {
            let details = match value {
                TypeInfoSubstate::Object(ObjectInfo {
                    module_versions,
                    blueprint_info,
                    global,
                }) => models::TypeInfoDetails::ObjectTypeInfoDetails {
                    module_versions: module_versions.iter()
                        .map(|(object_module_id, version)| -> Result<_, MappingError> {
                            Ok(models::ModuleVersion {
                                module: to_api_object_module_id(object_module_id),
                                version: to_api_blueprint_version(context, version)?,
                            })
                        })
                        .collect::<Result<_, _>>()?,
                    blueprint_info: Box::new(to_api_blueprint_info(context, blueprint_info)?),
                    global: *global,
                },
                TypeInfoSubstate::KeyValueStore(key_value_store_info) => {
                    models::TypeInfoDetails::KeyValueStoreTypeInfoDetails {
                        key_value_store_info: Box::new(to_api_key_value_store_info(
                            context,
                            key_value_store_info,
                        )?),
                    }
                }
                TypeInfoSubstate::GlobalAddressReservation(_) => {
                    return Err(MappingError::UnexpectedPersistedData {
                        message: "GlobalAddressReservation was persisted".to_string(),
                    })
                }
                TypeInfoSubstate::GlobalAddressPhantom(_) => {
                    return Err(MappingError::UnexpectedPersistedData {
                        message: "GlobalAddressPhantom was persisted".to_string(),
                    })
                }
            }
        },
        Value {
            details: Some(details),
        }
    ))
}

pub fn to_api_blueprint_info(
    context: &MappingContext,
    blueprint_info: &BlueprintInfo,
) -> Result<models::BlueprintInfo, MappingError> {
    let BlueprintInfo {
        blueprint_id:
            BlueprintId {
                package_address,
                blueprint_name,
            },
        outer_obj_info,
        features,
        generic_substitutions,
    } = blueprint_info;

    Ok(models::BlueprintInfo {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.to_string(),
        outer_object: match outer_obj_info {
            OuterObjectInfo::Some { outer_object } => {
                Some(to_api_global_address(context, outer_object)?)
            }
            OuterObjectInfo::None => None,
        },
        generic_substitutions: generic_substitutions
            .iter()
            .map(|substitution| to_api_generic_substitution(context, substitution))
            .collect::<Result<Vec<_>, _>>()?,
        features: features.iter().cloned().collect(),
    })
}

pub fn to_api_generic_substitution(
    context: &MappingContext,
    substitution: &GenericSubstitution,
) -> Result<models::TypeIdentifier, MappingError> {
    match substitution {
        GenericSubstitution::Local(type_identifier) => {
            to_api_type_identifier(context, type_identifier)
        }
    }
}

pub fn to_api_key_value_store_info(
    context: &MappingContext,
    key_value_store_info: &KeyValueStoreInfo,
) -> Result<models::KeyValueStoreInfo, MappingError> {
    let KeyValueStoreInfo {
        generic_substitutions,
    } = key_value_store_info;
    Ok(models::KeyValueStoreInfo {
        key_generic_substitution: Box::new(to_api_generic_substitution(
            context,
            &generic_substitutions.key_generic_substitutions,
        )?),
        value_generic_substitution: Box::new(to_api_generic_substitution(
            context,
            &generic_substitutions.value_generic_substitutions,
        )?),
        allow_ownership: generic_substitutions.allow_ownership,
    })
}
