use std::convert::TryFrom;
use std::str::FromStr;

use crate::core_api::*;

use crate::core_api::models;
use models::SubstateType;
use radix_engine::types::{
    ClockOffset, ComponentAddress, ComponentOffset, EpochManagerOffset, PackageAddress,
    PackageOffset, ResourceAddress,
};
use radix_engine_common::data::scrypto::scrypto_encode;
use radix_engine_common::types::{EntityType, GlobalAddress, ModuleId, NodeId, SubstateKey};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::data::scrypto::model::NonFungibleLocalId;
use radix_engine_interface::types::{
    AccessControllerOffset, AccessRulesOffset, AccountOffset, FungibleResourceManagerOffset,
    FungibleVaultOffset, NonFungibleResourceManagerOffset, NonFungibleVaultOffset, RoyaltyOffset,
    SysModuleId, TypeInfoOffset, ValidatorOffset,
};
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
    typed_substate_key: &TypedSubstateKey,
) -> Result<models::SubstateId, MappingError> {
    let mapped = to_mapped_substate_id(node_id, module_id, substate_key, typed_substate_key)?;
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
        let entity_type =
            to_api_entity_type(node_id.entity_type().ok_or(MappingError::EntityTypeError)?);
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
        EntityType::InternalFungibleVault => models::EntityType::FungibleVault,
        EntityType::InternalNonFungibleVault => models::EntityType::NonFungibleVault,
        EntityType::InternalAccount => models::EntityType::Account,
        EntityType::InternalKeyValueStore => models::EntityType::KeyValueStore,
        EntityType::InternalIndex => models::EntityType::Index,
        EntityType::InternalSortedIndex => models::EntityType::SortedIndex,
        EntityType::InternalGenericComponent => models::EntityType::NormalComponent,
    }
}

#[derive(Debug)]
pub struct MappedSubstateId(
    models::EntityType,
    Vec<u8>,
    models::SysModuleType,
    models::SubstateType,
    Vec<u8>,
);

