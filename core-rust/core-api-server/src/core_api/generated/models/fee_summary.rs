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
pub struct FeeSummary {
    /// An integer between `0` and `2^32 - 1`, representing the amount of cost units consumed by the transaction execution.
    #[serde(rename = "execution_cost_units_consumed")]
    pub execution_cost_units_consumed: i64,
    /// An integer between `0` and `2^32 - 1`, representing the amount of cost units consumed by the transaction finalization.
    #[serde(rename = "finalization_cost_units_consumed")]
    pub finalization_cost_units_consumed: i64,
    /// The string-encoded decimal representing the total amount of XRD burned in the transaction as part of execution costs. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "xrd_total_execution_cost")]
    pub xrd_total_execution_cost: String,
    /// The string-encoded decimal representing the total amount of XRD burned in the transaction as part of finalization costs. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "xrd_total_finalization_cost")]
    pub xrd_total_finalization_cost: String,
    /// The string-encoded decimal representing the total amount of XRD paid in royalties as part of the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "xrd_total_royalty_cost")]
    pub xrd_total_royalty_cost: String,
    /// The string-encoded decimal representing the total amount of XRD paid in state expansion costs as part of the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "xrd_total_storage_cost")]
    pub xrd_total_storage_cost: String,
    /// The string-encoded decimal representing the total amount of XRD tipped to validators in the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. 
    #[serde(rename = "xrd_total_tipping_cost")]
    pub xrd_total_tipping_cost: String,
}

impl FeeSummary {
    pub fn new(execution_cost_units_consumed: i64, finalization_cost_units_consumed: i64, xrd_total_execution_cost: String, xrd_total_finalization_cost: String, xrd_total_royalty_cost: String, xrd_total_storage_cost: String, xrd_total_tipping_cost: String) -> FeeSummary {
        FeeSummary {
            execution_cost_units_consumed,
            finalization_cost_units_consumed,
            xrd_total_execution_cost,
            xrd_total_finalization_cost,
            xrd_total_royalty_cost,
            xrd_total_storage_cost,
            xrd_total_tipping_cost,
        }
    }
}


