use std::convert::TryFrom;
use std::str::FromStr;

use crate::core_api::*;

use crate::core_api::models::ModuleType;
use models::{EntityType, SubstateKeyType, SubstateType};
use radix_engine::types::{
    scrypto_encode, AccessRulesChainOffset, ClockOffset, ComponentAddress, ComponentOffset,
    EpochManagerOffset, KeyValueStoreOffset, MetadataOffset, NonFungibleStoreOffset,
    PackageAddress, PackageOffset, RENodeId, ResourceAddress, ResourceManagerOffset, SubstateId,
    SubstateOffset, VaultOffset,
};
use radix_engine_interface::api::types::{
    AccessControllerOffset, AccountOffset, NodeModuleId, RoyaltyOffset, TypeInfoOffset,
    ValidatorOffset,
};
use radix_engine_interface::blueprints::resource::{NonFungibleIdType, NonFungibleLocalId};
use radix_engine_interface::data::model::Address;

pub fn to_api_component_address(
    context: &MappingContext,
    component_address: &ComponentAddress,
) -> String {
    context
        .bech32_encoder
        .encode_component_address_to_string(component_address)
}

pub fn to_api_resource_address(
    context: &MappingContext,
    resource_address: &ResourceAddress,
) -> String {
    context
        .bech32_encoder
        .encode_resource_address_to_string(resource_address)
}

pub fn to_api_package_address(
    context: &MappingContext,
    package_address: &PackageAddress,
) -> String {
    context
        .bech32_encoder
        .encode_package_address_to_string(package_address)
}

pub fn to_api_address(context: &MappingContext, address: &Address) -> String {
    match address {
        Address::Component(addr) => to_api_component_address(context, addr),
        Address::Package(addr) => to_api_package_address(context, addr),
        Address::Resource(addr) => to_api_resource_address(context, addr),
    }
}

pub fn get_entity_type_from_global_address(global_address: &Address) -> models::EntityType {
    match global_address {
        Address::Component(component) => match component {
            // Scrypto Components get mapped to EntityType::Component for now
            ComponentAddress::Normal(_) => models::EntityType::Component,
            ComponentAddress::Account(_) => models::EntityType::Component,
            ComponentAddress::EcdsaSecp256k1VirtualAccount(_) => models::EntityType::Component,
            ComponentAddress::EddsaEd25519VirtualAccount(_) => models::EntityType::Component,
            // Native Components get mapped to their own EntityType for now - but this will change when we have native packages
            ComponentAddress::EpochManager(_) => models::EntityType::EpochManager,
            ComponentAddress::Clock(_) => models::EntityType::Clock,
            ComponentAddress::Validator(_) => models::EntityType::Validator,
            ComponentAddress::AccessController(_) => models::EntityType::AccessController,
            ComponentAddress::Identity(_) => models::EntityType::Identity,
            ComponentAddress::EcdsaSecp256k1VirtualIdentity(_) => models::EntityType::Identity,
            ComponentAddress::EddsaEd25519VirtualIdentity(_) => models::EntityType::Identity,
        },
        Address::Package(_) => models::EntityType::Package,
        Address::Resource(_) => models::EntityType::ResourceManager,
    }
}

