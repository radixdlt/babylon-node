use crate::browse_api::*;

pub fn to_api_resolved_type_reference(
    context: &MappingContext,
    resolved_type: &ResolvedTypeMeta,
) -> Result<models::ResolvedTypeReference, MappingError> {
    let name = resolved_type.name().map(|str| str.to_string());
    Ok(match &resolved_type.type_reference {
        ResolvedTypeReference::WellKnown(type_id) => {
            models::ResolvedTypeReference::WellKnownTypeReference {
                index: to_api_index_as_i64(type_id.as_index())?,
                name,
            }
        }
        ResolvedTypeReference::SchemaBased(type_reference) => {
            models::ResolvedTypeReference::SchemaDefinedTypeReference {
                schema_reference: Box::new(to_api_schema_reference(
                    context,
                    &type_reference.schema_reference,
                )?),
                index: to_api_index_as_i64(type_reference.index)?,
                name,
            }
        }
    })
}

pub fn to_api_schema_reference(
    context: &MappingContext,
    schema_reference: &SchemaReference,
) -> Result<models::SchemaReference, MappingError> {
    let SchemaReference {
        node_id,
        schema_hash,
    } = schema_reference;
    Ok(models::SchemaReference {
        entity_address: to_api_entity_address(context, node_id)?,
        schema_hash: to_api_schema_hash(schema_hash),
    })
}

pub fn to_api_object_collection_kind(kind: &ObjectCollectionKind) -> models::ObjectCollectionKind {
    match kind {
        ObjectCollectionKind::KeyValueStore => models::ObjectCollectionKind::KeyValueStore,
        ObjectCollectionKind::Index => models::ObjectCollectionKind::Index,
        ObjectCollectionKind::SortedIndex => models::ObjectCollectionKind::SortedIndex,
    }
}
