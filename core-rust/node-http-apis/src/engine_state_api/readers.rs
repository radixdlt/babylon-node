use std::ops::Deref;

use radix_engine::types::*;

use radix_engine_store_interface::interface::SubstateDatabase;

use convert_case::{Case, Casing};
use itertools::Itertools;
use radix_engine::blueprints::package::{
    PackageCollection, VersionedPackageBlueprintVersionAuthConfig,
};

use radix_engine::system::system_db_reader::ObjectCollectionKey as ScryptoObjectCollectionKey;
use radix_engine::system::system_db_reader::{SystemDatabaseReader, SystemReaderError};
use radix_engine::system::system_type_checker::{BlueprintTypeTarget, SchemaValidationMeta};
use radix_engine::system::type_info::TypeInfoSubstate;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintInterface, BlueprintPayloadDef, BlueprintType, BlueprintVersion,
    BlueprintVersionKey, CanonicalBlueprintId, FunctionAuth, FunctionSchema, IndexedStateSchema,
    MethodAuthTemplate, RoleSpecification,
};

use crate::engine_state_api::handlers::RawCollectionKey;
use crate::engine_state_api::models::ErrorDetails;
use state_manager::store::traits::indices::{CreationId, EntityBlueprintIdV1, ReNodeListingIndex};
use state_manager::store::traits::SubstateNodeAncestryStore;

use super::*;

lazy_static::lazy_static! {
    /// The schema (and local type index) of the [`VersionedScryptoSchema`] Rust struct.
    /// Needed to return Scrypto schemas as (annotated) Programmatic JSON.
    static ref VERSIONED_SCHEMA_TYPE: (SchemaV1<ScryptoCustomSchema>, LocalTypeId) = {
        let (local_type_id, versioned_schema) =
            generate_full_schema_from_single_type::<VersionedScryptoSchema, ScryptoCustomSchema>();
        (versioned_schema.into_latest(), local_type_id)
    };
}

/// A loader of Engine State's metadata required by the Engine State API.
pub struct EngineStateMetaLoader<'s, S: SubstateDatabase> {
    // TODO(post-feature refactor): The implementation uses only the "reader" API provided by the
    // Engine, but many parts could be achieved in a more performant way (e.g. avoid loading the
    // same data multiple times, or avoid parsing large parts of SBOR). We can either extend the
    // Engine's reader, or implement required lower-level logic here.
    reader: SystemDatabaseReader<'s, S>,
}

