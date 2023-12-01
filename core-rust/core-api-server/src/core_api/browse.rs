use std::collections::VecDeque;
use std::{iter, mem};

use radix_engine::types::*;

use radix_engine_store_interface::interface::{DbPartitionKey, SubstateDatabase};

use convert_case::{Case, Casing};
use radix_engine::system::system_db_reader::{
    ResolvedPayloadSchema, SystemDatabaseReader, SystemReaderError,
};
use radix_engine_interface::blueprints::package::BlueprintPayloadIdentifier;
use radix_engine_store_interface::db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper};
use radix_engine_stores::hash_tree::tree_store::{
    Nibble, NibblePath, NodeKey, ReadableTreeStore, TreeNode,
};

use sbor::representations::{SerializationMode, SerializationParameters};
use state_manager::store::traits::QueryableProofStore;
use tracing::warn;

use super::*;

/// A loader of Engine State's metadata required by the Browse API.
pub struct EngineStateMetaLoader<'s, S: SubstateDatabase> {
    // TODO(post-feature refactor): The implementation uses only the "reader" API provided by the
    // Engine, but many parts could be achieved in a more performant way (e.g. avoid loading the
    // same data multiple times, or avoid parsing large parts of SBOR). We can either extend the
    // Engine's reader, or implement required lower-level logic here.
    reader: SystemDatabaseReader<'s, S>,
}

impl<'s, S: SubstateDatabase> EngineStateMetaLoader<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self {
            reader: SystemDatabaseReader::new(database),
        }
    }

    /// Loads metadata on all fields of the given object's module.
    pub fn load_object_field_set_meta(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<ObjectFieldSetMeta, EngineStateBrowsingError> {
        let type_target = self
            .reader
            .get_blueprint_type_target(node_id, module_id)
            .map_err(|error| match error {
                SystemReaderError::NodeIdDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Entity)
                }
                SystemReaderError::ModuleDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Module)
                }
                SystemReaderError::NotAnObject => EngineStateBrowsingError::RequestedItemInvalid(
                    ItemKind::Entity,
                    "not an object".to_string(),
                ),
                unexpected => EngineStateBrowsingError::UnexpectedEngineError(
                    unexpected,
                    "when getting type target".to_string(),
                ),
            })?;
        let blueprint_id = &type_target.blueprint_info.blueprint_id;
        let blueprint_definition =
            self.reader
                .get_blueprint_definition(blueprint_id)
                .map_err(|error| {
                    EngineStateBrowsingError::UnexpectedEngineError(
                        error,
                        "when getting blueprint definition".to_string(),
                    )
                })?;

        let fields = (0..blueprint_definition.interface.state.num_fields())
            .map(|field_index| {
                field_index
                    .try_into()
                    .expect("guaranteed by max field count")
            })
            .map(|field_index| {
                Ok(ObjectFieldMeta::new(
                    field_index,
                    blueprint_id.blueprint_name.as_str(),
                    self.reader
                        .get_blueprint_payload_schema(
                            &type_target,
                            &BlueprintPayloadIdentifier::Field(field_index),
                        )
                        .map_err(|error| {
                            EngineStateBrowsingError::UnexpectedEngineError(
                                error,
                                "when getting blueprint payload schema".to_string(),
                            )
                        })?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ObjectFieldSetMeta { fields })
    }
}

/// Metadata on all fields of a particular object's module.
pub struct ObjectFieldSetMeta {
    fields: Vec<ObjectFieldMeta>,
}

impl ObjectFieldSetMeta {
    /// Gets the particular field's metadata by its human-readable name.
    /// Please see [`ObjectFieldMeta::derive_field_name()`] to learn how a human-readable name is
    /// derived.
    /// Note: not every field has a name - either because it is inherently unnamed (e.g. within a
    /// tuple), or because its name does not strictly follow our naming convention (and thus cannot
    /// be automatically derived). For such cases, [`Self::by_index()`] is the only option.
    pub fn by_name(
        &self,
        name: impl Into<String>,
    ) -> Result<&ObjectFieldMeta, EngineStateBrowsingError> {
        let requested_derived_name = Some(name.into());
        let found_fields = self
            .fields
            .iter()
            .filter(|field| field.derived_field_name == requested_derived_name)
            .collect::<Vec<_>>();
        match found_fields.len() {
            0 => Err(EngineStateBrowsingError::RequestedItemNotFound(
                ItemKind::Field,
            )),
            1 => Ok(found_fields[0]),
            _ => Err(EngineStateBrowsingError::RequestedItemInvalid(
                ItemKind::Field,
                "derived name not unique".to_string(),
            )),
        }
    }

    /// Gets the particular field's metadata by its index.
    pub fn by_index(&self, index: u8) -> Result<&ObjectFieldMeta, EngineStateBrowsingError> {
        self.fields
            .get(usize::from(index))
            .ok_or(EngineStateBrowsingError::RequestedItemNotFound(
                ItemKind::Field,
            ))
    }
}

/// Metadata of a particular field.
pub struct ObjectFieldMeta {
    field_index: u8,
    derived_field_name: Option<String>,
    schema: SchemaV1<ScryptoCustomSchema>,
    local_type_id: LocalTypeId,
}

impl ObjectFieldMeta {
    /// Creates a self-contained field metadata: captures its index, name (if applicable) and a
    /// fully-resolved schema.
    /// The [`blueprint_name`] is only used for deriving the human-readable field name.
    fn new(
        field_index: u8,
        blueprint_name: &str,
        resolved_payload_schema: ResolvedPayloadSchema,
    ) -> Self {
        let schema = resolved_payload_schema.schema.into_latest();
        let local_type_id = resolved_payload_schema.type_id;
        let derived_field_name = schema
            .resolve_type_name_from_metadata(local_type_id)
            .and_then(|type_name| Self::derive_field_name(blueprint_name, type_name));
        Self {
            field_index,
            derived_field_name,
            schema,
            local_type_id,
        }
    }

    /// Performs a best-effort, heuristic resolution of a human-readable field name, given its type
    /// name.
    ///
    /// Implementation note:
    /// The type name most often is auto-generated by the Engine's blueprint macro, and thus follows
    /// a strict naming convention: `<BlueprintName><CamelCasedFieldName>FieldPayload`. This allows
    /// us to extract it and convert to `snake_case`.
    // TODO(after development in scrypto repo): It would be more bullet-proof to somehow capture the
    // field name (in the blueprint macro) and simply retrieve it here.
    fn derive_field_name(blueprint_name: &str, type_name: &str) -> Option<String> {
        type_name
            .strip_prefix(blueprint_name)
            .and_then(|name| name.strip_suffix("FieldPayload"))
            .map(|name| name.to_case(Case::Snake))
    }
}

/// A lister of Engine's Nodes.
pub struct EngineNodeLister<'s, S> {
    database: &'s S,
}

