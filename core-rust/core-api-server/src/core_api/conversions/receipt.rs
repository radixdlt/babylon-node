#![allow(clippy::too_many_arguments)]

use super::addressing::*;
use crate::core_api::*;
use crate::engine_prelude::*;

use state_manager::{
    ApplicationEvent, BySubstate, DetailedTransactionOutcome, LedgerStateChanges,
    LocalTransactionReceipt, PartitionChangeAction, PartitionReference, ReadableRocks,
    StateManagerDatabase, SubstateChangeAction, SubstateReference,
};

pub fn to_api_receipt(
    database: Option<&StateManagerDatabase<impl ReadableRocks>>,
    context: &MappingContext,
    receipt: LocalTransactionReceipt,
) -> Result<models::TransactionReceipt, MappingError> {
    let local_execution = receipt.local_execution;
    let (status, output, error_message) = match local_execution.outcome {
        DetailedTransactionOutcome::Success(output) => {
            (models::TransactionStatus::Succeeded, Some(output), None)
        }
        DetailedTransactionOutcome::Failure(error) => (
            models::TransactionStatus::Failed,
            None,
            Some(format!("{error:?}")),
        ),
    };

    let on_ledger = receipt.on_ledger;

    let api_state_updates = to_api_state_updates(
        database,
        context,
        &local_execution.substates_system_structure,
        &on_ledger.state_changes,
        &local_execution.state_update_summary,
    )?;

    let api_fee_summary = to_api_fee_summary(context, &local_execution.fee_summary)?;
    let api_costing_parameters = to_api_costing_parameters(
        context,
        &local_execution.engine_costing_parameters,
        &local_execution.transaction_costing_parameters,
    )?;
    let api_fee_source = to_api_fee_source(context, &local_execution.fee_source)?;
    let api_fee_destination = to_api_fee_destination(context, &local_execution.fee_destination)?;

    let api_events = on_ledger
        .application_events
        .into_iter()
        .map(|event| to_api_event(context, &local_execution.events_system_structure, event))
        .collect::<Result<Vec<_>, _>>()?;

    let api_output = output
        .map(|output| {
            output
                .into_iter()
                .map(|line_output| to_api_sbor_data_from_bytes(context, &line_output))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let next_epoch = local_execution
        .next_epoch
        .map(|epoch_change_event| to_api_next_epoch(context, epoch_change_event))
        .transpose()?
        .map(Box::new);

    Ok(models::TransactionReceipt {
        status,
        fee_summary: Box::new(api_fee_summary),
        costing_parameters: Box::new(api_costing_parameters),
        fee_source: Some(Box::new(api_fee_source)),
        fee_destination: Some(Box::new(api_fee_destination)),
        state_updates: Box::new(api_state_updates),
        events: Some(api_events),
        output: api_output,
        next_epoch,
        error_message,
    })
}

pub fn create_typed_substate_key(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, MappingError> {
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    to_typed_substate_key(entity_type, partition_number, substate_key).map_err(|msg| {
        MappingError::SubstateKey {
            entity_address: to_api_entity_address(context, node_id)
                .unwrap_or_else(|_| format!("NodeId[{}]", to_hex(node_id.as_bytes()))),
            partition_number,
            substate_key: Box::new(to_api_substate_key(substate_key)),
            message: msg,
        }
    })
}

pub fn create_typed_substate_value(
    typed_substate_key: &TypedSubstateKey,
    raw_value: &[u8],
) -> Result<TypedSubstateValue, MappingError> {
    to_typed_substate_value(typed_substate_key, raw_value).map_err(|msg| {
        MappingError::SubstateValue {
            bytes: raw_value.to_vec(),
            message: msg,
        }
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_value(
    context: &MappingContext,
    state_mapping_lookups: &StateMappingLookups,
    typed_substate_key: &TypedSubstateKey,
    raw_value: &[u8],
) -> Result<models::SubstateValue, MappingError> {
    Ok(models::SubstateValue {
        substate_hex: if context.substate_options.include_raw {
            Some(to_hex(raw_value))
        } else {
            None
        },
        substate_data_hash: if context.substate_options.include_hash {
            Some(to_hex(hash(raw_value)))
        } else {
            None
        },
        substate_data: if context.substate_options.include_typed {
            let typed_value = create_typed_substate_value(typed_substate_key, raw_value)?;
            Some(Box::new(to_api_substate(
                context,
                state_mapping_lookups,
                typed_substate_key,
                &typed_value,
            )?))
        } else {
            None
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_system_structure(
    context: &MappingContext,
    system_structure: &SubstateSystemStructure,
) -> Result<models::SubstateSystemStructure, MappingError> {
    Ok(match system_structure {
        SubstateSystemStructure::SystemField(SystemFieldStructure { field_kind }) => {
            match field_kind {
                SystemFieldKind::TypeInfo => {
                    models::SubstateSystemStructure::SystemFieldStructure {
                        field_kind: models::SystemFieldKind::TypeInfo,
                        boot_loader_type: None,
                    }
                }
                SystemFieldKind::BootLoader(boot_loader) => {
                    models::SubstateSystemStructure::SystemFieldStructure {
                        field_kind: models::SystemFieldKind::BootLoader,
                        boot_loader_type: Some(match boot_loader {
                            BootLoaderFieldKind::KernelBoot => models::BootLoaderType::KernelBoot,
                            BootLoaderFieldKind::SystemBoot => models::BootLoaderType::SystemBoot,
                            BootLoaderFieldKind::VmBoot => models::BootLoaderType::VmBoot,
                        }),
                    }
                }
            }
        }
        SubstateSystemStructure::SystemSchema => {
            models::SubstateSystemStructure::SystemSchemaStructure {}
        }
        SubstateSystemStructure::KeyValueStoreEntry(entry) => {
            let KeyValueStoreEntryStructure {
                key_full_type_id,
                value_full_type_id,
            } = entry;
            models::SubstateSystemStructure::KeyValueStoreEntryStructure {
                key_full_type_id: Box::new(to_api_fully_scoped_type_id(context, key_full_type_id)?),
                value_full_type_id: Box::new(to_api_fully_scoped_type_id(
                    context,
                    value_full_type_id,
                )?),
            }
        }
        SubstateSystemStructure::ObjectField(field) => {
            models::SubstateSystemStructure::ObjectFieldStructure {
                value_schema: Box::new(to_api_object_substate_type_reference(
                    context,
                    &field.value_schema,
                )?),
            }
        }
        SubstateSystemStructure::ObjectKeyValuePartitionEntry(entry) => {
            let KeyValuePartitionEntryStructure {
                key_schema,
                value_schema,
            } = entry;
            models::SubstateSystemStructure::ObjectKeyValuePartitionEntryStructure {
                key_schema: Box::new(to_api_object_substate_type_reference(context, key_schema)?),
                value_schema: Box::new(to_api_object_substate_type_reference(
                    context,
                    value_schema,
                )?),
            }
        }
        SubstateSystemStructure::ObjectIndexPartitionEntry(entry) => {
            let IndexPartitionEntryStructure {
                key_schema,
                value_schema,
            } = entry;
            models::SubstateSystemStructure::ObjectIndexPartitionEntryStructure {
                key_schema: Box::new(to_api_object_substate_type_reference(context, key_schema)?),
                value_schema: Box::new(to_api_object_substate_type_reference(
                    context,
                    value_schema,
                )?),
            }
        }
        SubstateSystemStructure::ObjectSortedIndexPartitionEntry(entry) => {
            let SortedIndexPartitionEntryStructure {
                key_schema,
                value_schema,
            } = entry;
            models::SubstateSystemStructure::ObjectSortedIndexPartitionEntryStructure {
                key_schema: Box::new(to_api_object_substate_type_reference(context, key_schema)?),
                value_schema: Box::new(to_api_object_substate_type_reference(
                    context,
                    value_schema,
                )?),
            }
        }
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_object_substate_type_reference(
    context: &MappingContext,
    substate_type_reference: &ObjectSubstateTypeReference,
) -> Result<models::ObjectSubstateTypeReference, MappingError> {
    Ok(match substate_type_reference {
        ObjectSubstateTypeReference::Package(package) => {
            let PackageTypeReference { full_type_id } = package;
            models::ObjectSubstateTypeReference::PackageObjectSubstateTypeReference {
                full_type_id: Box::new(to_api_fully_scoped_type_id(context, full_type_id)?),
            }
        }
        ObjectSubstateTypeReference::ObjectInstance(instance) => {
            let ObjectInstanceTypeReference {
                instance_type_id,
                resolved_full_type_id,
            } = instance;
            models::ObjectSubstateTypeReference::ObjectInstanceTypeReference {
                resolved_full_type_id: Box::new(to_api_fully_scoped_type_id(
                    context,
                    resolved_full_type_id,
                )?),
                generic_index: to_api_u8_as_i32(*instance_type_id),
            }
        }
    })
}

pub fn to_api_fully_scoped_type_id(
    context: &MappingContext,
    fully_scoped_type_id: &FullyScopedTypeId<impl AsRef<NodeId>>,
) -> Result<models::FullyScopedTypeId, MappingError> {
    let FullyScopedTypeId(address, schema_hash, local_type_id) = fully_scoped_type_id;
    Ok(models::FullyScopedTypeId {
        entity_address: to_api_entity_address(context, address.as_ref())?,
        schema_hash: to_api_schema_hash(schema_hash),
        local_type_id: Box::new(to_api_local_type_id(context, local_type_id)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_next_epoch(
    context: &MappingContext,
    epoch_change_event: EpochChangeEvent,
) -> Result<models::NextEpoch, MappingError> {
    let EpochChangeEvent {
        epoch,
        validator_set,
        significant_protocol_update_readiness,
    } = epoch_change_event;
    let next_epoch = models::NextEpoch {
        epoch: to_api_epoch(context, epoch)?,
        validators: validator_set
            .validators_by_stake_desc
            .into_iter()
            .map(|(address, validator)| to_api_active_validator(context, &address, &validator))
            .collect::<Result<_, _>>()?,
        significant_protocol_update_readiness: Some(
            significant_protocol_update_readiness
                .into_iter()
                .map(|(readiness_signal_name, signalled_stake)| {
                    models::SignificantProtocolUpdateReadinessEntry {
                        readiness_signal_name,
                        signalled_stake: signalled_stake.to_string(),
                    }
                })
                .collect(),
        ),
    };
    Ok(next_epoch)
}

#[tracing::instrument(skip_all)]
pub fn to_api_state_updates(
    database: Option<&StateManagerDatabase<impl ReadableRocks>>,
    context: &MappingContext,
    system_structures: &BySubstate<SubstateSystemStructure>,
    state_changes: &LedgerStateChanges,
    state_update_summary: &StateUpdateSummary,
) -> Result<models::StateUpdates, MappingError> {
    let LedgerStateChanges {
        partition_level_changes,
        substate_level_changes,
    } = state_changes;

    let deleted_partitions = partition_level_changes
        .iter()
        .map(|(partition_ref, action)| match action {
            PartitionChangeAction::Delete => {
                let PartitionReference(node_id, partition_num) = partition_ref;
                to_api_partition_id(context, &node_id, partition_num)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut created_substates = Vec::new();
    let mut updated_substates = Vec::new();
    let mut deleted_substates = Vec::new();

    // Step 1 - First, build actions
    let mut changes_to_map = Vec::new();
    for (substate_reference, action) in substate_level_changes.iter() {
        let SubstateReference(node_id, partition_number, substate_key) = &substate_reference;
        let typed_substate_key =
            create_typed_substate_key(context, node_id, *partition_number, substate_key)?;
        if !typed_substate_key.value_is_mappable() {
            continue;
        }
        changes_to_map.push((substate_reference, typed_substate_key, action))
    }

    // Step 2 - Build supplementary lookups from the database
    let state_mapping_lookups =
        StateMappingLookups::create_from_database(database, &changes_to_map)?;

    // Step 3 - Map the change actions
    for (substate_reference, typed_substate_key, action) in changes_to_map.into_iter() {
        let SubstateReference(node_id, partition_number, substate_key) = substate_reference;
        let system_structure = system_structures
            .get(&node_id, &partition_number, &substate_key)
            .ok_or(MappingError::MissingSystemStructure {
                message: format!(
                    "Missing system structure for substate {:?}:{:?}:{:?}",
                    node_id, partition_number, substate_key
                ),
            })?;
        let system_structure = Some(to_api_substate_system_structure(context, system_structure)?);
        let substate_id = Box::new(to_api_substate_id(
            context,
            &node_id,
            partition_number,
            &substate_key,
            &typed_substate_key,
        )?);
        match action {
            SubstateChangeAction::Create { new } => {
                created_substates.push(models::CreatedSubstate {
                    substate_id,
                    value: Box::new(to_api_substate_value(
                        context,
                        &state_mapping_lookups,
                        &typed_substate_key,
                        new,
                    )?),
                    system_structure,
                });
            }
            SubstateChangeAction::Update { previous, new } => {
                updated_substates.push(models::UpdatedSubstate {
                    substate_id,
                    new_value: Box::new(to_api_substate_value(
                        context,
                        &state_mapping_lookups,
                        &typed_substate_key,
                        new,
                    )?),
                    previous_value: if context.substate_options.include_previous {
                        Some(Box::new(to_api_substate_value(
                            context,
                            &state_mapping_lookups,
                            &typed_substate_key,
                            previous,
                        )?))
                    } else {
                        None
                    },
                    system_structure,
                });
            }
            SubstateChangeAction::Delete { previous } => {
                deleted_substates.push(models::DeletedSubstate {
                    substate_id,
                    previous_value: if context.substate_options.include_previous {
                        Some(Box::new(to_api_substate_value(
                            context,
                            &state_mapping_lookups,
                            &typed_substate_key,
                            previous,
                        )?))
                    } else {
                        None
                    },
                    system_structure,
                });
            }
        }
    }

    let mut new_global_entities = Vec::new();
    for package_address in &state_update_summary.new_packages {
        new_global_entities.push(to_api_entity_reference(
            context,
            package_address.as_node_id(),
        )?);
    }
    for component_address in &state_update_summary.new_components {
        new_global_entities.push(to_api_entity_reference(
            context,
            component_address.as_node_id(),
        )?);
    }
    for resource_address in &state_update_summary.new_resources {
        new_global_entities.push(to_api_entity_reference(
            context,
            resource_address.as_node_id(),
        )?);
    }

    Ok(models::StateUpdates {
        deleted_partitions,
        created_substates,
        updated_substates,
        deleted_substates,
        new_global_entities,
    })
}

/// This lookup was introduced near mainnet launch, to avoid needing Gateway to do the resolution of a
/// `GenericResolution::Remote(BlueprintTypeIdentifier)` into a `FullyScopedTypeId` in the data aggregator.
///
/// It is not ideal, and more of a pragamatic workaround to a pre-launch problem.
///
/// The `GenericResolution` is only found in the TypeInfo substate, so we first filter to only TypeInfo substates,
/// and then extract any relevant blueprints, and create a local lookup of those types, for the main mapping steps.
#[derive(Default)]
pub struct StateMappingLookups {
    blueprint_type_lookups: Option<IndexMap<BlueprintId, IndexMap<String, ScopedTypeId>>>,
}

impl StateMappingLookups {
    pub fn create_from_database(
        database: Option<&StateManagerDatabase<impl ReadableRocks>>,
        changes_to_map: &[(SubstateReference, TypedSubstateKey, &SubstateChangeAction)],
    ) -> Result<Self, MappingError> {
        let Some(database) = database else {
            return Ok(Self::default());
        };
        // First - filter to creating only the typed values which will be needed by lookup construction
        let mut typed_values = Vec::new();
        for (_, typed_substate_key, action) in changes_to_map.iter() {
            if matches!(typed_substate_key, TypedSubstateKey::TypeInfo(_)) {
                extract_typed_values(&mut typed_values, typed_substate_key, action)?;
            }
        }
        Ok(Self {
            blueprint_type_lookups: Some(Self::create_blueprint_type_lookups(
                database,
                &typed_values,
            )?),
        })
    }

    pub fn resolve_generic_remote(
        &self,
        blueprint_type_identifier: &BlueprintTypeIdentifier,
    ) -> Result<Option<FullyScopedTypeId<NodeId>>, MappingError> {
        let Some(lookup) = &self.blueprint_type_lookups else {
            return Ok(None);
        };
        let BlueprintTypeIdentifier {
            package_address,
            blueprint_name,
            type_name,
        } = blueprint_type_identifier;
        let package_types = lookup.get(&BlueprintId {
            package_address: *package_address,
            blueprint_name: blueprint_name.clone(),
        }).ok_or_else(|| MappingError::CouldNotResolveRemoteGenericSubstitution {
            message: "Could not find package in existing lookup - likely the lookup was somehow created incomplete".to_string()
        })?;
        let resolved = package_types.get(type_name).cloned().ok_or_else(|| {
            MappingError::CouldNotResolveRemoteGenericSubstitution {
                message: "Could not find type in package lookup".to_string(),
            }
        })?;
        Ok(Some(resolved.under_node(package_address.into_node_id())))
    }

    fn create_blueprint_type_lookups(
        database: &StateManagerDatabase<impl ReadableRocks>,
        typed_values: &[TypedSubstateValue],
    ) -> Result<IndexMap<BlueprintId, IndexMap<String, ScopedTypeId>>, MappingError> {
        // Step 1 - work out what database reads we need to do
        let mut blueprints_to_fetch_types = IndexSet::new();
        for typed_value in typed_values {
            if let TypedSubstateValue::TypeInfoModule(TypedTypeInfoModuleSubstateValue::TypeInfo(
                type_info,
            )) = typed_value
            {
                match type_info {
                    TypeInfoSubstate::Object(object_type_info) => {
                        for generic_substitution in
                            &object_type_info.blueprint_info.generic_substitutions
                        {
                            register_blueprint_for_adding_to_type_lookup(
                                &mut blueprints_to_fetch_types,
                                generic_substitution,
                            );
                        }
                    }
                    TypeInfoSubstate::KeyValueStore(key_value_store_type_info) => {
                        let KeyValueStoreGenericSubstitutions {
                            key_generic_substitution,
                            value_generic_substitution,
                            allow_ownership: _,
                        } = &key_value_store_type_info.generic_substitutions;
                        register_blueprint_for_adding_to_type_lookup(
                            &mut blueprints_to_fetch_types,
                            key_generic_substitution,
                        );
                        register_blueprint_for_adding_to_type_lookup(
                            &mut blueprints_to_fetch_types,
                            value_generic_substitution,
                        );
                    }
                    TypeInfoSubstate::GlobalAddressReservation(_) => {
                        return Err(MappingError::UnexpectedPersistedData {
                            message: "GlobalAddressReservation was persisted".to_string(),
                        })
                    }
                    TypeInfoSubstate::GlobalAddressPhantom(_) => {
                        return Err(MappingError::UnexpectedPersistedData {
                            message: "GlobalAddressPhantom was persisted".to_string(),
                        })
                    }
                }
            }
        }
        // Step 2 - Create the lookups from the database
        let mut blueprint_type_lookups = IndexMap::new();
        for (package_address, blueprint_name) in blueprints_to_fetch_types {
            let definition = database.get_mapped::<SpreadPrefixKeyMapper, PackageBlueprintVersionDefinitionEntrySubstate>(
                package_address.as_node_id(),
                PackagePartitionOffset::BlueprintVersionDefinitionKeyValue.as_main_partition(),
                &SubstateKey::Map(scrypto_encode(&PackageBlueprintVersionDefinitionKeyPayload::from_content_source(
                    BlueprintVersionKey::new_default(blueprint_name.clone())
                )).unwrap()),
            ).ok_or_else(|| MappingError::CouldNotResolveRemoteGenericSubstitution {
                message: "Could not find blueprint definition referenced in Remote Generic Substitution, but this was checked by the engine".to_string(),
            })?
            .into_value()
            .ok_or_else(|| MappingError::CouldNotResolveRemoteGenericSubstitution {
                message: "Blueprint definition was a deleted entry".to_string(),
            })?;
            blueprint_type_lookups.insert(
                BlueprintId {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                },
                definition
                    .fully_update_and_into_latest_version()
                    .interface
                    .types,
            );
        }
        Ok(blueprint_type_lookups)
    }
}

pub fn extract_typed_values(
    typed_values: &mut Vec<TypedSubstateValue>,
    typed_substate_key: &TypedSubstateKey,
    action: &SubstateChangeAction,
) -> Result<(), MappingError> {
    match action {
        SubstateChangeAction::Create { new } => {
            typed_values.push(create_typed_substate_value(typed_substate_key, new)?);
        }
        SubstateChangeAction::Update { new, previous } => {
            typed_values.push(create_typed_substate_value(typed_substate_key, new)?);
            typed_values.push(create_typed_substate_value(typed_substate_key, previous)?);
        }
        SubstateChangeAction::Delete { previous } => {
            typed_values.push(create_typed_substate_value(typed_substate_key, previous)?);
        }
    }
    Ok(())
}

pub fn register_blueprint_for_adding_to_type_lookup(
    blueprints_to_lookup: &mut IndexSet<(PackageAddress, String)>,
    generic_substitution: &GenericSubstitution,
) {
    match generic_substitution {
        GenericSubstitution::Local(_) => {}
        GenericSubstitution::Remote(BlueprintTypeIdentifier {
            package_address,
            blueprint_name,
            type_name: _,
        }) => {
            blueprints_to_lookup.insert((*package_address, blueprint_name.clone()));
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn to_api_event(
    context: &MappingContext,
    events_system_structure: &IndexMap<EventTypeIdentifier, EventSystemStructure>,
    event: ApplicationEvent,
) -> Result<models::Event, MappingError> {
    let ApplicationEvent { type_id, data } = event;
    let event_system_structure =
        events_system_structure
            .get(&type_id)
            .ok_or(MappingError::MissingSystemStructure {
                message: format!(
                    "Missing system structure for event of type ID {:?}",
                    type_id
                ),
            })?;
    let EventTypeIdentifier(emitter, name) = type_id;
    Ok(models::Event {
        _type: Box::new(models::EventTypeIdentifier {
            emitter: Some(match emitter {
                Emitter::Function(BlueprintId {
                    package_address,
                    blueprint_name,
                }) => models::EventEmitterIdentifier::FunctionEventEmitterIdentifier {
                    package_address: to_api_package_address(context, &package_address)?,
                    blueprint_name,
                },
                Emitter::Method(node_id, object_module_id) => {
                    models::EventEmitterIdentifier::MethodEventEmitterIdentifier {
                        entity: Box::new(to_api_entity_reference(context, &node_id)?),
                        object_module_id: to_api_module_id(&object_module_id),
                    }
                }
            }),
            type_reference: Box::new(to_api_package_type_reference(
                context,
                &event_system_structure.package_type_reference,
            )?),
            name,
        }),
        data: Box::new(to_api_sbor_data_from_bytes(context, &data)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(
    _context: &MappingContext,
    fee_summary: &TransactionFeeSummary,
) -> Result<models::FeeSummary, MappingError> {
    Ok(models::FeeSummary {
        execution_cost_units_consumed: to_api_u32_as_i64(
            fee_summary.total_execution_cost_units_consumed,
        ),
        finalization_cost_units_consumed: to_api_u32_as_i64(
            fee_summary.total_finalization_cost_units_consumed,
        ),
        xrd_total_execution_cost: to_api_decimal(&fee_summary.total_execution_cost_in_xrd),
        xrd_total_finalization_cost: to_api_decimal(&fee_summary.total_finalization_cost_in_xrd),
        xrd_total_tipping_cost: to_api_decimal(&fee_summary.total_tipping_cost_in_xrd),
        xrd_total_royalty_cost: to_api_decimal(&fee_summary.total_royalty_cost_in_xrd),
        xrd_total_storage_cost: to_api_decimal(&fee_summary.total_storage_cost_in_xrd),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_costing_parameters(
    _context: &MappingContext,
    engine_costing_parameters: &CostingParameters,
    transaction_costing_parameters: &TransactionCostingParametersReceipt,
) -> Result<models::CostingParameters, MappingError> {
    Ok(models::CostingParameters {
        execution_cost_unit_price: to_api_decimal(
            &engine_costing_parameters.execution_cost_unit_price,
        ),
        execution_cost_unit_limit: to_api_u32_as_i64(
            engine_costing_parameters.execution_cost_unit_limit,
        ),
        execution_cost_unit_loan: to_api_u32_as_i64(
            engine_costing_parameters.execution_cost_unit_loan,
        ),
        finalization_cost_unit_price: to_api_decimal(
            &engine_costing_parameters.finalization_cost_unit_price,
        ),
        finalization_cost_unit_limit: to_api_u32_as_i64(
            engine_costing_parameters.finalization_cost_unit_limit,
        ),
        xrd_usd_price: to_api_decimal(&engine_costing_parameters.usd_price),
        xrd_storage_price: to_api_decimal(&engine_costing_parameters.state_storage_price),
        xrd_archive_storage_price: to_api_decimal(&engine_costing_parameters.archive_storage_price),
        tip_percentage: to_api_u16_as_i32(transaction_costing_parameters.tip_percentage),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_source(
    context: &MappingContext,
    fee_source: &FeeSource,
) -> Result<models::FeeSource, MappingError> {
    Ok(models::FeeSource {
        from_vaults: fee_source
            .paying_vaults
            .iter()
            .map(|(vault_id, xrd_amount)| {
                Ok(models::PaymentFromVault {
                    vault_entity: Box::new(to_api_entity_reference(context, vault_id)?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_destination(
    context: &MappingContext,
    fee_destination: &FeeDestination,
) -> Result<models::FeeDestination, MappingError> {
    Ok(models::FeeDestination {
        to_proposer: to_api_decimal(&fee_destination.to_proposer),
        to_validator_set: to_api_decimal(&fee_destination.to_validator_set),
        to_burn: to_api_decimal(&fee_destination.to_burn),
        to_royalty_recipients: fee_destination
            .to_royalty_recipients
            .iter()
            .map(|(recipient, xrd_amount)| {
                let global_address: GlobalAddress = match recipient {
                    RoyaltyRecipient::Package(address, _) => (*address).into(),
                    RoyaltyRecipient::Component(address, _) => (*address).into(),
                };
                Ok(models::PaymentToRoyaltyRecipient {
                    royalty_recipient: Box::new(to_api_entity_reference(
                        context,
                        global_address.as_node_id(),
                    )?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}