impl<'s, S: SubstateDatabase + SubstateNodeAncestryStore> EngineStateMetaLoader<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self {
            reader: SystemDatabaseReader::new(database),
        }
    }

    /// Loads metadata on the given blueprint.
    pub fn load_blueprint_meta(
        &self,
        blueprint_id: &CanonicalBlueprintId,
    ) -> Result<BlueprintMeta, EngineStateBrowsingError> {
        if blueprint_id.version != BlueprintVersion::default() {
            return Err(EngineStateBrowsingError::RequestedItemInvalid(
                ItemKind::Blueprint,
                "only the default blueprint version is currently supported".to_string(),
            ));
        }
        let blueprint_id = BlueprintId::new(&blueprint_id.address, &blueprint_id.blueprint);
        let definition = self
            .reader
            .get_blueprint_definition(&blueprint_id)
            .map_err(|error| match error {
                SystemReaderError::BlueprintDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Blueprint)
                }
                unexpected => EngineStateBrowsingError::UnexpectedEngineError(
                    unexpected,
                    "when getting blueprint definition".to_string(),
                ),
            })?;

        let BlueprintInterface {
            blueprint_type,
            is_transient,
            generics,
            feature_set,
            state,
            functions,
            events,
            types,
        } = definition.interface.clone();
        let IndexedStateSchema {
            fields,
            collections,
            ..
        } = state;

        let node_id = blueprint_id.package_address.as_node_id();
        let blueprint_name = blueprint_id.blueprint_name.as_str();

        let AuthorizedCallablesMeta {
            functions,
            methods,
            roles,
        } = self.load_authorized_callables_meta(node_id, blueprint_name, functions)?;

        Ok(BlueprintMeta {
            outer_blueprint_name: match blueprint_type {
                BlueprintType::Outer => None,
                BlueprintType::Inner { outer_blueprint } => Some(outer_blueprint),
            },
            is_transient,
            generics,
            available_features: Vec::from_iter(feature_set),
            fields: fields
                .into_iter()
                .flat_map(|(_partition_description, fields)| fields)
                .enumerate()
                .map(|(index, field)| {
                    self.load_blueprint_field_meta(node_id, blueprint_name, index, field)
                })
                .collect::<Result<Vec<_>, _>>()?,
            collections: collections
                .into_iter()
                .enumerate()
                .map(|(index, (_partition_description, collection))| {
                    self.load_blueprint_collection_meta(node_id, blueprint_name, index, collection)
                })
                .collect::<Result<Vec<_>, _>>()?,
            functions,
            methods,
            roles,
            events: events
                .into_iter()
                .map(|(name, event)| self.load_blueprint_event_meta(node_id, name, event))
                .collect::<Result<Vec<_>, _>>()?,
            named_types: types
                .into_iter()
                .map(|(name, scoped_type_id)| {
                    self.load_blueprint_named_type_meta(node_id, name, scoped_type_id)
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Loads metadata on the given entity.
    /// Supports uninstantiated entities.
    pub fn load_entity_meta(
        &self,
        node_id: &NodeId,
    ) -> Result<EntityMeta, EngineStateBrowsingError> {
        let type_info = match self.reader.get_type_info(node_id) {
            Ok(type_info) => type_info,
            Err(error) => match error {
                SystemReaderError::NodeIdDoesNotExist => {
                    if node_id.is_global_virtual() {
                        return self.derive_uninstantiated_entity_meta(
                            node_id.entity_type().expect("we just checked its type"),
                        );
                    }
                    return Err(EngineStateBrowsingError::RequestedItemNotFound(
                        ItemKind::Entity,
                    ));
                }
                unexpected => {
                    return Err(EngineStateBrowsingError::UnexpectedEngineError(
                        unexpected,
                        "when getting type info".to_string(),
                    ))
                }
            },
        };
        match type_info {
            TypeInfoSubstate::Object(object_info) => Ok(EntityMeta::Object(
                self.load_object_meta(node_id, object_info)?,
            )),
            TypeInfoSubstate::KeyValueStore(kv_store_info) => Ok(EntityMeta::KeyValueStore(
                self.load_kv_store_meta(node_id, kv_store_info)?,
            )),
            TypeInfoSubstate::GlobalAddressReservation(_)
            | TypeInfoSubstate::GlobalAddressPhantom(_) => {
                Err(EngineStateBrowsingError::RequestedItemInvalid(
                    ItemKind::Entity,
                    "entity neither an object nor a KV store".to_string(),
                ))
            }
        }
    }

    /// Loads metadata on "state" (i.e. all fields and collections) of the given object's module.
    /// Does *not* support uninstantiated objects.
    ///
    /// API note: this is normally a part of the [`Self::load_entity_meta()`] result, but some
    /// clients are interested only in specific module and can use this cheaper method.
    pub fn load_object_module_state_meta(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<ObjectModuleStateMeta, EngineStateBrowsingError> {
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
        self.load_blueprint_state_meta(&type_target)
    }

    /// Loads extra metadata on the given field (a part of [`Self::load_blueprint_meta()`]).
    fn load_blueprint_field_meta(
        &self,
        node_id: &NodeId,
        blueprint_name: &str,
        index: usize,
        schema: FieldSchema<BlueprintPayloadDef>,
    ) -> Result<BlueprintFieldMeta, EngineStateBrowsingError> {
        let FieldSchema {
            field,
            condition,
            transience,
        } = schema;
        let declared_type = self.load_blueprint_type_meta(node_id, field)?;
        Ok(BlueprintFieldMeta {
            index: RichIndex::of(index)
                .with_derived_field_name(blueprint_name, declared_type.name()),
            declared_type,
            condition,
            transience_default_value_bytes: match transience {
                FieldTransience::NotTransient => None,
                FieldTransience::TransientStatic { default_value } => Some(default_value),
            },
        })
    }

    /// Loads extra metadata on authorization-aware callables (i.e. methods and functions) belonging
    /// to the given blueprint (a part of [`Self::load_blueprint_meta()`]).
    fn load_authorized_callables_meta(
        &self,
        node_id: &NodeId,
        blueprint_name: &str,
        callables: IndexMap<String, FunctionSchema>,
    ) -> Result<AuthorizedCallablesMeta, EngineStateBrowsingError> {
        let AuthConfig {
            function_auth,
            method_auth,
        } = self
            .reader
            .read_object_collection_entry::<_, VersionedPackageBlueprintVersionAuthConfig>(
                node_id,
                ModuleId::Main,
                ScryptoObjectCollectionKey::KeyValue(
                    PackageCollection::BlueprintVersionAuthConfigKeyValue.collection_index(),
                    &BlueprintVersionKey::new_default(blueprint_name),
                ),
            )
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when getting blueprint auth config".to_string(),
                )
            })?
            .ok_or_else(|| {
                EngineStateBrowsingError::EngineInvariantBroken(
                    "no auth config found for blueprint".to_string(),
                )
            })?
            .into_latest();

        let mut functions = Vec::new();
        let mut methods = Vec::new();

        for (name, schema) in callables {
            let FunctionSchema {
                receiver,
                input,
                output,
            } = schema;
            let declared_input_type = self.load_blueprint_type_meta(node_id, input)?;
            let declared_output_type = self.load_blueprint_type_meta(node_id, output)?;
            match receiver {
                None => {
                    let authorization = match &function_auth {
                        FunctionAuth::AllowAll => BlueprintFunctionAuthorization::Public,
                        FunctionAuth::AccessRules(rules) => {
                            let rule = rules.get(&name).ok_or_else(|| {
                                EngineStateBrowsingError::EngineInvariantBroken(
                                    "no rule found for function".to_string(),
                                )
                            })?;
                            BlueprintFunctionAuthorization::ByAccessRule(rule.clone())
                        }
                        FunctionAuth::RootOnly => BlueprintFunctionAuthorization::RootOnly,
                    };
                    functions.push(BlueprintFunctionMeta {
                        name,
                        declared_input_type,
                        declared_output_type,
                        authorization,
                    });
                }
                Some(receiver) => {
                    let authorization = match &method_auth {
                        MethodAuthTemplate::AllowAll => BlueprintMethodAuthorization::Public,
                        MethodAuthTemplate::StaticRoleDefinition(definitions) => {
                            let accessibility = definitions
                                .methods
                                .get(&MethodKey::new(&name))
                                .ok_or_else(|| {
                                    EngineStateBrowsingError::EngineInvariantBroken(
                                        "no accessibility found for method".to_string(),
                                    )
                                })?;
                            match accessibility {
                                MethodAccessibility::Public => BlueprintMethodAuthorization::Public,
                                MethodAccessibility::OuterObjectOnly => {
                                    BlueprintMethodAuthorization::OuterObjectOnly
                                }
                                MethodAccessibility::RoleProtected(role_list) => {
                                    BlueprintMethodAuthorization::ByRoles(role_list.list.clone())
                                }
                                MethodAccessibility::OwnPackageOnly => {
                                    BlueprintMethodAuthorization::OwnPackageOnly
                                }
                            }
                        }
                    };
                    methods.push(BlueprintMethodMeta {
                        name,
                        receiver,
                        declared_input_type,
                        declared_output_type,
                        authorization,
                    });
                }
            }
        }

        let roles = match method_auth {
            MethodAuthTemplate::AllowAll => BlueprintRolesDefinition::Local(Vec::new()),
            MethodAuthTemplate::StaticRoleDefinition(definition) => match definition.roles {
                RoleSpecification::Normal(roles) => BlueprintRolesDefinition::Local(
                    roles
                        .into_iter()
                        .map(|(key, updaters)| BlueprintRoleMeta {
                            key,
                            updater_role_keys: updaters.list,
                        })
                        .collect(),
                ),
                RoleSpecification::UseOuter => BlueprintRolesDefinition::Outer,
            },
        };

        Ok(AuthorizedCallablesMeta {
            functions,
            methods,
            roles,
        })
    }

    /// Loads extra metadata on the given collection (a part of [`Self::load_blueprint_meta()`]).
    fn load_blueprint_collection_meta(
        &self,
        node_id: &NodeId,
        blueprint_name: &str,
        index: usize,
        schema: BlueprintCollectionSchema<BlueprintPayloadDef>,
    ) -> Result<BlueprintCollectionMeta, EngineStateBrowsingError> {
        let (kind, collection_schema) = Self::destructure_collection_schema(schema);
        let BlueprintKeyValueSchema { key, value, .. } = collection_schema;
        let declared_key_type = self.load_blueprint_type_meta(node_id, key)?;
        let declared_value_type = self.load_blueprint_type_meta(node_id, value)?;
        Ok(BlueprintCollectionMeta {
            index: RichIndex::of(index)
                .with_derived_collection_name(blueprint_name, declared_value_type.name()),
            kind,
            declared_key_type,
            declared_value_type,
        })
    }

    /// Loads extra metadata on the given event (a part of [`Self::load_blueprint_meta()`]).
    fn load_blueprint_event_meta(
        &self,
        node_id: &NodeId,
        name: String,
        payload_def: BlueprintPayloadDef,
    ) -> Result<BlueprintEventMeta, EngineStateBrowsingError> {
        Ok(BlueprintEventMeta {
            name,
            declared_type: self.load_blueprint_type_meta(node_id, payload_def)?,
        })
    }

    /// Loads extra metadata on the given named type (a part of [`Self::load_blueprint_meta()`]).
    fn load_blueprint_named_type_meta(
        &self,
        node_id: &NodeId,
        name: String,
        scoped_type_id: ScopedTypeId,
    ) -> Result<BlueprintNamedTypeMeta, EngineStateBrowsingError> {
        Ok(BlueprintNamedTypeMeta {
            name,
            resolved_type: self
                .augment_with_schema(ResolvedTypeReference::new(*node_id, scoped_type_id))?,
        })
    }

    /// Loads extra metadata on the given blueprint-declared type (if it is statically defined;
    /// cannot really load anything more about a generic).
    fn load_blueprint_type_meta(
        &self,
        node_id: &NodeId,
        payload_def: BlueprintPayloadDef,
    ) -> Result<BlueprintTypeMeta, EngineStateBrowsingError> {
        Ok(match payload_def {
            BlueprintPayloadDef::Static(scoped_type_id) => BlueprintTypeMeta::Static(
                self.augment_with_schema(ResolvedTypeReference::new(*node_id, scoped_type_id))?,
            ),
            BlueprintPayloadDef::Generic(index) => BlueprintTypeMeta::Generic(index),
        })
    }

    /// An implementation delegate of [`Self::load_entity_meta()`] for `Object`s.
    fn load_object_meta(
        &self,
        node_id: &NodeId,
        object_info: ObjectInfo,
    ) -> Result<ObjectMeta, EngineStateBrowsingError> {
        let ObjectInfo {
            blueprint_info,
            object_type,
        } = object_info;
        Ok(ObjectMeta {
            is_instantiated: true,
            main_module_state: self.load_object_module_state_meta(node_id, ModuleId::Main)?,
            attached_module_states: match object_type {
                ObjectType::Global { modules } => modules
                    .into_keys() // deliberately ignored per-module blueprint versions
                    .map(|module_id| {
                        Ok((
                            module_id,
                            self.load_object_module_state_meta(node_id, module_id.into())?,
                        ))
                    })
                    .collect::<Result<IndexMap<_, _>, _>>()?,
                ObjectType::Owned => index_map_new(),
            },
            blueprint_reference: BlueprintReference {
                id: blueprint_info.blueprint_id,
                version: blueprint_info.blueprint_version,
            },
            instance_meta: ObjectInstanceMeta {
                outer_object: match blueprint_info.outer_obj_info {
                    OuterObjectInfo::Some { outer_object } => Some(outer_object),
                    OuterObjectInfo::None => None,
                },
                enabled_features: Vec::from_iter(blueprint_info.features),
                substituted_generic_types: blueprint_info
                    .generic_substitutions
                    .into_iter()
                    .map(|substitution| {
                        TypeReferenceResolver::new(&self.reader)
                            .resolve_generic_substitution(Some(node_id), substitution)
                            .and_then(|resolved_type| self.augment_with_schema(resolved_type))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
    }

    /// An implementation delegate of [`Self::load_entity_meta()`] for uninstantiated entities.
    // TODO(after development in scrypto repo): The implementation here hardcodes the results for
    // the only currently known uninstantiated entity types (accounts and identities). A more robust
    // solution could be implemented on the Engine's side (e.g. staged instantiation).
    fn derive_uninstantiated_entity_meta(
        &self,
        entity_type: EntityType,
    ) -> Result<EntityMeta, EngineStateBrowsingError> {
        let blueprint_id = match entity_type {
            EntityType::GlobalVirtualSecp256k1Account | EntityType::GlobalVirtualEd25519Account => {
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT)
            }
            EntityType::GlobalVirtualSecp256k1Identity
            | EntityType::GlobalVirtualEd25519Identity => {
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT)
            }
            _ => panic!("not an uninstantiated entity type"),
        };
        let blueprint_info = BlueprintInfo {
            blueprint_id,
            blueprint_version: BlueprintVersion::default(),
            outer_obj_info: OuterObjectInfo::None,
            features: index_set_new(),
            generic_substitutions: vec![],
        };
        Ok(EntityMeta::Object(ObjectMeta {
            is_instantiated: false,
            main_module_state: self.load_blueprint_state_meta(&BlueprintTypeTarget {
                blueprint_info: blueprint_info.clone(),
                meta: SchemaValidationMeta::Blueprint,
            })?,
            attached_module_states: index_map_new(),
            blueprint_reference: BlueprintReference {
                id: blueprint_info.blueprint_id,
                version: blueprint_info.blueprint_version,
            },
            instance_meta: ObjectInstanceMeta {
                outer_object: None,
                enabled_features: vec![],
                substituted_generic_types: vec![],
            },
        }))
    }

    /// An implementation delegate of [`Self::load_entity_meta()`] for `KeyValueStore`s.
    fn load_kv_store_meta(
        &self,
        node_id: &NodeId,
        kv_store_info: KeyValueStoreInfo,
    ) -> Result<KeyValueStoreMeta, EngineStateBrowsingError> {
        let KeyValueStoreGenericSubstitutions {
            key_generic_substitution,
            value_generic_substitution,
            ..
        } = kv_store_info.generic_substitutions;
        let resolver = TypeReferenceResolver::new(&self.reader);
        Ok(KeyValueStoreMeta {
            resolved_key_type: resolver
                .resolve_generic_substitution(Some(node_id), key_generic_substitution)
                .and_then(|resolved_type| self.augment_with_schema(resolved_type))?,
            resolved_value_type: resolver
                .resolve_generic_substitution(Some(node_id), value_generic_substitution)
                .and_then(|resolved_type| self.augment_with_schema(resolved_type))?,
        })
    }

    /// Loads metadata of all fields and collections within the blueprint referenced by the given
    /// [`BlueprintTypeTarget`]. The "type target" will also be used while resolving all types (see
    /// [`TypeReferenceResolver`]).
    fn load_blueprint_state_meta(
        &self,
        type_target: &BlueprintTypeTarget,
    ) -> Result<ObjectModuleStateMeta, EngineStateBrowsingError> {
        let blueprint_info = &type_target.blueprint_info;
        let blueprint_id = &blueprint_info.blueprint_id;
        let blueprint_name = blueprint_id.blueprint_name.as_str();
        let IndexedStateSchema {
            fields,
            collections,
            ..
        } = self
            .reader
            .get_blueprint_definition(blueprint_id)
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when getting blueprint definition".to_string(),
                )
            })?
            .interface
            .state
            .clone();

        let type_resolver = TypeReferenceResolver::new(&self.reader);
        let conditions_context = self.load_conditions_context(blueprint_info)?;

        let fields = fields
            .into_iter()
            .flat_map(|(_partition_description, fields)| fields)
            .enumerate()
            .filter(|(_index, schema)| conditions_context.meets(&schema.condition))
            .map(|(index, schema)| {
                Ok(ObjectFieldMeta::new(
                    index,
                    blueprint_id.blueprint_name.as_str(),
                    type_resolver
                        .resolve_type_from_blueprint_data(type_target, schema.field)
                        .and_then(|resolved_type| self.augment_with_schema(resolved_type))?,
                    schema.transience,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let collections = collections
            .into_iter()
            .enumerate()
            .map(|(index, (_partition_description, schema))| {
                let (kind, collection_schema) = Self::destructure_collection_schema(schema);
                let BlueprintKeyValueSchema { key, value, .. } = collection_schema;
                Ok(ObjectCollectionMeta::new(
                    index,
                    blueprint_name,
                    kind,
                    type_resolver
                        .resolve_type_from_blueprint_data(type_target, key)
                        .and_then(|resolved_type| self.augment_with_schema(resolved_type))?,
                    type_resolver
                        .resolve_type_from_blueprint_data(type_target, value)
                        .and_then(|resolved_type| self.augment_with_schema(resolved_type))?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ObjectModuleStateMeta {
            fields,
            collections,
        })
    }

    /// Constructs an [`ObjectConditionsContext`] from the given object's blueprint-related info.
    /// Note: the method belongs to the `load_*` family, since it may need to actually load the
    /// outer object's enabled feature set (if it exists) from database.
    fn load_conditions_context(
        &self,
        blueprint_info: &BlueprintInfo,
    ) -> Result<ObjectConditionsContext, EngineStateBrowsingError> {
        let object_features = blueprint_info.features.clone();
        let outer_object_features = match &blueprint_info.outer_obj_info {
            OuterObjectInfo::Some { outer_object } => {
                self.reader
                    .get_object_info(*outer_object.as_node_id())
                    .map_err(|error| {
                        EngineStateBrowsingError::UnexpectedEngineError(
                            error,
                            "when fetching outer object's info".to_string(),
                        )
                    })?
                    .blueprint_info
                    .features
            }
            OuterObjectInfo::None => index_set_new(),
        };
        Ok(ObjectConditionsContext {
            object_features,
            outer_object_features,
        })
    }

    /// Wraps the given [`ResolvedTypeReference`] into a [`ResolvedTypeMeta`] by *eagerly* loading
    /// the actual referenced schema.
    /// Note: the schema seems irrelevant for many "get meta information" methods, but it is needed
    /// to resolve human-readable type names (from which some field names are derived as well).
    fn augment_with_schema(
        &self,
        type_reference: ResolvedTypeReference,
    ) -> Result<ResolvedTypeMeta, EngineStateBrowsingError> {
        Ok(ResolvedTypeMeta {
            schema: match &type_reference {
                ResolvedTypeReference::SchemaBased(schema_based) => {
                    let SchemaReference {
                        node_id,
                        schema_hash,
                    } = &schema_based.schema_reference;
                    self.reader
                        .get_schema(node_id, schema_hash)
                        .map_err(|error| {
                            EngineStateBrowsingError::UnexpectedEngineError(
                                error,
                                "when locating schema".to_string(),
                            )
                        })?
                        .clone_into_latest()
                }
                ResolvedTypeReference::WellKnown(_) => SchemaV1::empty(),
            },
            type_reference,
        })
    }

    /// Converts the given [`BlueprintCollectionSchema`] to a more direct representation.
    fn destructure_collection_schema(
        schema: BlueprintCollectionSchema<BlueprintPayloadDef>,
    ) -> (
        ObjectCollectionKind,
        BlueprintKeyValueSchema<BlueprintPayloadDef>,
    ) {
        match schema {
            BlueprintCollectionSchema::KeyValueStore(schema) => {
                (ObjectCollectionKind::KeyValueStore, schema)
            }
            BlueprintCollectionSchema::Index(schema) => (ObjectCollectionKind::Index, schema),
            BlueprintCollectionSchema::SortedIndex(schema) => {
                (ObjectCollectionKind::SortedIndex, schema)
            }
        }
    }
}

/// Externally-relevant metadata on a blueprint (does not include all backend-specific information).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintMeta {
    pub outer_blueprint_name: Option<String>,
    pub is_transient: bool,
    pub generics: Vec<GenericBound>,
    pub available_features: Vec<String>,
    pub fields: Vec<BlueprintFieldMeta>,
    pub collections: Vec<BlueprintCollectionMeta>,
    pub functions: Vec<BlueprintFunctionMeta>,
    pub methods: Vec<BlueprintMethodMeta>,
    pub roles: BlueprintRolesDefinition,
    pub events: Vec<BlueprintEventMeta>,
    pub named_types: Vec<BlueprintNamedTypeMeta>,
}

/// Metadata on a field's definition within a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintFieldMeta {
    pub index: RichIndex,
    pub declared_type: BlueprintTypeMeta,
    pub condition: Condition,
    // Note: this field is not already-post-processed (and thus also not `pub`). The caller needs to
    // use the `self.transience()` method, due to Rust lifetimes reasons (namely, the returned
    // schema-aware "default value" references the `self.declared_type`).
    transience_default_value_bytes: Option<Vec<u8>>,
}

impl BlueprintFieldMeta {
    /// Post-processes and returns the [`Self::transience_default_value_bytes`] (see the note there).
    pub fn transience(&self) -> Option<FieldTransienceMeta> {
        self.transience_default_value_bytes
            .as_ref()
            .map(|default_value_bytes| FieldTransienceMeta {
                default_value: SborData::new(
                    default_value_bytes.clone(),
                    match &self.declared_type {
                        BlueprintTypeMeta::Static(resolved_type) => resolved_type,
                        BlueprintTypeMeta::Generic(_) => {
                            panic!("generic field cannot be transient")
                        }
                    },
                ),
            })
    }
}

/// Details relevant when a field is transient.
pub struct FieldTransienceMeta<'t> {
    pub default_value: SborData<'t>,
}

/// Metadata on a function's definition within a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintFunctionMeta {
    pub name: String,
    pub declared_input_type: BlueprintTypeMeta,
    pub declared_output_type: BlueprintTypeMeta,
    pub authorization: BlueprintFunctionAuthorization,
}

/// Authorization configuration of a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintFunctionAuthorization {
    Public,
    ByAccessRule(AccessRule),
    RootOnly,
}

/// Metadata on a method's definition within a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintMethodMeta {
    pub name: String,
    pub receiver: ReceiverInfo,
    pub declared_input_type: BlueprintTypeMeta,
    pub declared_output_type: BlueprintTypeMeta,
    pub authorization: BlueprintMethodAuthorization,
}

/// Authorization configuration of a method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintMethodAuthorization {
    Public,
    ByRoles(Vec<RoleKey>),
    OuterObjectOnly,
    OwnPackageOnly,
}

/// Roles defined by a blueprint (which may mean delegating to the outer object's blueprint).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintRolesDefinition {
    Local(Vec<BlueprintRoleMeta>),
    Outer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintRoleMeta {
    pub key: RoleKey,
    pub updater_role_keys: Vec<RoleKey>,
}

/// Metadata on an event's definition within a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintEventMeta {
    pub name: String,
    pub declared_type: BlueprintTypeMeta,
}

