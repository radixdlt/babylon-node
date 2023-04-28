use std::convert::TryFrom;
use std::str::FromStr;

use crate::core_api::*;

use crate::core_api::models;
use crate::core_api::models::ModuleType;
use models::{SubstateKeyType, SubstateType};
use radix_engine::types::{
    ClockOffset, ComponentAddress, ComponentOffset, EpochManagerOffset, PackageAddress,
    PackageOffset, ResourceAddress,
};
use radix_engine_common::types::{EntityType, GlobalAddress, ModuleId, NodeId, SubstateKey};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::data::scrypto::model::NonFungibleLocalId;
use radix_engine_interface::types::{
    AccessControllerOffset, AccessRulesOffset, AccountOffset, FungibleResourceManagerOffset,
    FungibleVaultOffset, NonFungibleResourceManagerOffset, NonFungibleVaultOffset, RoyaltyOffset,
    TypeInfoOffset, ValidatorOffset,
};
use radix_engine_queries::typed_substate_layout::to_typed_substate_key;
use radix_engine_queries::typed_substate_layout::{TypedObjectModuleSubstateKey, TypedSubstateKey};

pub fn to_api_component_address(
    context: &MappingContext,
    component_address: &ComponentAddress,
) -> String {
    context
        .bech32_encoder
        .encode(component_address.as_ref())
        .unwrap()
}

pub fn to_api_resource_address(
    context: &MappingContext,
    resource_address: &ResourceAddress,
) -> String {
    context
        .bech32_encoder
        .encode(resource_address.as_ref())
        .unwrap()
}

pub fn to_api_package_address(
    context: &MappingContext,
    package_address: &PackageAddress,
) -> String {
    context
        .bech32_encoder
        .encode(package_address.as_ref())
        .unwrap()
}

pub fn to_api_global_address(context: &MappingContext, global_address: &GlobalAddress) -> String {
    context
        .bech32_encoder
        .encode(global_address.as_ref())
        .unwrap()
}

pub fn to_api_entity_reference(node_id: NodeId) -> Result<models::EntityReference, MappingError> {
    let mapped = MappedEntityId::try_from(node_id)?;
    Ok(mapped.into())
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_id(
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Result<models::SubstateId, MappingError> {
    let mapped = to_mapped_substate_id(node_id, module_id, substate_key)?;
    Ok(mapped.into())
}

#[derive(Debug)]
pub struct MappedEntityId {
    entity_type: models::EntityType,
    entity_id_bytes: Vec<u8>,
}

impl From<MappedEntityId> for models::EntityReference {
    fn from(mapped_entity_id: MappedEntityId) -> Self {
        models::EntityReference {
            entity_type: mapped_entity_id.entity_type,
            entity_id_hex: to_hex(mapped_entity_id.entity_id_bytes),
        }
    }
}

impl TryFrom<NodeId> for MappedEntityId {
    type Error = MappingError;

    fn try_from(node_id: NodeId) -> Result<MappedEntityId, MappingError> {
        let entity_id_bytes = node_id_to_entity_id_bytes(&node_id);
        let entity_type = to_api_entity_type(node_id.entity_type().unwrap()); // TODO: handle error
        Ok(MappedEntityId {
            entity_type,
            entity_id_bytes,
        })
    }
}

pub fn to_api_entity_type(entity_type: EntityType) -> models::EntityType {
    match entity_type {
        EntityType::GlobalPackage => models::EntityType::Package,
        EntityType::GlobalFungibleResource => models::EntityType::FungibleResource,
        EntityType::GlobalNonFungibleResource => models::EntityType::NonFungibleResource,
        EntityType::GlobalEpochManager => models::EntityType::EpochManager,
        EntityType::GlobalValidator => models::EntityType::Validator,
        EntityType::GlobalClock => models::EntityType::Clock,
        EntityType::GlobalAccessController => models::EntityType::AccessController,
        EntityType::GlobalAccount => models::EntityType::Account,
        EntityType::GlobalIdentity => models::EntityType::Identity,
        EntityType::GlobalGenericComponent => models::EntityType::NormalComponent,
        EntityType::GlobalVirtualEcdsaAccount => models::EntityType::Account,
        EntityType::GlobalVirtualEddsaAccount => models::EntityType::Account,
        EntityType::GlobalVirtualEcdsaIdentity => models::EntityType::Identity,
        EntityType::GlobalVirtualEddsaIdentity => models::EntityType::Identity,
        EntityType::InternalFungibleVault => models::EntityType::Vault, // TODO: separate fungible/non-fungible
        EntityType::InternalNonFungibleVault => models::EntityType::Vault,
        EntityType::InternalAccount => models::EntityType::Account, // TODO: fixme
        EntityType::InternalKeyValueStore => models::EntityType::KeyValueStore,
        EntityType::InternalIndex => models::EntityType::Package, // TODO: fixme
        EntityType::InternalSortedIndex => models::EntityType::Package, // TODO: fixme
        EntityType::InternalGenericComponent => models::EntityType::NormalComponent,
    }
}

#[derive(Debug)]
pub struct MappedSubstateId(
    models::EntityType,
    Vec<u8>,
    ModuleType,
    SubstateType,
    SubstateKeyType,
    Vec<u8>,
);

impl From<MappedSubstateId> for models::SubstateId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::SubstateId {
            entity_type: mapped_substate_id.0,
            entity_id_hex: to_hex(mapped_substate_id.1),
            module_type: mapped_substate_id.2,
            substate_type: mapped_substate_id.3,
            substate_key_type: mapped_substate_id.4,
            substate_key_hex: to_hex(mapped_substate_id.5),
        }
    }
}

