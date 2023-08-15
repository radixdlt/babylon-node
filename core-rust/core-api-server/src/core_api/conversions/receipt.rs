use super::addressing::*;
use crate::core_api::*;
use radix_engine::types::*;

use radix_engine::system::system_modules::costing::*;
use radix_engine::transaction::{
    CostingParameters, FeeDestination, FeeSource, TransactionFeeSummary,
};
use radix_engine_queries::typed_substate_layout::*;
use transaction::prelude::TransactionCostingParameters;

use state_manager::{
    ApplicationEvent, ChangeAction, DetailedTransactionOutcome, LocalTransactionReceipt,
    SubstateReference,
};

pub fn to_api_receipt(
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

    let state_update_summary = local_execution.state_update_summary;
    let mut new_global_entities = Vec::new();

    for package_address in state_update_summary.new_packages {
        new_global_entities.push(to_api_entity_reference(
            context,
            package_address.as_node_id(),
        )?);
    }

    for component_address in state_update_summary.new_components {
        new_global_entities.push(to_api_entity_reference(
            context,
            component_address.as_node_id(),
        )?);
    }

    for resource_address in state_update_summary.new_resources {
        new_global_entities.push(to_api_entity_reference(
            context,
            resource_address.as_node_id(),
        )?);
    }

    let on_ledger = receipt.on_ledger;
    let mut created_substates = Vec::new();
    let mut updated_substates = Vec::new();
    let mut deleted_substates = Vec::new();
    for (substate_reference, action) in on_ledger.substate_changes.iter() {
        let SubstateReference(node_id, partition_number, substate_key) = substate_reference;
        let typed_substate_key =
            create_typed_substate_key(context, &node_id, partition_number, &substate_key)?;
        if !typed_substate_key.value_is_mappable() {
            continue;
        }

        match action.clone() {
            ChangeAction::Create { new } => {
                created_substates.push(to_api_created_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &typed_substate_key,
                    &ValueRepresentations::new(&typed_substate_key, new)?,
                )?);
            }
            ChangeAction::Update { previous, new } => {
                updated_substates.push(to_api_updated_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &typed_substate_key,
                    &ValueRepresentations::new(&typed_substate_key, new)?,
                    &ValueRepresentations::new(&typed_substate_key, previous)?,
                )?);
            }
            ChangeAction::Delete { previous } => {
                deleted_substates.push(to_api_deleted_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &typed_substate_key,
                    &ValueRepresentations::new(&typed_substate_key, previous)?,
                )?);
            }
        }
    }

    let api_state_updates = models::StateUpdates {
        created_substates,
        updated_substates,
        deleted_substates,
        new_global_entities,
    };

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
        .map(|event| to_api_event(context, event))
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
            substate_key: to_api_substate_key(substate_key),
            message: msg,
        }
    })
}

pub struct ValueRepresentations {
    pub typed: TypedSubstateValue,
    pub raw: Vec<u8>,
}

impl ValueRepresentations {
    pub fn new(typed_substate_key: &TypedSubstateKey, raw: Vec<u8>) -> Result<Self, MappingError> {
        Ok(Self {
            typed: to_typed_substate_value(typed_substate_key, raw.as_ref()).map_err(|msg| {
                MappingError::SubstateValue {
                    bytes: raw.clone(),
                    message: msg,
                }
            })?,
            raw,
        })
    }
}
#[tracing::instrument(skip_all)]
pub fn to_api_created_substate(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
    value_representations: &ValueRepresentations,
) -> Result<models::CreatedSubstate, MappingError> {
    Ok(models::CreatedSubstate {
        substate_id: Box::new(to_api_substate_id(
            context,
            node_id,
            partition_number,
            substate_key,
            typed_substate_key,
        )?),
        value: Box::new(to_api_substate_value(
            context,
            typed_substate_key,
            value_representations,
        )?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_updated_substate(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
    new_value_representations: &ValueRepresentations,
    previous_value_representations: &ValueRepresentations,
) -> Result<models::UpdatedSubstate, MappingError> {
    Ok(models::UpdatedSubstate {
        substate_id: Box::new(to_api_substate_id(
            context,
            node_id,
            partition_number,
            substate_key,
            typed_substate_key,
        )?),
        new_value: Box::new(to_api_substate_value(
            context,
            typed_substate_key,
            new_value_representations,
        )?),
        previous_value: if context.substate_options.include_previous {
            Some(Box::new(to_api_substate_value(
                context,
                typed_substate_key,
                previous_value_representations,
            )?))
        } else {
            None
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_substate_value(
    context: &MappingContext,
    typed_substate_key: &TypedSubstateKey,
    value_representations: &ValueRepresentations,
) -> Result<models::SubstateValue, MappingError> {
    Ok(models::SubstateValue {
        substate_hex: if context.substate_options.include_raw {
            Some(to_hex(&value_representations.raw))
        } else {
            None
        },
        substate_data_hash: if context.substate_options.include_hash {
            Some(to_hex(hash(&value_representations.raw)))
        } else {
            None
        },
        substate_data: if context.substate_options.include_typed {
            Some(Box::new(to_api_substate(
                context,
                typed_substate_key,
                &value_representations.typed,
            )?))
        } else {
            None
        },
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_deleted_substate(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    typed_substate_key: &TypedSubstateKey,
    previous_value_representations: &ValueRepresentations,
) -> Result<models::DeletedSubstate, MappingError> {
    let substate_id = to_api_substate_id(
        context,
        node_id,
        partition_number,
        substate_key,
        typed_substate_key,
    )?;
    Ok(models::DeletedSubstate {
        substate_id: Box::new(substate_id),
        previous_value: if context.substate_options.include_previous {
            Some(Box::new(to_api_substate_value(
                context,
                typed_substate_key,
                previous_value_representations,
            )?))
        } else {
            None
        },
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
        .. // TODO: expose `significant_protocol_update_readiness` when it becomes more important
    } = epoch_change_event;
    let next_epoch = models::NextEpoch {
        epoch: to_api_epoch(context, epoch)?,
        validators: validator_set
            .validators_by_stake_desc
            .into_iter()
            .map(|(address, validator)| to_api_active_validator(context, &address, &validator))
            .collect::<Result<_, _>>()?,
    };
    Ok(next_epoch)
}

#[tracing::instrument(skip_all)]
pub fn to_api_event(
    context: &MappingContext,
    event: ApplicationEvent,
) -> Result<models::Event, MappingError> {
    let ApplicationEvent {
        type_id: EventTypeIdentifier(emitter, name),
        data,
    } = event;
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
                        object_module_id: to_api_object_module_id(&object_module_id),
                    }
                }
            }),
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
    transaction_costing_parameters: &TransactionCostingParameters,
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
        xrd_usd_price: to_api_decimal(&engine_costing_parameters.finalization_cost_unit_price),
        xrd_storage_price: to_api_decimal(&engine_costing_parameters.finalization_cost_unit_price),
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