/// Metadata on a collection's definition within a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintCollectionMeta {
    pub index: RichIndex,
    pub kind: ObjectCollectionKind,
    pub declared_key_type: BlueprintTypeMeta,
    pub declared_value_type: BlueprintTypeMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintNamedTypeMeta {
    pub name: String,
    pub resolved_type: ResolvedTypeMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintTypeMeta {
    Static(ResolvedTypeMeta),
    Generic(u8),
}

impl BlueprintTypeMeta {
    /// Returns the type's name, if it is a concrete type (i.e. not generic) and its name is defined
    /// by the schema.
    pub fn name(&self) -> Option<&str> {
        match self {
            BlueprintTypeMeta::Static(resolved_type) => resolved_type.name(),
            BlueprintTypeMeta::Generic(_) => None,
        }
    }
}

/// Metadata on a particular object or key-value store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityMeta {
    Object(ObjectMeta),
    KeyValueStore(KeyValueStoreMeta),
}

/// Metadata on a particular object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMeta {
    pub is_instantiated: bool,
    pub main_module_state: ObjectModuleStateMeta,
    pub attached_module_states: IndexMap<AttachedModuleId, ObjectModuleStateMeta>,
    pub blueprint_reference: BlueprintReference,
    pub instance_meta: ObjectInstanceMeta,
}

