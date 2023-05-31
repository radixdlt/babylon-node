use crate::core_api::models;
use crate::core_api::*;
use models::SubstateType;
use radix_engine::track::db_key_mapper::*;
use radix_engine::types::*;
use radix_engine_interface::api::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_global_address(
    context: &MappingContext,
    global_address: &GlobalAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, global_address.as_node_id())
}

pub fn to_api_component_address(
    context: &MappingContext,
    component_address: &ComponentAddress,
) -> Result<String, MappingError> {
    to_api_entity_address(context, component_address.as_node_id())
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
        .bech32_encoder
        .encode(node_id.as_ref())
        .map_err(|err| MappingError::InvalidEntityAddress { encode_error: err })
}

pub fn to_api_entity_reference(
    context: &MappingContext,
    node_id: &NodeId,
) -> Result<models::EntityReference, MappingError> {
    Ok(models::EntityReference {
        entity_type: to_api_entity_type(
            node_id.entity_type().ok_or(MappingError::EntityTypeError)?,
        ),
        is_global: node_id.is_global(),
        entity_address: to_api_entity_address(context, node_id)?,
    })
}

pub fn to_api_entity_type(entity_type: EntityType) -> models::EntityType {
    match entity_type {
        EntityType::GlobalPackage => models::EntityType::GlobalPackage,
        EntityType::GlobalFungibleResourceManager => models::EntityType::GlobalFungibleResource,
        EntityType::GlobalNonFungibleResourceManager => models::EntityType::GlobalNonFungibleResource,
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
        EntityType::InternalAccount => models::EntityType::InternalAccount,
        EntityType::InternalKeyValueStore => models::EntityType::InternalKeyValueStore,
        EntityType::InternalGenericComponent => models::EntityType::InternalGenericComponent,
    }
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_id(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
) -> Result<models::SubstateId, MappingError> {
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    let entity_address = to_api_entity_address(context, node_id)?;
    let api_substate_key = to_api_substate_key(substate_key);

    let (substate_type, partition_kind) = match typed_substate_key {
        TypedSubstateKey::TypeInfoModuleField(TypeInfoField::TypeInfo) => (
            SubstateType::TypeInfoModuleFieldTypeInfo,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::AccessRulesModuleField(AccessRulesField::AccessRules) => (
            SubstateType::AccessRulesModuleFieldAccessRules,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::RoyaltyModuleField(RoyaltyField::RoyaltyConfig) => (
            SubstateType::RoyaltyModuleFieldConfig,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::RoyaltyModuleField(RoyaltyField::RoyaltyAccumulator) => (
            SubstateType::RoyaltyModuleFieldAccumulator,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MetadataModuleEntryKey(_) => (
            SubstateType::MetadataModuleEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageField(
            PackageField::Info,
        )) => (SubstateType::PackageFieldCode, models::PartitionKind::Field),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageField(
            PackageField::CodeType,
        )) => (
            SubstateType::PackageFieldCodeType,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageField(
            PackageField::Code,
        )) => (SubstateType::PackageFieldCode, models::PartitionKind::Field),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageField(
            PackageField::Royalty,
        )) => (
            SubstateType::PackageFieldRoyalty,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::PackageField(
            PackageField::FunctionAccessRules,
        )) => (
            SubstateType::PackageFieldFunctionAccessRules,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleResourceField(
            FungibleResourceManagerField::Divisibility,
        )) => (
            SubstateType::FungibleResourceManagerFieldDivisibility,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleResourceField(
            FungibleResourceManagerField::TotalSupply,
        )) => (
            SubstateType::FungibleResourceManagerFieldTotalSupply,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceField(
            NonFungibleResourceManagerField::IdType,
        )) => (
            SubstateType::NonFungibleResourceManagerFieldIdType,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceField(
            NonFungibleResourceManagerField::TotalSupply,
        )) => (
            SubstateType::NonFungibleResourceManagerFieldTotalSupply,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceField(
            NonFungibleResourceManagerField::MutableFields,
        )) => (
            SubstateType::NonFungibleResourceManagerFieldMutableFields,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceData(_)) => (
            SubstateType::NonFungibleResourceManagerDataEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVaultField(
            FungibleVaultField::LiquidFungible,
        )) => (
            SubstateType::FungibleVaultFieldBalance,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVaultField(
            FungibleVaultField::LockedFungible,
        )) => {
            return Err(MappingError::SubstateKey {
                entity_address,
                partition_number,
                substate_key: api_substate_key,
                message: "LockedFungible".to_string(),
            })
        }
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVaultField(
            NonFungibleVaultField::LiquidNonFungible,
        )) => (
            SubstateType::NonFungibleVaultFieldBalance,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVaultField(
            NonFungibleVaultField::LockedNonFungible,
        )) => {
            return Err(MappingError::SubstateKey {
                entity_address,
                partition_number,
                substate_key: api_substate_key,
                message: "LockedNonFungible".to_string(),
            })
        }
        TypedSubstateKey::MainModule(
            TypedMainModuleSubstateKey::NonFungibleVaultContentsIndexKey(_),
        ) => (
            SubstateType::NonFungibleVaultContentsIndexEntry,
            models::PartitionKind::Index,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::Config,
        )) => (
            SubstateType::ConsensusManagerFieldConfig,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::ConsensusManager,
        )) => (
            SubstateType::ConsensusManagerFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::CurrentProposalStatistic,
        )) => (
            SubstateType::ConsensusManagerFieldCurrentProposalStatistic,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::CurrentValidatorSet,
        )) => (
            SubstateType::ConsensusManagerFieldCurrentValidatorSet,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(
            TypedMainModuleSubstateKey::ConsensusManagerRegisteredValidatorsByStakeIndexKey(_),
        ) => (
            SubstateType::ConsensusManagerRegisteredValidatorsByStakeIndexEntry,
            models::PartitionKind::SortedIndex,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::CurrentTimeRoundedToMinutes,
        )) => (SubstateType::ConsensusManagerFieldCurrentTimeRoundedToMinutes, models::PartitionKind::Field),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerField(
            ConsensusManagerField::CurrentTime,
        )) => (SubstateType::ConsensusManagerFieldCurrentTime, models::PartitionKind::Field),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ValidatorField(
            ValidatorField::Validator,
        )) => (
            SubstateType::ValidatorFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountField(AccountField::Account)) => (
            SubstateType::AccountFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountVaultIndexKey(_)) => (
            SubstateType::AccountVaultIndexEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountResourceDepositRuleIndexKey(_)) => (
            SubstateType::AccountDepositRuleIndexEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccessControllerField(
            AccessControllerField::AccessController,
        )) => (
            SubstateType::AccessControllerFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::GenericScryptoComponentField(
            ComponentField::State0,
        )) => (
            SubstateType::GenericScryptoComponentFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::GenericKeyValueStoreKey(_)) => (
            SubstateType::GenericKeyValueStoreEntry,
            models::PartitionKind::KeyValue,
        ),
    };

    let entity_module = match typed_substate_key {
        TypedSubstateKey::TypeInfoModuleField(_) => models::EntityModule::TypeInfo,
        TypedSubstateKey::AccessRulesModuleField(_) => models::EntityModule::AccessRules,
        TypedSubstateKey::RoyaltyModuleField(_) => models::EntityModule::Royalty,
        TypedSubstateKey::MetadataModuleEntryKey(_) => models::EntityModule::Metadata,
        TypedSubstateKey::MainModule(_) => models::EntityModule::Main,
    };

    Ok(models::SubstateId {
        entity_type: to_api_entity_type(entity_type),
        entity_address,
        entity_module,
        partition_kind,
        partition_number: partition_number.0 as i32,
        substate_type,
        substate_key: Some(api_substate_key),
    })
}

