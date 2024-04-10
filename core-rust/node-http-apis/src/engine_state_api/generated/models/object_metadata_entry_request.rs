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
pub struct ObjectMetadataEntryRequest {
    /// A Bech32m-encoded, human readable rendering of an arbitrary Entity's address.
    #[serde(rename = "entity_address")]
    pub entity_address: String,
    /// The metadata key (i.e. always a string) to look up the entry by.
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "sbor_format_options", skip_serializing_if = "Option::is_none")]
    pub sbor_format_options: Option<Box<crate::engine_state_api::generated::models::SborFormatOptions>>,
    #[serde(rename = "at_ledger_state", skip_serializing_if = "Option::is_none")]
    pub at_ledger_state: Option<Box<crate::engine_state_api::generated::models::LedgerStateSelector>>,
}

impl ObjectMetadataEntryRequest {
    pub fn new(entity_address: String, key: String) -> ObjectMetadataEntryRequest {
        ObjectMetadataEntryRequest {
            entity_address,
            key,
            sbor_format_options: None,
            at_ledger_state: None,
        }
    }
}


