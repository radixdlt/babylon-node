use crate::core_api::models;
use crate::core_api::*;
use crate::engine_prelude::*;
use models::SubstateType;

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
        .address_encoder
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
        EntityType::GlobalNonFungibleResourceManager => {
            models::EntityType::GlobalNonFungibleResource
        }
        EntityType::GlobalConsensusManager => models::EntityType::GlobalConsensusManager,
        EntityType::GlobalValidator => models::EntityType::GlobalValidator,
        EntityType::GlobalAccessController => models::EntityType::GlobalAccessController,
        EntityType::GlobalAccount => models::EntityType::GlobalAccount,
        EntityType::GlobalAccountLocker => models::EntityType::GlobalAccountLocker,
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

#[tracing::instrument(skip_all)]
pub fn to_api_partition_id(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
) -> Result<models::PartitionId, MappingError> {
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    let entity_address = to_api_entity_address(context, node_id)?;
    let entity_module = to_api_entity_module(partition_number, entity_address.as_ref())?;
    let partition_kind =
        to_api_partition_kind(entity_type, partition_number, entity_address.as_ref())?;
    Ok(models::PartitionId {
        entity_type: to_api_entity_type(entity_type),
        entity_address,
        entity_module,
        partition_kind,
        partition_number: to_api_u8_as_i32(partition_number.0),
    })
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
        TypedSubstateKey::BootLoader(TypedBootLoaderSubstateKey::BootLoaderField(
            BootLoaderField::SystemBoot,
        )) => (
            SubstateType::BootLoaderModuleFieldSystemBoot,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::BootLoader(TypedBootLoaderSubstateKey::BootLoaderField(
            BootLoaderField::VmBoot,
        )) => (
            SubstateType::BootLoaderModuleFieldVmBoot,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::BootLoader(TypedBootLoaderSubstateKey::BootLoaderField(
            BootLoaderField::KernelBoot,
        )) => (
            SubstateType::BootLoaderModuleFieldKernelBoot,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::TypeInfo(TypedTypeInfoSubstateKey::TypeInfoField(
            TypeInfoField::TypeInfo,
        )) => (
            SubstateType::TypeInfoModuleFieldTypeInfo,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::RoleAssignmentModule(
            TypedRoleAssignmentSubstateKey::RoleAssignmentField(RoleAssignmentField::Owner),
        ) => (
            SubstateType::RoleAssignmentModuleFieldOwnerRole,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::RoleAssignmentModule(TypedRoleAssignmentSubstateKey::Rule(_)) => (
            SubstateType::RoleAssignmentModuleRuleEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::RoyaltyModule(TypedRoyaltyModuleSubstateKey::RoyaltyField(
            RoyaltyField::RoyaltyAccumulator,
        )) => (
            SubstateType::RoyaltyModuleFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::RoyaltyModule(
            TypedRoyaltyModuleSubstateKey::RoyaltyMethodRoyaltyEntryKey(_),
        ) => (
            SubstateType::RoyaltyModuleMethodRoyaltyEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(_)) => (
            SubstateType::MetadataModuleEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::Field(PackageField::RoyaltyAccumulator),
        )) => (
            SubstateType::PackageFieldRoyaltyAccumulator,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleResourceManager(
            FungibleResourceManagerTypedSubstateKey::Field(
                FungibleResourceManagerField::Divisibility,
            ),
        )) => (
            SubstateType::FungibleResourceManagerFieldDivisibility,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleResourceManager(
            FungibleResourceManagerTypedSubstateKey::Field(
                FungibleResourceManagerField::TotalSupply,
            ),
        )) => (
            SubstateType::FungibleResourceManagerFieldTotalSupply,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
            NonFungibleResourceManagerTypedSubstateKey::Field(
                NonFungibleResourceManagerField::IdType,
            ),
        )) => (
            SubstateType::NonFungibleResourceManagerFieldIdType,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
            NonFungibleResourceManagerTypedSubstateKey::Field(
                NonFungibleResourceManagerField::TotalSupply,
            ),
        )) => (
            SubstateType::NonFungibleResourceManagerFieldTotalSupply,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
            NonFungibleResourceManagerTypedSubstateKey::Field(
                NonFungibleResourceManagerField::MutableFields,
            ),
        )) => (
            SubstateType::NonFungibleResourceManagerFieldMutableFields,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
            NonFungibleResourceManagerTypedSubstateKey::DataKeyValueEntry(_),
        )) => (
            SubstateType::NonFungibleResourceManagerDataEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVault(
            FungibleVaultTypedSubstateKey::Field(FungibleVaultField::Balance),
        )) => (
            SubstateType::FungibleVaultFieldBalance,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVault(
            FungibleVaultTypedSubstateKey::Field(FungibleVaultField::FreezeStatus),
        )) => (
            SubstateType::FungibleVaultFieldFrozenStatus,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVault(
            FungibleVaultTypedSubstateKey::Field(FungibleVaultField::LockedBalance),
        )) => {
            return Err(MappingError::SubstateKey {
                entity_address,
                partition_number,
                substate_key: Box::new(api_substate_key),
                message: "LockedFungible".to_string(),
            })
        }
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::Field(NonFungibleVaultField::Balance),
        )) => (
            SubstateType::NonFungibleVaultFieldBalance,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::Field(NonFungibleVaultField::FreezeStatus),
        )) => (
            SubstateType::NonFungibleVaultFieldFrozenStatus,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::Field(NonFungibleVaultField::LockedResource),
        )) => {
            return Err(MappingError::SubstateKey {
                entity_address,
                partition_number,
                substate_key: Box::new(api_substate_key),
                message: "LockedNonFungible".to_string(),
            })
        }
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::NonFungibleIndexEntry(_),
        )) => (
            SubstateType::NonFungibleVaultContentsIndexEntry,
            models::PartitionKind::Index,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::Configuration),
        )) => (
            SubstateType::ConsensusManagerFieldConfig,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::State),
        )) => (
            SubstateType::ConsensusManagerFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(
                ConsensusManagerField::CurrentProposalStatistic,
            ),
        )) => (
            SubstateType::ConsensusManagerFieldCurrentProposalStatistic,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::CurrentValidatorSet),
        )) => (
            SubstateType::ConsensusManagerFieldCurrentValidatorSet,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::RegisteredValidatorByStakeSortedIndexEntry(_),
        )) => (
            SubstateType::ConsensusManagerRegisteredValidatorsByStakeIndexEntry,
            models::PartitionKind::SortedIndex,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::ProposerMinuteTimestamp),
        )) => (
            SubstateType::ConsensusManagerFieldCurrentTimeRoundedToMinutes,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::ProposerMilliTimestamp),
        )) => (
            SubstateType::ConsensusManagerFieldCurrentTime,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManager(
            ConsensusManagerTypedSubstateKey::Field(ConsensusManagerField::ValidatorRewards),
        )) => (
            SubstateType::ConsensusManagerFieldValidatorRewards,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ValidatorField(
            ValidatorTypedSubstateKey::Field(ValidatorField::State),
        )) => (
            SubstateType::ValidatorFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ValidatorField(
            ValidatorTypedSubstateKey::Field(ValidatorField::ProtocolUpdateReadinessSignal),
        )) => (
            SubstateType::ValidatorFieldProtocolUpdateReadinessSignal,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::Field(AccountField::DepositRule),
        )) => (
            SubstateType::AccountFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::ResourceVaultKeyValueEntry(_),
        )) => (
            SubstateType::AccountVaultEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::ResourcePreferenceKeyValueEntry(_),
        )) => (
            SubstateType::AccountResourcePreferenceEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Account(
            AccountTypedSubstateKey::AuthorizedDepositorKeyValueEntry(_),
        )) => (
            SubstateType::AccountAuthorizedDepositorEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccountLocker(
            AccountLockerTypedSubstateKey::AccountClaimsKeyValueEntry(_),
        )) => (
            SubstateType::AccountLockerAccountClaimsEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::AccessController(
            AccessControllerV2TypedSubstateKey::Field(AccessControllerV2Field::State),
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
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::OneResourcePool(
            OneResourcePoolTypedSubstateKey::Field(OneResourcePoolField::State),
        )) => (
            SubstateType::OneResourcePoolFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::TwoResourcePool(
            TwoResourcePoolTypedSubstateKey::Field(TwoResourcePoolField::State),
        )) => (
            SubstateType::TwoResourcePoolFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::MultiResourcePool(
            MultiResourcePoolTypedSubstateKey::Field(MultiResourcePoolField::State),
        )) => (
            SubstateType::MultiResourcePoolFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionDefinitionKeyValueEntry(_),
        )) => (
            SubstateType::PackageBlueprintDefinitionEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionDependenciesKeyValueEntry(_),
        )) => (
            SubstateType::PackageBlueprintDependenciesEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeVmTypeKeyValueEntry(_),
        )) => (
            SubstateType::PackageCodeVmTypeEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeOriginalCodeKeyValueEntry(_),
        )) => (
            SubstateType::PackageCodeOriginalCodeEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::CodeInstrumentedCodeKeyValueEntry(_),
        )) => (
            SubstateType::PackageCodeInstrumentedCodeEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionRoyaltyConfigKeyValueEntry(_),
        )) => (
            SubstateType::PackageBlueprintRoyaltyEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::BlueprintVersionAuthConfigKeyValueEntry(_),
        )) => (
            SubstateType::PackageBlueprintAuthTemplateEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::TransactionTrackerField(_)) => (
            SubstateType::TransactionTrackerFieldState,
            models::PartitionKind::Field,
        ),
        TypedSubstateKey::MainModule(
            TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(_),
        ) => (
            SubstateType::TransactionTrackerCollectionEntry,
            models::PartitionKind::KeyValue,
        ),
        TypedSubstateKey::Schema(TypedSchemaSubstateKey::SchemaKey(_)) => {
            (SubstateType::SchemaEntry, models::PartitionKind::KeyValue)
        }
    };

    let entity_module = match typed_substate_key {
        TypedSubstateKey::BootLoader(_) => models::EntityModule::BootLoader,
        TypedSubstateKey::TypeInfo(_) => models::EntityModule::TypeInfo,
        TypedSubstateKey::RoleAssignmentModule(_) => models::EntityModule::RoleAssignment,
        TypedSubstateKey::RoyaltyModule(_) => models::EntityModule::Royalty,
        TypedSubstateKey::MetadataModule(_) => models::EntityModule::Metadata,
        TypedSubstateKey::MainModule(_) => models::EntityModule::Main,
        TypedSubstateKey::Schema(_) => models::EntityModule::Schema,
    };

    Ok(models::SubstateId {
        entity_type: to_api_entity_type(entity_type),
        entity_address,
        entity_module,
        partition_kind,
        partition_number: to_api_u8_as_i32(partition_number.0),
        substate_type,
        substate_key: Some(api_substate_key),
    })
}

