/*
 * Engine State API - Babylon (Anemone)
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BlueprintStaticTypeReferenceAllOf {
    #[serde(rename = "static_type_reference")]
    pub static_type_reference: Option<crate::engine_state_api::generated::models::ResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
}

impl BlueprintStaticTypeReferenceAllOf {
    pub fn new(static_type_reference: crate::engine_state_api::generated::models::ResolvedTypeReference) -> BlueprintStaticTypeReferenceAllOf {
        BlueprintStaticTypeReferenceAllOf {
            static_type_reference: Option::Some(static_type_reference),
        }
    }
}


