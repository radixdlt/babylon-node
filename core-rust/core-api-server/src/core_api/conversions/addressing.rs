use std::convert::TryFrom;

use crate::core_api::*;

use models::{EntityType, SubstateKeyType, SubstateType};
use radix_engine::types::{
    ComponentOffset, GlobalAddress, GlobalOffset, KeyValueStoreOffset, NonFungibleIdType,
    NonFungibleStoreOffset, PackageOffset, ResourceManagerOffset, SubstateOffset, SystemAddress,
    VaultOffset,
};
use radix_engine::{
    model::GlobalAddressSubstate,
    types::{
        scrypto_encode, AccessRulesChainOffset, Bech32Decoder, Bech32Encoder, ClockOffset,
        ComponentAddress, EpochManagerOffset, MetadataOffset, NonFungibleId, PackageAddress,
        RENodeId, ResourceAddress, SubstateId,
    },
};

pub fn to_api_global_entity_assignment(
    bech32_encoder: &Bech32Encoder,
    global_substate_id: &SubstateId,
    global_address: &GlobalAddress,
    global_substate: &GlobalAddressSubstate,
) -> Result<models::GlobalEntityAssignment, MappingError> {
    let target_re_node_id = global_substate.node_deref();

    let target_entity = MappedEntityId::try_from(target_re_node_id)?;

    let global_entity_id_bytes = re_node_id_to_entity_id_bytes(&global_substate_id.0)?;

    Ok(models::GlobalEntityAssignment {
        target_entity_type: target_entity.entity_type,
        target_entity_id_hex: to_hex(target_entity.entity_id_bytes),
        global_entity_id_hex: to_hex(global_entity_id_bytes),
        global_address_hex: to_hex(global_address_to_vec(global_address)),
        global_address: encode_to_bech32m_string(bech32_encoder, global_address),
    })
}

pub fn encode_to_bech32m_string(
    bech32_encoder: &Bech32Encoder,
    global_address: &GlobalAddress,
) -> String {
    match global_address {
        GlobalAddress::Component(addr) => bech32_encoder.encode_component_address_to_string(addr),
        GlobalAddress::Package(addr) => bech32_encoder.encode_package_address_to_string(addr),
        GlobalAddress::Resource(addr) => bech32_encoder.encode_resource_address_to_string(addr),
        GlobalAddress::System(addr) => bech32_encoder.encode_system_address_to_string(addr),
    }
}

pub fn get_entity_type_from_global_address(global_address: &GlobalAddress) -> models::EntityType {
    match global_address {
        GlobalAddress::Component(_) => models::EntityType::Component,
        GlobalAddress::Package(_) => models::EntityType::Package,
        GlobalAddress::Resource(_) => models::EntityType::ResourceManager,
        GlobalAddress::System(SystemAddress::EpochManager(_)) => models::EntityType::EpochManager,
        GlobalAddress::System(SystemAddress::Clock(_)) => models::EntityType::Clock,
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
        let entity_type = match re_node_id {
            RENodeId::Global(_) => EntityType::Global,
            RENodeId::Component(_) => EntityType::Component,
            RENodeId::Package(_) => EntityType::Package,
            RENodeId::ResourceManager(_) => EntityType::ResourceManager,
            RENodeId::EpochManager(_) => EntityType::EpochManager,
            RENodeId::Clock(_) => EntityType::Clock,
            RENodeId::KeyValueStore(_) => EntityType::KeyValueStore,
            RENodeId::NonFungibleStore(_) => EntityType::NonFungibleStore,
            RENodeId::Vault(_) => EntityType::Vault,
            RENodeId::Bucket(_) => return Err(transient_renode_error("Bucket")),
            RENodeId::Proof(_) => return Err(transient_renode_error("Proof")),
            RENodeId::Worktop => return Err(transient_renode_error("Worktop")),
            RENodeId::AuthZoneStack(_) => return Err(transient_renode_error("AuthZoneStack")),
            RENodeId::FeeReserve(_) => return Err(transient_renode_error("FeeReserve")),
        };
        Ok(MappedEntityId {
            entity_type,
            entity_id_bytes,
        })
    }

    type Error = MappingError;
}

