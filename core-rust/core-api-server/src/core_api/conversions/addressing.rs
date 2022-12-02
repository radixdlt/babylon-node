use std::convert::TryFrom;

use crate::core_api::*;

use models::{EntityType, SubstateType};
use radix_engine::{
    model::GlobalAddressSubstate,
    types::{
        Bech32Decoder, Bech32Encoder, ComponentAddress, EpochManagerOffset, NonFungibleId,
        PackageAddress, RENodeId, ResourceAddress, SubstateId,
    },
};
use radix_engine::types::{
    BucketOffset, ComponentOffset, GlobalAddress, GlobalOffset, KeyValueStoreOffset,
    NonFungibleStoreOffset, PackageOffset, ProofOffset, ResourceManagerOffset, SubstateOffset,
    VaultOffset, WorktopOffset
};

pub fn to_api_global_entity_assignment(
    bech32_encoder: &Bech32Encoder,
    global_address: &GlobalAddress,
    global_substate: &GlobalAddressSubstate,
) -> Result<models::GlobalEntityAssignment, MappingError> {
    let target_entity_id = match global_substate {
        GlobalAddressSubstate::Component(id) => id,
        GlobalAddressSubstate::System(id) => id,
        GlobalAddressSubstate::Resource(id) => id,
        GlobalAddressSubstate::Package(id) => id,
    };

    Ok(models::GlobalEntityAssignment {
        target_entity_type: get_entity_type_from_global_address(global_address),
        target_entity_id_hex: to_hex(entity_id_to_bytes(target_entity_id)),
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
        GlobalAddress::System(_) => models::EntityType::EpochManager,
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

/// An entity id is formed from the transaction hash and a creation index, specifically:
/// (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
/// There is a separate exec_mode for the manifest and the standard Application executor
/// See id_allocator.rs for more information. - addresses are formed from (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
///
/// BEFORE updating this:
/// > NOTE that re_node_id only works properly if EntityId is of fixed length
/// > If EntityId became variable length, we'd need to do something else (eg sbor encode) to ensure a 1:1 mapping there
type EntityId = [u8; 36];

#[derive(Debug)]
pub struct MappedEntityId {
    entity_type: EntityType,
    entity_address: Vec<u8>,
}

impl MappedEntityId {
    pub fn new(entity_type: EntityType, address: Vec<u8>) -> Self {
        MappedEntityId {
            entity_type,
            entity_address: address,
        }
    }
}

impl From<MappedEntityId> for models::EntityReference {
    fn from(mapped_entity_id: MappedEntityId) -> Self {
        models::EntityReference {
            entity_type: mapped_entity_id.entity_type,
            entity_id_hex: to_hex(mapped_entity_id.entity_address),
        }
    }
}

impl TryFrom<RENodeId> for MappedEntityId {
    fn try_from(re_node_id: RENodeId) -> Result<MappedEntityId, MappingError> {
        Ok(match re_node_id {
            RENodeId::Global(addr) => {
                MappedEntityId::new(EntityType::Global, global_address_to_entity_id_bytes(&addr))
            }
            RENodeId::KeyValueStore(addr) => {
                MappedEntityId::new(EntityType::KeyValueStore, entity_id_to_bytes(&addr))
            }
            RENodeId::Component(id) => {
                MappedEntityId::new(EntityType::Component, entity_id_to_bytes(&id))
            }
            RENodeId::Vault(addr) => {
                MappedEntityId::new(EntityType::Vault, entity_id_to_bytes(&addr))
            }
            RENodeId::ResourceManager(addr) => {
                MappedEntityId::new(EntityType::ResourceManager, entity_id_to_bytes(&addr))
            }
            RENodeId::Package(addr) => {
                MappedEntityId::new(EntityType::Package, entity_id_to_bytes(&addr))
            }
            RENodeId::EpochManager(id) => {
                MappedEntityId::new(EntityType::EpochManager, entity_id_to_bytes(&id))
            }
            RENodeId::NonFungibleStore(id) => {
                MappedEntityId::new(EntityType::NonFungibleStore, entity_id_to_bytes(&id))
            }
            RENodeId::Bucket(_) => {
                return Err(MappingError::TransientSubstatePersisted {
                    message: "Bucket persisted".to_owned(),
                })
            }
            RENodeId::Proof(_) => {
                return Err(MappingError::TransientSubstatePersisted {
                    message: "Proof persisted".to_owned(),
                })
            }
            RENodeId::Worktop => {
                return Err(MappingError::TransientSubstatePersisted {
                    message: "Worktop persisted".to_owned(),
                })
            }
            RENodeId::AuthZoneStack(_) => {
                return Err(MappingError::TransientSubstatePersisted {
                    message: "AuthZoneStack persisted".to_owned(),
                })
            }
            RENodeId::FeeReserve(_) => {
                return Err(MappingError::TransientSubstatePersisted {
                    message: "FeeReserve persisted".to_owned(),
                })
            },
        })
    }

    type Error = MappingError;
}

#[derive(Debug)]
pub struct MappedSubstateId(EntityType, Vec<u8>, SubstateType, Vec<u8>);

impl From<MappedSubstateId> for models::SubstateId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::SubstateId {
            entity_type: mapped_substate_id.0,
            entity_id_hex: to_hex(mapped_substate_id.1),
            substate_type: mapped_substate_id.2,
            substate_key_hex: to_hex(mapped_substate_id.3),
        }
    }
}

impl From<MappedSubstateId> for MappedEntityId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        MappedEntityId {
            entity_type: mapped_substate_id.0,
            entity_address: mapped_substate_id.1,
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
    // It's crucial that we ensure all Entity Addresses are unique
    // It's crucial that we ensure all Substate keys are locally unique
    // NOTE: If you add any transient root spaces here, ensure they're added to to_api_virtual_substate_id
    Ok(match substate_id {
        // GLOBAL
        SubstateId(
            RENodeId::Global(global_address),
            SubstateOffset::Global(GlobalOffset::Global),
        ) => MappedSubstateId(
            EntityType::Global,
            global_address_to_entity_id_bytes(&global_address),
            SubstateType::Global,
            vec![0],
        ),

        // SYSTEM SUBSTATES
        SubstateId(
            RENodeId::EpochManager(component_id),
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
        ) => MappedSubstateId(
            EntityType::EpochManager,
            entity_id_to_bytes(&component_id),
            SubstateType::EpochManager,
            vec![0],
        ),

        // COMPONENT SUBSTATES
        SubstateId(
            RENodeId::Component(component_id),
            SubstateOffset::Component(ComponentOffset::Info),
        ) => MappedSubstateId(
            EntityType::Component,
            entity_id_to_bytes(&component_id),
            SubstateType::ComponentInfo,
            vec![0],
        ),
        SubstateId(
            RENodeId::Component(component_id),
            SubstateOffset::Component(ComponentOffset::State),
        ) => MappedSubstateId(
            EntityType::Component,
            entity_id_to_bytes(&component_id),
            SubstateType::ComponentState,
            vec![1],
        ),

        // PACKAGE SUBSTATES
        SubstateId(RENodeId::Package(addr), SubstateOffset::Package(PackageOffset::Info)) => {
            MappedSubstateId(
                EntityType::Package,
                entity_id_to_bytes(&addr),
                SubstateType::Package,
                vec![0],
            )
        }

        // RESOURCE SUBSTATES
        SubstateId(
            RENodeId::ResourceManager(addr),
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
        ) => MappedSubstateId(
            EntityType::ResourceManager,
            entity_id_to_bytes(&addr),
            SubstateType::ResourceManager,
            vec![0],
        ),
        SubstateId(
            RENodeId::NonFungibleStore(store_id),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
        ) => MappedSubstateId(
            EntityType::NonFungibleStore,
            entity_id_to_bytes(&store_id),
            SubstateType::NonFungible,
            prefix(vec![2], id.0),
        ),

        // KEY VALUE STORE SUBSTATES
        SubstateId(
            RENodeId::KeyValueStore(id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
        ) => MappedSubstateId(
            EntityType::KeyValueStore,
            entity_id_to_bytes(&id),
            SubstateType::KeyValueStoreEntry,
            prefix(vec![1], key),
        ),

        // VAULT SUBSTATES
        SubstateId(RENodeId::Vault(vault_id), SubstateOffset::Vault(VaultOffset::Vault)) => {
            MappedSubstateId(
                EntityType::Vault,
                entity_id_to_bytes(&vault_id),
                SubstateType::Vault,
                vec![0],
            )
        }

        // TRANSIENT? SUBSTATES
        SubstateId(RENodeId::Bucket(..), SubstateOffset::Bucket(BucketOffset::Bucket)) => {
            return Err(MappingError::TransientSubstatePersisted {
                message: "Bucket persisted".to_owned(),
            })
        }
        SubstateId(RENodeId::Proof(..), SubstateOffset::Proof(ProofOffset::Proof)) => {
            return Err(MappingError::TransientSubstatePersisted {
                message: "Proof persisted".to_owned(),
            })
        }
        SubstateId(RENodeId::Worktop, SubstateOffset::Worktop(WorktopOffset::Worktop)) => {
            return Err(MappingError::TransientSubstatePersisted {
                message: "Worktop persisted".to_owned(),
            })
        }
        _ => {
            return Err(MappingError::UnsupportedSubstatePersisted {
                message: format!("Unsupported substate persisted: {:?}", substate_id),
            })
        }
    })
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

pub fn to_entity_reference(
    entity_type: EntityType,
    entity_id: &EntityId,
) -> models::EntityReference {
    MappedEntityId {
        entity_type,
        entity_address: entity_id_to_bytes(entity_id),
    }
    .into()
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

pub fn extract_non_fungible_id(non_fungible_id: &str) -> Result<NonFungibleId, ExtractionError> {
    Ok(NonFungibleId(from_hex(non_fungible_id)?))
}

pub fn entity_id_to_bytes(entity_id: &EntityId) -> Vec<u8> {
    entity_id.to_vec()
}

/// This is used for Global Entities - the only entities which don't have use the standard EntityId format
pub fn global_address_to_entity_id_bytes(global_address: &GlobalAddress) -> Vec<u8> {
    // For now, we just use the global address bytes
    global_address_to_vec(global_address)
}

pub fn global_address_to_vec(global_address: &GlobalAddress) -> Vec<u8> {
    match global_address {
        GlobalAddress::Package(package_addr) => package_addr.to_vec(),
        GlobalAddress::Resource(resource_addr) => resource_addr.to_vec(),
        GlobalAddress::Component(component_addr) => component_addr.to_vec(),
        GlobalAddress::System(system_addr) => system_addr.to_vec(),
    }
}

fn prefix(mut prefix: Vec<u8>, mut suffix: Vec<u8>) -> Vec<u8> {
    prefix.append(&mut suffix);
    prefix
}
