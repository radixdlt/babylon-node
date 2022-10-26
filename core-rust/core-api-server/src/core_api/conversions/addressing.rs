use std::convert::TryFrom;

use crate::core_api::*;

use models::{EntityType, SubstateType};
use radix_engine::{
    model::GlobalAddressSubstate,
    types::{
        AddressError, Bech32Decoder, Bech32Encoder, ComponentAddress, NonFungibleId,
        PackageAddress, RENodeId, ResourceAddress, SubstateId,
    },
};
use scrypto::{
    address,
    engine::types::{
        BucketOffset, ComponentId, ComponentOffset, GlobalAddress, GlobalOffset,
        KeyValueStoreOffset, NonFungibleStoreOffset, PackageOffset, ProofOffset,
        ResourceManagerOffset, SubstateOffset, SystemOffset, VaultOffset, WorktopOffset,
    },
};

pub fn to_api_global_entity_id(
    bech32_encoder: &Bech32Encoder,
    global_address: &GlobalAddress,
    global_substate: &GlobalAddressSubstate,
) -> Result<models::GlobalEntityId, MappingError> {
    let global_address_bytes =
        MappedEntityId::try_from(RENodeId::Global(*global_address))?.entity_address;

    let target_entity_address = match global_substate {
        GlobalAddressSubstate::Component(scrypto::component::Component(id)) => id,
        GlobalAddressSubstate::SystemComponent(scrypto::component::Component(id)) => id,
        GlobalAddressSubstate::Resource(id) => id,
        GlobalAddressSubstate::Package(id) => id,
    };

    let target_entity_address_bytes = basic_address_to_vec(target_entity_address);

    let global_address_bech32m = match global_address {
        GlobalAddress::Component(addr) => bech32_encoder.encode_component_address_to_string(addr),
        GlobalAddress::Package(addr) => bech32_encoder.encode_package_address_to_string(addr),
        GlobalAddress::Resource(addr) => bech32_encoder.encode_resource_address_to_string(addr),
    };

    let entity_type = match global_address {
        GlobalAddress::Component(_) => EntityType::Component,
        GlobalAddress::Package(_) => EntityType::Package,
        GlobalAddress::Resource(_) => EntityType::ResourceManager,
    };

    Ok(models::GlobalEntityId {
        entity_type,
        entity_address_hex: to_hex(target_entity_address_bytes),
        global_address_hex: to_hex(global_address_bytes),
        global_address: global_address_bech32m,
    })
}

pub struct ParsedGlobalAddress(GlobalAddress);

impl TryFrom<&[u8]> for ParsedGlobalAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.is_empty() {
            return Err(AddressError::InvalidLength(slice.len()));
        }
        let entity_type = address::EntityType::try_from(slice[0])
            .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?;
        let global_address = match entity_type {
            address::EntityType::Resource => {
                GlobalAddress::Resource(ResourceAddress::try_from(slice)?)
            }
            address::EntityType::Package => {
                GlobalAddress::Package(PackageAddress::try_from(slice)?)
            }
            address::EntityType::NormalComponent => {
                GlobalAddress::Component(ComponentAddress::try_from(slice)?)
            }
            address::EntityType::AccountComponent => {
                GlobalAddress::Component(ComponentAddress::try_from(slice)?)
            }
            address::EntityType::SystemComponent => {
                GlobalAddress::Component(ComponentAddress::try_from(slice)?)
            }
        };
        Ok(Self(global_address))
    }
}

