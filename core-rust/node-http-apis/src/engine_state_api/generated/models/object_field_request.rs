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
pub struct ObjectFieldRequest {
    /// A Bech32m-encoded, human readable rendering of an arbitrary Entity's address.
    #[serde(rename = "entity_address")]
    pub entity_address: String,
    #[serde(rename = "attached_module_id", skip_serializing_if = "Option::is_none")]
    pub attached_module_id: Option<crate::engine_state_api::generated::models::AttachedModuleId>,
    /// Name of the field to read. Either this or `field_index` is required. Note: getting a field by name is only a convenience feature, meant for manual, ad-hoc requests (i.e. allowing a human to use the human-readable `DerivedName` when available). In general, a field's primary identifier is its index, and any production-grade integration should use it. 
    #[serde(rename = "field_name", skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    /// Index of the field to read. Either this or `field_name` is required.
    #[serde(rename = "field_index", skip_serializing_if = "Option::is_none")]
    pub field_index: Option<i32>,
    #[serde(rename = "sbor_format_options", skip_serializing_if = "Option::is_none")]
    pub sbor_format_options: Option<Box<crate::engine_state_api::generated::models::SborFormatOptions>>,
}

impl ObjectFieldRequest {
    pub fn new(entity_address: String) -> ObjectFieldRequest {
        ObjectFieldRequest {
            entity_address,
            attached_module_id: None,
            field_name: None,
            field_index: None,
            sbor_format_options: None,
        }
    }
}


