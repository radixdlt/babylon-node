/*
 * Engine State API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ObjectCollectionInfo {
    /// Collection's index - please note that it will not necessarily be equal to an index of this item within `ObjectModuleStateInfo.collections`. 
    #[serde(rename = "index")]
    pub index: i32,
    #[serde(rename = "kind")]
    pub kind: crate::engine_state_api::generated::models::ObjectCollectionKind,
    /// A human-readable name, derived on a best-effort basis from the type info/blueprint/schema. May be missing either because the subject deliberately has no defined name (e.g. in case of an unnamed tuple) or because the name resolution was not successful (e.g. when certain naming conventions are not observed within the relevant definitions). 
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "key_type_reference")]
    pub key_type_reference: Option<crate::engine_state_api::generated::models::ResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
    #[serde(rename = "value_type_reference")]
    pub value_type_reference: Option<crate::engine_state_api::generated::models::ResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
}

impl ObjectCollectionInfo {
    pub fn new(index: i32, kind: crate::engine_state_api::generated::models::ObjectCollectionKind, key_type_reference: crate::engine_state_api::generated::models::ResolvedTypeReference, value_type_reference: crate::engine_state_api::generated::models::ResolvedTypeReference) -> ObjectCollectionInfo {
        ObjectCollectionInfo {
            index,
            kind,
            name: None,
            key_type_reference: Option::Some(key_type_reference),
            value_type_reference: Option::Some(value_type_reference),
        }
    }
}