pub fn to_api_substate_key(substate_key: &SubstateKey) -> models::SubstateKey {
    let db_sort_key_hex = to_hex(SpreadPrefixKeyMapper::to_db_sort_key(substate_key).0);
    match substate_key {
        SubstateKey::Field(field_key) => models::SubstateKey::FieldSubstateKey {
            db_sort_key_hex,
            id: to_api_u8_as_i32(*field_key),
        },
        SubstateKey::Map(map_key) => models::SubstateKey::MapSubstateKey {
            db_sort_key_hex,
            key_hex: to_hex(map_key),
        },
        SubstateKey::Sorted((sort_key, map_key)) => models::SubstateKey::SortedSubstateKey {
            db_sort_key_hex,
            sort_prefix_hex: to_hex(sort_key),
            key_hex: to_hex(map_key),
        },
    }
}

pub fn to_api_entity_module(
    partition_number: PartitionNumber,
    entity_address: &str, // for error-surfacing purposes only
) -> Result<models::EntityModule, MappingError> {
    Ok(match partition_number {
        TYPE_INFO_FIELD_PARTITION => models::EntityModule::TypeInfo,
        SCHEMAS_PARTITION => models::EntityModule::Schema,
        METADATA_BASE_PARTITION => models::EntityModule::Metadata,
        ROYALTY_FIELDS_PARTITION | ROYALTY_CONFIG_PARTITION => models::EntityModule::Royalty,
        ROLE_ASSIGNMENT_FIELDS_PARTITION | ROLE_ASSIGNMENT_ROLE_DEF_PARTITION => {
            models::EntityModule::RoleAssignment
        }
        _ => {
            if partition_number > MAIN_BASE_PARTITION {
                models::EntityModule::Main
            } else {
                return Err(MappingError::PartitionNumber {
                    entity_address: entity_address.to_owned(),
                    partition_number,
                    message: "unknown partition".to_owned(),
                });
            }
        }
    })
}

