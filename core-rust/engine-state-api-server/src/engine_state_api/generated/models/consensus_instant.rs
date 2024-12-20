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
pub struct ConsensusInstant {
    /// A decimal string-encoded 64-bit signed integer, marking the unix timestamp in milliseconds.  Note: this field accurately represents the full range of possible values (i.e. `-2^63 <= milliseconds < 2^63`). 
    #[serde(rename = "unix_timestamp_ms")]
    pub unix_timestamp_ms: String,
    /// The RFC 3339 / ISO 8601 string representation of the timestamp. Will always use \"Z\" (denoting UTC) and include milliseconds (even if `000`). E.g.: `2023-01-26T18:30:09.453Z`.  Note: This field will *not* be present if the `unix_timestamp_ms` value is outside the basic range supported by the RFC 3339 / ISO 8601 standard, which starts at year 1583 (i.e. the beginning of the Gregorian calendar) and ends at year 9999 (inclusive). 
    #[serde(rename = "date_time", skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
}

impl ConsensusInstant {
    pub fn new(unix_timestamp_ms: String) -> ConsensusInstant {
        ConsensusInstant {
            unix_timestamp_ms,
            date_time: None,
        }
    }
}


