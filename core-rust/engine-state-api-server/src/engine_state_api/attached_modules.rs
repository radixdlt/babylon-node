use std::any::type_name;
use std::ops::Deref;

use crate::engine_prelude::*;

use super::*;

lazy_static::lazy_static! {
    /// A statically-known type information of the [`MetadataCollection::EntryKeyValue`].
    static ref METADATA_COLLECTION_META: ObjectCollectionMeta = create_object_collection_meta::<String, VersionedMetadataEntry>(MetadataCollection::EntryKeyValue);

    /// A statically-known type information of the [`RoleAssignmentField::Owner`].
    static ref ROLE_ASSIGNMENT_OWNER_META: ObjectFieldMeta = create_object_field_meta::<VersionedRoleAssignmentOwner>(RoleAssignmentField::Owner);

    /// A statically-known type information of the [`RoleAssignmentCollection::AccessRuleKeyValue`].
    static ref ROLE_ASSIGNMENT_COLLECTION_META: ObjectCollectionMeta = create_object_collection_meta::<ModuleRoleKey, VersionedRoleAssignmentAccessRule>(RoleAssignmentCollection::AccessRuleKeyValue);

    /// A statically-known type information of the [`ComponentRoyaltyCollection::MethodAmountKeyValue`].
    static ref METHOD_ROYALTY_COLLECTION_META: ObjectCollectionMeta = create_object_collection_meta::<String, VersionedComponentRoyaltyMethodAmount>(ComponentRoyaltyCollection::MethodAmountKeyValue);
}

/// A loader of Object's attached Royalty information.
///
/// Note: as evident by its sole [`EngineStateDataLoader`] dependency, this loader operates at an
/// abstraction layer higher than the rest of the Engine State API (i.e. it interprets the data that
/// can be read using other, lower-level means).
pub struct ObjectRoyaltyLoader<'s, S: SubstateDatabase> {
    pub meta_loader: EngineStateMetaLoader<'s, S>,
    pub data_loader: EngineStateDataLoader<'s, S>,
}

impl<'s, S: SubstateDatabase> ObjectRoyaltyLoader<'s, S> {
    /// Returns Package and Component royalty amounts for all methods of the given object.
    pub fn load_method_amounts(
        &self,
        node_id: &NodeId,
    ) -> Result<Vec<MethodRoyaltyAmount>, AttachedModuleBrowsingError> {
        let EntityMeta::Object(object_meta) = self.meta_loader.load_entity_meta(node_id)? else {
            return Err(AttachedModuleBrowsingError::NotAnObject);
        };

        let mut component_royalties = if object_meta
            .attached_module_states
            .contains_key(&AttachedModuleId::Royalty)
        {
            self.data_loader
                .iter_object_collection(
                    node_id,
                    ModuleId::Royalty,
                    METHOD_ROYALTY_COLLECTION_META.deref(),
                    None,
                )?
                .map(|(key, value)| {
                    Ok::<_, AttachedModuleBrowsingError>((
                        decode_kv_collection_key::<String>(key),
                        decode_latest::<VersionedComponentRoyaltyMethodAmount>(value)?,
                    ))
                })
                .collect::<Result<NonIterMap<_, _>, _>>()?
        } else {
            // It is ok for an object to NOT have Royalty module (and it means free methods):
            NonIterMap::new()
        };

        Ok(self
            .meta_loader
            .load_blueprint_meta(&object_meta.blueprint_reference)?
            .methods
            .into_iter()
            .map(|method| MethodRoyaltyAmount {
                for_component: component_royalties
                    .remove(&method.name)
                    .unwrap_or(RoyaltyAmount::Free),
                for_package: method.royalty,
                name: method.name,
            })
            .collect())
    }
}

/// Resolved Package and Component royalty amounts of a specific method.
pub struct MethodRoyaltyAmount {
    pub name: String,
    pub for_component: RoyaltyAmount,
    pub for_package: RoyaltyAmount,
}

/// A loader of Object's attached Role Assignments.
///
/// Note: as evident by its "Engine State reader" dependencies, this loader operates at an
/// abstraction layer higher than the rest of the Engine State API (i.e. it interprets the data that
/// can be read using other, lower-level means).
pub struct ObjectRoleAssignmentLoader<'s, S: SubstateDatabase> {
    pub meta_loader: EngineStateMetaLoader<'s, S>,
    pub data_loader: EngineStateDataLoader<'s, S>,
}

