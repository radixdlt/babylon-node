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
pub struct NonFungibleLocalIdMetadataValue {
    #[serde(rename = "type")]
    pub _type: crate::engine_state_api::generated::models::MetadataValueType,
    /// A simple string representation of a non-fungible local ID, with a type-dependent formatting: * For string ids, this is `<the-string-id>` * For integer ids, this is `#the-integer-id#` * For bytes ids, this is `[the-lower-case-hex-representation]` * For RUID ids, this is `{...-...-...-...}` where `...` are each 16 hex characters. 
    #[serde(rename = "value")]
    pub value: String,
}

impl NonFungibleLocalIdMetadataValue {
    pub fn new(_type: crate::engine_state_api::generated::models::MetadataValueType, value: String) -> NonFungibleLocalIdMetadataValue {
        NonFungibleLocalIdMetadataValue {
            _type,
            value,
        }
    }
}