pub fn to_api_entity_id(node_id: RENodeId) -> Result<models::EntityId, MappingError> {
    let mapped: MappedEntityId = node_id.try_into()?;

    Ok(mapped.into())
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_id(substate_id: SubstateId) -> Result<models::SubstateId, MappingError> {
    let mapped = to_mapped_substate_id(substate_id)?;

    Ok(models::SubstateId {
        entity_type: mapped.0,
        entity_address_hex: to_hex(mapped.1),
        substate_type: mapped.2,
        substate_key_hex: to_hex(mapped.3),
    })
}

/// A basic address is formed from the transaction hash and a creation index, specifically:
/// (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
/// There is a separate exec_mode for the manifest and the standard Application executor
/// See id_allocator.rs for more information. - addresses are formed from (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
///
/// BEFORE updating this:
/// > NOTE that basic_address_to_vec only works properly if basic_address is of fixed length
/// > If basic_address became variable length, we'd need to do something else (eg sbor encode) to ensure a 1:1 mapping there
type BasicAddress = (scrypto::crypto::Hash, u32);

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

impl From<MappedEntityId> for models::EntityId {
    fn from(mapped_entity_id: MappedEntityId) -> Self {
        models::EntityId {
            entity_type: mapped_entity_id.entity_type,
            entity_address_hex: to_hex(mapped_entity_id.entity_address),
        }
    }
}

impl TryFrom<RENodeId> for MappedEntityId {
    fn try_from(re_node_id: RENodeId) -> Result<MappedEntityId, MappingError> {
        Ok(match re_node_id {
            RENodeId::Global(addr) => MappedEntityId::new(
                EntityType::Global,
                match addr {
                    GlobalAddress::Package(addr) => addr.to_vec(),
                    GlobalAddress::Component(addr) => addr.to_vec(),
                    GlobalAddress::Resource(addr) => addr.to_vec(),
                },
            ),
            RENodeId::KeyValueStore(addr) => {
                MappedEntityId::new(EntityType::KeyValueStore, basic_address_to_vec(&addr))
            }
            RENodeId::Component(id) => {
                MappedEntityId::new(EntityType::Component, basic_address_to_vec(&id))
            }
            RENodeId::Vault(addr) => {
                MappedEntityId::new(EntityType::Vault, basic_address_to_vec(&addr))
            }
            RENodeId::ResourceManager(addr) => {
                MappedEntityId::new(EntityType::ResourceManager, basic_address_to_vec(&addr))
            }
            RENodeId::Package(addr) => {
                MappedEntityId::new(EntityType::Package, basic_address_to_vec(&addr))
            }
            RENodeId::System(id) => {
                MappedEntityId::new(EntityType::System, basic_address_to_vec(&id))
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
            RENodeId::NonFungibleStore(non_fungible_store_id) => MappedEntityId::new(
                EntityType::NonFungibleStore,
                basic_address_to_vec(&non_fungible_store_id),
            ),
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
            entity_address_hex: to_hex(mapped_substate_id.1),
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

impl From<MappedSubstateId> for models::EntityId {
    fn from(mapped_substate_id: MappedSubstateId) -> Self {
        models::EntityId {
            entity_type: mapped_substate_id.0,
            entity_address_hex: to_hex(mapped_substate_id.1),
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
            match global_address {
                GlobalAddress::Package(package_addr) => package_addr.to_vec(),
                GlobalAddress::Resource(resource_addr) => resource_addr.to_vec(),
                GlobalAddress::Component(component_addr) => component_addr.to_vec(),
            },
            SubstateType::Global,
            vec![0],
        ),

        // SYSTEM SUBSTATES
        SubstateId(
            RENodeId::System(component_id),
            SubstateOffset::System(SystemOffset::System),
        ) => MappedSubstateId(
            EntityType::System,
            basic_address_to_vec(&component_id),
            SubstateType::System,
            vec![0],
        ),
        // COMPONENT SUBSTATES
        SubstateId(
            RENodeId::Component(component_id),
            SubstateOffset::Component(ComponentOffset::Info),
        ) => MappedSubstateId(
            EntityType::Component,
            basic_address_to_vec(&component_id),
            SubstateType::ComponentInfo,
            vec![0],
        ),
        SubstateId(
            RENodeId::Component(component_id),
            SubstateOffset::Component(ComponentOffset::State),
        ) => MappedSubstateId(
            EntityType::Component,
            basic_address_to_vec(&component_id),
            SubstateType::ComponentState,
            vec![1],
        ),
        // PACKAGE SUBSTATES
        SubstateId(RENodeId::Package(addr), SubstateOffset::Package(PackageOffset::Package)) => {
            MappedSubstateId(
                EntityType::Package,
                basic_address_to_vec(&addr),
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
            basic_address_to_vec(&addr),
            SubstateType::ResourceManager,
            vec![0],
        ),
        SubstateId(
            RENodeId::NonFungibleStore(store_id),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
        ) => MappedSubstateId(
            EntityType::NonFungibleStore,
            basic_address_to_vec(&store_id),
            SubstateType::NonFungible,
            prefix(vec![2], id.0),
        ),

        SubstateId(
            RENodeId::NonFungibleStore(..),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
        ) => {
            return Err(MappingError::VirtualRootSubstatePersisted {
                message: "No state_update known/possible for NonFungibleSpace".to_owned(),
            })
        }

        // KEY VALUE STORE SUBSTATES
        SubstateId(
            RENodeId::KeyValueStore(..),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
        ) => {
            return Err(MappingError::VirtualRootSubstatePersisted {
                message: "No state_update known/possible for KeyValueStoreSpace".to_owned(),
            })
        }
        SubstateId(
            RENodeId::KeyValueStore(id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
        ) => MappedSubstateId(
            EntityType::KeyValueStore,
            basic_address_to_vec(&id),
            SubstateType::KeyValueStoreEntry,
            prefix(vec![1], key),
        ),

        // VAULT SUBSTATES
        SubstateId(RENodeId::Vault(vault_id), SubstateOffset::Vault(VaultOffset::Vault)) => {
            MappedSubstateId(
                EntityType::Vault,
                basic_address_to_vec(&vault_id),
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

pub fn to_api_virtual_substate_id(
    root_substate_id: SubstateId,
    key: Vec<u8>,
) -> Result<models::SubstateId, MappingError> {
    // These should match the ids of the keys
    let sub_id = match root_substate_id {
        // NonFungibleSpace key is downed to create a NonFungible
        SubstateId(
            RENodeId::NonFungibleStore(store_id),
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
        ) => MappedSubstateId(
            EntityType::NonFungibleStore,
            basic_address_to_vec(&store_id),
            SubstateType::NonFungible,
            prefix(vec![2], key),
        ),
        // KeyValueStoreSpace key is downed to create a KeyValueStoreEntry
        SubstateId(
            RENodeId::KeyValueStore(store_id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
        ) => MappedSubstateId(
            EntityType::KeyValueStore,
            basic_address_to_vec(&store_id),
            SubstateType::KeyValueStoreEntry,
            prefix(vec![1], key),
        ),
        // Assume all other substates are not root spaces
        other => {
            return Err(MappingError::VirtualSubstateDownedWithInvalidParent {
                message: format!("{:?}", other),
            })
        }
    };
    Ok(models::SubstateId {
        entity_type: sub_id.0,
        entity_address_hex: to_hex(sub_id.1),
        substate_type: sub_id.2,
        substate_key_hex: to_hex(sub_id.3),
    })
}

pub fn to_global_component_entity_id(component_address: &ComponentAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::Global,
        entity_address: component_address.to_vec(),
    }
}

pub fn to_component_entity_id(component_id: &ComponentId) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::Component,
        entity_address: basic_address_to_vec(component_id),
    }
}

pub fn to_resource_entity_id(resource_address: &ResourceAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::ResourceManager,
        entity_address: resource_address.to_vec(),
    }
}

pub fn to_global_resource_entity_id(resource_address: &ResourceAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::Global,
        entity_address: resource_address.to_vec(),
    }
}

#[allow(dead_code)]
pub fn to_package_entity_id(package_address: &PackageAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::Package,
        entity_address: package_address.to_vec(),
    }
}

pub fn to_vault_entity_id(basic_address: &BasicAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::Vault,
        entity_address: basic_address_to_vec(basic_address),
    }
}

pub fn to_key_value_store_entity_id(basic_address: &BasicAddress) -> MappedEntityId {
    MappedEntityId {
        entity_type: EntityType::KeyValueStore,
        entity_address: basic_address_to_vec(basic_address),
    }
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

// NB - see id_allocator.rs - addresses are formed from (tx_hash, index_in_tx_for_exec_mode + offset_for_exec_mode)
// There is a separate exec_mode for the manifest and the standard Application executor
pub fn basic_address_to_vec(basic_address: &BasicAddress) -> Vec<u8> {
    // NOTE - this only works because the trunc of basic_address is of fixed length.
    // If basic_address became variable length, we'd need to do something else (eg sbor encode) to ensure a 1:1 mapping here

    prefix(
        basic_address.0.to_vec(),
        basic_address.1.to_le_bytes().to_vec(),
    )
}

fn prefix(mut prefix: Vec<u8>, mut suffix: Vec<u8>) -> Vec<u8> {
    prefix.append(&mut suffix);
    prefix
}
