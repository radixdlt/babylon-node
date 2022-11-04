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
pub struct RequireFixedProofRuleAllOf {
    #[serde(rename = "resource")]
    pub resource: Option<crate::core_api::generated::models::FixedResourceDescriptor>, // Using Option permits Default trait; Will always be Some in normal use
}

impl RequireFixedProofRuleAllOf {
    pub fn new(resource: crate::core_api::generated::models::FixedResourceDescriptor) -> RequireFixedProofRuleAllOf {
        RequireFixedProofRuleAllOf {
            resource: Option::Some(resource),
        }
    }
}


