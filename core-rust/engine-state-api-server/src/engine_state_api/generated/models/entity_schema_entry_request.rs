/*
 * Engine State API - Babylon (Anemone)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.1-beta
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct EntitySchemaEntryRequest {
    /// A Bech32m-encoded, human readable rendering of an arbitrary Entity's address.
    #[serde(rename = "entity_address")]
    pub entity_address: String,
    /// A hex-encoded Schema hash, capturing the identity of an SBOR schema.
    #[serde(rename = "schema_hash")]
    pub schema_hash: String,
    #[serde(rename = "sbor_format_options", skip_serializing_if = "Option::is_none")]
    pub sbor_format_options: Option<Box<crate::engine_state_api::generated::models::SborFormatOptions>>,
    #[serde(rename = "at_ledger_state", skip_serializing_if = "Option::is_none")]
    pub at_ledger_state: Option<Box<crate::engine_state_api::generated::models::LedgerStateSelector>>,
}

impl EntitySchemaEntryRequest {
    pub fn new(entity_address: String, schema_hash: String) -> EntitySchemaEntryRequest {
        EntitySchemaEntryRequest {
            entity_address,
            schema_hash,
            sbor_format_options: None,
            at_ledger_state: None,
        }
    }
}