impl From<MappedSubstateId> for MappedEntityId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        MappedEntityId {
            entity_type: mapped_substate_id.0,
            entity_id_bytes: mapped_substate_id.1,
        }
    }
}

impl From<MappedSubstateId> for models::EntityReference {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::EntityReference {
            entity_type: mapped_substate_id.0,
            entity_id_hex: to_hex(mapped_substate_id.1),
        }
    }
}

fn to_mapped_substate_id(
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Result<MappedSubstateId, MappingError> {
    let entity_type = node_id.entity_type().unwrap(); // TODO: handle error
    let typed_substate_key = to_typed_substate_key(entity_type, module_id, substate_key).unwrap();

    // TODO: check value_is_mappable ?

    let entity_id_bytes = node_id_to_entity_id_bytes(node_id);
    let module_type = to_api_module_type(&typed_substate_key);
    let substate_key_bytes = vec![]; // TODO: fixme   substate_offset_to_substate_key_bytes(&substate_id.2)?;

    // TODO: fix all this...
    let (substate_type, substate_key_type) = match typed_substate_key {
        TypedSubstateKey::TypeInfoModule(TypeInfoOffset::TypeInfo) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::AccessRulesModule(AccessRulesOffset::AccessRules) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::RoyaltyModule(RoyaltyOffset::RoyaltyConfig) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::RoyaltyModule(RoyaltyOffset::RoyaltyAccumulator) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::MetadataModule(_String) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Info,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::CodeType,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Code,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Royalty,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::FunctionAccessRules,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleResource(
            FungibleResourceManagerOffset::ResourceManager,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleResource(
            NonFungibleResourceManagerOffset::ResourceManager,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::Divisibility,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::LiquidFungible,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::LockedFungible,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::IdType,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::LiquidNonFungible,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::LockedNonFungible,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::EpochManager,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::CurrentValidatorSet,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::RegisteredValidatorSet,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Clock(
            ClockOffset::CurrentTimeRoundedToMinutes,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Validator(
            ValidatorOffset::Validator,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Account(
            AccountOffset::Account,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::AccessController(
            AccessControllerOffset::AccessController,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericScryptoComponent(
            ComponentOffset::State0,
        )) => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericKeyValueStore(_)) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericIndex(_)) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericSortedU16Index(_)) => {
            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
        }
    };

    Ok(MappedSubstateId(
        to_api_entity_type(entity_type),
        entity_id_bytes,
        module_type,
        substate_type,
        substate_key_type,
        substate_key_bytes,
    ))
}

pub fn to_global_entity_reference(
    context: &MappingContext,
    global_address: &GlobalAddress,
) -> Result<models::GlobalEntityReference, MappingError> {
    let reference = models::GlobalEntityReference {
        entity_reference: Box::new(to_api_entity_reference(*global_address.as_node_id())?),
        global_address_hex: to_hex(global_address.to_vec()),
        global_address: to_api_global_address(context, global_address),
    };

    Ok(reference)
}

pub fn extract_package_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<PackageAddress, ExtractionError> {
    PackageAddress::try_from_bech32(&extraction_context.bech32_decoder, package_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_component_address(
    extraction_context: &ExtractionContext,
    component_address: &str,
) -> Result<ComponentAddress, ExtractionError> {
    ComponentAddress::try_from_bech32(&extraction_context.bech32_decoder, component_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_resource_address(
    extraction_context: &ExtractionContext,
    resource_address: &str,
) -> Result<ResourceAddress, ExtractionError> {
    ResourceAddress::try_from_bech32(&extraction_context.bech32_decoder, resource_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_non_fungible_id_from_simple_representation(
    simple_rep: &str,
) -> Result<NonFungibleLocalId, ExtractionError> {
    Ok(NonFungibleLocalId::from_str(simple_rep)?)
}

pub fn node_id_to_entity_id_bytes(node_id: &NodeId) -> Vec<u8> {
    // TODO: take ENTITY_ID_LENGTH
    vec![node_id.0[0]]
}

pub fn to_api_module_type(typed_substate_key: &TypedSubstateKey) -> ModuleType {
    match typed_substate_key {
        TypedSubstateKey::TypeInfoModule(_) => ModuleType::TypeInfo,
        TypedSubstateKey::AccessRulesModule(_) => ModuleType::AccessRules,
        TypedSubstateKey::RoyaltyModule(_) => ModuleType::ComponentRoyalty,
        TypedSubstateKey::MetadataModule(_) => ModuleType::Metadata,
        TypedSubstateKey::ObjectModule(_) => ModuleType::_Self, // TODO: update api module types
    }
}

pub fn to_api_module_type_from_obj(object_module_id: &ObjectModuleId) -> ModuleType {
    match object_module_id {
        ObjectModuleId::SELF => ModuleType::_Self,
        ObjectModuleId::Metadata => ModuleType::Metadata,
        ObjectModuleId::Royalty => ModuleType::ComponentRoyalty,
        ObjectModuleId::AccessRules => ModuleType::AccessRules,
    }
}
