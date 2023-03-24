use super::addressing::*;
use crate::core_api::*;
use radix_engine::blueprints::epoch_manager::Validator;
use radix_engine::system::kernel_modules::costing::{FeeSummary, RoyaltyRecipient};
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::indexmap::IndexMap;
use radix_engine::types::{
    Address, ComponentAddress, ObjectId, RENodeId, SubstateOffset, VaultOffset,
};
use radix_engine::{
    ledger::OutputValue,
    types::{hash, scrypto_encode, Decimal, SubstateId},
};

use radix_engine_interface::api::types::{Emitter, EventTypeIdentifier};
use radix_engine_interface::blueprints::resource::ResourceType;

use std::collections::{BTreeMap, HashMap};

use state_manager::{
    ApplicationEvent, ChangeAction, DeletedSubstateVersion, DetailedTransactionOutcome,
    LocalTransactionReceipt,
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
        new_global_entities.push(to_global_entity_reference(
            context,
            &package_address.into(),
        )?);
    }

    for component_address in state_update_summary.new_components {
        new_global_entities.push(to_global_entity_reference(
            context,
            &component_address.into(),
        )?);
    }

    for resource_address in state_update_summary.new_resources {
        new_global_entities.push(to_global_entity_reference(
            context,
            &resource_address.into(),
        )?);
    }

    // This was added as a temporary workaround to the Vault substate abstraction for the RCNet release
    fn filter_out_incorrect_vault_substates_for_gateway(
        created_substates: Vec<(SubstateId, OutputValue)>,
    ) -> Vec<(SubstateId, OutputValue)> {
        // First pass -> create mapping of Vault => ResourceType
        let vault_resource_type_map: HashMap<ObjectId, ResourceType> = created_substates
            .iter()
            .filter_map(|(substate_id, output_value)| match substate_id {
                SubstateId(
                    RENodeId::Object(vault_id),
                    _,
                    SubstateOffset::Vault(VaultOffset::Info),
                ) => match &output_value.substate {
                    PersistedSubstate::VaultInfo(substate) => {
                        Some((*vault_id, substate.resource_type))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();
        // Second pass -> filter out incorrect substates
        created_substates
            .into_iter()
            .filter_map(|(substate_id, output_value)| {
                let resource_type = match substate_id.0 {
                    RENodeId::Object(object_id) => vault_resource_type_map.get(&object_id),
                    _ => None,
                };
                let keep_substate = match (&output_value.substate, resource_type) {
                    (
                        PersistedSubstate::VaultLiquidFungible(_),
                        Some(ResourceType::Fungible { .. }),
                    ) => true,
                    (PersistedSubstate::VaultLiquidFungible(_), _) => false,
                    (
                        PersistedSubstate::VaultLiquidNonFungible(_),
                        Some(ResourceType::NonFungible { .. }),
                    ) => true,
                    (PersistedSubstate::VaultLiquidNonFungible(_), _) => false,
                    (PersistedSubstate::VaultLockedFungible(_), _) => false,
                    (PersistedSubstate::VaultLockedNonFungible(_), _) => false,
                    _ => true,
                };
                if keep_substate {
                    Some((substate_id, output_value))
                } else {
                    None
                }
            })
            .collect()
    }

    let mut unfiltered_creations = Vec::new();
    let mut updated_substates = Vec::new();
    let mut deleted_substates = Vec::new();
    for substate_change in receipt.on_ledger.substate_changes {
        let id = substate_change.substate_id;
        match substate_change.action {
            ChangeAction::Create(value) => {
                unfiltered_creations.push((id, value));
            }
            ChangeAction::Update(value) => {
                updated_substates.push(to_api_new_substate_version(context, id, value)?);
            }
            ChangeAction::Delete(version) => {
                deleted_substates.push(to_api_deleted_substate(id, version)?);
            }
        }
    }

    let created_substates = filter_out_incorrect_vault_substates_for_gateway(unfiltered_creations)
        .into_iter()
        .map(|(id, value)| to_api_new_substate_version(context, id, value))
        .collect::<Result<Vec<_>, _>>()?;

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
pub fn to_api_new_substate_version(
    context: &MappingContext,
    substate_id: SubstateId,
    output_value: OutputValue,
) -> Result<models::NewSubstateVersion, MappingError> {
    let substate_bytes =
        scrypto_encode(&output_value.substate).map_err(|err| MappingError::SborEncodeError {
            encode_error: err,
            message: "Substate bytes could not be encoded".to_string(),
        })?;
    let hash = to_hex(hash(&substate_bytes));

    let api_substate_data = Some(to_api_substate(
        context,
        &substate_id,
        &output_value.substate,
    )?);

    Ok(models::NewSubstateVersion {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        version: to_api_substate_version(output_value.version)?,
        substate_hex: to_hex(substate_bytes),
        substate_data_hash: hash,
        substate_data: api_substate_data,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_deleted_substate(
    substate_id: SubstateId,
    deleted_substate: DeletedSubstateVersion,
) -> Result<models::DeletedSubstateVersionRef, MappingError> {
    Ok(models::DeletedSubstateVersionRef {
        substate_id: Box::new(to_api_substate_id(substate_id)?),
        substate_data_hash: to_hex(deleted_substate.substate_hash),
        version: to_api_substate_version(deleted_substate.version)?,
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

    let mut validators = Vec::new();
    for (address, validator) in sorted_validators {
        let api_validator = to_api_active_validator(context, &address, &validator);
        validators.push(api_validator);
    }

    let next_epoch = models::NextEpoch {
        epoch: to_api_epoch(context, next_epoch.1)?,
        validators,
    };

    Ok(next_epoch)
}

#[tracing::instrument(skip_all)]
pub fn to_api_event(
    context: &MappingContext,
    event: ApplicationEvent,
) -> Result<models::Event, MappingError> {
    let EventTypeIdentifier(emitter, local_type_index) = event.type_id;
    Ok(models::Event {
        _type: Box::new(models::EventTypeIdentifier {
            emitter: Some(match emitter {
                Emitter::Function(node_id, node_module_id, blueprint_name) => {
                    models::EventEmitterIdentifier::FunctionEventEmitterIdentifier {
                        entity: Box::new(to_api_entity_reference(node_id)?),
                        module_type: node_module_id_to_module_type(&node_module_id),
                        blueprint_name,
                    }
                }
                Emitter::Method(node_id, node_module_id) => {
                    models::EventEmitterIdentifier::MethodEventEmitterIdentifier {
                        entity: Box::new(to_api_entity_reference(node_id)?),
                        module_type: node_module_id_to_module_type(&node_module_id),
                    }
                }
            }),
            local_type_index: Box::new(to_api_local_type_index(context, &local_type_index)?),
        }),
        data: Box::new(to_api_sbor_data_from_bytes(context, &event.data)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_fee_summary(
    context: &MappingContext,
    fee_summary: &FeeSummary,
    fee_payments: &IndexMap<ObjectId, Decimal>,
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
                    vault_entity: Box::new(to_api_entity_reference(RENodeId::Object(*vault_id))?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
        xrd_royalty_receivers: fee_summary
            .royalty_cost_breakdown
            .iter()
            .map(|(receiver, (_, xrd_amount))| {
                let global_address = match receiver {
                    RoyaltyRecipient::Package(address) => Address::Package(*address),
                    RoyaltyRecipient::Component(address) => Address::Component(*address),
                };
                Ok(models::RoyaltyPayment {
                    royalty_receiver: Box::new(to_global_entity_reference(
                        context,
                        &global_address,
                    )?),
                    xrd_amount: to_api_decimal(xrd_amount),
                })
            })
            .collect::<Result<_, _>>()?,
    })
}
