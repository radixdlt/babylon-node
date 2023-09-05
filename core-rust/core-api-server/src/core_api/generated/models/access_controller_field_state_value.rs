/*
 * Babylon Core API - RCnet v3.1
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct AccessControllerFieldStateValue {
    #[serde(rename = "controlled_vault")]
    pub controlled_vault: Box<crate::core_api::generated::models::EntityReference>,
    /// An integer between `0` and `2^32 - 1`, specifying the amount of time (in minutes) that it takes for timed recovery to be done. When not present, then timed recovery can not be performed through this access controller. 
    #[serde(rename = "timed_recovery_delay_minutes", skip_serializing_if = "Option::is_none")]
    pub timed_recovery_delay_minutes: Option<i64>,
    /// The Bech32m-encoded human readable version of the resource address
    #[serde(rename = "recovery_badge_resource_address")]
    pub recovery_badge_resource_address: String,
    /// Whether the primary role is currently locked.
    #[serde(rename = "is_primary_role_locked")]
    pub is_primary_role_locked: bool,
    #[serde(rename = "primary_role_recovery_attempt", skip_serializing_if = "Option::is_none")]
    pub primary_role_recovery_attempt: Option<Box<crate::core_api::generated::models::PrimaryRoleRecoveryAttempt>>,
    /// Whether the primary role badge withdraw is currently being attempted.
    #[serde(rename = "has_primary_role_badge_withdraw_attempt")]
    pub has_primary_role_badge_withdraw_attempt: bool,
    #[serde(rename = "recovery_role_recovery_attempt", skip_serializing_if = "Option::is_none")]
    pub recovery_role_recovery_attempt: Option<Box<crate::core_api::generated::models::RecoveryRoleRecoveryAttempt>>,
    /// Whether the recovery role badge withdraw is currently being attempted.
    #[serde(rename = "has_recovery_role_badge_withdraw_attempt")]
    pub has_recovery_role_badge_withdraw_attempt: bool,
}

impl AccessControllerFieldStateValue {
    pub fn new(controlled_vault: crate::core_api::generated::models::EntityReference, recovery_badge_resource_address: String, is_primary_role_locked: bool, has_primary_role_badge_withdraw_attempt: bool, has_recovery_role_badge_withdraw_attempt: bool) -> AccessControllerFieldStateValue {
        AccessControllerFieldStateValue {
            controlled_vault: Box::new(controlled_vault),
            timed_recovery_delay_minutes: None,
            recovery_badge_resource_address,
            is_primary_role_locked,
            primary_role_recovery_attempt: None,
            has_primary_role_badge_withdraw_attempt,
            recovery_role_recovery_attempt: None,
            has_recovery_role_badge_withdraw_attempt,
        }
    }
}


