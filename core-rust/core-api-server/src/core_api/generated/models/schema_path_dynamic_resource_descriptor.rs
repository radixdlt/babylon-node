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
pub struct SchemaPathDynamicResourceDescriptor {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::DynamicResourceDescriptorType,
    #[serde(rename = "schema_path")]
    pub schema_path: Vec<crate::core_api::generated::models::SchemaSubpath>,
}

impl SchemaPathDynamicResourceDescriptor {
    pub fn new(_type: crate::core_api::generated::models::DynamicResourceDescriptorType, schema_path: Vec<crate::core_api::generated::models::SchemaSubpath>) -> SchemaPathDynamicResourceDescriptor {
        SchemaPathDynamicResourceDescriptor {
            _type,
            schema_path,
        }
    }
}


