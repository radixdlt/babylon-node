/*
 * Babylon Core API - RCnet v3.1
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct SortedSubstateKey {
    #[serde(rename = "key_type")]
    pub key_type: crate::core_api::generated::models::SubstateKeyType,
    /// The hex-encoded bytes of the partially-hashed DB sort key, under the given entity partition
    #[serde(rename = "db_sort_key_hex")]
    pub db_sort_key_hex: String,
    /// The hex-encoded bytes of the sorted part of the key
    #[serde(rename = "sort_prefix_hex")]
    pub sort_prefix_hex: String,
    /// The hex-encoded remaining bytes of the key
    #[serde(rename = "key_hex")]
    pub key_hex: String,
}

impl SortedSubstateKey {
    pub fn new(key_type: crate::core_api::generated::models::SubstateKeyType, db_sort_key_hex: String, sort_prefix_hex: String, key_hex: String) -> SortedSubstateKey {
        SortedSubstateKey {
            key_type,
            db_sort_key_hex,
            sort_prefix_hex,
            key_hex,
        }
    }
}


