use super::addressing::*;
use crate::core_api::*;
use radix_engine::types::*;

use radix_engine::system::system_modules::costing::*;
use radix_engine_queries::typed_substate_layout::*;

use state_manager::{
    ApplicationEvent, ChangeAction, DetailedTransactionOutcome, LocalTransactionReceipt,
    SubstateReference,
};

pub fn to_api_receipt(
    context: &MappingContext,
    receipt: LocalTransactionReceipt,
) -> Result<models::TransactionReceipt, MappingError> {
    let (status, output, error_message) = match receipt.local_execution.outcome {
        DetailedTransactionOutcome::Success(output) => {
            (models::TransactionStatus::Succeeded, Some(output), None)
        }
        DetailedTransactionOutcome::Failure(error) => (
            models::TransactionStatus::Failed,
            None,
            Some(format!("{error:?}")),
        ),
    };

    let state_update_summary = receipt.local_execution.state_update_summary;
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

    let mut created_substates = Vec::new();
    let mut updated_substates = Vec::new();
    let mut deleted_substates = Vec::new();
    for (substate_reference, action) in receipt.on_ledger.substate_changes.iter() {
        let SubstateReference(node_id, partition_number, substate_key) = substate_reference;
        let typed_substate_key =
            create_typed_substate_key(context, &node_id, partition_number, &substate_key)?;
        if !typed_substate_key.value_is_mappable() {
            continue;
        }

        match action.clone() {
            ChangeAction::Create(value) => {
                created_substates.push(to_api_created_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &typed_substate_key,
                    &ValueRepresentations::new(&typed_substate_key, value)?,
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
            ChangeAction::Delete => {
                deleted_substates.push(to_api_deleted_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &typed_substate_key,
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

    let api_fee_summary = to_api_fee_summary(context, &receipt.local_execution.fee_summary)?;

    let api_events = receipt
        .on_ledger
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

    let next_epoch = receipt
        .local_execution
        .next_epoch
        .map(|epoch_change_event| to_api_next_epoch(context, epoch_change_event))
        .transpose()?
        .map(Box::new);

    Ok(models::TransactionReceipt {
        status,
        fee_summary: Some(Box::new(api_fee_summary)),
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
        type_id: EventTypeIdentifier(emitter, type_pointer),
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
            type_pointer: Some(to_api_type_pointer(context, &type_pointer)?),
        }),
        data: Box::new(to_api_sbor_data_from_bytes(context, &data)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(
    context: &MappingContext,
    fee_summary: &FeeSummary,
) -> Result<models::FeeSummary, MappingError> {
    Ok(models::FeeSummary {
        cost_unit_price: to_api_decimal(&fee_summary.cost_unit_price),
        tip_percentage: to_api_u16_as_i32(fee_summary.tip_percentage),
        cost_unit_limit: to_api_u32_as_i64(fee_summary.cost_unit_limit),
        cost_units_consumed: to_api_u32_as_i64(fee_summary.execution_cost_sum),
        xrd_total_execution_cost: to_api_decimal(&fee_summary.total_execution_cost_xrd),
        xrd_total_state_expansion_cost: to_api_decimal(&fee_summary.total_state_expansion_cost_xrd),
        xrd_total_royalty_cost: to_api_decimal(&fee_summary.total_royalty_cost_xrd),
        xrd_total_tipped: to_api_decimal(&fee_summary.total_tipping_cost_xrd),
        cost_unit_execution_breakdown: fee_summary
            .execution_cost_breakdown
            .iter()
            .map(|(key, cost_unit_amount)| (key.to_string(), to_api_u32_as_i64(*cost_unit_amount)))
            .collect(),
        xrd_vault_payments: fee_summary
            .fee_payments
            .iter()
            .map(|(vault_id, xrd_amount)| {
                Ok(models::VaultPayment {
                    vault_entity: Box::new(to_api_entity_reference(context, vault_id)?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
        xrd_royalty_receivers: fee_summary
            .royalty_cost_breakdown
            .iter()
            .map(|(receiver, (_, xrd_amount))| {
                let global_address: GlobalAddress = match receiver {
                    RoyaltyRecipient::Package(address) => (*address).into(),
                    RoyaltyRecipient::Component(address) => (*address).into(),
                };
                Ok(models::RoyaltyPayment {
                    royalty_receiver: Box::new(to_api_entity_reference(
                        context,
                        global_address.as_node_id(),
                    )?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}
