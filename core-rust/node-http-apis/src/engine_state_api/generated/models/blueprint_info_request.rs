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
pub struct BlueprintInfoRequest {
    /// A Bech32m-encoded, human readable rendering of a Package address.
    #[serde(rename = "package_address")]
    pub package_address: String,
    #[serde(rename = "blueprint_name")]
    pub blueprint_name: String,
    /// A string of format `Major.Minor.Patch` (all parts being `u32`). Defaults to `1.0.0`. 
    #[serde(rename = "blueprint_version", skip_serializing_if = "Option::is_none")]
    pub blueprint_version: Option<String>,
    #[serde(rename = "sbor_format_options", skip_serializing_if = "Option::is_none")]
    pub sbor_format_options: Option<Box<crate::engine_state_api::generated::models::SborFormatOptions>>,
    #[serde(rename = "at_ledger_state", skip_serializing_if = "Option::is_none")]
    pub at_ledger_state: Option<Box<crate::engine_state_api::generated::models::LedgerStateSelector>>,
}

impl BlueprintInfoRequest {
    pub fn new(package_address: String, blueprint_name: String) -> BlueprintInfoRequest {
        BlueprintInfoRequest {
            package_address,
            blueprint_name,
            blueprint_version: None,
            sbor_format_options: None,
            at_ledger_state: None,
        }
    }
}


