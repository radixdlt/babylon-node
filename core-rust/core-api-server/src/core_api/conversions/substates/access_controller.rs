use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_access_controller_substate(
    context: &MappingContext,
    substate: &AccessControllerStateFieldSubstate,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate_versioned!(
        substate,
        AccessControllerFieldState,
        AccessControllerSubstate {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state: (
                primary_role_locking_state,
                primary_role_recovery_attempt_state,
                primary_role_badge_withdraw_attempt_state,
                recovery_role_recovery_attempt_state,
                recovery_role_badge_withdraw_attempt_state,
            )
        },
        Value {
            controlled_vault: Box::new(to_api_entity_reference(
                context,
                controlled_asset.0.as_node_id()
            )?),
            timed_recovery_delay_minutes: timed_recovery_delay_in_minutes
                .as_ref()
                .map(|minutes| to_api_u32_as_i64(*minutes)),
            recovery_badge_resource_address: to_api_resource_address(context, recovery_badge)?,
            is_primary_role_locked: match primary_role_locking_state {
                PrimaryRoleLockingState::Locked => true,
                PrimaryRoleLockingState::Unlocked => false,
            },
            primary_role_recovery_attempt: match primary_role_recovery_attempt_state {
                PrimaryRoleRecoveryAttemptState::NoRecoveryAttempt => None,
                PrimaryRoleRecoveryAttemptState::RecoveryAttempt(recovery_proposal) =>
                    Some(Box::new(models::PrimaryRoleRecoveryAttempt {
                        recovery_proposal: Box::new(to_api_recovery_proposal(
                            context,
                            recovery_proposal
                        )?)
                    })),
            },
            has_primary_role_badge_withdraw_attempt: match primary_role_badge_withdraw_attempt_state
            {
                PrimaryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt => true,
                PrimaryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt => false,
            },
            recovery_role_recovery_attempt: match recovery_role_recovery_attempt_state {
                RecoveryRoleRecoveryAttemptState::NoRecoveryAttempt => None,
                RecoveryRoleRecoveryAttemptState::RecoveryAttempt(attempt_state) => {
                    let (recovery_proposal, timed_recovery_allowed_after) = match attempt_state {
                        RecoveryRoleRecoveryState::UntimedRecovery(proposal) => (proposal, None),
                        RecoveryRoleRecoveryState::TimedRecovery {
                            proposal,
                            timed_recovery_allowed_after,
                        } => (proposal, Some(timed_recovery_allowed_after)),
                    };
                    Some(Box::new(models::RecoveryRoleRecoveryAttempt {
                        recovery_proposal: Box::new(to_api_recovery_proposal(
                            context,
                            recovery_proposal,
                        )?),
                        timed_recovery_allowed_after: timed_recovery_allowed_after
                            .map(to_api_instant)
                            .transpose()?
                            .map(Box::new),
                    }))
                }
            },
            has_recovery_role_badge_withdraw_attempt:
                match recovery_role_badge_withdraw_attempt_state {
                    RecoveryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt => true,
                    RecoveryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt => false,
                },
        }
    ))
}

pub fn to_api_recovery_proposal(
    context: &MappingContext,
    proposal: &RecoveryProposal,
) -> Result<models::RecoveryProposal, MappingError> {
    let RecoveryProposal {
        rule_set:
            RuleSet {
                primary_role,
                recovery_role,
                confirmation_role,
            },
        timed_recovery_delay_in_minutes,
    } = proposal;
    Ok(models::RecoveryProposal {
        primary_role: Some(to_api_access_rule(context, primary_role)?),
        recovery_role: Some(to_api_access_rule(context, recovery_role)?),
        confirmation_role: Some(to_api_access_rule(context, confirmation_role)?),
        timed_recovery_delay_minutes: timed_recovery_delay_in_minutes
            .as_ref()
            .map(|minutes| to_api_u32_as_i64(*minutes)),
    })
}
