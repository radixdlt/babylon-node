/*
 * Browse API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ObjectModuleStateInfo : Information about the state held by a particular module of the object. 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ObjectModuleStateInfo {
    #[serde(rename = "fields")]
    pub fields: Vec<crate::browse_api::generated::models::ObjectFieldInfo>,
    #[serde(rename = "collections")]
    pub collections: Vec<crate::browse_api::generated::models::ObjectCollectionInfo>,
}

impl ObjectModuleStateInfo {
    /// Information about the state held by a particular module of the object. 
    pub fn new(fields: Vec<crate::browse_api::generated::models::ObjectFieldInfo>, collections: Vec<crate::browse_api::generated::models::ObjectCollectionInfo>) -> ObjectModuleStateInfo {
        ObjectModuleStateInfo {
            fields,
            collections,
        }
    }
}