pub fn to_api_entity_reference(node_id: RENodeId) -> Result<models::EntityReference, MappingError> {
    let mapped = MappedEntityId::try_from(node_id)?;

    Ok(mapped.into())
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_id(substate_id: SubstateId) -> Result<models::SubstateId, MappingError> {
    let mapped = to_mapped_substate_id(substate_id)?;

    Ok(mapped.into())
}

#[derive(Debug)]
pub struct MappedEntityId {
    entity_type: EntityType,
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

impl TryFrom<RENodeId> for MappedEntityId {
    fn try_from(re_node_id: RENodeId) -> Result<MappedEntityId, MappingError> {
        // Helper function
        fn transient_renode_error(name: &'static str) -> MappingError {
            MappingError::TransientRENodePersisted {
                message: format!("{} persisted", name),
            }
        }

        // Start body of method
        let entity_id_bytes = re_node_id_to_entity_id_bytes(&re_node_id)?;
        let entity_type =
            match re_node_id {
                // Gateway understands "Component" to be "Component with Scrypto Package" for now. This will change when we have Native Packages
                RENodeId::GlobalComponent(ComponentAddress::Normal(..))
                | RENodeId::Component(_) => EntityType::Component,
                RENodeId::GlobalPackage(_) => EntityType::Package,
                RENodeId::GlobalResourceManager(_) => EntityType::ResourceManager,
                // Native Components
                RENodeId::GlobalComponent(ComponentAddress::EpochManager(..))
                | RENodeId::EpochManager(_) => EntityType::EpochManager,
                RENodeId::GlobalComponent(ComponentAddress::Clock(..)) | RENodeId::Clock(_) => {
                    EntityType::Clock
                }
                RENodeId::GlobalComponent(ComponentAddress::Validator(..))
                | RENodeId::Validator(_) => EntityType::Validator,
                RENodeId::GlobalComponent(ComponentAddress::AccessController(..))
                | RENodeId::AccessController(_) => EntityType::AccessController,
                RENodeId::GlobalComponent(ComponentAddress::Identity(..))
                | RENodeId::GlobalComponent(ComponentAddress::EcdsaSecp256k1VirtualIdentity(..))
                | RENodeId::GlobalComponent(ComponentAddress::EddsaEd25519VirtualIdentity(..))
                | RENodeId::Identity(_) => EntityType::Identity,
                RENodeId::KeyValueStore(_) => EntityType::KeyValueStore,
                RENodeId::NonFungibleStore(_) => EntityType::NonFungibleStore,
                RENodeId::Vault(_) => EntityType::Vault,
                RENodeId::GlobalComponent(ComponentAddress::Account(..))
                | RENodeId::GlobalComponent(ComponentAddress::EcdsaSecp256k1VirtualAccount(..))
                | RENodeId::GlobalComponent(ComponentAddress::EddsaEd25519VirtualAccount(..))
                | RENodeId::Account(_) => EntityType::Account,
                RENodeId::Bucket(_) => return Err(transient_renode_error("Bucket")),
                RENodeId::Proof(_) => return Err(transient_renode_error("Proof")),
                RENodeId::Worktop => return Err(transient_renode_error("Worktop")),
                RENodeId::AuthZoneStack => return Err(transient_renode_error("AuthZoneStack")),
                RENodeId::TransactionRuntime => {
                    return Err(transient_renode_error("TransactionRuntime"))
                }
            };
        Ok(MappedEntityId {
            entity_type,
            entity_id_bytes,
        })
    }

    type Error = MappingError;
}

#[derive(Debug)]
pub struct MappedSubstateId(
    EntityType,
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

#[tracing::instrument(skip_all)]
fn to_mapped_substate_id(substate_id: SubstateId) -> Result<MappedSubstateId, MappingError> {
    // Helper methods
    fn unknown_substate_error(renode_name: &'static str, substate_id: &SubstateId) -> MappingError {
        MappingError::UnsupportedSubstatePersisted {
            message: format!(
                "Unsupported substate persisted for {} RENode: {:?}",
                renode_name, substate_id
            ),
        }
    }
    fn transient_substate_error(
        renode_name: &'static str,
        substate_id: &SubstateId,
    ) -> MappingError {
        MappingError::TransientSubstatePersisted {
            message: format!(
                "Transient substate persisted for {} RENode: {:?}",
                renode_name, substate_id
            ),
        }
    }

    // Start body of method
    let entity_id_bytes = re_node_id_to_entity_id_bytes(&substate_id.0)?;
    let module_type = node_module_id_to_module_type(&substate_id.1);
    let substate_key_bytes = substate_offset_to_substate_key_bytes(&substate_id.2)?;

    // In the below, we nest match statements to ensure we get as much help from the compiler as possible to ensure
    //   we capture all possible substate types at compile time...
    // We can't capture new offset types under an RENode though - check nodes.rs after each merge to check we're not missing any
    let (entity_type, substate_type_key) = match &substate_id {
        SubstateId(
            RENodeId::GlobalComponent(ComponentAddress::Normal(..)) | RENodeId::Component(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Component(offset) => match offset {
                    ComponentOffset::State0 => (
                        SubstateType::ComponentState,
                        SubstateKeyType::ComponentState,
                    ),
                },
                SubstateOffset::Royalty(offset) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("Component", &substate_id)),
            };
            (EntityType::Component, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(
                ComponentAddress::Account(..)
                | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                | ComponentAddress::EddsaEd25519VirtualAccount(..),
            )
            | RENodeId::Account(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Account(offset) => match offset {
                    AccountOffset::Account => (SubstateType::Account, SubstateKeyType::Account),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("Account", &substate_id)),
            };
            (EntityType::Account, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(ComponentAddress::AccessController(..))
            | RENodeId::AccessController(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::AccessController(offset) => match offset {
                    AccessControllerOffset::AccessController => (
                        SubstateType::AccessController,
                        SubstateKeyType::AccessController,
                    ),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("AccessController", &substate_id)),
            };
            (EntityType::AccessController, substate_type_key)
        }

        SubstateId(RENodeId::GlobalPackage(_), _, offset) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Package(offset) => match offset {
                    PackageOffset::Info => {
                        (SubstateType::PackageInfo, SubstateKeyType::PackageInfo)
                    }
                    PackageOffset::Code => {
                        (SubstateType::PackageCode, SubstateKeyType::PackageCode)
                    }
                    PackageOffset::CodeType => (
                        SubstateType::PackageCodeType,
                        SubstateKeyType::PackageCodeType,
                    ),
                },
                SubstateOffset::Royalty(offset) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::PackageRoyaltyConfig,
                        SubstateKeyType::PackageRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::PackageRoyaltyAccumulator,
                        SubstateKeyType::PackageRoyaltyAccumulator,
                    ),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                SubstateOffset::PackageAccessRules => (
                    SubstateType::FunctionAccessRules,
                    SubstateKeyType::FunctionAccessRules,
                ),
                _ => return Err(unknown_substate_error("Package", &substate_id)),
            };
            (EntityType::Package, substate_type_key)
        }

        SubstateId(RENodeId::GlobalResourceManager(_), _, offset) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::ResourceManager(offset) => match offset {
                    ResourceManagerOffset::ResourceManager => (
                        SubstateType::ResourceManager,
                        SubstateKeyType::ResourceManager,
                    ),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("ResourceManager", &substate_id)),
            };
            (EntityType::ResourceManager, substate_type_key)
        }

        SubstateId(RENodeId::NonFungibleStore(_), _, offset) => {
            let substate_type_key = match offset {
                SubstateOffset::NonFungibleStore(offset) => match offset {
                    NonFungibleStoreOffset::Entry(_) => (
                        SubstateType::NonFungibleStoreEntry,
                        SubstateKeyType::NonFungibleStoreEntry,
                    ),
                },
                _ => return Err(unknown_substate_error("NonFungibleStore", &substate_id)),
            };
            (EntityType::NonFungibleStore, substate_type_key)
        }

        SubstateId(RENodeId::KeyValueStore(_), _, offset) => {
            let substate_type_key = match offset {
                SubstateOffset::KeyValueStore(offset) => match offset {
                    KeyValueStoreOffset::Entry(_) => (
                        SubstateType::KeyValueStoreEntry,
                        SubstateKeyType::KeyValueStoreEntry,
                    ),
                },
                _ => return Err(unknown_substate_error("KeyValueStore", &substate_id)),
            };
            (EntityType::KeyValueStore, substate_type_key)
        }

        SubstateId(RENodeId::Vault(_), _, offset) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Vault(offset) => match offset {
                    VaultOffset::Info => (SubstateType::VaultInfo, SubstateKeyType::VaultInfo),
                    VaultOffset::LiquidNonFungible | VaultOffset::LockedNonFungible => (
                        SubstateType::VaultNonFungible,
                        SubstateKeyType::VaultNonFungible,
                    ),
                    VaultOffset::LiquidFungible | VaultOffset::LockedFungible => {
                        (SubstateType::VaultFungible, SubstateKeyType::VaultFungible)
                    }
                },
                _ => return Err(unknown_substate_error("Vault", &substate_id)),
            };
            (EntityType::Vault, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(ComponentAddress::EpochManager(..))
            | RENodeId::EpochManager(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::EpochManager(offset) => match offset {
                    EpochManagerOffset::EpochManager => {
                        (SubstateType::EpochManager, SubstateKeyType::EpochManager)
                    }
                    EpochManagerOffset::CurrentValidatorSet => (
                        SubstateType::EpochManager,
                        SubstateKeyType::CurrentValidatorSet,
                    ),
                    EpochManagerOffset::PreparingValidatorSet => (
                        SubstateType::EpochManager,
                        SubstateKeyType::PreparingValidatorSet,
                    ),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("EpochManager", &substate_id)),
            };
            (EntityType::EpochManager, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(ComponentAddress::Validator(..)) | RENodeId::Validator(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Validator(offset) => match offset {
                    ValidatorOffset::Validator => {
                        (SubstateType::Validator, SubstateKeyType::Validator)
                    }
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("Validator", &substate_id)),
            };
            (EntityType::Validator, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(ComponentAddress::Clock(..)) | RENodeId::Clock(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Clock(offset) => match offset {
                    ClockOffset::CurrentTimeRoundedToMinutes => (
                        SubstateType::ClockCurrentMinute,
                        SubstateKeyType::ClockCurrentMinute,
                    ),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("Clock", &substate_id)),
            };
            (EntityType::Clock, substate_type_key)
        }

        SubstateId(
            RENodeId::GlobalComponent(
                ComponentAddress::Identity(..)
                | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                | ComponentAddress::EddsaEd25519VirtualIdentity(..),
            )
            | RENodeId::Identity(_),
            _,
            offset,
        ) => {
            let substate_type_key = match offset {
                SubstateOffset::TypeInfo(offset) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                SubstateOffset::Metadata(offset) => match offset {
                    MetadataOffset::Metadata => (SubstateType::Metadata, SubstateKeyType::Metadata),
                },
                SubstateOffset::AccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::AccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("Identity", &substate_id)),
            };
            (EntityType::Identity, substate_type_key)
        }

        // TRANSIENT SUBSTATES
        SubstateId(RENodeId::Bucket(..), ..) => {
            return Err(transient_substate_error("Bucket", &substate_id))
        }
        SubstateId(RENodeId::Proof(..), ..) => {
            return Err(transient_substate_error("Proof", &substate_id))
        }
        SubstateId(RENodeId::Worktop, ..) => {
            return Err(transient_substate_error("Worktop", &substate_id))
        }
        SubstateId(RENodeId::AuthZoneStack, ..) => {
            return Err(transient_substate_error("AuthZoneStack", &substate_id))
        }
        SubstateId(RENodeId::TransactionRuntime, ..) => {
            return Err(transient_substate_error("TransactionRuntime", &substate_id))
        }
    };

    Ok(MappedSubstateId(
        entity_type,
        entity_id_bytes,
        module_type,
        substate_type_key.0,
        substate_type_key.1,
        substate_key_bytes,
    ))
}