#[derive(Debug)]
pub struct MappedSubstateId(EntityType, Vec<u8>, SubstateType, SubstateKeyType, Vec<u8>);

impl From<MappedSubstateId> for models::SubstateId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::SubstateId {
            entity_type: mapped_substate_id.0,
            entity_id_hex: to_hex(mapped_substate_id.1),
            substate_type: mapped_substate_id.2,
            substate_key_type: mapped_substate_id.3,
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
    let substate_key_bytes = substate_offset_to_substate_key_bytes(&substate_id.1)?;

    // In the below, we nest match statements to ensure we get as much help from the compiler as possible to ensure
    //   we capture all possible substate types at compile time...
    // We can't capture new offset types under an RENode though - check nodes.rs after each merge to check we're not missing any
    let (entity_type, substate_type_key) = match &substate_id {
        SubstateId(RENodeId::Global(_), offset) => {
            let substate_type_key = match offset {
                SubstateOffset::Global(offset) => match offset {
                    GlobalOffset::Global => {
                        (SubstateType::GlobalAddress, SubstateKeyType::GlobalAddress)
                    }
                },
                _ => return Err(unknown_substate_error("Global", &substate_id)),
            };
            (EntityType::Global, substate_type_key)
        }

        SubstateId(RENodeId::Component(_), offset) => {
            let substate_type_key = match offset {
                SubstateOffset::Component(offset) => match offset {
                    ComponentOffset::Info => {
                        (SubstateType::ComponentInfo, SubstateKeyType::ComponentInfo)
                    }
                    ComponentOffset::State => (
                        SubstateType::ComponentState,
                        SubstateKeyType::ComponentState,
                    ),
                    ComponentOffset::RoyaltyConfig => (
                        SubstateType::ComponentRoyaltyConfig,
                        SubstateKeyType::ComponentRoyaltyConfig,
                    ),
                    ComponentOffset::RoyaltyAccumulator => (
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

        SubstateId(RENodeId::Package(_), offset) => {
            let substate_type_key = match offset {
                SubstateOffset::Package(offset) => match offset {
                    PackageOffset::Info => {
                        (SubstateType::PackageInfo, SubstateKeyType::PackageInfo)
                    }
                    PackageOffset::RoyaltyConfig => (
                        SubstateType::PackageRoyaltyConfig,
                        SubstateKeyType::PackageRoyaltyConfig,
                    ),
                    PackageOffset::RoyaltyAccumulator => (
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
                _ => return Err(unknown_substate_error("Package", &substate_id)),
            };
            (EntityType::Package, substate_type_key)
        }

        SubstateId(RENodeId::ResourceManager(_), offset) => {
            let substate_type_key = match offset {
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
                SubstateOffset::VaultAccessRulesChain(offset) => match offset {
                    AccessRulesChainOffset::AccessRulesChain => (
                        SubstateType::AccessRulesChain,
                        SubstateKeyType::ResourceManagerVaultAccessRulesChain,
                    ),
                },
                _ => return Err(unknown_substate_error("ResourceManager", &substate_id)),
            };
            (EntityType::ResourceManager, substate_type_key)
        }

        SubstateId(RENodeId::NonFungibleStore(_), offset) => {
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

        SubstateId(RENodeId::KeyValueStore(_), offset) => {
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

        SubstateId(RENodeId::Vault(_), offset) => {
            let substate_type_key = match offset {
                SubstateOffset::Vault(offset) => match offset {
                    VaultOffset::Vault => (SubstateType::Vault, SubstateKeyType::Vault),
                },
                _ => return Err(unknown_substate_error("Vault", &substate_id)),
            };
            (EntityType::Vault, substate_type_key)
        }

        SubstateId(RENodeId::EpochManager(_), offset) => {
            let substate_type_key = match offset {
                SubstateOffset::EpochManager(offset) => match offset {
                    EpochManagerOffset::EpochManager => {
                        (SubstateType::EpochManager, SubstateKeyType::EpochManager)
                    }
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

        SubstateId(RENodeId::Clock(_), offset) => {
            let substate_type_key = match offset {
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

        // TRANSIENT SUBSTATES
        SubstateId(RENodeId::Bucket(..), _) => {
            return Err(transient_substate_error("Bucket", &substate_id))
        }
        SubstateId(RENodeId::Proof(..), _) => {
            return Err(transient_substate_error("Proof", &substate_id))
        }
        SubstateId(RENodeId::Worktop, _) => {
            return Err(transient_substate_error("Worktop", &substate_id))
        }
        SubstateId(RENodeId::AuthZoneStack(_), _) => {
            return Err(transient_substate_error("AuthZoneStack", &substate_id))
        }
        SubstateId(RENodeId::FeeReserve(_), _) => {
            return Err(transient_substate_error("FeeReserve", &substate_id))
        }
    };

    Ok(MappedSubstateId(
        entity_type,
        entity_id_bytes,
        substate_type_key.0,
        substate_type_key.1,
        substate_key_bytes,
    ))
}

pub fn to_global_entity_reference(
    bech32_encoder: &Bech32Encoder,
    global_address: &GlobalAddress,
) -> models::GlobalEntityReference {
    models::GlobalEntityReference {
        entity_type: get_entity_type_from_global_address(global_address),
        global_address_hex: to_hex(global_address_to_vec(global_address)),
        global_address: encode_to_bech32m_string(bech32_encoder, global_address),
    }
}

pub fn to_entity_reference(re_node_id: RENodeId) -> Result<models::EntityReference, MappingError> {
    Ok(MappedEntityId::try_from(re_node_id)?.into())
}

pub fn extract_package_address(
    bech32_decoder: &Bech32Decoder,
    package_address: &str,
) -> Result<PackageAddress, ExtractionError> {
    bech32_decoder
        .validate_and_decode_package_address(package_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_component_address(
    bech32_decoder: &Bech32Decoder,
    component_address: &str,
) -> Result<ComponentAddress, ExtractionError> {
    bech32_decoder
        .validate_and_decode_component_address(component_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_resource_address(
    bech32_decoder: &Bech32Decoder,
    resource_address: &str,
) -> Result<ResourceAddress, ExtractionError> {
    bech32_decoder
        .validate_and_decode_resource_address(resource_address)
        .map_err(ExtractionError::InvalidAddress)
}

pub fn extract_non_fungible_id_from_simple_representation(
    id_type: NonFungibleIdType,
    simple_rep: &str,
) -> Result<NonFungibleId, ExtractionError> {
    Ok(NonFungibleId::try_from_simple_string(id_type, simple_rep)?)
}

pub fn re_node_id_to_entity_id_bytes(re_node_id: &RENodeId) -> Result<Vec<u8>, MappingError> {
    scrypto_encode(re_node_id).map_err(|err| MappingError::SborEncodeError {
        encode_error: err,
        message: "Could not encode re node id".to_string(),
    })
}

pub fn substate_offset_to_substate_key_bytes(
    substate_offset: &SubstateOffset,
) -> Result<Vec<u8>, MappingError> {
    scrypto_encode(substate_offset).map_err(|err| MappingError::SborEncodeError {
        encode_error: err,
        message: "Could not encode substate offset".to_string(),
    })
}

pub fn global_address_to_vec(global_address: &GlobalAddress) -> Vec<u8> {
    match global_address {
        GlobalAddress::Package(package_addr) => package_addr.to_vec(),
        GlobalAddress::Resource(resource_addr) => resource_addr.to_vec(),
        GlobalAddress::Component(component_addr) => component_addr.to_vec(),
        GlobalAddress::System(system_addr) => system_addr.to_vec(),
    }
}