/// A fully-disambiguated reference to a blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintReference {
    pub id: BlueprintId,
    pub version: BlueprintVersion,
}

/// Object's metadata details defined on a per-instance basis (i.e. not in blueprint).
/// In other words: the information that would be required to instantiate an object using a
/// particular blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInstanceMeta {
    pub outer_object: Option<GlobalAddress>,
    pub enabled_features: Vec<String>,
    pub substituted_generic_types: Vec<ResolvedTypeMeta>,
}

/// Metadata on a particular key-value store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValueStoreMeta {
    pub resolved_key_type: ResolvedTypeMeta,
    pub resolved_value_type: ResolvedTypeMeta,
}

/// A fully-disambiguated reference to a well-known or schema-defined type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTypeReference {
    WellKnown(WellKnownTypeId),
    SchemaBased(SchemaBasedTypeReference),
}

impl ResolvedTypeReference {
    /// Creates a type reference from the Engine's over-specified representation.
    fn new(node_id: NodeId, scoped_type_id: ScopedTypeId) -> ResolvedTypeReference {
        let ScopedTypeId(schema_hash, local_type_id) = scoped_type_id;
        match local_type_id {
            LocalTypeId::WellKnown(id) => ResolvedTypeReference::WellKnown(id),
            LocalTypeId::SchemaLocalIndex(index) => {
                ResolvedTypeReference::SchemaBased(SchemaBasedTypeReference {
                    schema_reference: SchemaReference {
                        node_id,
                        schema_hash,
                    },
                    index,
                })
            }
        }
    }

