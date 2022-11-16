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
pub struct LedgerTransactionBase {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::LedgerTransactionType,
    /// The hex-encoded full ledger transaction payload
    #[serde(rename = "payload_hex")]
    pub payload_hex: String,
}

impl LedgerTransactionBase {
    pub fn new(_type: crate::core_api::generated::models::LedgerTransactionType, payload_hex: String) -> LedgerTransactionBase {
        LedgerTransactionBase {
            _type,
            payload_hex,
        }
    }
}