impl<'s, S: SubstateDatabase> ObjectRoleAssignmentLoader<'s, S> {
    /// Loads full information from the [`ModuleId::RoleAssignment`] module:
    /// - the Owner rule and updater,
    /// - the role-to-rule assignment for all roles defined by the object and its attached modules.
    ///
    /// This correctly resolves the [`BlueprintRolesDefinition::Outer`], and list also the roles
    /// for which no rule is assigned (which denotes an Owner rule fallback).
    pub fn load_role_assignment(
        &self,
        node_id: &NodeId,
    ) -> Result<ObjectRoleAssignment, AttachedModuleBrowsingError> {
        let EntityMeta::Object(object_meta) = self.meta_loader.load_entity_meta(node_id)? else {
            return Err(AttachedModuleBrowsingError::NotAnObject);
        };
        let main_blueprint_meta = self
            .meta_loader
            .load_blueprint_meta(&object_meta.blueprint_reference)?;
        let main_roles = match main_blueprint_meta.roles {
            BlueprintRolesDefinition::Local(roles) => roles,
            BlueprintRolesDefinition::Outer => {
                let Some(outer_object_address) = object_meta.instance_meta.outer_object else {
                    return Err(AttachedModuleBrowsingError::AttachedModulesInvariantBroken(
                        "roles delegated to outer object which does not exist".to_string(),
                    ));
                };
                let EntityMeta::Object(outer_object_meta) = self
                    .meta_loader
                    .load_entity_meta(outer_object_address.as_node_id())?
                else {
                    return Err(AttachedModuleBrowsingError::AttachedModulesInvariantBroken(
                        "outer object turned out to not be an object".to_string(),
                    ));
                };
                let BlueprintRolesDefinition::Local(roles) = self
                    .meta_loader
                    .load_blueprint_meta(&outer_object_meta.blueprint_reference)?
                    .roles
                else {
                    return Err(AttachedModuleBrowsingError::AttachedModulesInvariantBroken(
                        "delegate outer object's blueprint does not define roles".to_string(),
                    ));
                };
                roles
            }
        };

        let owner_role_entry =
            decode_latest::<VersionedRoleAssignmentOwner>(self.data_loader.load_field_value(
                node_id,
                ModuleId::RoleAssignment,
                ROLE_ASSIGNMENT_OWNER_META.deref(),
            )?)?
            .owner_role_entry;

        let mut role_key_to_rule = self
            .data_loader
            .iter_object_collection(
                node_id,
                ModuleId::RoleAssignment,
                ROLE_ASSIGNMENT_COLLECTION_META.deref(),
                None,
            )?
            .map(|(key, value)| {
                Ok::<_, AttachedModuleBrowsingError>((
                    ModuleRoleKey::from(key),
                    decode_latest::<VersionedRoleAssignmentAccessRule>(value)?,
                ))
            })
            .collect::<Result<NonIterMap<_, _>, _>>()?;

        Ok(ObjectRoleAssignment {
            owner_role_entry,
            main_module_roles: Self::resolve_module_roles(
                main_roles,
                ModuleId::Main,
                &mut role_key_to_rule,
            ),
            attached_modules: object_meta
                .attached_module_states
                .into_keys()
                .map(|attached_module_id| {
                    let module_blueprint_reference = BlueprintReference {
                        id: attached_module_id.static_blueprint(),
                        version: BlueprintVersion::default(),
                    };
                    let BlueprintRolesDefinition::Local(module_roles) = self
                        .meta_loader
                        .load_blueprint_meta(&module_blueprint_reference)?
                        .roles
                    else {
                        return Err(AttachedModuleBrowsingError::AttachedModulesInvariantBroken(
                            format!("{:?}'s blueprint does not define roles", attached_module_id),
                        ));
                    };
                    Ok((
                        attached_module_id,
                        Self::resolve_module_roles(
                            module_roles,
                            attached_module_id.into(),
                            &mut role_key_to_rule,
                        ),
                    ))
                })
                .collect::<Result<IndexMap<_, _>, _>>()?,
        })
    }

    fn resolve_module_roles(
        module_roles: Vec<BlueprintRoleMeta>,
        module_id: ModuleId,
        role_key_to_rule: &mut NonIterMap<ModuleRoleKey, AccessRule>,
    ) -> ModuleRoles {
        module_roles
            .into_iter()
            .map(|role| {
                (
                    role.key.clone(),
                    role_key_to_rule
                        .remove(&ModuleRoleKey::new(module_id, role.key))
                        .map(Assignment::Explicit)
                        .unwrap_or(Assignment::Owner),
                )
            })
            .collect()
    }
}

/// Fully resolved information from the [`ModuleId::RoleAssignment`] module.
pub struct ObjectRoleAssignment {
    pub owner_role_entry: OwnerRoleEntry,
    pub main_module_roles: ModuleRoles,
    pub attached_modules: IndexMap<AttachedModuleId, ModuleRoles>,
}

/// A role -> rule map alias.
pub type ModuleRoles = IndexMap<RoleKey, Assignment>;

