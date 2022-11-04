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
pub struct ResourceManagerSubstateAllOf {
    #[serde(rename = "resource_type")]
    pub resource_type: crate::core_api::generated::models::ResourceType,
    #[serde(rename = "fungible_divisibility", skip_serializing_if = "Option::is_none")]
    pub fungible_divisibility: Option<i32>,
    #[serde(rename = "metadata")]
    pub metadata: Vec<crate::core_api::generated::models::ResourceManagerSubstateAllOfMetadata>,
    /// The string-encoded decimal representing the total supply of this resource. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`.     owned_entities: 
    #[serde(rename = "total_supply")]
    pub total_supply: String,
    #[serde(rename = "owned_nf_store", skip_serializing_if = "Option::is_none")]
    pub owned_nf_store: Option<Box<crate::core_api::generated::models::EntityReference>>,
}

impl ResourceManagerSubstateAllOf {
    pub fn new(resource_type: crate::core_api::generated::models::ResourceType, metadata: Vec<crate::core_api::generated::models::ResourceManagerSubstateAllOfMetadata>, total_supply: String) -> ResourceManagerSubstateAllOf {
        ResourceManagerSubstateAllOf {
            resource_type,
            fungible_divisibility: None,
            metadata,
            total_supply,
            owned_nf_store: None,
        }
    }
}