pub fn to_api_partition_kind(
    entity_type: EntityType,
    partition_number: PartitionNumber,
    entity_address: &str, // only for error-surfacing purposes only
) -> Result<models::PartitionKind, MappingError> {
    Ok(match partition_number {
        TYPE_INFO_FIELD_PARTITION => models::PartitionKind::Field,
        SCHEMAS_PARTITION => models::PartitionKind::KeyValue,
        METADATA_BASE_PARTITION => models::PartitionKind::KeyValue,
        ROYALTY_FIELDS_PARTITION => models::PartitionKind::Field,
        ROYALTY_CONFIG_PARTITION => models::PartitionKind::KeyValue,
        ROLE_ASSIGNMENT_FIELDS_PARTITION => models::PartitionKind::Field,
        ROLE_ASSIGNMENT_ROLE_DEF_PARTITION => models::PartitionKind::KeyValue,
        _ => {
            if partition_number >= MAIN_BASE_PARTITION {
                to_api_object_module_partition_kind(
                    entity_type,
                    PartitionOffset(partition_number.0 - MAIN_BASE_PARTITION.0),
                )
                .map_err(|_| MappingError::PartitionNumber {
                    entity_address: entity_address.to_owned(),
                    partition_number,
                    message: "invalid offset within the main partition".to_owned(),
                })?
            } else {
                return Err(MappingError::PartitionNumber {
                    entity_address: entity_address.to_owned(),
                    partition_number,
                    message: "unknown partition".to_owned(),
                });
            }
        }
    })
}

