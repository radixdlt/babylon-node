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
pub struct AllOfDynamicProofRuleAllOf {
    #[serde(rename = "list")]
    pub list: Option<crate::core_api::generated::models::DynamicResourceDescriptorList>, // Using Option permits Default trait; Will always be Some in normal use
}

impl AllOfDynamicProofRuleAllOf {
    pub fn new(list: crate::core_api::generated::models::DynamicResourceDescriptorList) -> AllOfDynamicProofRuleAllOf {
        AllOfDynamicProofRuleAllOf {
            list: Option::Some(list),
        }
    }
}


