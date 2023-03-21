use std::convert::TryFrom;
use std::str::FromStr;

use crate::core_api::*;

use crate::core_api::models::ModuleType;
use models::{EntityType, SubstateKeyType, SubstateType};
use radix_engine::types::{
    scrypto_encode, ClockOffset, ComponentAddress, ComponentOffset, EpochManagerOffset,
    KeyValueStoreOffset, PackageAddress, PackageOffset, RENodeId, ResourceAddress,
    ResourceManagerOffset, SubstateId, SubstateOffset, VaultOffset,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::data::scrypto::model::{
    Address, NonFungibleIdType, NonFungibleLocalId,
};

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
                message: format!("{name} persisted"),
            }
        }

        // Start body of method
        let entity_id_bytes = re_node_id_to_entity_id_bytes(&re_node_id);
        let entity_type = match re_node_id {
            // Gateway understands "Component" to be "Component with Scrypto Package" for now. This will change when we have Native Packages
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Normal(..))) => {
                EntityType::NormalComponent
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::EpochManager(..))) => {
                EntityType::EpochManager
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Clock(..))) => {
                EntityType::Clock
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Validator(..))) => {
                EntityType::Validator
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::AccessController(..))) => {
                EntityType::AccessController
            }
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Identity(..)
                | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                | ComponentAddress::EddsaEd25519VirtualIdentity(..),
            )) => EntityType::Identity,
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Account(..)
                | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                | ComponentAddress::EddsaEd25519VirtualAccount(..),
            )) => EntityType::Account,
            RENodeId::GlobalObject(Address::Package(PackageAddress::Normal(..))) => {
                EntityType::Package
            }
            RENodeId::GlobalObject(Address::Resource(ResourceAddress::Fungible(..))) => {
                EntityType::FungibleResource
            }
            RENodeId::GlobalObject(Address::Resource(ResourceAddress::NonFungible(..))) => {
                EntityType::NonFungibleResource
            }
            RENodeId::KeyValueStore(_) => EntityType::KeyValueStore,
            RENodeId::Object([INTERNAL_OBJECT_VAULT_ID, ..]) => EntityType::Vault,
            RENodeId::Object([INTERNAL_OBJECT_NORMAL_COMPONENT_ID, ..]) => {
                EntityType::NormalComponent
            }
            RENodeId::Object([INTERNAL_KV_STORE_ID, ..]) => EntityType::KeyValueStore,
            RENodeId::Object(addr) => {
                return Err(MappingError::UnknownNodeTypePersisted {
                    message: format!("Unknown object RENode address type persisted: {:?}", addr),
                })
            }
            RENodeId::AuthZoneStack => return Err(transient_renode_error("AuthZoneStack")),
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

