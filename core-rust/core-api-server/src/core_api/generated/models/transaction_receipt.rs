/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// TransactionReceipt : The transaction execution receipt



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct TransactionReceipt {
    #[serde(rename = "status")]
    pub status: crate::core_api::generated::models::TransactionStatus,
    #[serde(rename = "fee_summary")]
    pub fee_summary: Box<crate::core_api::generated::models::FeeSummary>,
    #[serde(rename = "state_updates")]
    pub state_updates: Box<crate::core_api::generated::models::StateUpdates>,
    /// The manifest line-by-line engine return data (only present if status is Succeeded)
    #[serde(rename = "output", skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<crate::core_api::generated::models::SborData>>,
    /// Error message (only present if status is Failed or Rejected)
    #[serde(rename = "error_message", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl TransactionReceipt {
    /// The transaction execution receipt
    pub fn new(status: crate::core_api::generated::models::TransactionStatus, fee_summary: crate::core_api::generated::models::FeeSummary, state_updates: crate::core_api::generated::models::StateUpdates) -> TransactionReceipt {
        TransactionReceipt {
            status,
            fee_summary: Box::new(fee_summary),
            state_updates: Box::new(state_updates),
            output: None,
            error_message: None,
        }
    }
}


