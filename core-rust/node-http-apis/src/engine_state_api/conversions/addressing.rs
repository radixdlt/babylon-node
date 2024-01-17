use crate::engine_state_api::models;
use crate::engine_state_api::*;

use radix_engine::types::*;

pub fn to_api_global_address(
    context: &MappingContext,
    global_address: &GlobalAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, global_address.as_node_id())
}

pub fn to_api_resource_address(
    context: &MappingContext,
    resource_address: &ResourceAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, resource_address.as_node_id())
}

pub fn to_api_package_address(
    context: &MappingContext,
    package_address: &PackageAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, package_address.as_node_id())
}

pub fn to_api_entity_address(
    context: &MappingContext,
    node_id: &NodeId,
) -> Result<String, MappingError> {
    context
        .address_encoder
        .encode(node_id.as_ref())
        .map_err(|err| MappingError::InvalidEntityAddress { encode_error: err })
}

pub fn to_api_entity_type(entity_type: EntityType) -> models::EntityType {
    match entity_type {
        EntityType::GlobalPackage => models::EntityType::GlobalPackage,
        EntityType::GlobalFungibleResourceManager => models::EntityType::GlobalFungibleResource,
        EntityType::GlobalNonFungibleResourceManager => {
            models::EntityType::GlobalNonFungibleResource
        }
        EntityType::GlobalConsensusManager => models::EntityType::GlobalConsensusManager,
        EntityType::GlobalValidator => models::EntityType::GlobalValidator,
        EntityType::GlobalAccessController => models::EntityType::GlobalAccessController,
        EntityType::GlobalAccount => models::EntityType::GlobalAccount,
        EntityType::GlobalIdentity => models::EntityType::GlobalIdentity,
        EntityType::GlobalGenericComponent => models::EntityType::GlobalGenericComponent,
        EntityType::GlobalVirtualSecp256k1Account => {
            models::EntityType::GlobalVirtualSecp256k1Account
        }
        EntityType::GlobalVirtualEd25519Account => models::EntityType::GlobalVirtualEd25519Account,
        EntityType::GlobalVirtualSecp256k1Identity => {
            models::EntityType::GlobalVirtualSecp256k1Identity
        }
        EntityType::GlobalVirtualEd25519Identity => {
            models::EntityType::GlobalVirtualEd25519Identity
        }
        EntityType::InternalFungibleVault => models::EntityType::InternalFungibleVault,
        EntityType::InternalNonFungibleVault => models::EntityType::InternalNonFungibleVault,
        EntityType::InternalKeyValueStore => models::EntityType::InternalKeyValueStore,
        EntityType::InternalGenericComponent => models::EntityType::InternalGenericComponent,
        EntityType::GlobalOneResourcePool => models::EntityType::GlobalOneResourcePool,
        EntityType::GlobalTwoResourcePool => models::EntityType::GlobalTwoResourcePool,
        EntityType::GlobalMultiResourcePool => models::EntityType::GlobalMultiResourcePool,
        EntityType::GlobalTransactionTracker => models::EntityType::GlobalTransactionTracker,
    }
}

pub fn extract_package_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<PackageAddress, ExtractionError> {
    PackageAddress::try_from_bech32(&extraction_context.address_decoder, package_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn to_api_attached_module_id(module_id: &AttachedModuleId) -> models::AttachedModuleId {
    match module_id {
        AttachedModuleId::Metadata => models::AttachedModuleId::Metadata,
        AttachedModuleId::Royalty => models::AttachedModuleId::Royalty,
        AttachedModuleId::RoleAssignment => models::AttachedModuleId::RoleAssignment,
    }
}

pub fn extract_api_module_id(module_id: &models::ModuleId) -> ModuleId {
    match module_id {
        models::ModuleId::Main => ModuleId::Main,
        models::ModuleId::Metadata => ModuleId::Metadata,
        models::ModuleId::Royalty => ModuleId::Royalty,
        models::ModuleId::RoleAssignment => ModuleId::RoleAssignment,
    }
}

pub fn extract_address_as_node_id(
    extraction_context: &ExtractionContext,
    address: &str,
) -> Result<NodeId, ExtractionError> {
    let (_entity_type, bytes) = extraction_context
        .address_decoder
        .validate_and_decode(address)
        .map_err(|_error| ExtractionError::InvalidAddress)?;
    if bytes.len() != NodeId::LENGTH {
        return Err(ExtractionError::InvalidAddress);
    }
    Ok(NodeId::from(copy_u8_array(&bytes)))
}

pub fn to_api_non_fungible_global_id(
    context: &MappingContext,
    non_fungible_global_id: &NonFungibleGlobalId,
) -> Result<models::NonFungibleGlobalId, MappingError> {
    Ok(models::NonFungibleGlobalId {
        resource_address: to_api_resource_address(
            context,
            &non_fungible_global_id.resource_address(),
        )?,
        local_id: non_fungible_global_id.local_id().to_string(),
    })
}