impl<'s, S: QueryableProofStore + ReadableTreeStore> EngineNodeLister<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self { database }
    }

    /// Returns an iterator of all Engine Node IDs, starting from the given one (or its successor,
    /// if it does not exist), in an arbitrary but deterministic order used by the backing storage.
    pub fn iter_node_ids(
        &self,
        from_node_id: Option<&NodeId>,
    ) -> impl Iterator<Item = NodeId> + '_ {
        let current_version = self.database.max_state_version();
        let from_nibbles = from_node_id
            .map(|node_id| NibblePath::new_even(SpreadPrefixKeyMapper::to_db_node_key(node_id)))
            .map(|nibble_path| nibble_path.nibbles().collect::<VecDeque<_>>())
            .unwrap_or_default();
        self.recurse_until_full_leaf_paths(
            NodeKey::new_empty_path(current_version.number()),
            from_nibbles,
        )
        .map(|path| Self::node_id_from(path.bytes()))
    }

    /// Returns an iterator of all state hash tree's *leaf* [`NibblePath`]s, starting from some
    /// specific point.
    /// The implementation first loads the starting node by `key`, and then drills down the tree,
    /// following the `from_nibbles` chain - as long as it is possible.
    ///
    /// This is the recursive part of the [`Self::iter_node_ids()`] implementation.
    fn recurse_until_full_leaf_paths(
        &self,
        key: NodeKey,
        from_nibbles: VecDeque<Nibble>,
    ) -> Box<dyn Iterator<Item = NibblePath> + '_> {
        let Some(node) = self.database.get_node(&key) else {
            panic!("{:?} referenced but not found in the storage", key);
        };
        match node {
            TreeNode::Internal(internal) => {
                let mut child_from_nibbles = from_nibbles;
                let from_nibble = child_from_nibbles
                    .pop_front()
                    .unwrap_or_else(|| Nibble::from(0));
                Box::new(
                    internal
                        .children
                        .into_iter()
                        .filter(move |child| child.nibble >= from_nibble)
                        .flat_map(move |child| {
                            let child_key = key.gen_child_node_key(child.version, child.nibble);
                            let child_from_nibbles = if child.nibble == from_nibble {
                                mem::take(&mut child_from_nibbles)
                            } else {
                                VecDeque::new()
                            };
                            self.recurse_until_full_leaf_paths(child_key, child_from_nibbles)
                        }),
                )
            }
            TreeNode::Leaf(leaf) => Box::new(
                Some(leaf.key_suffix.nibbles())
                    .filter(|suffix_nibbles| {
                        suffix_nibbles
                            .remaining_nibbles()
                            .ge(from_nibbles.iter().cloned())
                    })
                    .map(|suffix_nibbles| {
                        NibblePath::from_iter(key.nibble_path().nibbles().chain(suffix_nibbles))
                    })
                    .into_iter(),
            ),
            TreeNode::Null => Box::new(iter::empty()),
        }
    }

    /// Extracts an Engine's Node ID from the state hash tree's full Node key.
    // TODO(after development in scrypto repo): The implementation here fakes a `partition_num: 0`,
    // but it would be better if `SpreadPrefixKeyMapper` just offered an API to convert the Node ID
    // alone.
    fn node_id_from(node_key_bytes: &[u8]) -> NodeId {
        SpreadPrefixKeyMapper::from_db_partition_key(&DbPartitionKey {
            node_key: node_key_bytes.to_vec(),
            partition_num: 0,
        })
        .0
    }
}

