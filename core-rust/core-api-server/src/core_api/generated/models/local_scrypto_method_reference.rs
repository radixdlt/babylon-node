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
pub struct LocalScryptoMethodReference {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::LocalMethodReferenceType,
    #[serde(rename = "name")]
    pub name: String,
}

impl LocalScryptoMethodReference {
    pub fn new(_type: crate::core_api::generated::models::LocalMethodReferenceType, name: String) -> LocalScryptoMethodReference {
        LocalScryptoMethodReference {
            _type,
            name,
        }
    }
}


