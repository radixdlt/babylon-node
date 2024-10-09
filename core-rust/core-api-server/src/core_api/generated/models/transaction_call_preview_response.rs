/*
 * Radix Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct TransactionCallPreviewResponse {
    #[serde(rename = "at_ledger_state")]
    pub at_ledger_state: Box<crate::core_api::generated::models::LedgerStateSummary>,
    #[serde(rename = "status")]
    pub status: crate::core_api::generated::models::TransactionStatus,
    #[serde(rename = "output", skip_serializing_if = "Option::is_none")]
    pub output: Option<Box<crate::core_api::generated::models::SborData>>,
    /// Error message (only present if status is Failed or Rejected)
    #[serde(rename = "error_message", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl TransactionCallPreviewResponse {
    pub fn new(at_ledger_state: crate::core_api::generated::models::LedgerStateSummary, status: crate::core_api::generated::models::TransactionStatus) -> TransactionCallPreviewResponse {
        TransactionCallPreviewResponse {
            at_ledger_state: Box::new(at_ledger_state),
            status,
            output: None,
            error_message: None,
        }
    }
}


