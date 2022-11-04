/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewRequest {
    /// The logical name of the network
    #[serde(rename = "network")]
    pub network: String,
    /// A text-representation of a transaction manifest
    #[serde(rename = "manifest")]
    pub manifest: String,
    /// An array of hex-encoded blob data (optional)
    #[serde(rename = "blobs_hex", skip_serializing_if = "Option::is_none")]
    pub blobs_hex: Option<Vec<String>>,
    /// An integer between `0` and `10^10`, marking the epoch at which the transaction starts being valid
    #[serde(rename = "start_epoch_inclusive")]
    pub start_epoch_inclusive: i64,
    /// An integer between `0` and `10^10`, marking the epoch at which the transaction is no longer valid
    #[serde(rename = "end_epoch_exclusive")]
    pub end_epoch_exclusive: i64,
    #[serde(rename = "notary_public_key", skip_serializing_if = "Option::is_none")]
    pub notary_public_key: Option<Box<crate::core_api::generated::models::PublicKey>>,
    /// Whether the notary should count as a signatory (optional, default false)
    #[serde(rename = "notary_as_signatory", skip_serializing_if = "Option::is_none")]
    pub notary_as_signatory: Option<bool>,
    /// An integer between `0` and `2^32 - 1`, giving the maximum number of cost units available for transaction execution
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: i64,
    /// An integer between `0` and `2^32 - 1`, specifying the validator tip as a percentage amount. A value of `1` corresponds to 1% of the fee.
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: i64,
    /// A decimal-string-encoded integer between `0` and `2^64 - 1`, used to ensure the transaction intent is unique.
    #[serde(rename = "nonce")]
    pub nonce: String,
    /// A list of public keys to be used as transaction signers
    #[serde(rename = "signer_public_keys")]
    pub signer_public_keys: Vec<crate::core_api::generated::models::PublicKey>,
    #[serde(rename = "flags")]
    pub flags: Box<crate::core_api::generated::models::TransactionPreviewRequestFlags>,
}

impl TransactionPreviewRequest {
    pub fn new(network: String, manifest: String, start_epoch_inclusive: i64, end_epoch_exclusive: i64, cost_unit_limit: i64, tip_percentage: i64, nonce: String, signer_public_keys: Vec<crate::core_api::generated::models::PublicKey>, flags: crate::core_api::generated::models::TransactionPreviewRequestFlags) -> TransactionPreviewRequest {
        TransactionPreviewRequest {
            network,
            manifest,
            blobs_hex: None,
            start_epoch_inclusive,
            end_epoch_exclusive,
            notary_public_key: None,
            notary_as_signatory: None,
            cost_unit_limit,
            tip_percentage,
            nonce,
            signer_public_keys,
            flags: Box::new(flags),
        }
    }
}


