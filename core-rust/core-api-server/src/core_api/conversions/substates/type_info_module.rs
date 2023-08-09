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
        instance_schema,
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
        instance_schema: instance_schema
            .as_ref()
            .map(|instance_schema| Ok(Box::new(to_api_instance_schema(context, instance_schema)?)))
            .transpose()?,
        features: features.iter().cloned().collect(),
    })
}

pub fn to_api_instance_schema(
    context: &MappingContext,
    instance_schema: &InstanceSchema,
) -> Result<models::InstanceSchema, MappingError> {
    if instance_schema.instance_type_lookup.is_empty() {
        return Err(MappingError::ExpectedDataInvariantBroken {
            message: "Expected instance schema to have at least 1 instance type".to_string(),
        });
    }
    let schema_hash = &instance_schema.instance_type_lookup[0].0;
    for instance_type in &instance_schema.instance_type_lookup {
        if &instance_type.0 != schema_hash {
            return Err(MappingError::ExpectedDataInvariantBroken {
                message: "Expected all instance type identifiers to point at the same instance schema, with the same hash".to_string(),
            });
        }
    }
    Ok(models::InstanceSchema {
        schema: Box::new(to_api_scrypto_schema(context, &instance_schema.schema)?),
        schema_hash: to_api_hash(schema_hash),
        instance_type_lookup: instance_schema
            .instance_type_lookup
            .iter()
            .map(|type_identifier| to_api_local_type_index(context, &type_identifier.1))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_key_value_store_info(
    context: &MappingContext,
    key_value_store_info: &KeyValueStoreInfo,
) -> Result<models::KeyValueStoreInfo, MappingError> {
    let KeyValueStoreInfo { schema } = key_value_store_info;
    Ok(models::KeyValueStoreInfo {
        kv_store_schema: Box::new(to_api_key_value_store_schema(context, schema)?),
    })
}

pub fn to_api_key_value_store_schema(
    context: &MappingContext,
    key_value_store_schema: &KeyValueStoreSchema,
) -> Result<models::KeyValueStoreSchema, MappingError> {
    let KeyValueStoreSchema {
        key,
        value,
        can_own,
        schema,
    } = key_value_store_schema;
    let schema_hash = &key.0;
    // Really not sure why the schema hash is split onto each of the key type and value types
    if &value.0 != schema_hash {
        return Err(MappingError::ExpectedDataInvariantBroken {
            message: "Expected both key and value to point at the same key value store schema, with the same hash".to_string(),
        });
    }
    Ok(models::KeyValueStoreSchema {
        schema: Box::new(to_api_scrypto_schema(context, schema)?),
        schema_hash: to_api_hash(schema_hash),
        key_type: Box::new(to_api_local_type_index(context, &key.1)?),
        value_type: Box::new(to_api_local_type_index(context, &value.1)?),
        can_own: *can_own,
    })
}
