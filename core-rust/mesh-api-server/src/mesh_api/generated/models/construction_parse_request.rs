/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionParseRequest : ConstructionParseRequest is the input to the `/construction/parse` endpoint. It allows the caller to parse either an unsigned or signed transaction. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ConstructionParseRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::mesh_api::generated::models::NetworkIdentifier>,
    /// Signed is a boolean indicating whether the transaction is signed. 
    #[serde(rename = "signed")]
    pub signed: bool,
    /// This must be either the unsigned transaction blob returned by `/construction/payloads` or the signed transaction blob returned by `/construction/combine`. 
    #[serde(rename = "transaction")]
    pub transaction: String,
}

impl ConstructionParseRequest {
    /// ConstructionParseRequest is the input to the `/construction/parse` endpoint. It allows the caller to parse either an unsigned or signed transaction. 
    pub fn new(network_identifier: crate::mesh_api::generated::models::NetworkIdentifier, signed: bool, transaction: String) -> ConstructionParseRequest {
        ConstructionParseRequest {
            network_identifier: Box::new(network_identifier),
            signed,
            transaction,
        }
    }
}