    /// Creates a [`LocalTypeId`] from this type reference.
    /// This is used to interact back with the Engine's "reader" API.
    fn to_local_type_id(&self) -> LocalTypeId {
        match self {
            ResolvedTypeReference::WellKnown(id) => LocalTypeId::WellKnown(*id),
            ResolvedTypeReference::SchemaBased(id) => LocalTypeId::SchemaLocalIndex(id.index),
        }
    }
}

/// A fully-disambiguated reference to a type defined by the given schema at the given index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaBasedTypeReference {
    pub schema_reference: SchemaReference,
    pub index: usize,
}

/// Metadata on all fields/collections of a particular object's *module*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectModuleStateMeta {
    pub fields: Vec<ObjectFieldMeta>,
    pub collections: Vec<ObjectCollectionMeta>,
}

impl ObjectModuleStateMeta {
    /// Gets the particular field's metadata by its human-readable name.
    /// Please see [`RichIndex::with_derived_field_name()`] to learn how a human-readable name is
    /// derived.
    /// Note: not every field has a name - either because it is inherently unnamed (e.g. within a
    /// tuple), or because its name does not strictly follow our naming convention (and thus cannot
    /// be automatically derived). For such cases, [`Self::field_by_index()`] is the only option.
    pub fn field_by_name(
        &self,
        name: impl Into<String>,
    ) -> Result<&ObjectFieldMeta, EngineStateBrowsingError> {
        let requested_derived_name = Some(name.into());
        Self::exactly_one_with_derived_name(
            self.fields
                .iter()
                .filter(|field| field.index.derived_name == requested_derived_name),
            ItemKind::Field,
        )
    }

    /// Gets the particular field's metadata by its index.
    pub fn field_by_index(&self, index: u8) -> Result<&ObjectFieldMeta, EngineStateBrowsingError> {
        self.fields
            .get(usize::from(index))
            .ok_or(EngineStateBrowsingError::RequestedItemNotFound(
                ItemKind::Field,
            ))
    }

    /// Gets the particular collection's metadata by its human-readable name.
    /// Please see [`RichIndex::with_derived_collection_name()`] to learn how a human-readable name
    /// is derived.
    /// Note: not every collection has a name - either because it is inherently unnamed (e.g. within
    /// a tuple), or because its name does not strictly follow our naming convention (and thus
    /// cannot be automatically derived). For such cases, [`Self::collection_by_index()`] is the
    /// only option.
    pub fn collection_by_name(
        &self,
        name: impl Into<String>,
    ) -> Result<&ObjectCollectionMeta, EngineStateBrowsingError> {
        let requested_derived_name = Some(name.into());
        Self::exactly_one_with_derived_name(
            self.collections
                .iter()
                .filter(|collection| collection.index.derived_name == requested_derived_name),
            ItemKind::Collection,
        )
    }

    /// Gets the particular collection's metadata by its index.
    pub fn collection_by_index(
        &self,
        index: u8,
    ) -> Result<&ObjectCollectionMeta, EngineStateBrowsingError> {
        self.collections.get(usize::from(index)).ok_or(
            EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Collection),
        )
    }

    /// Returns the only item (supposedly found by its derived name), or an error if not *exactly*
    /// one such item was found.
    fn exactly_one_with_derived_name<T>(
        found_items: impl IntoIterator<Item = T>,
        item_kind: ItemKind,
    ) -> Result<T, EngineStateBrowsingError> {
        let mut found_items = found_items.into_iter().collect::<Vec<_>>();
        match found_items.len() {
            0 => Err(EngineStateBrowsingError::RequestedItemNotFound(item_kind)),
            1 => Ok(found_items.remove(0)),
            _ => Err(EngineStateBrowsingError::RequestedItemInvalid(
                item_kind,
                "derived name not unique".to_string(),
            )),
        }
    }
}

/// A type reference accompanied by its schema.
/// Note: the [`ResolvedTypeReference`] already contains a *reference* to the schema, but in many
/// cases we need the actual schema value - this structure simply allows to load the schema once and
/// pass it around.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypeMeta {
    pub type_reference: ResolvedTypeReference,
    pub schema: SchemaV1<ScryptoCustomSchema>,
}

/// A fully-disambiguated reference to a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaReference {
    pub node_id: NodeId,
    pub schema_hash: SchemaHash,
}

impl ResolvedTypeMeta {
    /// Returns the type's name, if it is defined by the schema.
    pub fn name(&self) -> Option<&str> {
        self.schema
            .resolve_type_name_from_metadata(self.type_reference.to_local_type_id())
    }
}

/// An index (of a field or collection), accompanied by a human-readable name derived from available
/// metadata (on a best-effort basis - see [`Self::derive_from_generated()`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichIndex {
    pub number: u8,
    pub derived_name: Option<String>,
}

impl RichIndex {
    /// Creates an instance with unknown name.
    fn of(number: usize) -> Self {
        Self {
            number: number
                .try_into()
                .expect("guaranteed by maximum partition count"),
            derived_name: None,
        }
    }

