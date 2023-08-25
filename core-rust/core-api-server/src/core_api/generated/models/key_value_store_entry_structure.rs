/*
 * Babylon Core API - RCnet v3
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyValueStoreEntryStructure {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::SubstateSystemStructureType,
    /// Bech32m-encoded human readable version of the entity's address (ie the entity's node id)
    #[serde(rename = "key_value_store_address")]
    pub key_value_store_address: String,
    /// The hex-encoded schema hash, capturing the identity of an SBOR schema.
    #[serde(rename = "key_schema_hash")]
    pub key_schema_hash: String,
    #[serde(rename = "key_local_type_index")]
    pub key_local_type_index: Box<crate::core_api::generated::models::LocalTypeIndex>,
    /// The hex-encoded schema hash, capturing the identity of an SBOR schema.
    #[serde(rename = "value_schema_hash")]
    pub value_schema_hash: String,
    #[serde(rename = "value_local_type_index")]
    pub value_local_type_index: Box<crate::core_api::generated::models::LocalTypeIndex>,
}

impl KeyValueStoreEntryStructure {
    pub fn new(_type: crate::core_api::generated::models::SubstateSystemStructureType, key_value_store_address: String, key_schema_hash: String, key_local_type_index: crate::core_api::generated::models::LocalTypeIndex, value_schema_hash: String, value_local_type_index: crate::core_api::generated::models::LocalTypeIndex) -> KeyValueStoreEntryStructure {
        KeyValueStoreEntryStructure {
            _type,
            key_value_store_address,
            key_schema_hash,
            key_local_type_index: Box::new(key_local_type_index),
            value_schema_hash,
            value_local_type_index: Box::new(value_local_type_index),
        }
    }
}

