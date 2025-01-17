/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Block : Blocks contain an array of Transactions that occurred at a particular BlockIdentifier.  A hard requirement for blocks returned by Rosetta implementations is that they MUST be _inalterable_: once a client has requested and received a block identified by a specific BlockIndentifier, all future calls for that same BlockIdentifier must return the same block contents. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Block {
    #[serde(rename = "block_identifier")]
    pub block_identifier: Box<crate::mesh_api::generated::models::BlockIdentifier>,
    #[serde(rename = "parent_block_identifier")]
    pub parent_block_identifier: Box<crate::mesh_api::generated::models::BlockIdentifier>,
    /// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second. 
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "transactions")]
    pub transactions: Vec<crate::mesh_api::generated::models::Transaction>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Block {
    /// Blocks contain an array of Transactions that occurred at a particular BlockIdentifier.  A hard requirement for blocks returned by Rosetta implementations is that they MUST be _inalterable_: once a client has requested and received a block identified by a specific BlockIndentifier, all future calls for that same BlockIdentifier must return the same block contents. 
    pub fn new(block_identifier: crate::mesh_api::generated::models::BlockIdentifier, parent_block_identifier: crate::mesh_api::generated::models::BlockIdentifier, timestamp: i64, transactions: Vec<crate::mesh_api::generated::models::Transaction>) -> Block {
        Block {
            block_identifier: Box::new(block_identifier),
            parent_block_identifier: Box::new(parent_block_identifier),
            timestamp,
            transactions,
            metadata: None,
        }
    }
}