pub fn to_global_entity_reference(
    context: &MappingContext,
    global_address: &Address,
) -> models::GlobalEntityReference {
    models::GlobalEntityReference {
        entity_type: get_entity_type_from_global_address(global_address),
        global_address_hex: to_hex(global_address_to_vec(global_address)),
        global_address: to_api_address(context, global_address),
    }
}

pub fn extract_package_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<PackageAddress, ExtractionError> {
    extraction_context
        .bech32_decoder
        .validate_and_decode_package_address(package_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_component_address(
    extraction_context: &ExtractionContext,
    component_address: &str,
) -> Result<ComponentAddress, ExtractionError> {
    extraction_context
        .bech32_decoder
        .validate_and_decode_component_address(component_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_resource_address(
    extraction_context: &ExtractionContext,
    resource_address: &str,
) -> Result<ResourceAddress, ExtractionError> {
    extraction_context
        .bech32_decoder
        .validate_and_decode_resource_address(resource_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_non_fungible_id_from_simple_representation(
    _id_type: NonFungibleIdType,
    simple_rep: &str,
) -> Result<NonFungibleLocalId, ExtractionError> {
    let non_fungible_local_id = NonFungibleLocalId::from_str(simple_rep)?;
    Ok(non_fungible_local_id)
}

pub fn re_node_id_to_entity_id_bytes(re_node_id: &RENodeId) -> Result<Vec<u8>, MappingError> {
    scrypto_encode(re_node_id).map_err(|err| MappingError::SborEncodeError {
        encode_error: err,
        message: "Could not encode re node id".to_string(),
    })
}

pub fn node_module_id_to_module_type(node_module_id: &NodeModuleId) -> ModuleType {
    match node_module_id {
        NodeModuleId::SELF => ModuleType::_Self,
        NodeModuleId::Metadata => ModuleType::Metadata,
        NodeModuleId::AccessRules => ModuleType::AccessRules,
        NodeModuleId::AccessRules1 => ModuleType::AccessRules1,
        NodeModuleId::FunctionAccessRules => ModuleType::FunctionAccessRules,
        NodeModuleId::ComponentRoyalty => ModuleType::ComponentRoyalty,
        NodeModuleId::PackageRoyalty => ModuleType::PackageRoyalty,
        NodeModuleId::TypeInfo => ModuleType::TypeInfo,
    }
}

pub fn substate_offset_to_substate_key_bytes(
    substate_offset: &SubstateOffset,
) -> Result<Vec<u8>, MappingError> {
    scrypto_encode(substate_offset).map_err(|err| MappingError::SborEncodeError {
        encode_error: err,
        message: "Could not encode substate offset".to_string(),
    })
}

pub fn global_address_to_vec(global_address: &Address) -> Vec<u8> {
    match global_address {
        Address::Package(package_addr) => package_addr.to_vec(),
        Address::Resource(resource_addr) => resource_addr.to_vec(),
        Address::Component(component_addr) => component_addr.to_vec(),
    }
}
