use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_package_royalty_accumulator_substate(
    context: &MappingContext,
    substate: &PackageRoyaltyAccumulatorFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        PackageFieldRoyaltyAccumulator,
        PackageRoyaltyAccumulator { royalty_vault },
        Value {
            vault_entity: Box::new(to_api_entity_reference(
                context,
                royalty_vault.0.as_node_id(),
            )?),
        }
    ))
}

pub fn to_api_package_code_vm_type_entry_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageCodeVmTypeEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeVmTypeKeyValueEntry(PackageCodeVmTypeKeyPayload {
                content: hash
            })
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageCodeVmTypeEntry,
        models::PackageCodeKey {
            code_hash: to_api_code_hash(hash),
        },
        PackageCodeVmType { vm_type } => {
            vm_type: match vm_type {
                VmType::Native => models::VmType::Native,
                VmType::ScryptoV1 => models::VmType::ScryptoV1,
            }
        }
    ))
}

pub fn to_api_package_code_original_code_entry_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageCodeOriginalCodeEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeOriginalCodeKeyValueEntry(
                PackageCodeOriginalCodeKeyPayload { content: hash }
            )
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageCodeOriginalCodeEntry,
        models::PackageCodeKey {
            code_hash: to_api_code_hash(hash),
        },
        PackageCodeOriginalCode { code } => {
            code_hex: to_hex(code),
        }
    ))
}

pub fn to_api_package_code_instrumented_code_entry_substate(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageCodeInstrumentedCodeEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeInstrumentedCodeKeyValueEntry(
                PackageCodeInstrumentedCodeKeyPayload { content: hash }
            )
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageCodeInstrumentedCodeEntry,
        models::PackageCodeKey {
            code_hash: to_api_code_hash(hash),
        },
        PackageCodeInstrumentedCode { instrumented_code } => {
            code_hex: to_hex(instrumented_code),
        }
    ))
}

pub fn to_api_schema_entry_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<VersionedScryptoSchema>,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::Schema(TypedSchemaSubstateKey::SchemaKey(hash))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        SchemaEntry,
        models::SchemaKey {
            schema_hash: to_api_schema_hash(hash),
        },
        value => {
            schema: Box::new(to_api_scrypto_schema(context, value)?),
        }
    ))
}

pub fn to_api_package_blueprint_definition_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageBlueprintVersionDefinitionEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionDefinitionKeyValueEntry(
                PackageBlueprintVersionDefinitionKeyPayload {
                    content: blueprint_version_key
                }
            )
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageBlueprintDefinitionEntry,
        to_api_blueprint_version_key(context, blueprint_version_key)?,
        value => {
            definition: Box::new(to_api_blueprint_definition(context, value)?),
        }
    ))
}

pub fn to_api_package_blueprint_dependencies_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageBlueprintVersionDependenciesEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionDependenciesKeyValueEntry(
                PackageBlueprintVersionDependenciesKeyPayload {
                    content: blueprint_version_key
                }
            )
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageBlueprintDependenciesEntry,
        to_api_blueprint_version_key(context, blueprint_version_key)?,
        value => {
            dependencies: Box::new(to_api_blueprint_dependencies(context, value)?),
        }
    ))
}

pub fn to_api_package_blueprint_royalty_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageBlueprintVersionRoyaltyConfigEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionRoyaltyConfigKeyValueEntry(
                PackageBlueprintVersionRoyaltyConfigKeyPayload {
                    content: blueprint_version_key
                }
            )
        ))
    );

    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageBlueprintRoyaltyEntry,
        to_api_blueprint_version_key(context, blueprint_version_key)?,
        value => {
            royalty_config: Box::new(to_api_package_blueprint_royalty_config(value)),
        }
    ))
}

pub fn to_api_package_blueprint_royalty_config(
    royalty_config: &PackageRoyaltyConfig,
) -> models::BlueprintRoyaltyConfig {
    let (is_enabled, royalties) = match royalty_config {
        PackageRoyaltyConfig::Disabled => (false, None),
        PackageRoyaltyConfig::Enabled(rules) => (true, Some(rules)),
    };
    models::BlueprintRoyaltyConfig {
        is_enabled,
        method_rules: royalties.map(|rules| {
            rules
                .iter()
                .map(
                    |(method_name, royalty_amount)| models::BlueprintMethodRoyalty {
                        method_name: method_name.to_owned(),
                        royalty_amount: to_api_royalty_amount(royalty_amount).map(Box::new),
                    },
                )
                .collect()
        }),
    }
}

