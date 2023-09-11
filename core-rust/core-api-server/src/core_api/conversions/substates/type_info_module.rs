use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_type_info_substate(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(system_field_substate!(
        substate,
        TypeInfoModuleFieldTypeInfo,
        value => {
            let details = match value {
                TypeInfoSubstate::Object(ObjectInfo {blueprint_info, object_type}) => {
                    let (global, module_versions) = match object_type {
                        ObjectType::Global { modules } => (true, Some(modules)),
                        ObjectType::Owned => (false, None)
                    };
                    models::TypeInfoDetails::ObjectTypeInfoDetails {
                        module_versions: module_versions
                            .iter()
                            .flat_map(|modules| modules.iter())
                            .map(|(module_id, version)| -> Result<_, MappingError> {
                                Ok(models::ModuleVersion {
                                    module: to_api_attached_module_id(module_id),
                                    version: to_api_blueprint_version(context, version)?,
                                })
                            })
                            .collect::<Result<_, _>>()?,
                        blueprint_info: Box::new(to_api_blueprint_info(
                            context,
                            state_mapping_lookups,
                            blueprint_info,
                        )?),
                        global,
                    }
                },
                TypeInfoSubstate::KeyValueStore(key_value_store_info) => {
                    models::TypeInfoDetails::KeyValueStoreTypeInfoDetails {
                        key_value_store_info: Box::new(to_api_key_value_store_info(
                            context,
                            state_mapping_lookups,
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
    state_mapping_lookups: &StateMappingLookups,
    blueprint_info: &BlueprintInfo,
) -> Result<models::BlueprintInfo, MappingError> {
    let BlueprintInfo {
        blueprint_id:
            BlueprintId {
                package_address,
                blueprint_name,
            },
        blueprint_version,
        outer_obj_info,
        features,
        generic_substitutions,
    } = blueprint_info;

    Ok(models::BlueprintInfo {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.to_string(),
        blueprint_version: to_api_blueprint_version(context, blueprint_version)?,
        outer_object: match outer_obj_info {
            OuterObjectInfo::Some { outer_object } => {
                Some(to_api_global_address(context, outer_object)?)
            }
            OuterObjectInfo::None => None,
        },
        generic_substitutions: generic_substitutions
            .iter()
            .map(|substitution| {
                to_api_generic_substitution(context, state_mapping_lookups, substitution)
            })
            .collect::<Result<Vec<_>, _>>()?,
        features: features.iter().cloned().collect(),
    })
}

pub fn to_api_generic_substitution(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    substitution: &GenericSubstitution,
) -> Result<models::GenericSubstitution, MappingError> {
    Ok(match substitution {
        GenericSubstitution::Local(scoped_type_id) => {
            models::GenericSubstitution::LocalGenericSubstition {
                scoped_type_id: Box::new(to_api_scoped_type_id(context, scoped_type_id)?),
            }
        }
        GenericSubstitution::Remote(blueprint_type_identifier) => {
            let resolved =
                state_mapping_lookups.resolve_generic_remote(blueprint_type_identifier)?;
            models::GenericSubstitution::RemoteGenericSubstition {
                blueprint_type_identifier: Box::new(to_api_blueprint_type_identifier(
                    context,
                    blueprint_type_identifier,
                )?),
                resolved_scoped_type_id: resolved
                    .map(|scoped_type_id| -> Result<_, MappingError> {
                        Ok(Box::new(to_api_scoped_type_id(context, &scoped_type_id)?))
                    })
                    .transpose()?,
            }
        }
    })
}

pub fn to_api_blueprint_type_identifier(
    context: &MappingContext,
    blueprint_type_identifier: &BlueprintTypeIdentifier,
) -> Result<models::BlueprintTypeIdentifier, MappingError> {
    let BlueprintTypeIdentifier {
        package_address,
        blueprint_name,
        type_name,
    } = blueprint_type_identifier;
    Ok(models::BlueprintTypeIdentifier {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.clone(),
        type_name: type_name.clone(),
    })
}

pub fn to_api_key_value_store_info(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    key_value_store_info: &KeyValueStoreInfo,
) -> Result<models::KeyValueStoreInfo, MappingError> {
    let KeyValueStoreInfo {
        generic_substitutions:
            KeyValueStoreGenericSubstitutions {
                key_generic_substitution,
                value_generic_substitution,
                allow_ownership,
            },
    } = key_value_store_info;
    Ok(models::KeyValueStoreInfo {
        key_generic_substitution: Some(to_api_generic_substitution(
            context,
            state_mapping_lookups,
            key_generic_substitution,
        )?),
        value_generic_substitution: Some(to_api_generic_substitution(
            context,
            state_mapping_lookups,
            value_generic_substitution,
        )?),
        allow_ownership: *allow_ownership,
    })
}