/// An explicitly assigned rule, or fallback to Owner rule.
pub enum Assignment {
    Explicit(AccessRule),
    Owner,
}

/// A lister and loader of Object's attached Metadata entries.
///
/// Note: as evident by its sole [`EngineStateDataLoader`] dependency, this loader operates at an
/// abstraction layer higher than the rest of the Engine State API (i.e. it interprets the data that
/// can be read using other, lower-level means).
pub struct ObjectMetadataLoader<'s, S: SubstateDatabase> {
    pub loader: EngineStateDataLoader<'s, S>,
}

impl<'s, S: SubstateDatabase> ObjectMetadataLoader<'s, S> {
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
            .map(MetadataKey::from))
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
        decode_latest::<VersionedMetadataEntry>(entry_data)
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
    NotAnObject,
    AttachedModulesInvariantBroken(String),
}

impl From<String> for MetadataKey {
    fn from(string: String) -> Self {
        Self { string }
    }
}

impl<'t> From<SborCollectionKey<'t>> for MetadataKey {
    fn from(key: SborCollectionKey<'t>) -> Self {
        Self::from(decode_kv_collection_key::<String>(key))
    }
}

impl<'t> From<SborCollectionKey<'t>> for ModuleRoleKey {
    fn from(key: SborCollectionKey<'t>) -> Self {
        decode_kv_collection_key(key)
    }
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
            AttachedModuleBrowsingError::NotAnObject => ResponseError::new(
                StatusCode::BAD_REQUEST,
                "The requested Entity is not an Object",
            )
            .with_public_details(models::ErrorDetails::RequestedItemInvalidDetails {
                item_type: models::RequestedItemType::Entity,
            }),
            AttachedModuleBrowsingError::AttachedModulesInvariantBroken(message) => {
                ResponseError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid Engine state: {}", message),
                )
            }
        }
    }
}

fn decode_kv_collection_key<D: ScryptoDecode>(key: SborCollectionKey) -> D {
    let SborCollectionKey::KeyValueStore(sbor_data) = key else {
        panic!("expected the Key-Value collection key; got {:?}", key)
    };
    let Ok(decoded) = scrypto_decode(sbor_data.as_bytes()) else {
        panic!(
            "expected the collection key to be a {}; got {:?}",
            type_name::<D>(),
            sbor_data
        )
    };
    decoded
}

fn decode_latest<V: Versioned + ScryptoDecode>(
    entry_data: SborData,
) -> Result<V::LatestVersion, AttachedModuleBrowsingError> {
    Ok(scrypto_decode::<V>(entry_data.as_bytes())
        .map_err(AttachedModuleBrowsingError::SborDecode)?
        .fully_update_and_into_latest_version())
}

fn create_object_collection_meta<K: ScryptoDescribe, V: ScryptoDescribe>(
    descriptor: impl CollectionDescriptor,
) -> ObjectCollectionMeta {
    ObjectCollectionMeta {
        index: RichIndex::of(descriptor.collection_index() as usize),
        kind: ObjectCollectionKind::KeyValueStore,
        resolved_key_type: create_type_meta::<K>(),
        resolved_value_type: create_type_meta::<V>(),
    }
}

fn create_object_field_meta<T: ScryptoDescribe>(
    descriptor: impl FieldDescriptor,
) -> ObjectFieldMeta {
    ObjectFieldMeta {
        index: RichIndex::of(descriptor.field_index() as usize),
        resolved_type: create_type_meta::<T>(),
        transience_default_value_bytes: None, // field transience is irrelevant in our use-case
    }
}

fn create_type_meta<T: ScryptoDescribe>() -> ResolvedTypeMeta {
    let (local_type_id, versioned_schema) =
        generate_full_schema_from_single_type::<T, ScryptoCustomSchema>();
    ResolvedTypeMeta {
        type_reference: match local_type_id {
            LocalTypeId::WellKnown(well_known_type_id) => {
                ResolvedTypeReference::WellKnown(well_known_type_id)
            }
            LocalTypeId::SchemaLocalIndex(index) => {
                // Create a dummy reference: required by the `SchemaBasedTypeReference` helper type, but not
                // actually used (since we have the actual `SchemaV1` instance and type index).
                let schema_reference = SchemaReference {
                    node_id: NodeId::new(
                        EntityType::GlobalPackage as u8,
                        &[0u8; NodeId::RID_LENGTH],
                    ),
                    schema_hash: SchemaHash(Hash::from_bytes([0; Hash::LENGTH])),
                };
                ResolvedTypeReference::SchemaBased(SchemaBasedTypeReference {
                    schema_reference,
                    index,
                })
            }
        },
        schema: versioned_schema.fully_update_and_into_latest_version(),
    }
}
