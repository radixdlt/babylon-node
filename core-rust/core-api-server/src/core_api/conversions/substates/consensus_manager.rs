use super::super::*;
use super::*;
use crate::core_api::models;
use radix_engine_interface::blueprints::consensus_manager::*;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_registered_validators_by_stake_index_entry_substate(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &Validator,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::ConsensusManagerRegisteredValidatorsByStakeIndexKey(ValidatorByStakeKey { divided_stake, validator_address })) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "ValidatorByStakeKey".to_string() });
    };
    let validator = substate;
    Ok(index_substate!(
        substate,
        ConsensusManagerRegisteredValidatorsByStakeIndexEntry,
        models::ActiveValidatorKey {
            stake_weighting: to_api_u16_as_i32(*divided_stake),
            validator_address: to_api_component_address(context, validator_address)?,
        },
        {
            active_validator: Box::new(to_api_active_validator(
                context,
                validator_address,
                validator,
            )?),
        },
    ))
}

pub fn to_api_current_validator_set_substate(
    context: &MappingContext,
    substate: &FieldSubstate<CurrentValidatorSetSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldCurrentValidatorSet,
        CurrentValidatorSetSubstate { validator_set } => {
            let validator_set = validator_set
                .validators_by_stake_desc
                .iter()
                .map(|(address, validator)| to_api_active_validator(context, address, validator))
                .collect::<Result<_, _>>()?;
        },
        Value { validator_set }
    ))
}

pub fn to_api_current_proposal_statistic_substate(
    _context: &MappingContext,
    substate: &FieldSubstate<CurrentProposalStatisticSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldCurrentProposalStatistic,
        CurrentProposalStatisticSubstate {
            validator_statistics,
        },
        Value {
            completed: validator_statistics
                .iter()
                .map(|s| to_api_ten_trillion_capped_u64(s.made, "completed_proposals"))
                .collect::<Result<_, _>>()?,
            missed: validator_statistics
                .iter()
                .map(|s| to_api_ten_trillion_capped_u64(s.missed, "missed_proposals"))
                .collect::<Result<_, _>>()?,
        },
    ))
}

pub fn to_api_validator_rewards_substate(
    context: &MappingContext,
    substate: &FieldSubstate<ValidatorRewardsSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldValidatorRewards,
        ValidatorRewardsSubstate {
            proposer_rewards,
            rewards_vault,
        },
        Value {
            proposer_rewards: proposer_rewards
                .iter()
                .map(|(validator_index, xrd_amount)| {
                    to_api_proposer_reward(context, validator_index, xrd_amount)
                })
                .collect::<Result<Vec<_>, MappingError>>()?,
            rewards_vault: Box::new(to_api_entity_reference(
                context,
                rewards_vault.0.as_node_id(),
            )?),
        },
    ))
}

pub fn to_api_active_validator(
    context: &MappingContext,
    address: &ComponentAddress,
    validator: &Validator,
) -> Result<models::ActiveValidator, MappingError> {
    Ok(models::ActiveValidator {
        address: to_api_component_address(context, address)?,
        key: Box::new(to_api_ecdsa_secp256k1_public_key(&validator.key)),
        stake: to_api_decimal(&validator.stake),
    })
}

pub fn to_api_proposer_reward(
    _context: &MappingContext,
    validator_index: &ValidatorIndex,
    xrd_amount: &Decimal,
) -> Result<models::ProposerReward, MappingError> {
    Ok(models::ProposerReward {
        validator_index: Box::new(to_api_active_validator_index(*validator_index)),
        xrd_amount: to_api_decimal(xrd_amount),
    })
}

pub fn to_api_validator_substate(
    context: &MappingContext,
    substate: &FieldSubstate<ValidatorSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ValidatorFieldState,
        ValidatorSubstate {
            sorted_key,
            key,
            is_registered,
            accepts_delegated_stake,
            validator_fee_factor,
            validator_fee_change_request,
            stake_unit_resource,
            stake_xrd_vault_id,
            claim_nft,
            pending_xrd_withdraw_vault_id,
            locked_owner_stake_unit_vault_id,
            pending_owner_stake_unit_unlock_vault_id,
            pending_owner_stake_unit_withdrawals,
            already_unlocked_owner_stake_unit_amount,
        },
        Value {
            sorted_key: sorted_key.as_ref().map(|key| {
                Box::new(to_api_substate_key(&SubstateKey::Sorted((
                    key.0,
                    key.1.clone(),
                ))))
            }),
            public_key: Box::new(to_api_ecdsa_secp256k1_public_key(key)),
            is_registered: *is_registered,
            accepts_delegated_stake: *accepts_delegated_stake,
            validator_fee_factor: to_api_decimal(validator_fee_factor),
            validator_fee_change_request: validator_fee_change_request
                .as_ref()
                .map(|validator_fee_change_request| -> Result<_, _> {
                    let ValidatorFeeChangeRequest {
                        epoch_effective,
                        new_fee_factor,
                    } = validator_fee_change_request;
                    Ok(Box::new(models::ValidatorFeeChangeRequest {
                        epoch_effective: to_api_epoch(context, *epoch_effective)?,
                        new_fee_factor: to_api_decimal(new_fee_factor),
                    }))
                })
                .transpose()?,
            stake_unit_resource_address: to_api_resource_address(context, stake_unit_resource)?,
            stake_xrd_vault: Box::new(to_api_entity_reference(
                context,
                stake_xrd_vault_id.as_node_id(),
            )?),
            claim_token_resource_address: to_api_resource_address(context, claim_nft)?,
            pending_xrd_withdraw_vault: Box::new(to_api_entity_reference(
                context,
                pending_xrd_withdraw_vault_id.as_node_id(),
            )?),
            locked_owner_stake_unit_vault: Box::new(to_api_entity_reference(
                context,
                locked_owner_stake_unit_vault_id.as_node_id(),
            )?),
            pending_owner_stake_unit_unlock_vault: Box::new(to_api_entity_reference(
                context,
                pending_owner_stake_unit_unlock_vault_id.as_node_id(),
            )?),
            pending_owner_stake_unit_withdrawals: pending_owner_stake_unit_withdrawals
                .iter()
                .map(|(epoch, amount)| -> Result<_, _> {
                    Ok(models::PendingOwnerStakeWithdrawal {
                        epoch_unlocked: to_api_epoch(context, *epoch)?,
                        stake_unit_amount: to_api_decimal(amount),
                    })
                })
                .collect::<Result<_, _>>()?,
            already_unlocked_owner_stake_unit_amount: to_api_decimal(
                already_unlocked_owner_stake_unit_amount,
            ),
        }
    ))
}

