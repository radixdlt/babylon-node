/*
 * Babylon Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ParsedNotarizedTransactionAllOf {
    #[serde(rename = "notarized_transaction", skip_serializing_if = "Option::is_none")]
    pub notarized_transaction: Option<Box<crate::core_api::generated::models::NotarizedTransaction>>,
    #[serde(rename = "identifiers")]
    pub identifiers: Box<crate::core_api::generated::models::ParsedNotarizedTransactionAllOfIdentifiers>,
    #[serde(rename = "validation_error", skip_serializing_if = "Option::is_none")]
    pub validation_error: Option<Box<crate::core_api::generated::models::ParsedNotarizedTransactionAllOfValidationError>>,
}

impl ParsedNotarizedTransactionAllOf {
    pub fn new(identifiers: crate::core_api::generated::models::ParsedNotarizedTransactionAllOfIdentifiers) -> ParsedNotarizedTransactionAllOf {
        ParsedNotarizedTransactionAllOf {
            notarized_transaction: None,
            identifiers: Box::new(identifiers),
            validation_error: None,
        }
    }
}


