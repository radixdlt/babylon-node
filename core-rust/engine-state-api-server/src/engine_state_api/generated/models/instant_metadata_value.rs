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
pub struct InstantMetadataValue {
    #[serde(rename = "type")]
    pub _type: crate::engine_state_api::generated::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: Box<crate::engine_state_api::generated::models::ScryptoInstant>,
}

impl InstantMetadataValue {
    pub fn new(_type: crate::engine_state_api::generated::models::MetadataValueType, value: crate::engine_state_api::generated::models::ScryptoInstant) -> InstantMetadataValue {
        InstantMetadataValue {
            _type,
            value: Box::new(value),
        }
    }
}


