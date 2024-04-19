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
pub struct BlueprintInfoResponse {
    #[serde(rename = "at_ledger_state")]
    pub at_ledger_state: Box<crate::engine_state_api::generated::models::LedgerStateSummary>,
    #[serde(rename = "info")]
    pub info: Box<crate::engine_state_api::generated::models::DetailedBlueprintInfo>,
}

impl BlueprintInfoResponse {
    pub fn new(at_ledger_state: crate::engine_state_api::generated::models::LedgerStateSummary, info: crate::engine_state_api::generated::models::DetailedBlueprintInfo) -> BlueprintInfoResponse {
        BlueprintInfoResponse {
            at_ledger_state: Box::new(at_ledger_state),
            info: Box::new(info),
        }
    }
}

