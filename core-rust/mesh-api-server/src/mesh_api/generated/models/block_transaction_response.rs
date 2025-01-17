/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlockTransactionResponse : A BlockTransactionResponse contains information about a block transaction. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BlockTransactionResponse {
    #[serde(rename = "transaction")]
    pub transaction: Box<crate::mesh_api::generated::models::Transaction>,
}

impl BlockTransactionResponse {
    /// A BlockTransactionResponse contains information about a block transaction. 
    pub fn new(transaction: crate::mesh_api::generated::models::Transaction) -> BlockTransactionResponse {
        BlockTransactionResponse {
            transaction: Box::new(transaction),
        }
    }
}

