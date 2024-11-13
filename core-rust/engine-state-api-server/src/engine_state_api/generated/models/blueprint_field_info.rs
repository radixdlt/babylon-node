/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BlueprintFieldInfo {
    #[serde(rename = "index")]
    pub index: i32,
    /// A human-readable name, derived on a best-effort basis from the type info/blueprint/schema. May be missing either because the subject deliberately has no defined name (e.g. in case of an unnamed tuple) or because the name resolution was not successful (e.g. when certain naming conventions are not observed within the relevant definitions). 
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type_reference")]
    pub type_reference: Option<crate::engine_state_api::generated::models::BlueprintResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
    #[serde(rename = "condition", skip_serializing_if = "Option::is_none")]
    pub condition: Option<Box<crate::engine_state_api::generated::models::BlueprintFieldCondition>>,
    #[serde(rename = "transience", skip_serializing_if = "Option::is_none")]
    pub transience: Option<Box<crate::engine_state_api::generated::models::BlueprintFieldTransience>>,
}

impl BlueprintFieldInfo {
    pub fn new(index: i32, type_reference: crate::engine_state_api::generated::models::BlueprintResolvedTypeReference) -> BlueprintFieldInfo {
        BlueprintFieldInfo {
            index,
            name: None,
            type_reference: Option::Some(type_reference),
            condition: None,
            transience: None,
        }
    }
}