pub fn to_api_object_module_partition_kind(
    entity_type: EntityType,
    partition_offset: PartitionOffset,
) -> Result<models::PartitionKind, ()> {
    Ok(match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            models::PartitionKind::Field
        }
        EntityType::GlobalPackage => match PackagePartitionOffset::try_from(partition_offset)? {
            PackagePartitionOffset::Field => models::PartitionKind::Field,
            PackagePartitionOffset::BlueprintVersionDefinitionKeyValue
            | PackagePartitionOffset::BlueprintVersionDependenciesKeyValue
            | PackagePartitionOffset::BlueprintVersionRoyaltyConfigKeyValue
            | PackagePartitionOffset::BlueprintVersionAuthConfigKeyValue
            | PackagePartitionOffset::CodeVmTypeKeyValue
            | PackagePartitionOffset::CodeOriginalCodeKeyValue
            | PackagePartitionOffset::CodeInstrumentedCodeKeyValue => {
                models::PartitionKind::KeyValue
            }
        },
        EntityType::GlobalFungibleResourceManager => {
            match FungibleResourceManagerPartitionOffset::try_from(partition_offset)? {
                FungibleResourceManagerPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalNonFungibleResourceManager => {
            match NonFungibleResourceManagerPartitionOffset::try_from(partition_offset)? {
                NonFungibleResourceManagerPartitionOffset::Field => models::PartitionKind::Field,
                NonFungibleResourceManagerPartitionOffset::DataKeyValue => {
                    models::PartitionKind::KeyValue
                }
            }
        }
        EntityType::GlobalConsensusManager => {
            match ConsensusManagerPartitionOffset::try_from(partition_offset)? {
                ConsensusManagerPartitionOffset::Field => models::PartitionKind::Field,
                ConsensusManagerPartitionOffset::RegisteredValidatorByStakeSortedIndex => {
                    models::PartitionKind::SortedIndex
                }
            }
        }
        EntityType::GlobalValidator => {
            match ValidatorPartitionOffset::try_from(partition_offset)? {
                ValidatorPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalAccessController => {
            match AccessControllerPartitionOffset::try_from(partition_offset)? {
                AccessControllerPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalVirtualSecp256k1Account
        | EntityType::GlobalVirtualEd25519Account
        | EntityType::GlobalAccount => match AccountPartitionOffset::try_from(partition_offset)? {
            AccountPartitionOffset::Field => models::PartitionKind::Field,
            AccountPartitionOffset::ResourceVaultKeyValue => models::PartitionKind::KeyValue,
            AccountPartitionOffset::ResourcePreferenceKeyValue => models::PartitionKind::KeyValue,
            AccountPartitionOffset::AuthorizedDepositorKeyValue => models::PartitionKind::KeyValue,
        },
        EntityType::GlobalAccountLocker => {
            match AccountLockerPartitionOffset::try_from(partition_offset)? {
                AccountLockerPartitionOffset::AccountClaimsKeyValue => {
                    models::PartitionKind::KeyValue
                }
            }
        }
        EntityType::GlobalVirtualSecp256k1Identity
        | EntityType::GlobalVirtualEd25519Identity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => {
            match FungibleVaultPartitionOffset::try_from(partition_offset)? {
                FungibleVaultPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::InternalNonFungibleVault => {
            match NonFungibleVaultPartitionOffset::try_from(partition_offset)? {
                NonFungibleVaultPartitionOffset::Field => models::PartitionKind::Field,
                NonFungibleVaultPartitionOffset::NonFungibleIndex => models::PartitionKind::Index,
            }
        }
        EntityType::GlobalOneResourcePool => {
            match OneResourcePoolPartitionOffset::try_from(partition_offset)? {
                OneResourcePoolPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalTwoResourcePool => {
            match TwoResourcePoolPartitionOffset::try_from(partition_offset)? {
                TwoResourcePoolPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalMultiResourcePool => {
            match MultiResourcePoolPartitionOffset::try_from(partition_offset)? {
                MultiResourcePoolPartitionOffset::Field => models::PartitionKind::Field,
            }
        }
        EntityType::GlobalTransactionTracker => match partition_offset {
            PartitionOffset(0) => models::PartitionKind::Field,
            _ => models::PartitionKind::KeyValue,
        },
        EntityType::InternalKeyValueStore => models::PartitionKind::KeyValue,
    })
}

pub fn extract_global_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<GlobalAddress, ExtractionError> {
    GlobalAddress::try_from_bech32(&extraction_context.address_decoder, package_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_package_address(
    extraction_context: &ExtractionContext,
    package_address: &str,
) -> Result<PackageAddress, ExtractionError> {
    PackageAddress::try_from_bech32(&extraction_context.address_decoder, package_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_component_address(
    extraction_context: &ExtractionContext,
    component_address: &str,
) -> Result<ComponentAddress, ExtractionError> {
    ComponentAddress::try_from_bech32(&extraction_context.address_decoder, component_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_resource_address(
    extraction_context: &ExtractionContext,
    resource_address: &str,
) -> Result<ResourceAddress, ExtractionError> {
    ResourceAddress::try_from_bech32(&extraction_context.address_decoder, resource_address)
        .ok_or(ExtractionError::InvalidAddress)
}

pub fn extract_non_fungible_id_from_simple_representation(
    simple_rep: &str,
) -> Result<NonFungibleLocalId, ExtractionError> {
    Ok(NonFungibleLocalId::from_str(simple_rep)?)
}

pub fn to_api_attached_module_id(module_id: &AttachedModuleId) -> models::AttachedModuleId {
    match module_id {
        AttachedModuleId::Metadata => models::AttachedModuleId::Metadata,
        AttachedModuleId::Royalty => models::AttachedModuleId::Royalty,
        AttachedModuleId::RoleAssignment => models::AttachedModuleId::RoleAssignment,
    }
}

pub fn to_api_module_id(object_module_id: &ModuleId) -> models::ModuleId {
    match object_module_id {
        ModuleId::Main => models::ModuleId::Main,
        ModuleId::Metadata => models::ModuleId::Metadata,
        ModuleId::Royalty => models::ModuleId::Royalty,
        ModuleId::RoleAssignment => models::ModuleId::RoleAssignment,
    }
}
