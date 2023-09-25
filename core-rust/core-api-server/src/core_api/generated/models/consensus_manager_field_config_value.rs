/*
 * Radix Core API - Babylon
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ConsensusManagerFieldConfigValue {
    /// An integer between `0` and `10^10`, specifying the maximum number of validators in the active validator set. 
    #[serde(rename = "max_validators")]
    pub max_validators: i64,
    #[serde(rename = "epoch_change_condition")]
    pub epoch_change_condition: Box<crate::core_api::generated::models::EpochChangeCondition>,
    /// An integer between `0` and `10^10`, specifying the minimum number of epochs before an unstaker can withdraw their XRD. 
    #[serde(rename = "num_unstake_epochs")]
    pub num_unstake_epochs: i64,
    /// A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "total_emission_xrd_per_epoch")]
    pub total_emission_xrd_per_epoch: String,
    /// A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "min_validator_reliability")]
    pub min_validator_reliability: String,
    /// An integer between `0` and `10^10`, specifying the minimum number of epochs before an owner can take their stake units after attempting to withdraw them. 
    #[serde(rename = "num_owner_stake_units_unlock_epochs")]
    pub num_owner_stake_units_unlock_epochs: i64,
    /// An integer between `0` and `10^10`, specifying the minimum number of epochs before a fee increase takes effect. 
    #[serde(rename = "num_fee_increase_delay_epochs")]
    pub num_fee_increase_delay_epochs: i64,
    /// The defining decimal cost of a validator in USD. This is turned into an XRD cost through the current protocol-based USD/XRD multiplier. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "validator_creation_usd_equivalent_cost")]
    pub validator_creation_usd_equivalent_cost: String,
    /// The decimal amount of XRD required to be passed in a bucket to create a validator. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "validator_creation_xrd_cost")]
    pub validator_creation_xrd_cost: String,
}

impl ConsensusManagerFieldConfigValue {
    pub fn new(max_validators: i64, epoch_change_condition: crate::core_api::generated::models::EpochChangeCondition, num_unstake_epochs: i64, total_emission_xrd_per_epoch: String, min_validator_reliability: String, num_owner_stake_units_unlock_epochs: i64, num_fee_increase_delay_epochs: i64, validator_creation_usd_equivalent_cost: String, validator_creation_xrd_cost: String) -> ConsensusManagerFieldConfigValue {
        ConsensusManagerFieldConfigValue {
            max_validators,
            epoch_change_condition: Box::new(epoch_change_condition),
            num_unstake_epochs,
            total_emission_xrd_per_epoch,
            min_validator_reliability,
            num_owner_stake_units_unlock_epochs,
            num_fee_increase_delay_epochs,
            validator_creation_usd_equivalent_cost,
            validator_creation_xrd_cost,
        }
    }
}


