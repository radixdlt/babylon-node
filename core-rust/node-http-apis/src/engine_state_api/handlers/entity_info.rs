use crate::engine_state_api::*;
use radix_engine::types::*;
use state_manager::store::traits::{SubstateNodeAncestryRecord, SubstateNodeAncestryStore};
use std::ops::Deref;

pub(crate) async fn handle_entity_info(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntityInfoRequest>,
) -> Result<Json<models::EntityInfoResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let entity_meta = meta_loader.load_entity_meta(&node_id)?;

    // Technically, the ancestry information could be interpreted as part of "meta" and included in
    // the `EntityMeta` above. However, we plan to move `EngineStateMetaLoader` (among others) into
    // the Scrypto repo (e.g. make it a part of `SystemDatabaseReader`), and ancestry index is not
    // available there. So let's load the ancestry directly from our database here.
    let entity_ancestry = database.deref().get_ancestry(&node_id);

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::EntityInfoResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        info: Some(to_api_entity_info(
            &mapping_context,
            &node_id,
            &entity_meta,
            entity_ancestry.as_ref(),
        )?),
    }))
}

fn to_api_entity_info(
    context: &MappingContext,
    node_id: &NodeId,
    meta: &EntityMeta,
    ancestry: Option<&SubstateNodeAncestryRecord>,
) -> Result<models::EntityInfo, MappingError> {
    let ancestry = ancestry
        .map(|ancestry| to_api_entity_ancestry_info(context, ancestry))
        .transpose()?
        .map(Box::new);
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    Ok(match meta {
        EntityMeta::Object(meta) => models::EntityInfo::ObjectEntityInfo {
            ancestry,
            entity_type: to_api_entity_type(entity_type),
            is_global: entity_type.is_global(),
            is_instantiated: meta.is_instantiated,
            main_module_state: Box::new(to_api_object_module_state_info(
                context,
                &meta.main_module_state,
            )?),
            attached_modules: meta
                .attached_module_states
                .iter()
                .map(|(module_id, module_state)| {
                    Ok(models::ObjectEntityInfoAllOfAttachedModules {
                        module_id: to_api_attached_module_id(module_id),
                        state: Box::new(to_api_object_module_state_info(context, module_state)?),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            blueprint_reference: Box::new(to_api_blueprint_reference(
                context,
                &meta.blueprint_reference,
            )?),
            instance_info: Box::new(to_api_object_instance_info(context, &meta.instance_meta)?),
        },
        EntityMeta::KeyValueStore(meta) => models::EntityInfo::KeyValueStoreEntityInfo {
            ancestry,
            key_type_reference: Box::new(to_api_resolved_type_reference(
                context,
                &meta.resolved_key_type,
            )?),
            value_type_reference: Box::new(to_api_resolved_type_reference(
                context,
                &meta.resolved_value_type,
            )?),
        },
    })
}

fn to_api_entity_ancestry_info(
    context: &MappingContext,
    ancestry: &SubstateNodeAncestryRecord,
) -> Result<models::EntityAncestryInfo, MappingError> {
    Ok(models::EntityAncestryInfo {
        parent_entity_address: to_api_entity_address(context, &ancestry.parent.0)?,
        root_entity_address: to_api_entity_address(context, &ancestry.root.0)?,
    })
}

fn to_api_blueprint_reference(
    context: &MappingContext,
    blueprint_reference: &BlueprintReference,
) -> Result<models::BlueprintReference, MappingError> {
    Ok(models::BlueprintReference {
        package_address: to_api_package_address(context, &blueprint_reference.id.package_address)?,
        blueprint_name: blueprint_reference.id.blueprint_name.clone(),
        blueprint_version: to_api_blueprint_version(context, &blueprint_reference.version)?,
    })
}

fn to_api_object_instance_info(
    context: &MappingContext,
    instance_meta: &ObjectInstanceMeta,
) -> Result<models::ObjectInstanceInfo, MappingError> {
    let ObjectInstanceMeta {
        outer_object,
        enabled_features,
        substituted_generic_types,
    } = instance_meta;
    Ok(models::ObjectInstanceInfo {
        outer_object_address: outer_object
            .as_ref()
            .map(|address| to_api_global_address(context, address))
            .transpose()?,
        enabled_features: enabled_features.clone(),
        substituted_generic_types: substituted_generic_types
            .iter()
            .map(|resolved_type| to_api_resolved_type_reference(context, resolved_type))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn to_api_object_module_state_info(
    context: &MappingContext,
    object_module_state: &ObjectModuleStateMeta,
) -> Result<models::ObjectModuleStateInfo, MappingError> {
    let ObjectModuleStateMeta {
        fields,
        collections,
    } = object_module_state;
    Ok(models::ObjectModuleStateInfo {
        fields: fields
            .iter()
            .map(|field| to_api_object_field_info(context, field))
            .collect::<Result<Vec<_>, _>>()?,
        collections: collections
            .iter()
            .map(|collection| to_api_object_collection_info(context, collection))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn to_api_object_field_info(
    context: &MappingContext,
    object_field_meta: &ObjectFieldMeta,
) -> Result<models::ObjectFieldInfo, MappingError> {
    let ObjectFieldMeta {
        index,
        resolved_type,
        ..
    } = object_field_meta;
    Ok(models::ObjectFieldInfo {
        index: to_api_u8_as_i32(index.number),
        name: index.derived_name.clone(),
        type_reference: Some(to_api_resolved_type_reference(context, resolved_type)?),
    })
}

fn to_api_object_collection_info(
    context: &MappingContext,
    object_collection_meta: &ObjectCollectionMeta,
) -> Result<models::ObjectCollectionInfo, MappingError> {
    let ObjectCollectionMeta {
        index,
        kind,
        resolved_key_type,
        resolved_value_type,
    } = object_collection_meta;
    Ok(models::ObjectCollectionInfo {
        index: to_api_u8_as_i32(index.number),
        name: index.derived_name.clone(),
        kind: to_api_object_collection_kind(kind),
        key_type_reference: Some(to_api_resolved_type_reference(context, resolved_key_type)?),
        value_type_reference: Some(to_api_resolved_type_reference(
            context,
            resolved_value_type,
        )?),
    })
}