pub fn to_api_package_auth_template_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &PackageBlueprintVersionAuthConfigEntrySubstate,
) -> Result<models::Substate, MappingError> {
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionAuthConfigKeyValueEntry(
                PackageBlueprintVersionAuthConfigKeyPayload {
                    content: blueprint_version_key
                }
            )
        ))
    );
    Ok(key_value_store_mandatory_substate_versioned!(
        substate,
        PackageBlueprintAuthTemplateEntry,
        to_api_blueprint_version_key(context, blueprint_version_key)?,
        value => {
            auth_config: Box::new(to_api_auth_config(context, value)?),
        }
    ))
}

pub fn to_api_auth_config(
    context: &MappingContext,
    config: &AuthConfig,
) -> Result<models::AuthConfig, MappingError> {
    let AuthConfig {
        function_auth,
        method_auth,
    } = config;
    let (function_auth_type, function_access_rules) = match function_auth {
        FunctionAuth::AllowAll => (models::FunctionAuthType::AllowAll, None),
        FunctionAuth::AccessRules(access_rules) => {
            let access_rules = access_rules
                .iter()
                .map(|(identifier, access_rule)| {
                    Ok((
                        identifier.to_string(),
                        to_api_access_rule(context, access_rule)?,
                    ))
                })
                .collect::<Result<_, _>>()?;
            (
                models::FunctionAuthType::FunctionAccessRules,
                Some(access_rules),
            )
        }
        FunctionAuth::RootOnly => (models::FunctionAuthType::RootOnly, None),
    };
    let (method_auth_type, method_roles) = match method_auth {
        MethodAuthTemplate::AllowAll => (models::MethodAuthType::AllowAll, None),
        MethodAuthTemplate::StaticRoleDefinition(definition) => {
            let static_roles = to_api_static_role_definition(context, definition)?;
            (
                models::MethodAuthType::StaticRoleDefinition,
                Some(Box::new(static_roles)),
            )
        }
    };
    Ok(models::AuthConfig {
        function_auth_type,
        function_access_rules,
        method_auth_type,
        method_roles,
    })
}

fn to_api_blueprint_version_key(
    context: &MappingContext,
    BlueprintVersionKey { blueprint, version }: &BlueprintVersionKey,
) -> Result<models::BlueprintVersionKey, MappingError> {
    Ok(models::BlueprintVersionKey {
        blueprint_name: blueprint.to_string(),
        blueprint_version: to_api_blueprint_version(context, version)?,
    })
}

pub fn to_api_static_role_definition(
    context: &MappingContext,
    definition: &StaticRoleDefinition,
) -> Result<models::StaticRoleDefinitionAuthTemplate, MappingError> {
    let StaticRoleDefinition { roles, methods } = definition;

    let (role_specification, roles) = match roles {
        RoleSpecification::Normal(roles) => {
            let roles = roles
                .iter()
                .map(|(identifier, updater_roles)| {
                    (
                        identifier.key.to_string(),
                        models::RoleDetails {
                            updater_roles: updater_roles
                                .list
                                .iter()
                                .map(|role| role.key.to_owned())
                                .collect(),
                        },
                    )
                })
                .collect();
            (models::RoleSpecification::Normal, Some(roles))
        }
        RoleSpecification::UseOuter => (models::RoleSpecification::UseOuter, None),
    };
    let method_accessibility_map = methods
        .iter()
        .map(|(method_key, method_accessibility)| {
            Ok((
                method_key.ident.to_string(),
                to_api_method_accessibility(context, method_accessibility)?,
            ))
        })
        .collect::<Result<_, _>>()?;
    Ok(models::StaticRoleDefinitionAuthTemplate {
        role_specification,
        roles,
        method_accessibility_map,
    })
}

pub fn to_api_method_accessibility(
    _context: &MappingContext,
    permission: &MethodAccessibility,
) -> Result<models::MethodAccessibility, MappingError> {
    Ok(match permission {
        MethodAccessibility::Public => models::MethodAccessibility::PublicMethodAccessibility {},
        MethodAccessibility::OuterObjectOnly => {
            models::MethodAccessibility::OuterObjectOnlyMethodAccessibility {}
        }
        MethodAccessibility::RoleProtected(role_list) => {
            models::MethodAccessibility::RoleProtectedMethodAccessibility {
                allowed_roles: role_list
                    .list
                    .iter()
                    .map(|key| key.key.to_string())
                    .collect::<Vec<_>>(),
            }
        }
        MethodAccessibility::OwnPackageOnly => {
            models::MethodAccessibility::OwnPackageOnlyMethodAccessibility {}
        }
    })
}