fn to_mapped_substate_id(substate_id: SubstateId) -> Result<MappedSubstateId, MappingError> {
    // Helper methods
    fn unknown_substate_error(renode_name: &'static str, substate_id: &SubstateId) -> MappingError {
        MappingError::UnsupportedSubstatePersisted {
            message: format!(
                "Unsupported substate persisted for {renode_name} RENode: {substate_id:?}"
            ),
        }
    }
    fn transient_substate_error(
        renode_name: &'static str,
        substate_id: &SubstateId,
    ) -> MappingError {
        MappingError::TransientSubstatePersisted {
            message: format!(
                "Transient substate persisted for {renode_name} RENode: {substate_id:?}"
            ),
        }
    }

    // Start body of method
    let entity_id_bytes = re_node_id_to_entity_id_bytes(&substate_id.0);
    let module_type = node_module_id_to_module_type(&substate_id.1);
    let substate_key_bytes = substate_offset_to_substate_key_bytes(&substate_id.2)?;

    // In the below, we nest match statements to ensure we get as much help from the compiler as possible to ensure
    //   we capture all possible substate types at compile time...
    // We can't capture new offset types under an RENode though - check nodes.rs after each merge to check we're not missing any
    let (entity_type, substate_type_key) = match &substate_id {
        // GLOBAL NORMAL COMPONENTS
        SubstateId(
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Normal(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::Component(offset)) => match offset {
                    ComponentOffset::State0 => (
                        SubstateType::ComponentState,
                        SubstateKeyType::ComponentState,
                    ),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("Component", &substate_id)),
            };
            (EntityType::NormalComponent, substate_type_key)
        }
        // GLOBAL ACCOUNT COMPONENTS
        SubstateId(
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Account(..)
                | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                | ComponentAddress::EddsaEd25519VirtualAccount(..),
            )),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::Account(offset)) => match offset {
                    AccountOffset::Account => (SubstateType::Account, SubstateKeyType::Account),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("Account", &substate_id)),
            };
            (EntityType::Account, substate_type_key)
        }

        // GLOBAL ACCESS CONTROLLERS
        SubstateId(
            RENodeId::GlobalObject(Address::Component(ComponentAddress::AccessController(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::AccessController(offset)) => match offset {
                    AccessControllerOffset::AccessController => (
                        SubstateType::AccessController,
                        SubstateKeyType::AccessController,
                    ),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("AccessController", &substate_id)),
            };
            (EntityType::AccessController, substate_type_key)
        }

        // GLOBAL PACKAGES
        SubstateId(RENodeId::GlobalObject(Address::Package(..)), module_id, offset) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::Package(offset)) => match offset {
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
                    PackageOffset::Royalty => (
                        SubstateType::PackageRoyalty,
                        SubstateKeyType::PackageRoyalty,
                    ),
                    PackageOffset::FunctionAccessRules => (
                        SubstateType::PackageFunctionAccessRules,
                        SubstateKeyType::PackageFunctionAccessRules,
                    ),
                    PackageOffset::EventSchema => {
                        return Err(unknown_substate_error(
                            "PackageEventSchema should have been removed",
                            &substate_id,
                        ))
                    }
                },
                // Packages still have ComponentRoyalties, because in the context of _their_ package (PACKAGE_LOADER),
                // they themselves have methods (eg setting the PackageRoyalty details - which is for the components of the package)
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("Package", &substate_id)),
            };
            (EntityType::Package, substate_type_key)
        }

        // GLOBAL FUNGIBLE RESOURCES
        SubstateId(
            RENodeId::GlobalObject(Address::Resource(ResourceAddress::Fungible(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::ResourceManager(offset)) => match offset {
                    ResourceManagerOffset::ResourceManager => (
                        SubstateType::FungibleResourceManager,
                        SubstateKeyType::FungibleResourceManager,
                    ),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (
                    NodeModuleId::AccessRules | NodeModuleId::AccessRules1,
                    SubstateOffset::AccessRules(offset),
                ) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => {
                    return Err(unknown_substate_error(
                        "FungibleResourceManager",
                        &substate_id,
                    ))
                }
            };
            (EntityType::FungibleResource, substate_type_key)
        }

        // GLOBAL NON-FUNGIBLE RESOURCES
        SubstateId(
            RENodeId::GlobalObject(Address::Resource(ResourceAddress::NonFungible(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::ResourceManager(offset)) => match offset {
                    ResourceManagerOffset::ResourceManager => (
                        SubstateType::NonFungibleResourceManager,
                        SubstateKeyType::NonFungibleResourceManager,
                    ),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (
                    NodeModuleId::AccessRules | NodeModuleId::AccessRules1,
                    SubstateOffset::AccessRules(offset),
                ) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => {
                    return Err(unknown_substate_error(
                        "NonFungibleResourceManager",
                        &substate_id,
                    ))
                }
            };
            (EntityType::NonFungibleResource, substate_type_key)
        }

        // KEY VALUE STORES
        SubstateId(RENodeId::KeyValueStore(_), module_id, offset) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::KeyValueStore(offset)) => match offset {
                    KeyValueStoreOffset::Entry(_) => (
                        SubstateType::KeyValueStoreEntry,
                        SubstateKeyType::KeyValueStoreEntry,
                    ),
                },
                _ => return Err(unknown_substate_error("KeyValueStore", &substate_id)),
            };
            (EntityType::KeyValueStore, substate_type_key)
        }

        // GLOBAL EPOCH MANAGER
        SubstateId(
            RENodeId::GlobalObject(Address::Component(ComponentAddress::EpochManager(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::EpochManager(offset)) => match offset {
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
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                _ => return Err(unknown_substate_error("EpochManager", &substate_id)),
            };
            (EntityType::EpochManager, substate_type_key)
        }

        // GLOBAL VALIDATOR
        SubstateId(
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Validator(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::Validator(offset)) => match offset {
                    ValidatorOffset::Validator => {
                        (SubstateType::Validator, SubstateKeyType::Validator)
                    }
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("Validator", &substate_id)),
            };
            (EntityType::Validator, substate_type_key)
        }

        // GLOBAL CLOCK
        SubstateId(
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Clock(..))),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::SELF, SubstateOffset::Clock(offset)) => match offset {
                    ClockOffset::CurrentTimeRoundedToMinutes => {
                        (SubstateType::Clock, SubstateKeyType::Clock)
                    }
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                _ => return Err(unknown_substate_error("Clock", &substate_id)),
            };
            (EntityType::Clock, substate_type_key)
        }

        // GLOBAL IDENTITY
        SubstateId(
            RENodeId::GlobalObject(Address::Component(
                ComponentAddress::Identity(..)
                | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                | ComponentAddress::EddsaEd25519VirtualIdentity(..),
            )),
            module_id,
            offset,
        ) => {
            let substate_type_key = match (module_id, offset) {
                (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                    TypeInfoOffset::TypeInfo => (SubstateType::TypeInfo, SubstateKeyType::TypeInfo),
                },
                (NodeModuleId::ComponentRoyalty, SubstateOffset::Royalty(offset)) => match offset {
                    RoyaltyOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    RoyaltyOffset::RoyaltyAccumulator => (
                        SubstateType::ComponentRoyaltyAccumulator,
                        SubstateKeyType::ComponentRoyaltyAccumulator,
                    ),
                },
                (
                    NodeModuleId::Metadata,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => (SubstateType::MetadataEntry, SubstateKeyType::MetadataEntry),
                (NodeModuleId::AccessRules, SubstateOffset::AccessRules(offset)) => match offset {
                    AccessRulesOffset::AccessRules => {
                        (SubstateType::AccessRules, SubstateKeyType::AccessRules)
                    }
                },
                _ => return Err(unknown_substate_error("Identity", &substate_id)),
            };
            (EntityType::Identity, substate_type_key)
        }

        // INTERNAL OBJECTS
        SubstateId(RENodeId::Object(object_id), module_id, offset) => match object_id {
            [INTERNAL_OBJECT_VAULT_ID, ..] => {
                let substate_type_key = match (module_id, offset) {
                    (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                        TypeInfoOffset::TypeInfo => {
                            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
                        }
                    },
                    (NodeModuleId::SELF, SubstateOffset::Vault(offset)) => match offset {
                        VaultOffset::Info => (SubstateType::VaultInfo, SubstateKeyType::VaultInfo),
                        VaultOffset::LiquidNonFungible => (
                            SubstateType::VaultNonFungible,
                            SubstateKeyType::VaultNonFungible,
                        ),
                        VaultOffset::LockedNonFungible => (
                            SubstateType::VaultLockedNonFungible,
                            SubstateKeyType::VaultLockedNonFungible,
                        ),
                        VaultOffset::LiquidFungible => {
                            (SubstateType::VaultFungible, SubstateKeyType::VaultFungible)
                        }
                        VaultOffset::LockedFungible => (
                            SubstateType::VaultLockedFungible,
                            SubstateKeyType::VaultLockedFungible,
                        ),
                    },
                    _ => return Err(unknown_substate_error("Vault", &substate_id)),
                };
                (EntityType::Vault, substate_type_key)
            }
            [INTERNAL_OBJECT_NORMAL_COMPONENT_ID, ..] => {
                let substate_type_key = match (module_id, offset) {
                    (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                        TypeInfoOffset::TypeInfo => {
                            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
                        }
                    },
                    (NodeModuleId::SELF, SubstateOffset::Component(offset)) => match offset {
                        ComponentOffset::State0 => (
                            SubstateType::ComponentState,
                            SubstateKeyType::ComponentState,
                        ),
                    },
                    _ => return Err(unknown_substate_error("Internal Component", &substate_id)),
                };

                (EntityType::NormalComponent, substate_type_key)
            }
            [INTERNAL_KV_STORE_ID, ..] => {
                let substate_type_key = match (module_id, offset) {
                    (NodeModuleId::TypeInfo, SubstateOffset::TypeInfo(offset)) => match offset {
                        TypeInfoOffset::TypeInfo => {
                            (SubstateType::TypeInfo, SubstateKeyType::TypeInfo)
                        }
                    },
                    (NodeModuleId::SELF, SubstateOffset::KeyValueStore(offset)) => match offset {
                        KeyValueStoreOffset::Entry(_) => (
                            SubstateType::KeyValueStoreEntry,
                            SubstateKeyType::KeyValueStoreEntry,
                        ),
                    },
                    _ => return Err(unknown_substate_error("KeyValueStore", &substate_id)),
                };
                (EntityType::KeyValueStore, substate_type_key)
            }
            _ => return Err(unknown_substate_error("Unmapped Object Type", &substate_id)),
        },

        // TRANSIENT SUBSTATES
        SubstateId(RENodeId::AuthZoneStack, ..) => {
            return Err(transient_substate_error("AuthZoneStack", &substate_id))
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
) -> Result<models::GlobalEntityReference, MappingError> {
    let reference = models::GlobalEntityReference {
        entity_reference: Box::new(to_api_entity_reference((*global_address).into())?),
        global_address_hex: to_hex(global_address_to_vec(global_address)),
        global_address: to_api_address(context, global_address),
    };

    Ok(reference)
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

pub fn re_node_id_to_entity_id_bytes(re_node_id: &RENodeId) -> Vec<u8> {
    (*re_node_id).into()
}

pub fn node_module_id_to_module_type(node_module_id: &NodeModuleId) -> ModuleType {
    match node_module_id {
        NodeModuleId::SELF => ModuleType::_Self,
        NodeModuleId::Metadata => ModuleType::Metadata,
        NodeModuleId::AccessRules => ModuleType::AccessRules,
        NodeModuleId::AccessRules1 => ModuleType::AccessRules1,
        NodeModuleId::ComponentRoyalty => ModuleType::ComponentRoyalty,
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
