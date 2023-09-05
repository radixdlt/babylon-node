/*
 * Babylon Core API - RCnet v3.1
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */



#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "error_type")]
pub enum ErrorResponse {
    #[serde(rename="Basic")]
    BasicErrorResponse {
        /// A numeric code corresponding to the given HTTP error code.
        #[serde(rename = "code")]
        code: i32,
        /// A human-readable error message.
        #[serde(rename = "message")]
        message: String,
        /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
        #[serde(rename = "trace_id", skip_serializing_if = "Option::is_none")]
        trace_id: Option<String>,
    },
    #[serde(rename="LtsTransactionSubmit")]
    LtsTransactionSubmitErrorResponse {
        /// A numeric code corresponding to the given HTTP error code.
        #[serde(rename = "code")]
        code: i32,
        /// A human-readable error message.
        #[serde(rename = "message")]
        message: String,
        /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
        #[serde(rename = "trace_id", skip_serializing_if = "Option::is_none")]
        trace_id: Option<String>,
        #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
        details: Option<Box<crate::core_api::generated::models::LtsTransactionSubmitErrorDetails>>,
    },
    #[serde(rename="TransactionSubmit")]
    TransactionSubmitErrorResponse {
        /// A numeric code corresponding to the given HTTP error code.
        #[serde(rename = "code")]
        code: i32,
        /// A human-readable error message.
        #[serde(rename = "message")]
        message: String,
        /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
        #[serde(rename = "trace_id", skip_serializing_if = "Option::is_none")]
        trace_id: Option<String>,
        #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
        details: Option<Box<crate::core_api::generated::models::TransactionSubmitErrorDetails>>,
    },
}