pub fn to_api_blueprint_definition(
    context: &MappingContext,
    blueprint_definition: &BlueprintDefinition,
) -> Result<models::BlueprintDefinition, MappingError> {
    let BlueprintDefinition {
        interface,
        function_exports,
        hook_exports,
    } = blueprint_definition;
    Ok(models::BlueprintDefinition {
        interface: Box::new(to_api_blueprint_interface(context, interface)?),
        function_exports: function_exports
            .iter()
            .map(|(function_name, package_export)| {
                Ok((
                    function_name.to_string(),
                    to_api_package_export(context, package_export)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        hook_exports: hook_exports
            .iter()
            .map(|(blueprint_hook, package_export)| {
                Ok(models::HookExport {
                    object_hook: match blueprint_hook {
                        BlueprintHook::OnVirtualize => models::ObjectHook::OnVirtualize,
                        BlueprintHook::OnMove => models::ObjectHook::OnMove,
                        BlueprintHook::OnDrop => models::ObjectHook::OnDrop,
                    },
                    export: Box::new(to_api_package_export(context, package_export)?),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_blueprint_dependencies(
    context: &MappingContext,
    dependencies: &BlueprintDependencies,
) -> Result<models::BlueprintDependencies, MappingError> {
    let BlueprintDependencies { dependencies } = dependencies;
    Ok(models::BlueprintDependencies {
        dependencies: dependencies
            .iter()
            .map(|address| to_api_global_address(context, address))
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_package_export(
    _context: &MappingContext,
    package_export: &PackageExport,
) -> Result<models::PackageExport, MappingError> {
    let PackageExport {
        code_hash,
        export_name,
    } = package_export;
    Ok(models::PackageExport {
        code_hash: to_api_code_hash(code_hash),
        export_name: export_name.to_string(),
    })
}

pub fn to_api_blueprint_interface(
    context: &MappingContext,
    blueprint_interface: &BlueprintInterface,
) -> Result<models::BlueprintInterface, MappingError> {
    let BlueprintInterface {
        blueprint_type,
        is_transient,
        generics,
        state,
        functions,
        feature_set,
        events,
        types,
    } = blueprint_interface;
    Ok(models::BlueprintInterface {
        outer_blueprint: match blueprint_type {
            BlueprintType::Outer => None,
            BlueprintType::Inner { outer_blueprint } => Some(outer_blueprint.to_string()),
        },
        is_transient: *is_transient,
        generic_type_parameters: generics
            .iter()
            .map(|generic| match generic {
                GenericBound::Any => models::GenericTypeParameter {
                    constraints: models::GenericTypeParameterConstraints::Any,
                },
            })
            .collect::<Vec<_>>(),
        features: feature_set.iter().cloned().collect(),
        state: Box::new(to_api_indexed_state_schema(context, state)?),
        functions: functions
            .iter()
            .map(|(function_name, function_schema)| {
                Ok((
                    function_name.to_string(),
                    to_api_function_schema(context, function_schema)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        events: events
            .iter()
            .map(|(event_name, blueprint_payload_def)| {
                Ok((
                    event_name.to_string(),
                    to_api_blueprint_payload_def(context, blueprint_payload_def)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        types: types
            .iter()
            .map(|(type_name, scoped_type_id)| {
                Ok((
                    type_name.to_string(),
                    to_api_scoped_type_id(context, scoped_type_id)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn to_api_indexed_state_schema(
    context: &MappingContext,
    indexed_state_schema: &IndexedStateSchema,
) -> Result<models::IndexedStateSchema, MappingError> {
    let IndexedStateSchema {
        fields,
        collections,
        num_logical_partitions,
    } = indexed_state_schema;
    Ok(models::IndexedStateSchema {
        fields: fields
            .as_ref()
            .map(|(partition_description, schemas)| {
                to_api_blueprint_schema_fields_partition(context, partition_description, schemas)
            })
            .transpose()?
            .map(Box::new),
        collections: collections
            .iter()
            .map(|(partition_description, schema)| {
                to_api_blueprint_schema_collection_partition(context, partition_description, schema)
            })
            .collect::<Result<_, _>>()?,
        num_partitions: to_api_u8_as_i32(*num_logical_partitions),
    })
}

pub fn to_api_blueprint_payload_def(
    context: &MappingContext,
    blueprint_payload_def: &BlueprintPayloadDef,
) -> Result<models::BlueprintPayloadDef, MappingError> {
    Ok(match blueprint_payload_def {
        BlueprintPayloadDef::Static(scoped_type_id) => {
            models::BlueprintPayloadDef::StaticBlueprintPayloadDef {
                type_id: Box::new(to_api_scoped_type_id(context, scoped_type_id)?),
            }
        }
        BlueprintPayloadDef::Generic(index) => {
            models::BlueprintPayloadDef::GenericBlueprintPayloadDef {
                generic_index: to_api_u8_as_i32(*index),
            }
        }
    })
}

pub fn to_api_scoped_type_id(
    context: &MappingContext,
    scoped_type_id: &ScopedTypeId,
) -> Result<models::ScopedTypeId, MappingError> {
    let ScopedTypeId(schema_hash, local_type_id) = scoped_type_id;
    Ok(models::ScopedTypeId {
        schema_hash: to_api_schema_hash(schema_hash),
        local_type_id: Box::new(to_api_local_type_id(context, local_type_id)?),
    })
}

pub fn to_api_package_type_reference(
    context: &MappingContext,
    reference: &PackageTypeReference,
) -> Result<models::PackageTypeReference, MappingError> {
    let PackageTypeReference { full_type_id } = reference;
    Ok(models::PackageTypeReference {
        full_type_id: Box::new(to_api_fully_scoped_type_id(context, full_type_id)?),
    })
}

pub fn to_api_local_type_id(
    context: &MappingContext,
    local_type_id: &LocalTypeId,
) -> Result<models::LocalTypeId, MappingError> {
    Ok(match local_type_id {
        LocalTypeId::WellKnown(index) => models::LocalTypeId {
            kind: models::local_type_id::Kind::WellKnown,
            id: to_api_well_known_type_id(index)?,
            as_sbor: Box::new(to_api_sbor_data_from_encodable(context, local_type_id)?),
        },
        LocalTypeId::SchemaLocalIndex(index) => models::LocalTypeId {
            kind: models::local_type_id::Kind::SchemaLocal,
            id: to_api_index_as_i64(*index)?,
            as_sbor: Box::new(to_api_sbor_data_from_encodable(context, local_type_id)?),
        },
    })
}

pub fn to_api_function_schema(
    context: &MappingContext,
    function_schema: &FunctionSchema,
) -> Result<models::FunctionSchema, MappingError> {
    let FunctionSchema {
        receiver,
        input,
        output,
    } = function_schema;
    Ok(models::FunctionSchema {
        receiver_info: receiver
            .as_ref()
            .map(|receiver_info| Box::new(to_api_receiver_info(receiver_info))),
        input: Some(to_api_blueprint_payload_def(context, input)?),
        output: Some(to_api_blueprint_payload_def(context, output)?),
    })
}

pub fn to_api_receiver_info(receiver_info: &ReceiverInfo) -> models::ReceiverInfo {
    let ReceiverInfo {
        receiver,
        ref_types,
    } = receiver_info;
    models::ReceiverInfo {
        receiver: match receiver {
            Receiver::SelfRef => models::receiver_info::Receiver::SelfRef,
            Receiver::SelfRefMut => models::receiver_info::Receiver::SelfRefMut,
        },
        reference_type: Box::new(models::ReferenceType {
            raw_bits: to_api_u32_as_i64(ref_types.bits()),
            normal: ref_types.intersects(RefTypes::NORMAL),
            direct_access: ref_types.intersects(RefTypes::DIRECT_ACCESS),
        }),
    }
}

pub fn to_api_blueprint_schema_fields_partition(
    context: &MappingContext,
    partition_description: &PartitionDescription,
    schemas: &[FieldSchema<BlueprintPayloadDef>],
) -> Result<models::BlueprintSchemaFieldPartition, MappingError> {
    Ok(models::BlueprintSchemaFieldPartition {
        partition_description: Box::new(to_api_partition_description(partition_description)?),
        fields: schemas
            .iter()
            .map(|schema| to_api_blueprint_field_schema(context, schema))
            .collect::<Result<_, MappingError>>()?,
    })
}

pub fn to_api_blueprint_field_schema(
    context: &MappingContext,
    field_schema: &FieldSchema<BlueprintPayloadDef>,
) -> Result<models::FieldSchema, MappingError> {
    let FieldSchema {
        field,
        condition,
        transience,
    } = field_schema;
    Ok(models::FieldSchema {
        field_type_ref: Some(to_api_blueprint_payload_def(context, field)?),
        condition: Some(Box::new(match condition {
            Condition::Always => models::FieldSchemaFeatureCondition::FieldSchemaFeatureConditionAlways {},
            Condition::IfFeature(feature) => models::FieldSchemaFeatureCondition::FieldSchemaFeatureConditionIfOwnFeature {
                feature_name: feature.to_string(),
            },
            Condition::IfOuterFeature(feature) => models::FieldSchemaFeatureCondition::FieldSchemaFeatureConditionIfOuterObjectFeature {
                feature_name: feature.to_string(),
            },
        })),
        transient_default_value_hex: match transience {
            FieldTransience::NotTransient => None,
            FieldTransience::TransientStatic { default_value } => Some(to_hex(default_value)),
        },
    })
}

pub fn to_api_blueprint_schema_collection_partition(
    context: &MappingContext,
    partition_description: &PartitionDescription,
    collection_schema: &BlueprintCollectionSchema<BlueprintPayloadDef>,
) -> Result<models::BlueprintSchemaCollectionPartition, MappingError> {
    Ok(models::BlueprintSchemaCollectionPartition {
        partition_description: Box::new(to_api_partition_description(partition_description)?),
        collection_schema: Some(to_api_blueprint_collection_schema(
            context,
            collection_schema,
        )?),
    })
}

pub fn to_api_partition_description(
    partition_description: &PartitionDescription,
) -> Result<models::PartitionDescription, MappingError> {
    let (description_type, value) = match partition_description {
        PartitionDescription::Logical(logical) => {
            (models::PartitionDescriptionType::Logical, logical.0)
        }
        PartitionDescription::Physical(physical) => {
            (models::PartitionDescriptionType::Physical, physical.0)
        }
    };
    Ok(models::PartitionDescription {
        _type: description_type,
        value: to_api_u8_as_i32(value),
    })
}

pub fn to_api_blueprint_collection_schema(
    context: &MappingContext,
    collection_schema: &BlueprintCollectionSchema<BlueprintPayloadDef>,
) -> Result<models::BlueprintCollectionSchema, MappingError> {
    Ok(match collection_schema {
        BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueSchema::<
            BlueprintPayloadDef,
        > {
            key,
            value,
            allow_ownership,
        }) => models::BlueprintCollectionSchema::KeyValueBlueprintCollectionSchema {
            key_type_ref: Box::new(to_api_blueprint_payload_def(context, key)?),
            value_type_ref: Box::new(to_api_blueprint_payload_def(context, value)?),
            allow_ownership: *allow_ownership,
        },
        BlueprintCollectionSchema::Index(BlueprintKeyValueSchema::<BlueprintPayloadDef> {
            key,
            value,
            allow_ownership,
        }) => models::BlueprintCollectionSchema::IndexBlueprintCollectionSchema {
            key_type_ref: Box::new(to_api_blueprint_payload_def(context, key)?),
            value_type_ref: Box::new(to_api_blueprint_payload_def(context, value)?),
            allow_ownership: *allow_ownership,
        },
        BlueprintCollectionSchema::SortedIndex(
            BlueprintKeyValueSchema::<BlueprintPayloadDef> {
                key,
                value,
                allow_ownership,
            },
        ) => models::BlueprintCollectionSchema::SortedIndexBlueprintCollectionSchema {
            key_type_ref: Box::new(to_api_blueprint_payload_def(context, key)?),
            value_type_ref: Box::new(to_api_blueprint_payload_def(context, value)?),
            allow_ownership: *allow_ownership,
        },
    })
}

pub fn to_api_scrypto_schema(
    context: &MappingContext,
    schema: &Schema<ScryptoCustomSchema>,
) -> Result<models::ScryptoSchema, MappingError> {
    Ok(models::ScryptoSchema {
        sbor_data: Box::new(to_api_sbor_data_from_encodable(context, schema)?),
    })
}
