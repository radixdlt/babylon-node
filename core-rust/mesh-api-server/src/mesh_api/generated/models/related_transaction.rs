/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// RelatedTransaction : The related_transaction allows implementations to link together multiple transactions. An unpopulated network identifier indicates that the related transaction is on the same network. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct RelatedTransaction {
    #[serde(rename = "network_identifier", skip_serializing_if = "Option::is_none")]
    pub network_identifier: Option<Box<crate::mesh_api::generated::models::NetworkIdentifier>>,
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: Box<crate::mesh_api::generated::models::TransactionIdentifier>,
    #[serde(rename = "direction")]
    pub direction: crate::mesh_api::generated::models::Direction,
}

impl RelatedTransaction {
    /// The related_transaction allows implementations to link together multiple transactions. An unpopulated network identifier indicates that the related transaction is on the same network. 
    pub fn new(transaction_identifier: crate::mesh_api::generated::models::TransactionIdentifier, direction: crate::mesh_api::generated::models::Direction) -> RelatedTransaction {
        RelatedTransaction {
            network_identifier: None,
            transaction_identifier: Box::new(transaction_identifier),
            direction,
        }
    }
}


