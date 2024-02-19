use crate::engine_prelude::*;
use crate::engine_state_api::*;

use std::ops::Deref;

pub(crate) async fn handle_blueprint_info(
    state: State<EngineStateApiState>,
    Json(request): Json<models::BlueprintInfoRequest>,
) -> Result<Json<models::BlueprintInfoResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let package_address = extract_package_address(&extraction_context, &request.package_address)
        .map_err(|err| err.into_response_error("package_address"))?;
    let blueprint_reference = BlueprintReference {
        id: BlueprintId {
            package_address,
            blueprint_name: request.blueprint_name,
        },
        version: request
            .blueprint_version
            .map(|blueprint_version| extract_blueprint_version(blueprint_version.as_str()))
            .transpose()
            .map_err(|err| err.into_response_error("blueprint_version"))?
            .unwrap_or_default(),
    };

    let database = state.state_manager.database.snapshot();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let blueprint_meta = meta_loader.load_blueprint_meta(&blueprint_reference)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::BlueprintInfoResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        info: Box::new(to_api_blueprint_info(&mapping_context, blueprint_meta)?),
    }))
}

fn to_api_blueprint_info(
    context: &MappingContext,
    meta: BlueprintMeta,
) -> Result<models::DetailedBlueprintInfo, MappingError> {
    let BlueprintMeta {
        outer_blueprint_name,
        is_transient,
        generics,
        available_features,
        fields,
        collections,
        functions,
        methods,
        roles,
        events,
        named_types,
    } = meta;
    Ok(models::DetailedBlueprintInfo {
        outer_blueprint_name,
        is_transient,
        generic_type_parameters: generics
            .into_iter()
            .map(|generic| match generic {
                GenericBound::Any => models::GenericTypeParameter::AnyGenericTypeParameter {},
            })
            .collect(),
        available_features,
        fields: fields
            .into_iter()
            .map(|field| to_api_blueprint_field_info(context, field))
            .collect::<Result<Vec<_>, _>>()?,
        collections: collections
            .into_iter()
            .map(|collection| to_api_blueprint_collection_info(context, collection))
            .collect::<Result<Vec<_>, _>>()?,
        functions: functions
            .into_iter()
            .map(|function| to_api_blueprint_function_info(context, function))
            .collect::<Result<Vec<_>, _>>()?,
        methods: methods
            .into_iter()
            .map(|method| to_api_blueprint_method_info(context, method))
            .collect::<Result<Vec<_>, _>>()?,
        roles: Some(to_api_blueprint_roles_definition(roles)),
        events: events
            .into_iter()
            .map(|event| to_api_blueprint_event_info(context, event))
            .collect::<Result<Vec<_>, _>>()?,
        named_types: named_types
            .into_iter()
            .map(|named_type| to_api_blueprint_named_type_info(context, named_type))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn to_api_blueprint_field_info(
    context: &MappingContext,
    field: BlueprintFieldMeta,
) -> Result<models::BlueprintFieldInfo, MappingError> {
    // cannot destructure due to the required getter usage below (see its note)
    let transience = field
        .transience()
        .map(|transience| {
            Ok(Box::new(models::BlueprintFieldTransience {
                default_value: Box::new(to_api_sbor_data(context, transience.default_value)?),
            }))
        })
        .transpose()?;
    Ok(models::BlueprintFieldInfo {
        index: to_api_u8_as_i32(field.index.number),
        name: field.index.derived_name,
        type_reference: Some(to_api_blueprint_type_info(context, field.declared_type)?),
        condition: match field.condition {
            Condition::Always => None,
            Condition::IfFeature(feature_name) => Some(Box::new(
                models::BlueprintFieldCondition::IfOwnFeatureFieldCondition { feature_name },
            )),
            Condition::IfOuterFeature(feature_name) => Some(Box::new(
                models::BlueprintFieldCondition::IfOwnFeatureFieldCondition { feature_name },
            )),
        },
        transience,
    })
}

fn to_api_blueprint_collection_info(
    context: &MappingContext,
    collection: BlueprintCollectionMeta,
) -> Result<models::BlueprintCollectionInfo, MappingError> {
    let BlueprintCollectionMeta {
        index,
        kind,
        declared_key_type,
        declared_value_type,
    } = collection;
    Ok(models::BlueprintCollectionInfo {
        index: to_api_u8_as_i32(index.number),
        name: index.derived_name,
        kind: to_api_object_collection_kind(&kind),
        key_type_reference: Some(to_api_blueprint_type_info(context, declared_key_type)?),
        value_type_reference: Some(to_api_blueprint_type_info(context, declared_value_type)?),
    })
}

fn to_api_blueprint_function_info(
    context: &MappingContext,
    function: BlueprintFunctionMeta,
) -> Result<models::BlueprintFunctionInfo, MappingError> {
    let BlueprintFunctionMeta {
        name,
        declared_input_type,
        declared_output_type,
        authorization,
        royalty,
    } = function;
    Ok(models::BlueprintFunctionInfo {
        name,
        input_type_reference: Some(to_api_blueprint_type_info(context, declared_input_type)?),
        output_type_reference: Some(to_api_blueprint_type_info(context, declared_output_type)?),
        authorization: Some(match authorization {
            BlueprintFunctionAuthorization::Public => {
                models::BlueprintFunctionAuthorization::PublicBlueprintFunctionAuthorization {}
            }
            BlueprintFunctionAuthorization::RootOnly => {
                models::BlueprintFunctionAuthorization::RootOnlyBlueprintFunctionAuthorization {}
            }
            BlueprintFunctionAuthorization::ByAccessRule(access_rule) => {
                models::BlueprintFunctionAuthorization::ByAccessRuleBlueprintFunctionAuthorization {
                    rule: Box::new(to_api_access_rule(context, &access_rule)?),
                }
            }
        }),
        royalty_amount: to_api_royalty_amount(&royalty).map(Box::new),
    })
}

fn to_api_blueprint_method_info(
    context: &MappingContext,
    method: BlueprintMethodMeta,
) -> Result<models::BlueprintMethodInfo, MappingError> {
    let BlueprintMethodMeta {
        name,
        receiver,
        declared_input_type,
        declared_output_type,
        authorization,
        royalty,
    } = method;
    Ok(models::BlueprintMethodInfo {
        name,
        receiver: Box::new(to_api_receiver_info(context, receiver)?),
        input_type_reference: Some(to_api_blueprint_type_info(context, declared_input_type)?),
        output_type_reference: Some(to_api_blueprint_type_info(context, declared_output_type)?),
        authorization: Some(match authorization {
            BlueprintMethodAuthorization::Public => {
                models::BlueprintMethodAuthorization::PublicBlueprintMethodAuthorization {}
            }
            BlueprintMethodAuthorization::OuterObjectOnly => {
                models::BlueprintMethodAuthorization::OuterObjectOnlyBlueprintMethodAuthorization {}
            }
            BlueprintMethodAuthorization::OwnPackageOnly => {
                models::BlueprintMethodAuthorization::OwnPackageOnlyBlueprintMethodAuthorization {}
            }
            BlueprintMethodAuthorization::ByRoles(roles) => {
                models::BlueprintMethodAuthorization::ByRolesBlueprintMethodAuthorization {
                    role_keys: roles.into_iter().map(|key| key.key).collect(),
                }
            }
        }),
        royalty_amount: to_api_royalty_amount(&royalty).map(Box::new),
    })
}

fn to_api_receiver_info(
    _context: &MappingContext,
    receiver: ReceiverInfo,
) -> Result<models::BlueprintMethodReceiverInfo, MappingError> {
    let ReceiverInfo {
        receiver,
        ref_types,
    } = receiver;
    Ok(models::BlueprintMethodReceiverInfo {
        receiver_type: match receiver {
            Receiver::SelfRef => models::MethodReceiverType::SelfRef,
            Receiver::SelfRefMut => models::MethodReceiverType::SelfRefMut,
        },
        reference_types: [
            (
                ref_types.contains(RefTypes::NORMAL),
                models::MethodReceiverReferenceType::Normal,
            ),
            (
                ref_types.contains(RefTypes::DIRECT_ACCESS),
                models::MethodReceiverReferenceType::DirectAccess,
            ),
        ]
        .into_iter()
        .filter(|(contains, _)| *contains)
        .map(|(_, value)| value)
        .collect(),
    })
}

fn to_api_blueprint_roles_definition(
    roles: BlueprintRolesDefinition,
) -> models::BlueprintRolesDefinition {
    match roles {
        BlueprintRolesDefinition::Local(roles) => {
            models::BlueprintRolesDefinition::LocalBlueprintRolesDefinition {
                definitions: roles
                    .into_iter()
                    .map(|role| models::BlueprintRoleInfo {
                        key: role.key.key,
                        updater_role_keys: role
                            .updater_role_keys
                            .into_iter()
                            .map(|key| key.key)
                            .collect(),
                    })
                    .collect(),
            }
        }
        BlueprintRolesDefinition::Outer => {
            models::BlueprintRolesDefinition::OuterBlueprintRolesDefinition {}
        }
    }
}

fn to_api_blueprint_event_info(
    context: &MappingContext,
    event: BlueprintEventMeta,
) -> Result<models::BlueprintEventInfo, MappingError> {
    let BlueprintEventMeta {
        name,
        declared_type,
    } = event;
    Ok(models::BlueprintEventInfo {
        name,
        type_reference: Some(to_api_blueprint_type_info(context, declared_type)?),
    })
}

fn to_api_blueprint_named_type_info(
    context: &MappingContext,
    named_type: BlueprintNamedTypeMeta,
) -> Result<models::BlueprintNamedTypeInfo, MappingError> {
    let BlueprintNamedTypeMeta {
        name,
        resolved_type,
    } = named_type;
    Ok(models::BlueprintNamedTypeInfo {
        name,
        type_reference: Some(to_api_resolved_type_reference(context, &resolved_type)?),
    })
}

fn to_api_blueprint_type_info(
    context: &MappingContext,
    type_meta: BlueprintTypeMeta,
) -> Result<models::BlueprintResolvedTypeReference, MappingError> {
    Ok(match type_meta {
        BlueprintTypeMeta::Static(type_meta) => {
            models::BlueprintResolvedTypeReference::BlueprintStaticTypeReference {
                static_type_reference: Box::new(to_api_resolved_type_reference(
                    context, &type_meta,
                )?),
            }
        }
        BlueprintTypeMeta::Generic(index) => {
            models::BlueprintResolvedTypeReference::BlueprintGenericTypeReference {
                generic_type_parameter_index: to_api_u8_as_i32(index),
            }
        }
    })
}