pub fn to_api_substate_key(substate_key: &SubstateKey) -> models::SubstateKey {
    let db_sort_key_hex = to_hex(SpreadPrefixKeyMapper::to_db_sort_key(substate_key).0);
    match substate_key {
        SubstateKey::Tuple(tuple_key) => models::SubstateKey::FieldSubstateKey {
            db_sort_key_hex,
            id: to_api_u8_as_i32(*tuple_key),
        },
        SubstateKey::Map(map_key) => models::SubstateKey::MapSubstateKey {
            db_sort_key_hex,
            key_hex: to_hex(map_key),
        },
        SubstateKey::Sorted((sort_key, map_key)) => models::SubstateKey::SortedSubstateKey {
            db_sort_key_hex,
            sort_prefix: to_api_u16_as_i32(*sort_key),
            key_hex: to_hex(map_key),
        },
    }
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

pub fn to_api_object_module_id(object_module_id: &ObjectModuleId) -> models::ObjectModuleId {
    match object_module_id {
        ObjectModuleId::Main => models::ObjectModuleId::Main,
        ObjectModuleId::Metadata => models::ObjectModuleId::Metadata,
        ObjectModuleId::Royalty => models::ObjectModuleId::Royalty,
        ObjectModuleId::AccessRules => models::ObjectModuleId::AccessRules,
    }
}