    /// Adds a human-readable field name (if successfully derived from the given blueprint name and
    /// type name).
    fn with_derived_field_name(self, blueprint_name: &str, type_name: Option<&str>) -> Self {
        Self {
            number: self.number,
            derived_name: Self::derive_from_generated(blueprint_name, type_name, "FieldPayload"),
        }
    }

    /// Adds a human-readable collection name (if successfully derived from the given blueprint name
    /// and type name).
    fn with_derived_collection_name(self, blueprint_name: &str, type_name: Option<&str>) -> Self {
        Self {
            number: self.number,
            derived_name: Self::derive_from_generated(blueprint_name, type_name, "EntryPayload"),
        }
    }

    /// Performs a best-effort, heuristic resolution of a human-readable field/collection name,
    /// given its blueprint name and the auto-generated type name.
    ///
    /// Implementation note:
    /// The type name most often is auto-generated by the Engine's blueprint macro, and thus follows
    /// a strict naming convention: `<BlueprintName><CamelCaseTypeName><KnownSuffix>`. This allows
    /// us to extract it and convert to `snake_case`.
    // TODO(after development in scrypto repo): It would be more bullet-proof to somehow capture the
    // field/collection name (in the blueprint macro) and simply retrieve it here.
    fn derive_from_generated(
        blueprint_name: &str,
        type_name: Option<&str>,
        known_suffix: &str,
    ) -> Option<String> {
        type_name.and_then(|type_name| {
            type_name
                .strip_prefix(blueprint_name)
                .and_then(|name| name.strip_suffix(known_suffix))
                .map(|name| name.to_case(Case::Snake))
        })
    }
}

/// Metadata of a particular field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectFieldMeta {
    pub index: RichIndex,
    pub resolved_type: ResolvedTypeMeta,
    pub transience_default_value_bytes: Option<Vec<u8>>,
}

impl ObjectFieldMeta {
    /// Creates a self-contained field metadata: captures its index, name (if applicable) and a
    /// fully-resolved type information.
    /// The [`blueprint_name`] is only used for deriving the human-readable field name.
    fn new(
        field_index: usize,
        blueprint_name: &str,
        resolved_type: ResolvedTypeMeta,
        transience: FieldTransience,
    ) -> Self {
        let index = RichIndex::of(field_index)
            .with_derived_field_name(blueprint_name, resolved_type.name());
        Self {
            index,
            resolved_type,
            transience_default_value_bytes: match transience {
                FieldTransience::NotTransient => None,
                FieldTransience::TransientStatic { default_value } => Some(default_value),
            },
        }
    }
}

/// One of supported kinds of collections within an Object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectCollectionKind {
    KeyValueStore,
    Index,
    SortedIndex,
}

/// Metadata of a particular collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectCollectionMeta {
    pub index: RichIndex,
    pub kind: ObjectCollectionKind,
    pub resolved_key_type: ResolvedTypeMeta,
    pub resolved_value_type: ResolvedTypeMeta,
}

impl ObjectCollectionMeta {
    /// Creates a self-contained collection metadata: captures its index, name (if applicable) and a
    /// fully-resolved schema.
    /// The [`blueprint_name`] is only used for deriving the human-readable field name.
    fn new(
        collection_index: usize,
        blueprint_name: &str,
        kind: ObjectCollectionKind,
        resolved_key_type: ResolvedTypeMeta,
        resolved_value_type: ResolvedTypeMeta,
    ) -> Self {
        let index = RichIndex::of(collection_index)
            .with_derived_collection_name(blueprint_name, resolved_value_type.name());
        Self {
            index,
            kind,
            resolved_key_type,
            resolved_value_type,
        }
    }
}

/// Authorization-aware callables (i.e. methods and functions) belonging to a specific blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorizedCallablesMeta {
    functions: Vec<BlueprintFunctionMeta>,
    methods: Vec<BlueprintMethodMeta>,
    roles: BlueprintRolesDefinition,
}

/// A lister of entities.
pub struct EngineEntityLister<'s, S> {
    database: &'s S,
}

/// Basic information about an entity (i.e. read from a DB index, for listing purposes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySummary {
    pub node_id: NodeId,
    pub creation_id: CreationId,
    pub blueprint_id: Option<BlueprintId>, // only present for Object entities
}

impl<'s, S: ReNodeListingIndex> EngineEntityLister<'s, S> {
    /// Creates an instance reading from the given database.
    pub fn new(database: &'s S) -> Self {
        Self { database }
    }

    /// Returns an iterator of all entities, starting from the given [`CreationId`] (or its
    /// successor, if it does not exist), in the [`CreationId`]'s natural order (ascending).
    pub fn iter_entities(
        &self,
        from_creation_id: Option<&CreationId>,
    ) -> impl Iterator<Item = EntitySummary> + '_ {
        Self::all_entity_types()
            .map(|entity_type| {
                self.database
                    .get_created_entity_iter(entity_type, from_creation_id)
            })
            .kmerge_by(|(a_creation_id, _), (b_creation_id, _)| a_creation_id < b_creation_id)
            .map(
                |(
                    creation_id,
                    EntityBlueprintIdV1 {
                        node_id,
                        blueprint_id,
                    },
                )| EntitySummary {
                    node_id,
                    creation_id,
                    blueprint_id,
                },
            )
    }

    /// Lists all possible [`EntityType`]s.
    fn all_entity_types() -> impl Iterator<Item = EntityType> {
        (0..=u8::MAX).filter_map(EntityType::from_repr)
    }
}

