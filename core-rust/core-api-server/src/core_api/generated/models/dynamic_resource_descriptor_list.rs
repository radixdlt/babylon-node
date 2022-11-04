/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */



#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DynamicResourceDescriptorList {
    #[serde(rename="List")]
    ListDynamicResourceDescriptorList {
        #[serde(rename = "resources")]
        resources: Vec<crate::core_api::generated::models::DynamicResourceDescriptor>,
    },
    #[serde(rename="SchemaPath")]
    SchemaPathDynamicResourceDescriptorList {
        #[serde(rename = "schema_path")]
        schema_path: Vec<crate::core_api::generated::models::SchemaSubpath>,
    },
}