/// A loader of Engine State's data (i.e. values) required by the Browse API.
pub struct EngineStateDataLoader<'s, S: SubstateDatabase> {
    reader: SystemDatabaseReader<'s, S>,
}

impl<'s, S: SubstateDatabase> EngineStateDataLoader<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self {
            reader: SystemDatabaseReader::new(database),
        }
    }

    /// Loads an SBOR-encoded value of the given field.
    /// Note: technically, loading an SBOR does not need the fully-resolved field metadata (just its
    /// index); however, the object we return is schema-aware, so that it can render itself
    /// together with field names. Hence the field metadata must first be obtained from the
    /// [`EngineStateMetaLoader`].
    pub fn load_field_value<'m>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        field_meta: &'m ObjectFieldMeta,
    ) -> Result<SborData<'m>, EngineStateBrowsingError> {
        let indexed_value = self
            .reader
            .read_object_field(node_id, module_id, field_meta.field_index)
            .map_err(|error| match error {
                SystemReaderError::NodeIdDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Entity)
                }
                SystemReaderError::ModuleDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Module)
                }
                SystemReaderError::FieldDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Field)
                }
                SystemReaderError::NotAnObject => EngineStateBrowsingError::RequestedItemInvalid(
                    ItemKind::Entity,
                    "not an object".to_string(),
                ),
                unexpected => EngineStateBrowsingError::UnexpectedEngineError(
                    unexpected,
                    "when reading object field".to_string(),
                ),
            })?;
        Ok(SborData::new(
            indexed_value.into(),
            &field_meta.schema,
            field_meta.local_type_id,
        ))
    }
}

/// A top-level SBOR value aware of its schema.
pub struct SborData<'s> {
    payload_bytes: Vec<u8>,
    schema: &'s SchemaV1<ScryptoCustomSchema>,
    local_type_id: LocalTypeId,
}

impl<'s> SborData<'s> {
    /// Creates an instance.
    fn new(
        payload_bytes: Vec<u8>,
        schema: &'s SchemaV1<ScryptoCustomSchema>,
        local_type_id: LocalTypeId,
    ) -> Self {
        Self {
            payload_bytes,
            schema,
            local_type_id,
        }
    }

    /// Converts this instance to a schema-annotated programmatic JSON (already rendered as a
    /// `serde` JSON tree).
    pub fn into_programmatic_json(
        self,
        mapping_context: &MappingContext,
    ) -> Result<serde_json::Value, MappingError> {
        let raw_payload = RawScryptoPayload::new_from_valid_owned(self.payload_bytes);
        let serializable = raw_payload.serializable(SerializationParameters::WithSchema {
            mode: SerializationMode::Programmatic,
            custom_context: ScryptoValueDisplayContext::with_optional_bech32(Some(
                &mapping_context.address_encoder,
            )),
            schema: self.schema,
            type_id: self.local_type_id,
            depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
        });
        serde_json::to_value(serializable).map_err(|_error| MappingError::SubstateValue {
            bytes: raw_payload.payload_bytes().to_vec(),
            message: "cannot render as programmatic json".to_string(),
        })
    }
}

/// An error that can be encountered while browsing Engine State.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineStateBrowsingError {
    /// The caller's input referenced some non-existent item.
    RequestedItemNotFound(ItemKind),
    /// The caller's input referenced an item which should not be referenced in the current context.
    RequestedItemInvalid(ItemKind, String),
    /// The Engine's "reader" API returned an error which should never occur in the current context.
    UnexpectedEngineError(SystemReaderError, String),
}

/// A kind of browsable item within Engine State.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Entity,
    Module,
    Field,
}

impl<E: ErrorDetails> From<EngineStateBrowsingError> for ResponseError<E> {
    fn from(error: EngineStateBrowsingError) -> Self {
        match error {
            EngineStateBrowsingError::RequestedItemNotFound(item_kind) => {
                client_error(format!("{:?} not found", item_kind))
            }
            EngineStateBrowsingError::RequestedItemInvalid(item_kind, reason) => {
                client_error(format!("Invalid {:?}: {}", item_kind, reason))
            }
            EngineStateBrowsingError::UnexpectedEngineError(system_reader_error, circumstances) => {
                let public_message = format!("Unexpected state encountered {}", circumstances);
                warn!(?system_reader_error, public_message);
                server_error(public_message)
            }
        }
    }
}