/// A loader of Engine State's data (i.e. values) required by the Engine State API.
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
        let ObjectFieldMeta {
            index,
            resolved_type,
            transience_default_value_bytes,
        } = field_meta;
        let value_bytes = match transience_default_value_bytes {
            Some(value_bytes) => value_bytes.clone(),
            None => self
                .reader
                .read_object_field(node_id, module_id, index.number)
                // if `ObjectFieldMeta` exists, then the object, module and field must exist - no errors expected:
                .map_err(|error| {
                    EngineStateBrowsingError::UnexpectedEngineError(
                        error,
                        "when reading object field".to_string(),
                    )
                })?
                .into(),
        };
        Ok(SborData::new(value_bytes, resolved_type))
    }

    /// Loads an SBOR-encoded value of the given field.
    /// Note: technically, loading an SBOR does not need the fully-resolved field metadata (just its
    /// index); however, the object we return is schema-aware, so that it can render itself
    /// together with field names. Hence the field metadata must first be obtained from the
    /// [`EngineStateMetaLoader`].
    pub fn load_collection_entry<'m>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        collection_meta: &'m ObjectCollectionMeta,
        key: &RawCollectionKey,
    ) -> Result<SborData<'m>, EngineStateBrowsingError> {
        let collection_key = Self::to_scrypto_object_collection_key(key, collection_meta)?;
        let mapped_value = self
            .reader
            .read_object_collection_entry::<_, ScryptoValue>(node_id, module_id, collection_key)
            // if `ObjectCollectionMeta` exists, then the object, module and collection must exist - no errors expected:
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when reading object collection".to_string(),
                )
            })?
            .ok_or(EngineStateBrowsingError::RequestedItemNotFound(
                ItemKind::EntryKey,
            ))?;
        Ok(SborData::new(
            scrypto_encode(&mapped_value).expect("it was just decoded"),
            &collection_meta.resolved_value_type,
        ))
    }

    /// Loads an SBOR-encoded value associated with the given key in the given Key-Value Store.
    /// Note: technically, loading an SBOR does not need the fully-resolved field metadata (just its
    /// index); however, the object we return is schema-aware, so that it can render itself
    /// together with field names. Hence the field metadata must first be obtained from the
    /// [`EngineStateMetaLoader`].
    pub fn load_kv_store_entry<'m>(
        &self,
        node_id: &NodeId,
        kv_store_meta: &'m KeyValueStoreMeta,
        key: &ScryptoValue,
    ) -> Result<SborData<'m>, EngineStateBrowsingError> {
        let mapped_value = self
            .reader
            .read_typed_kv_entry::<_, ScryptoValue>(node_id, key)
            .ok_or(EngineStateBrowsingError::RequestedItemNotFound(
                ItemKind::EntryKey,
            ))?;
        Ok(SborData::new(
            scrypto_encode(&mapped_value).expect("it was just decoded"),
            &kv_store_meta.resolved_value_type,
        ))
    }

    /// Returns an iterator over all keys of the given object's collection, starting from the given
    /// key (or its successor, if it does not exist), in an arbitrary but deterministic order used
    /// by the backing storage.
    pub fn iter_object_collection_keys(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        collection_meta: &'s ObjectCollectionMeta,
        from_key: Option<&RawCollectionKey>,
    ) -> Result<impl Iterator<Item = ObjectCollectionKey> + '_, EngineStateBrowsingError> {
        let collection_index = collection_meta.index.number;
        let from_key = from_key.map(|key| Self::to_substate_key(key));
        Ok(self
            .reader
            .collection_iter_advanced(node_id, module_id, collection_index, from_key.as_ref())
            // if `ObjectCollectionMeta` exists, then the object, module and collection must exist - no errors expected:
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when reading object collection".to_string(),
                )
            })?
            .0
            .map(|(substate_key, _)| Self::to_object_collection_key(substate_key, collection_meta)))
    }

    /// Returns an iterator over all keys of the given Key-Value Store entity, starting from the
    /// given key (or its successor, if it does not exist), in an arbitrary but deterministic order
    /// used by the backing storage.
    pub fn iter_kv_store_keys(
        &self,
        node_id: &NodeId,
        kv_store_meta: &'s KeyValueStoreMeta,
        from_key: Option<&MapKey>,
    ) -> Result<impl Iterator<Item = SborData> + '_, EngineStateBrowsingError> {
        Ok(self
            .reader
            .key_value_store_iter(node_id, from_key)
            // if `KeyValueStoreMeta` exists, then the object, module and collection must exist - no errors expected:
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when iterating over Key-Value Store".to_string(),
                )
            })?
            .map(|(map_key, _)| SborData::new(map_key, &kv_store_meta.resolved_key_type)))
    }

    /// Loads schema by its reference, returning SBOR bytes of [`VersionedScryptoSchema`].
    pub fn load_schema(
        &self,
        reference: &SchemaReference,
    ) -> Result<SborData, EngineStateBrowsingError> {
        let versioned_schema = self
            .reader
            .get_schema(&reference.node_id, &reference.schema_hash)
            .map_err(|error| match error {
                SystemReaderError::SchemaDoesNotExist => {
                    EngineStateBrowsingError::RequestedItemNotFound(ItemKind::Schema)
                }
                _ => EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when fetching schema".to_string(),
                ),
            })?;
        let (meta_schema, meta_type_id) = VERSIONED_SCHEMA_TYPE.deref();
        Ok(SborData {
            payload_bytes: scrypto_encode(&versioned_schema).expect("it was just decoded"),
            schema: meta_schema,
            local_type_id: *meta_type_id,
        })
    }

    /// Creates an API *output* representation from the low-level object collection's substate key.
    fn to_object_collection_key(
        substate_key: SubstateKey,
        collection_meta: &ObjectCollectionMeta,
    ) -> ObjectCollectionKey {
        match (&collection_meta.kind, substate_key) {
            (ObjectCollectionKind::KeyValueStore, SubstateKey::Map(key)) => {
                ObjectCollectionKey::KeyValueStore(SborData::new(
                    key,
                    &collection_meta.resolved_key_type,
                ))
            }
            (ObjectCollectionKind::Index, SubstateKey::Map(key)) => {
                ObjectCollectionKey::Index(SborData::new(key, &collection_meta.resolved_key_type))
            }
            (ObjectCollectionKind::SortedIndex, SubstateKey::Sorted((sorted_prefix, key))) => {
                ObjectCollectionKey::SortedIndex(
                    sorted_prefix,
                    SborData::new(key, &collection_meta.resolved_key_type),
                )
            }
            _ => panic!("persisted key type does not match persisted collection type"),
        }
    }

    /// Creates a low-level collection key (i.e. for interfacing with the Engine's Substate store)
    /// from the API *input* representation.
    fn to_substate_key(collection_key: &RawCollectionKey) -> SubstateKey {
        match collection_key {
            RawCollectionKey::Sorted(sort_prefix, value) => SubstateKey::Sorted((
                *sort_prefix,
                scrypto_encode(value).expect("scrypto value must be encodable"),
            )),
            RawCollectionKey::Unsorted(value) => {
                SubstateKey::Map(scrypto_encode(value).expect("scrypto value must be encodable"))
            }
        }
    }

    /// Creates a mid-level collection key (i.e. for interfacing with the "system reader") from the
    /// API *input* representation.
    fn to_scrypto_object_collection_key<'k>(
        key: &'k RawCollectionKey,
        collection_meta: &ObjectCollectionMeta,
    ) -> Result<ScryptoObjectCollectionKey<'k, ScryptoValue>, EngineStateBrowsingError> {
        let index = collection_meta.index.number;
        Ok(match (&collection_meta.kind, key) {
            (ObjectCollectionKind::KeyValueStore, RawCollectionKey::Unsorted(value)) => {
                ScryptoObjectCollectionKey::KeyValue(index, value)
            }
            (ObjectCollectionKind::Index, RawCollectionKey::Unsorted(value)) => {
                ScryptoObjectCollectionKey::Index(index, value)
            }
            (ObjectCollectionKind::SortedIndex, RawCollectionKey::Sorted(sort_prefix, value)) => {
                ScryptoObjectCollectionKey::SortedIndex(
                    index,
                    u16::from_be_bytes(*sort_prefix),
                    value,
                )
            }
            _ => {
                return Err(EngineStateBrowsingError::RequestedItemInvalid(
                    ItemKind::EntryKey,
                    "requested key type does not match persisted collection type".to_string(),
                ))
            }
        })
    }
}

