/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NetworkOptionsResponse : NetworkOptionsResponse contains information about the versioning of the node and the allowed operation statuses, operation types, and errors. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct NetworkOptionsResponse {
    #[serde(rename = "version")]
    pub version: Box<crate::mesh_api::generated::models::Version>,
    #[serde(rename = "allow")]
    pub allow: Box<crate::mesh_api::generated::models::Allow>,
}

impl NetworkOptionsResponse {
    /// NetworkOptionsResponse contains information about the versioning of the node and the allowed operation statuses, operation types, and errors. 
    pub fn new(version: crate::mesh_api::generated::models::Version, allow: crate::mesh_api::generated::models::Allow) -> NetworkOptionsResponse {
        NetworkOptionsResponse {
            version: Box::new(version),
            allow: Box::new(allow),
        }
    }
}


