/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// LedgerStateSummary : A state version and summarized header of the ledger proof which can be used to verify the returned on-ledger data.  Please note that: - For \"current top-of-ledger\" requests (i.e. not specifying any `LedgerStateSelector`),   this will always be the most recent ledger header, proving exactly the version at which   the on-ledger data was read. - For historical requests (i.e. using a `LedgerStateSelector`), this will be the *nearest*   ledger header at *or after* the requested past state version - depending on the   granularity of the consensus progress (and the granularity of the ledger proofs actually   persisted by the queried Node). 



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct LedgerStateSummary {
    #[serde(rename = "state_version")]
    pub state_version: i64,
    #[serde(rename = "header_summary")]
    pub header_summary: Box<crate::engine_state_api::generated::models::LedgerHeaderSummary>,
}

impl LedgerStateSummary {
    /// A state version and summarized header of the ledger proof which can be used to verify the returned on-ledger data.  Please note that: - For \"current top-of-ledger\" requests (i.e. not specifying any `LedgerStateSelector`),   this will always be the most recent ledger header, proving exactly the version at which   the on-ledger data was read. - For historical requests (i.e. using a `LedgerStateSelector`), this will be the *nearest*   ledger header at *or after* the requested past state version - depending on the   granularity of the consensus progress (and the granularity of the ledger proofs actually   persisted by the queried Node). 
    pub fn new(state_version: i64, header_summary: crate::engine_state_api::generated::models::LedgerHeaderSummary) -> LedgerStateSummary {
        LedgerStateSummary {
            state_version,
            header_summary: Box::new(header_summary),
        }
    }
}


