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
pub struct ResourceManagerSubstate {
    #[serde(rename = "entity_type")]
    pub entity_type: crate::core_api::generated::models::EntityType,
    #[serde(rename = "substate_type")]
    pub substate_type: crate::core_api::generated::models::SubstateType,
    #[serde(rename = "resource_type")]
    pub resource_type: crate::core_api::generated::models::ResourceType,
    #[serde(rename = "fungible_divisibility", skip_serializing_if = "Option::is_none")]
    pub fungible_divisibility: Option<i32>,
    #[serde(rename = "metadata")]
    pub metadata: Vec<crate::core_api::generated::models::ResourceManagerSubstateAllOfMetadata>,
    /// The string-encoded decimal representing the total supply of this resource. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. 
    #[serde(rename = "total_supply")]
    pub total_supply: String,
}

impl ResourceManagerSubstate {
    pub fn new(entity_type: crate::core_api::generated::models::EntityType, substate_type: crate::core_api::generated::models::SubstateType, resource_type: crate::core_api::generated::models::ResourceType, metadata: Vec<crate::core_api::generated::models::ResourceManagerSubstateAllOfMetadata>, total_supply: String) -> ResourceManagerSubstate {
        ResourceManagerSubstate {
            entity_type,
            substate_type,
            resource_type,
            fungible_divisibility: None,
            metadata,
            total_supply,
        }
    }
}


