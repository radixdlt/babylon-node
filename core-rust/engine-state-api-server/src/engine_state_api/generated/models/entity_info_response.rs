/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct EntityInfoResponse {
    #[serde(rename = "at_ledger_state")]
    pub at_ledger_state: Box<crate::engine_state_api::generated::models::LedgerStateSummary>,
    #[serde(rename = "info")]
    pub info: Option<crate::engine_state_api::generated::models::EntityInfo>, // Using Option permits Default trait; Will always be Some in normal use
}

impl EntityInfoResponse {
    pub fn new(at_ledger_state: crate::engine_state_api::generated::models::LedgerStateSummary, info: crate::engine_state_api::generated::models::EntityInfo) -> EntityInfoResponse {
        EntityInfoResponse {
            at_ledger_state: Box::new(at_ledger_state),
            info: Option::Some(info),
        }
    }
}