pub fn to_api_validator_protocol_update_readiness_signal_substate(
    _context: &MappingContext,
    substate: &FieldSubstate<ValidatorProtocolUpdateReadinessSignalSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ValidatorFieldProtocolUpdateReadinessSignal,
        ValidatorProtocolUpdateReadinessSignalSubstate {
            protocol_version_name,
        },
        Value {
            protocol_version_name: protocol_version_name.as_ref().map(|name| name.to_string()),
        }
    ))
}

pub fn to_api_consensus_manager_state_substate(
    context: &MappingContext,
    substate: &FieldSubstate<ConsensusManagerSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldState,
        ConsensusManagerSubstate {
            epoch,
            round,
            started,
            effective_epoch_start_milli,
            actual_epoch_start_milli,
            current_leader,
        },
        Value {
            epoch: to_api_epoch(context, *epoch)?,
            round: to_api_round(*round)?,
            is_started: *started,
            effective_epoch_start: Box::new(to_api_instant_from_safe_timestamp(
                *effective_epoch_start_milli,
            )?),
            actual_epoch_start: Box::new(to_api_instant_from_safe_timestamp(
                *actual_epoch_start_milli,
            )?),
            current_leader: current_leader
                .as_ref()
                .map(|validator_index| to_api_active_validator_index(*validator_index))
                .map(Box::new),
        }
    ))
}

pub fn to_api_consensus_manager_config_substate(
    substate: &FieldSubstate<ConsensusManagerConfigSubstate>,
) -> Result<models::Substate, MappingError> {
    let usd_price_in_xrd = Decimal::try_from(USD_PRICE_IN_XRD).unwrap();
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldConfig,
        ConsensusManagerConfigSubstate {
            config: ConsensusManagerConfig {
                max_validators,
                epoch_change_condition,
                num_unstake_epochs,
                total_emission_xrd_per_epoch,
                min_validator_reliability,
                num_owner_stake_units_unlock_epochs,
                num_fee_increase_delay_epochs,
                validator_creation_usd_cost,
            },
        },
        Value {
            max_validators: to_api_ten_trillion_capped_u64(
                u64::from(*max_validators),
                "max_validators",
            )?,
            epoch_change_condition: Box::new(to_api_epoch_change_condition(
                epoch_change_condition
            )?),
            num_unstake_epochs: to_api_ten_trillion_capped_u64(
                *num_unstake_epochs,
                "num_unstake_epochs",
            )?,
            total_emission_xrd_per_epoch: to_api_decimal(total_emission_xrd_per_epoch),
            min_validator_reliability: to_api_decimal(min_validator_reliability),
            num_owner_stake_units_unlock_epochs: to_api_ten_trillion_capped_u64(
                *num_owner_stake_units_unlock_epochs,
                "num_owner_stake_units_unlock_epochs",
            )?,
            num_fee_increase_delay_epochs: to_api_ten_trillion_capped_u64(
                *num_fee_increase_delay_epochs,
                "num_fee_increase_delay_epochs",
            )?,
            validator_creation_usd_equivalent_cost: to_api_decimal(validator_creation_usd_cost),
            validator_creation_xrd_cost: to_api_decimal(
                &(validator_creation_usd_cost.mul_or_panic(usd_price_in_xrd))
            ),
        }
    ))
}

pub fn to_api_epoch_change_condition(
    epoch_change_condition: &EpochChangeCondition,
) -> Result<models::EpochChangeCondition, MappingError> {
    let EpochChangeCondition {
        min_round_count,
        max_round_count,
        target_duration_millis,
    } = epoch_change_condition;
    Ok(models::EpochChangeCondition {
        min_round_count: to_api_ten_trillion_capped_u64(*min_round_count, "min_round_count")?,
        max_round_count: to_api_ten_trillion_capped_u64(*max_round_count, "max_round_count")?,
        target_duration_millis: to_api_ten_trillion_capped_u64(
            *target_duration_millis,
            "target_duration_millis",
        )?,
    })
}

pub fn to_api_current_time_substate(
    substate: &FieldSubstate<ProposerMilliTimestampSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldCurrentTime,
        ProposerMilliTimestampSubstate { epoch_milli },
        Value {
            proposer_timestamp: Box::new(to_api_instant_from_safe_timestamp(*epoch_milli)?),
        }
    ))
}

pub fn to_api_current_time_rounded_to_minutes_substate(
    substate: &FieldSubstate<ProposerMinuteTimestampSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        ConsensusManagerFieldCurrentTimeRoundedToMinutes,
        ProposerMinuteTimestampSubstate { epoch_minute },
        Value {
            proposer_timestamp_rounded_down_to_minute: Box::new(
                to_api_instant_from_safe_timestamp(i64::from(*epoch_minute) * 60 * 1000)?,
            ),
        }
    ))
}
