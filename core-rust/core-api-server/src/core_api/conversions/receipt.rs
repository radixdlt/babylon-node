use super::addressing::*;
use crate::core_api::*;
use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::types::*;

use radix_engine::system::system_modules::costing::*;
use radix_engine_queries::typed_substate_layout::*;

use state_manager::{
    ApplicationEvent, ChangeAction, DetailedTransactionOutcome, LocalTransactionReceipt,
    SubstateChange,
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
    for SubstateChange {
        node_id,
        partition_number,
        substate_key,
        action,
    } in receipt.on_ledger.substate_changes
    {
        let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
        let typed_substate_key =
            to_typed_substate_key(entity_type, partition_number, &substate_key).map_err(|msg| {
                MappingError::SubstateKey {
                    entity_address: to_api_entity_address(context, &node_id)
                        .unwrap_or_else(|_| format!("NodeId[{}]", to_hex(node_id.as_bytes()))),
                    partition_number,
                    substate_key: to_api_substate_key(&substate_key),
                    message: msg,
                }
            })?;
        if !typed_substate_key.value_is_mappable() {
            continue;
        }

        match action {
            ChangeAction::Create(value) => {
                let typed_substate_value =
                    to_typed_substate_value(&typed_substate_key, value.as_ref()).map_err(
                        |msg| MappingError::SubstateValue {
                            bytes: value.clone(),
                            message: msg,
                        },
                    )?;
                created_substates.push(to_api_created_or_updated_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &value,
                    &typed_substate_key,
                    &typed_substate_value,
                )?);
            }
            ChangeAction::Update(value) => {
                let typed_substate_value =
                    to_typed_substate_value(&typed_substate_key, value.as_ref()).map_err(
                        |msg| MappingError::SubstateValue {
                            bytes: value.clone(),
                            message: msg,
                        },
                    )?;
                updated_substates.push(to_api_created_or_updated_substate(
                    context,
                    &node_id,
                    partition_number,
                    &substate_key,
                    &value,
                    &typed_substate_key,
                    &typed_substate_value,
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

    let api_fee_summary = to_api_fee_summary(
        context,
        &receipt.local_execution.fee_summary,
        &receipt.local_execution.fee_payments,
    )?;

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
        .map(|next_epoch| to_api_next_epoch(context, next_epoch))
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

#[tracing::instrument(skip_all)]
pub fn to_api_created_or_updated_substate(
    context: &MappingContext,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
    value: &Vec<u8>,
    typed_substate_key: &TypedSubstateKey,
    typed_substate_value: &TypedSubstateValue,
) -> Result<models::CreatedOrUpdatedSubstate, MappingError> {
    let substate_hex = to_hex(value);
    let substate_data_hash = to_hex(hash(value));
    let substate_id = to_api_substate_id(
        context,
        node_id,
        partition_number,
        substate_key,
        typed_substate_key,
    )?;
    let substate_data = Some(to_api_substate(
        context,
        substate_key,
        typed_substate_value,
    )?);
    Ok(models::CreatedOrUpdatedSubstate {
        substate_id: Box::new(substate_id),
        substate_hex,
        substate_data_hash,
        substate_data,
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
    next_epoch: (BTreeMap<ComponentAddress, Validator>, u64),
) -> Result<models::NextEpoch, MappingError> {
    let mut sorted_validators: Vec<(ComponentAddress, Validator)> =
        next_epoch.0.into_iter().map(|e| (e.0, e.1)).collect();
    sorted_validators.sort_by(|a, b| b.1.stake.cmp(&a.1.stake));

    let next_epoch = models::NextEpoch {
        epoch: to_api_epoch(context, next_epoch.1)?,
        validators: sorted_validators
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
        type_id: EventTypeIdentifier(emitter, local_type_index),
        data,
    } = event;
    Ok(models::Event {
        _type: Box::new(models::EventTypeIdentifier {
            emitter: Some(match emitter {
                Emitter::Function(node_id, object_module_id, blueprint_name) => {
                    models::EventEmitterIdentifier::FunctionEventEmitterIdentifier {
                        entity: Box::new(to_api_entity_reference(context, &node_id)?),
                        object_module_id: to_api_object_module_id(&object_module_id),
                        blueprint_name,
                    }
                }
                Emitter::Method(node_id, object_module_id) => {
                    models::EventEmitterIdentifier::MethodEventEmitterIdentifier {
                        entity: Box::new(to_api_entity_reference(context, &node_id)?),
                        object_module_id: to_api_object_module_id(&object_module_id),
                    }
                }
            }),
            local_type_index: Box::new(to_api_local_type_index(context, &local_type_index)?),
        }),
        data: Box::new(to_api_sbor_data_from_bytes(context, &data)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(
    context: &MappingContext,
    fee_summary: &FeeSummary,
    fee_payments: &IndexMap<NodeId, Decimal>,
) -> Result<models::FeeSummary, MappingError> {
    Ok(models::FeeSummary {
        cost_unit_price: to_api_decimal(&fee_summary.cost_unit_price),
        tip_percentage: to_api_u16_as_i32(fee_summary.tip_percentage),
        cost_unit_limit: to_api_u32_as_i64(fee_summary.cost_unit_limit),
        cost_units_consumed: to_api_u32_as_i64(fee_summary.execution_cost_sum),
        xrd_total_execution_cost: to_api_decimal(&fee_summary.total_execution_cost_xrd),
        xrd_total_royalty_cost: to_api_decimal(&fee_summary.total_royalty_cost_xrd),
        xrd_total_tipped: to_api_decimal(&Decimal::ZERO),
        cost_unit_execution_breakdown: fee_summary
            .execution_cost_breakdown
            .iter()
            .map(|(key, cost_unit_amount)| (key.to_string(), to_api_u32_as_i64(*cost_unit_amount)))
            .collect(),
        xrd_vault_payments: fee_payments
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