impl From<MappedSubstateId> for models::SubstateId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::SubstateId {
            entity_type: mapped_substate_id.0,
            entity_id_hex: to_hex(mapped_substate_id.1),
            module_type: mapped_substate_id.2,
            substate_type: mapped_substate_id.3,
            substate_key_hex: to_hex(mapped_substate_id.4),
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
    typed_substate_key: &TypedSubstateKey,
) -> Result<MappedSubstateId, MappingError> {
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    let entity_id_bytes = node_id_to_entity_id_bytes(node_id);
    let module_type = to_api_sys_module_type(module_id)?;
    let substate_key_bytes = scrypto_encode(&substate_key).unwrap();

    let substate_type = match typed_substate_key {
        TypedSubstateKey::TypeInfoModule(TypeInfoOffset::TypeInfo) => SubstateType::TypeInfo,
        TypedSubstateKey::AccessRulesModule(AccessRulesOffset::AccessRules) => {
            SubstateType::MethodAccessRules
        }
        TypedSubstateKey::RoyaltyModule(RoyaltyOffset::RoyaltyConfig) => {
            SubstateType::ComponentRoyaltyConfig
        }
        TypedSubstateKey::RoyaltyModule(RoyaltyOffset::RoyaltyAccumulator) => {
            SubstateType::ComponentRoyaltyAccumulator
        }
        TypedSubstateKey::MetadataModule(_) => SubstateType::MetadataValue,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Info,
        )) => SubstateType::PackageInfo,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::CodeType,
        )) => SubstateType::PackageCode,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Code,
        )) => SubstateType::PackageCode,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::Royalty,
        )) => SubstateType::PackageRoyalty,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Package(
            PackageOffset::FunctionAccessRules,
        )) => SubstateType::PackageFunctionAccessRules,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleResource(
            FungibleResourceManagerOffset::Divisibility,
        )) => SubstateType::FungibleResourceManagerDivisibility,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleResource(
            FungibleResourceManagerOffset::TotalSupply,
        )) => SubstateType::FungibleResourceManagerTotalSupply,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleResource(
            NonFungibleResourceManagerOffset::IdType,
        )) => SubstateType::NonFungibleResourceManagerIdType,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleResource(
            NonFungibleResourceManagerOffset::TotalSupply,
        )) => SubstateType::NonFungibleResourceManagerTotalSupply,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleResource(
            NonFungibleResourceManagerOffset::DataSchema,
        )) => SubstateType::NonFungibleResourceManagerDataSchema,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleResource(
            NonFungibleResourceManagerOffset::Data,
        )) => SubstateType::NonFungibleResourceManagerData,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::LiquidFungible,
        )) => SubstateType::FungibleVaultBalance,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::LockedFungible,
        )) => {
            return Err(MappingError::SubstateKeyMappingError {
                entity_type_hex: to_hex(entity_id_bytes),
                module_id: module_id.0,
                substate_key_hex: to_hex(substate_key_bytes),
                message: "LockedFungible".to_string(),
            })
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::LiquidNonFungible,
        )) => SubstateType::NonFungibleVaultBalance,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::LockedNonFungible,
        )) => {
            return Err(MappingError::SubstateKeyMappingError {
                entity_type_hex: to_hex(entity_id_bytes),
                module_id: module_id.0,
                substate_key_hex: to_hex(substate_key_bytes),
                message: "LockedNonFungible".to_string(),
            })
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::Config,
        )) => SubstateType::EpochManagerConfig,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::EpochManager,
        )) => SubstateType::EpochManager,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::CurrentValidatorSet,
        )) => SubstateType::CurrentValidatorSet,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManager(
            EpochManagerOffset::RegisteredValidators,
        )) => SubstateType::RegisteredValidators,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Clock(
            ClockOffset::CurrentTimeRoundedToMinutes,
        )) => SubstateType::Clock,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Validator(
            ValidatorOffset::Validator,
        )) => SubstateType::Validator,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::Account(
            AccountOffset::Account,
        )) => SubstateType::Account,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::AccessController(
            AccessControllerOffset::AccessController,
        )) => SubstateType::AccessController,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericScryptoComponent(
            ComponentOffset::State0,
        )) => SubstateType::GenericScryptoComponentState,
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericKeyValueStore(_)) => {
            SubstateType::GenericKeyValueStore
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericIndex(_)) => {
            SubstateType::GenericIndex
        }
        TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericSortedU16Index(_)) => {
            SubstateType::GenericSortedU16Index
        }
    };

    Ok(MappedSubstateId(
        to_api_entity_type(entity_type),
        entity_id_bytes,
        module_type,
        substate_type,
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

pub fn extract_global_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<GlobalAddress, ExtractionError> {
    GlobalAddress::try_from_bech32(&extraction_context.bech32_decoder, package_address)
        .ok_or(ExtractionError::InvalidAddress)
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
    node_id.0[0..NodeId::ENTITY_ID_LENGTH].to_vec()
}

pub fn to_api_sys_module_type(module_id: ModuleId) -> Result<models::SysModuleType, MappingError> {
    let sys_module_id =
        SysModuleId::try_from(module_id).map_err(|_| MappingError::ModuleTypeError {
            message: format!("Could not convert SysModuleId {:?}", module_id),
        })?;

    Ok(match sys_module_id {
        SysModuleId::TypeInfo => models::SysModuleType::TypeInfo,
        SysModuleId::Metadata => models::SysModuleType::Metadata,
        SysModuleId::Royalty => models::SysModuleType::Royalty,
        SysModuleId::AccessRules => models::SysModuleType::AccessRules,
        SysModuleId::Object => models::SysModuleType::Object,
        SysModuleId::Virtualized => models::SysModuleType::Virtualized,
    })
}

pub fn to_api_object_module_type(object_module_id: &ObjectModuleId) -> models::ObjectModuleType {
    match object_module_id {
        ObjectModuleId::SELF => models::ObjectModuleType::_Self,
        ObjectModuleId::Metadata => models::ObjectModuleType::Metadata,
        ObjectModuleId::Royalty => models::ObjectModuleType::Royalty,
        ObjectModuleId::AccessRules => models::ObjectModuleType::AccessRules,
    }
}