/// An [`SborData`] in a wrapper depending on the object collection kind.
pub enum ObjectCollectionKey<'t> {
    KeyValueStore(SborData<'t>),
    Index(SborData<'t>),
    SortedIndex([u8; 2], SborData<'t>),
}

/// A top-level SBOR value aware of its schema.
pub struct SborData<'t> {
    payload_bytes: Vec<u8>,
    schema: &'t SchemaV1<ScryptoCustomSchema>,
    local_type_id: LocalTypeId,
}

impl<'t> SborData<'t> {
    /// Creates an instance.
    fn new(payload_bytes: Vec<u8>, resolved_type: &'t ResolvedTypeMeta) -> Self {
        Self {
            payload_bytes,
            schema: &resolved_type.schema,
            local_type_id: resolved_type.type_reference.to_local_type_id(),
        }
    }

    /// Converts this instance to a schema-annotated programmatic JSON (already rendered as a
    /// `serde` JSON tree).
    pub fn into_programmatic_json(
        self,
        mapping_context: &MappingContext,
    ) -> Result<serde_json::Value, MappingError> {
        ProgrammaticJsonEncoder::new(mapping_context).encode(
            self.payload_bytes,
            self.schema,
            self.local_type_id,
        )
    }

    /// Returns raw SBOR bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.payload_bytes
    }

    /// Creates a [`ScryptoValue`] representation of these SBOR bytes.
    pub fn to_scrypto_value(&self) -> ScryptoValue {
        scrypto_decode(self.as_bytes()).expect("bytes read from substate store")
    }
}

/// An internal helper holding instance-specific context relevant to [`Condition`] checks.
struct ObjectConditionsContext {
    object_features: IndexSet<String>,
    outer_object_features: IndexSet<String>, // empty when there is no outer object
}

impl ObjectConditionsContext {
    /// Checks whether the object's state captured by this context meets the given condition.
    pub fn meets(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Always => true,
            Condition::IfFeature(feature) => self.object_features.contains(feature),
            Condition::IfOuterFeature(feature) => self.outer_object_features.contains(feature),
        }
    }
}

/// An internal helper for resolving concrete type references.
struct TypeReferenceResolver<'s, S: SubstateDatabase> {
    reader: &'s SystemDatabaseReader<'s, S>,
}

impl<'s, S: SubstateDatabase> TypeReferenceResolver<'s, S> {
    /// Creates an instance relying on the given reader.
    pub fn new(reader: &'s SystemDatabaseReader<'s, S>) -> Self {
        Self { reader }
    }

    /// Returns a type reference resolved from the given, already-loada [`BlueprintPayloadDef`]
    /// using the context from the [`BlueprintTypeTarget`].
    /// Note: this method does not load anything more from the store; technically it could get rid
    /// of the `&self` parameter.
    pub fn resolve_type_from_blueprint_data(
        &self,
        type_target: &BlueprintTypeTarget,
        payload_def: BlueprintPayloadDef,
    ) -> Result<ResolvedTypeReference, EngineStateBrowsingError> {
        let BlueprintTypeTarget {
            blueprint_info,
            meta,
        } = type_target;
        match payload_def {
            BlueprintPayloadDef::Static(scoped_type_id) => Ok(ResolvedTypeReference::new(
                blueprint_info.blueprint_id.package_address.into_node_id(),
                scoped_type_id,
            )),
            BlueprintPayloadDef::Generic(instance_index) => {
                let generic_substitution = blueprint_info
                    .generic_substitutions
                    .get(usize::from(instance_index))
                    .expect("missing generic substitution");
                let schemas_node_id = match meta {
                    SchemaValidationMeta::ExistingObject { additional_schemas } => {
                        Some(additional_schemas)
                    }
                    SchemaValidationMeta::NewObject { .. } | SchemaValidationMeta::Blueprint => {
                        None
                    }
                };
                self.resolve_generic_substitution(schemas_node_id, generic_substitution.clone())
            }
        }
    }

    /// Returns a type reference resolved from the given [`GenericSubstitution`].
    /// The local Node ID must be present if the substitution points to a [`ScopedTypeId`].
    pub fn resolve_generic_substitution(
        &self,
        local_node_id: Option<&NodeId>,
        generic_substitution: GenericSubstitution,
    ) -> Result<ResolvedTypeReference, EngineStateBrowsingError> {
        match generic_substitution {
            GenericSubstitution::Local(scoped_type_id) => {
                let schemas_node_id = local_node_id.ok_or_else(|| {
                    EngineStateBrowsingError::EngineInvariantBroken(
                        "local generic substitution requires known entity".to_string(),
                    )
                })?;
                Ok(ResolvedTypeReference::new(*schemas_node_id, scoped_type_id))
            }
            GenericSubstitution::Remote(blueprint_type_identifier) => {
                self.resolve_type_from_blueprint_reference(blueprint_type_identifier)
            }
        }
    }

    /// Returns a type reference resolved by name from a fetched blueprint (according to
    /// specification from the given [`BlueprintTypeIdentifier`]).
    fn resolve_type_from_blueprint_reference(
        &self,
        blueprint_type_identifier: BlueprintTypeIdentifier,
    ) -> Result<ResolvedTypeReference, EngineStateBrowsingError> {
        let BlueprintTypeIdentifier {
            package_address,
            blueprint_name,
            type_name,
        } = blueprint_type_identifier.clone();
        let blueprint_id = BlueprintId {
            package_address,
            blueprint_name,
        };
        let blueprint_definition = self
            .reader
            .get_blueprint_payload_def(&blueprint_id)
            .map_err(|error| {
                EngineStateBrowsingError::UnexpectedEngineError(
                    error,
                    "when getting def by ID known to exist".to_string(),
                )
            })?;
        let scoped_type_id = blueprint_definition.interface.types.get(&type_name).ok_or(
            EngineStateBrowsingError::EngineInvariantBroken(
                "no type of declared name found in blueprint definition".to_string(),
            ),
        )?;
        Ok(ResolvedTypeReference::new(
            package_address.into_node_id(),
            *scoped_type_id,
        ))
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
    /// The Engine's "reader" API returned data inconsistent with its declared behaviors.
    EngineInvariantBroken(String),
}

/// A kind of browsable item within Engine State.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Blueprint,
    Schema,
    Entity,
    Module,
    Field,
    Collection,
    EntryKey,
}

impl From<EngineStateBrowsingError> for ResponseError {
    fn from(error: EngineStateBrowsingError) -> Self {
        match error {
            EngineStateBrowsingError::RequestedItemNotFound(item_kind) => {
                ResponseError::new(StatusCode::NOT_FOUND, format!("{:?} not found", item_kind))
                    .with_public_details(ErrorDetails::RequestedItemNotFoundDetails {
                        item_type: to_api_requested_item_type(item_kind),
                    })
            }
            EngineStateBrowsingError::RequestedItemInvalid(item_kind, reason) => {
                ResponseError::new(
                    StatusCode::BAD_REQUEST,
                    format!("Invalid {:?}: {}", item_kind, reason),
                )
                .with_public_details(ErrorDetails::RequestedItemInvalidDetails {
                    item_type: to_api_requested_item_type(item_kind),
                })
            }
            EngineStateBrowsingError::UnexpectedEngineError(system_reader_error, circumstances) => {
                ResponseError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unexpected error encountered {}", circumstances),
                )
                .with_internal_message(format!("{:?}", system_reader_error))
            }
            EngineStateBrowsingError::EngineInvariantBroken(message) => ResponseError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid Engine state: {}", message),
            ),
        }
    }
}
