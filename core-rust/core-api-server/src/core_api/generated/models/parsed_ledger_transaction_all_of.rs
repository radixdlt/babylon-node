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
pub struct ParsedLedgerTransactionAllOf {
    #[serde(rename = "ledger_transaction", skip_serializing_if = "Option::is_none")]
    pub ledger_transaction: Option<Box<crate::core_api::generated::models::LedgerTransaction>>,
    #[serde(rename = "identifiers")]
    pub identifiers: Box<crate::core_api::generated::models::ParsedLedgerTransactionAllOfIdentifiers>,
}

impl ParsedLedgerTransactionAllOf {
    pub fn new(identifiers: crate::core_api::generated::models::ParsedLedgerTransactionAllOfIdentifiers) -> ParsedLedgerTransactionAllOf {
        ParsedLedgerTransactionAllOf {
            ledger_transaction: None,
            identifiers: Box::new(identifiers),
        }
    }
}


