use super::*;
use super::super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_type_info_substate(
    context: &MappingContext,
    substate: &TypeInfoSubstate,
) -> Result<models::Substate, MappingError> {
    // Use compiler to unpack to ensure we map all fields
    let details = match substate {
        TypeInfoSubstate::Object(ObjectInfo {
            global,
            blueprint_id:
                BlueprintId {
                    package_address,
                    blueprint_name,
                },
            blueprint_info,
            version,
            features,
            instance_schema,
        }) => models::TypeInfoDetails::ObjectTypeInfoDetails {
            package_address: to_api_package_address(context, package_address)?,
            blueprint_name: blueprint_name.to_string(),
            blueprint_version: to_api_blueprint_version(context, version)?,
            global: *global,
            outer_object: match blueprint_info {
                ObjectBlueprintInfo::Inner { outer_object } => {
                    Some(to_api_global_address(context, outer_object)?)
                }
                ObjectBlueprintInfo::Outer => None,
            },
            instance_schema: instance_schema
                .as_ref()
                .map(|instance_schema| {
                    Ok(Box::new(to_api_instance_schema(context, instance_schema)?))
                })
                .transpose()?,
            features: features.iter().cloned().collect(),
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
    };

    Ok(field_substate!(
        substate,
        TypeInfoModuleFieldTypeInfo,
        {
            details: Box::new(details),
        }
    ))
}

pub fn to_api_instance_schema(
    context: &MappingContext,
    instance_schema: &InstanceSchema,
) -> Result<models::InstanceSchema, MappingError> {
    Ok(models::InstanceSchema {
        schema: Box::new(to_api_scrypto_schema(context, &instance_schema.schema)?),
        provided_types: instance_schema
            .type_index
            .iter()
            .map(|local_type_index| to_api_local_type_index(context, local_type_index))
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
    Ok(models::KeyValueStoreSchema {
        schema: Box::new(to_api_scrypto_schema(context, schema)?),
        key_type: Box::new(to_api_local_type_index(context, key)?),
        value_type: Box::new(to_api_local_type_index(context, value)?),
        can_own: *can_own,
    })
}