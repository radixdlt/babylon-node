use std::ops::Deref;

use radix_engine::types::*;

use radix_engine_store_interface::interface::SubstateDatabase;

use radix_engine::system::attached_modules::metadata::MetadataCollection;
use radix_engine::system::attached_modules::metadata::VersionedMetadataEntry;

use sbor::basic_well_known_types::STRING_TYPE;

use super::*;

lazy_static::lazy_static! {
    /// A statically-known type information of the [`MetadataCollection::EntryKeyValue`].
    static ref METADATA_COLLECTION_META: ObjectCollectionMeta = {
        let (local_type_id, versioned_schema) =
            generate_full_schema_from_single_type::<VersionedMetadataEntry, ScryptoCustomSchema>();
        let LocalTypeId::SchemaLocalIndex(index) = local_type_id else {
            panic!("VersionedMetadataEntry is a custom type");
        };
        // Create a dummy reference: required by the `SchemaBasedTypeReference` helper type, but not
        // actually used (since we have the actual `SchemaV1` instance and type index).
        let schema_reference = SchemaReference {
            node_id: NodeId::new(EntityType::GlobalPackage as u8, &[0u8; NodeId::RID_LENGTH]),
            schema_hash: SchemaHash(Hash::from_bytes([0; Hash::LENGTH])),
        };
        ObjectCollectionMeta {
            index: RichIndex::of(MetadataCollection::EntryKeyValue.collection_index() as usize),
            kind: ObjectCollectionKind::KeyValueStore,
            resolved_key_type: ResolvedTypeMeta {
                type_reference: ResolvedTypeReference::WellKnown(STRING_TYPE),
                schema: SchemaV1::empty(),
            },
            resolved_value_type: ResolvedTypeMeta {
                type_reference: ResolvedTypeReference::SchemaBased(SchemaBasedTypeReference {
                    schema_reference,
                    index,
                }),
                schema: versioned_schema.into_latest(),
            },
        }
    };
}

/// A lister and loader of Object's attached Metadata entries.
///
/// Note: as evident by its sole [`EngineStateDataLoader`] dependency, this loader operates at an
/// abstraction layer higher than the rest of the Engine State API (i.e. it interprets the data that
/// can be read using other, lower-level means).
pub struct ObjectMetadataLoader<'s, S: SubstateDatabase> {
    loader: EngineStateDataLoader<'s, S>,
}

impl<'s, S: SubstateDatabase> ObjectMetadataLoader<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self {
            loader: EngineStateDataLoader::new(database),
        }
    }

    /// Returns an iterator of keys within the Metadata module attached to the given object,
    /// starting at the given key.
    pub fn iter_keys(
        &self,
        object_node_id: &NodeId,
        from_key: Option<&MetadataKey>,
    ) -> Result<impl Iterator<Item = MetadataKey> + '_, AttachedModuleBrowsingError> {
        Ok(self
            .loader
            .iter_object_collection_keys(
                object_node_id,
                ModuleId::Metadata,
                METADATA_COLLECTION_META.deref(),
                from_key
                    .map(|key| {
                        RawCollectionKey::Unsorted(ScryptoValue::String {
                            value: key.string.clone(),
                        })
                    })
                    .as_ref(),
            )?
            .map(|key| to_metadata_key(key)))
    }

    /// Loads a value of the given Metadata entry.
    pub fn load_entry(
        &self,
        object_node_id: &NodeId,
        key: &MetadataKey,
    ) -> Result<MetadataValue, AttachedModuleBrowsingError> {
        let entry_data = self.loader.load_collection_entry(
            object_node_id,
            ModuleId::Metadata,
            METADATA_COLLECTION_META.deref(),
            &RawCollectionKey::Unsorted(ScryptoValue::String {
                value: key.string.clone(),
            }),
        )?;
        Ok(
            scrypto_decode::<VersionedMetadataEntry>(entry_data.as_bytes())
                .map_err(AttachedModuleBrowsingError::SborDecode)?
                .into_latest(),
        )
    }
}

fn to_metadata_key(key: ObjectCollectionKey) -> MetadataKey {
    let ObjectCollectionKey::KeyValueStore(sbor_data) = key else {
        panic!("metadata collection must be Key-Value; got {:?}", key)
    };
    MetadataKey {
        string: scrypto_decode(sbor_data.as_bytes()).expect("metadata keys must be strings"),
    }
}

/// A type-safe Metadata key; always a string.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct MetadataKey {
    pub string: String,
}

/// An error that can be encountered while browsing Object's attached modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachedModuleBrowsingError {
    UnderlyingError(EngineStateBrowsingError),
    SborDecode(DecodeError),
}

impl From<EngineStateBrowsingError> for AttachedModuleBrowsingError {
    fn from(error: EngineStateBrowsingError) -> Self {
        AttachedModuleBrowsingError::UnderlyingError(error)
    }
}

impl From<AttachedModuleBrowsingError> for ResponseError {
    fn from(error: AttachedModuleBrowsingError) -> Self {
        match error {
            AttachedModuleBrowsingError::UnderlyingError(error) => ResponseError::from(error),
            AttachedModuleBrowsingError::SborDecode(error) => ResponseError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not decode value from database",
            )
            .with_internal_message(format!("{:?}", error)),
        }
    }
}
